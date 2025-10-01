use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, OnceLock};
use std::time::{Duration, Instant};
use std::{error::Error, path::Path};

use anyhow::{Context, Result, bail};
use clap::Parser;
use flexi_logger::LogSpecification;
use parking_lot::Mutex;
use sysinfo::{Pid, ProcessStatus, System};
use termusiclib::config::v2::server::config_extra::ServerConfigVersionedDefaulted;
use termusiclib::config::v2::server::{ComProtocol, ScanDepth};
use termusiclib::config::v2::tui::config_extra::TuiConfigVersionedDefaulted;
use termusiclib::config::{
    ServerOverlay, SharedServerSettings, SharedTuiSettings, TuiOverlay, new_shared_server_settings,
    new_shared_tui_settings,
};
use termusiclib::player::music_player_client::MusicPlayerClient;
use termusiclib::{podcast, utils};
use tokio::io::AsyncReadExt;
use tokio::process::Child;
use tokio::sync::RwLock;
use tokio::task::AbortHandle;
use tokio_util::sync::CancellationToken;

use ui::UI;

mod cli;
mod logger;
mod ui;

#[macro_use]
extern crate log;

pub const MAX_DEPTH: usize = 4;

/// Combined Shared Settings, because the TUI needs access to both configs
///
/// also this exists instead of a unnamed tuple
#[derive(Debug, Clone)]
pub struct CombinedSettings {
    pub server: SharedServerSettings,
    pub tui: SharedTuiSettings,
}

fn main() -> Result<()> {
    // print error to the log and then throw it
    if let Err(err) = actual_main() {
        error!("Error: {err:?}");
        return Err(err);
    }

    Ok(())
}

pub static SERVER_PID: OnceLock<Pid> = OnceLock::new();

/// Handles CLI args, potentially starts termusic-server, then runs UI loop
#[tokio::main]
async fn actual_main() -> Result<()> {
    let args = cli::Args::parse();
    let mut logger_handle = logger::setup(&args);
    let config = get_config(&args)?;

    ctrl_c_handler().expect("Error setting Ctrl-C handler");

    if let Some(action) = args.action {
        return execute_action(action, &config).await;
    }

    // launch the daemon if it isn't already
    let (pid, child) = {
        let active_pid = find_active_server_process();

        if let Some(pid) = active_pid {
            (pid.as_u32(), None)
        } else {
            let child = launch_server(&args)?;
            (child.id().unwrap(), Some(child))
        }
    };
    drop(child);

    // let server_output = child.map(collect_server_output);

    println!("Server process ID: {pid}");
    SERVER_PID
        .set(Pid::from_u32(pid))
        .unwrap_or_else(|_| error!("Could not set SERVER_PID."));

    // this is a bad implementation, but there is no way to currently only shut off stderr / stdout
    // see https://github.com/emabee/flexi_logger/issues/142
    if !args.log_options.log_to_file {
        logger_handle.set_new_spec(LogSpecification::off());
    } else if let Err(err) =
        logger_handle.adapt_duplication_to_stderr(flexi_logger::Duplicate::None)
    {
        warn!("flexi_logger error: {err}");
    }

    info!("Waiting until connected");

    let (client, addr) = match wait_till_connected(&config, pid).await {
        Ok(v) => v,
        Err(err) => {
            // if let Some(server_output) = server_output {
            //     server_output.cancel_token.cancel();
            //     let stdout = server_output.stdout.read().await;
            //     let stderr = server_output.stderr.read().await;

            //     let stdout = String::from_utf8_lossy(&stdout).to_string();
            //     let stderr = String::from_utf8_lossy(&stderr).to_string();

            //     return Err(err.context(format!(
            //         "Server output during start:\n---STDOUT---\n{stdout}\n---STDERR---\n{stderr}\n---"
            //     )));
            // }

            return Err(err);
        }
    };
    info!("Connected on {addr}");

    // if let Some(server_output) = server_output {
    //     server_output.cancel_token.cancel();
    // }

    let mut ui = UI::new(config, client).await?;
    ui.run()?;

    info!("Bye");

    Ok(())
}

#[allow(dead_code)]
#[derive(Debug)]
struct ServerOutput {
    stdout: RwLock<Vec<u8>>,
    stderr: RwLock<Vec<u8>>,
    cancel_token: CancellationToken,
}

/// Spawn a task that collects the server's log output, until cancelled.
#[expect(dead_code)] // Disabled because flexi_logger seems to have a issue of completely disabling logging once stdio closes.
fn collect_server_output(mut child: Child) -> Arc<ServerOutput> {
    let output = Arc::new(ServerOutput {
        stdout: RwLock::new(Vec::new()),
        stderr: RwLock::new(Vec::new()),
        cancel_token: CancellationToken::new(),
    });
    let res = output.clone();

    let mut stdout = child.stdout.take().unwrap();
    let mut stderr = child.stderr.take().unwrap();

    tokio::spawn(async move {
        let mut handle_stdout = output.stdout.write().await;
        let mut handle_stderr = output.stderr.write().await;
        let cancel_token = output.cancel_token.clone();

        loop {
            tokio::select! {
                _ = stdout.read_buf(&mut *handle_stdout) => {

                },
                _ = stderr.read_buf(&mut *handle_stderr) => {

                }
                () = cancel_token.cancelled() => {
                    break;
                }
            }
        }

        debug!("Server log collection task ended");
    });

    res
}

/// Launch the server, passing along some arguments like `--backend`.
///
/// Returns the launched [`Child`].
fn launch_server(args: &cli::Args) -> Result<Child> {
    let termusic_server_prog = get_server_binary_exe()?;

    let mut server_args = vec![];

    // dont clone over "log-to-file", because default is "true" now, and otherwise can be controlled via TMS_LOGTOFILE or TMS_LOGFILE
    // server_args.push("--log-to-file");
    // if args.log_options.log_to_file {
    //     server_args.push("true");
    // } else {
    //     server_args.push("false");
    // }

    if args.log_options.file_color_log {
        server_args.push("--log-filecolor");
    }

    if let Some(backend) = args.backend {
        server_args.push("--backend");
        server_args.push(backend.as_str());
    }

    // server can stay around after client exits (if supported by the system)
    #[allow(clippy::zombie_processes)]
    let proc = utils::spawn_process(&termusic_server_prog, false, false, &server_args).context(
        format!(
            "Could not start binary \"{}\"",
            termusic_server_prog.display()
        ),
    )?;

    Ok(proc)
}

/// Try to find a active server process, returning its [`Pid`].
/// Otherwise if not found, returns [`None`].
fn find_active_server_process() -> Option<Pid> {
    let mut system = System::new();
    system.refresh_all();
    for (id, proc) in system.processes() {
        let Some(exe) = proc.exe().map(|v| v.display().to_string()) else {
            continue;
        };
        if exe.contains("termusic-server") {
            return Some(*id);
        }
    }

    None
}

/// Try to find the server binary adjacent to the current executable path.
/// Otherwise return command to let system PATH figure it out.
fn get_server_binary_exe() -> Result<PathBuf> {
    let mut termusic_server_prog = std::path::PathBuf::from("termusic-server");

    // try to find the server binary adjacent to the currently executing binary path
    let potential_server_exe = {
        let mut exe = std::env::current_exe()?;
        exe.pop();
        exe.join(&termusic_server_prog)
    };
    if potential_server_exe.exists() {
        termusic_server_prog = potential_server_exe;
    }

    Ok(termusic_server_prog)
}

/// Timeout to give up connecting.
const WAIT_TIMEOUT: Duration = Duration::from_secs(30);
/// Timeout until writing "Taking longer than expected" message.
const WAIT_MESSAGE_TIME: Duration = Duration::from_secs(5);
/// Time to sleep between connect tries.
const WAIT_INTERVAL: Duration = Duration::from_millis(100);

/// Wait until the [`MusicPlayerClient`] is connected on the correct transport protocol.
async fn wait_till_connected(
    config: &CombinedSettings,
    pid: u32,
) -> Result<(MusicPlayerClient<tonic::transport::Channel>, String)> {
    let protocol = config.tui.read().settings.get_com().unwrap().protocol;
    let player = match protocol {
        ComProtocol::HTTP => wait_till_connected_tcp(config, pid).await?,
        ComProtocol::UDS => wait_till_connected_uds(config, pid).await?,
    };

    Ok(player)
}

/// Wait until tonic is connected, or:
/// - tonic errors anything other than `ConnectionRefused`
/// - given PID does not exist anymore
/// - timeout of [`WAIT_TIMEOUT`] reached
async fn wait_till_connected_tcp(
    config: &CombinedSettings,
    pid: u32,
) -> Result<(MusicPlayerClient<tonic::transport::Channel>, String)> {
    let addr = {
        let config_read = config.tui.read();
        SocketAddr::from(config_read.settings.get_com().ok_or(anyhow::anyhow!(
            "Expected tui-com settings to be resolved at this point"
        ))?)
    };
    let addr = format!("http://{addr}");

    let mut sys = sysinfo::System::new();
    let sys_pid = Pid::from_u32(pid);
    let start_time = Instant::now();
    let _msg_handle = start_message_timeout();

    loop {
        if Instant::now() > start_time + WAIT_TIMEOUT {
            anyhow::bail!(
                "Could not connect within {} timeout.",
                WAIT_TIMEOUT.as_secs()
            );
        }

        sys.refresh_processes(sysinfo::ProcessesToUpdate::Some(&[sys_pid]), true);

        let status = sys.process(sys_pid);

        // dont endlessly try to connect, if the server exited / crashed
        if status.is_none() || status.is_some_and(|v| v.status() == ProcessStatus::Zombie) {
            anyhow::bail!("Process {pid} exited before being able to connect!");
        }

        match MusicPlayerClient::connect(addr.clone()).await {
            Err(err) => {
                // downcast "tonic::transport::Error" to a "std::io::Error"(kind: Os)
                if let Some(os_err) = find_source::<std::io::Error>(&err) {
                    if os_err.kind() == std::io::ErrorKind::ConnectionRefused {
                        debug!("Connection refused found!");
                        tokio::time::sleep(WAIT_INTERVAL).await;
                        continue;
                    }
                }

                // return the error and stop if it is anything other than "Connection Refused"
                anyhow::bail!(err);
            }
            Ok(client) => return Ok((client, addr)),
        }
    }
}

/// Wait until tonic is connected, or:
/// - tonic errors anything other than `ConnectionRefused`(server not accepting the connection yet) or `NotFound`(path does not exit yet)
/// - given PID does not exist anymore
/// - timeout of [`WAIT_TIMEOUT`] reached
async fn wait_till_connected_uds(
    config: &CombinedSettings,
    pid: u32,
) -> Result<(MusicPlayerClient<tonic::transport::Channel>, String)> {
    let addr = {
        let config_read = config.tui.read();
        let addr = config_read
            .settings
            .get_com()
            .unwrap()
            .socket_path
            .to_string_lossy();
        format!("unix://{addr}")
    };

    let mut sys = sysinfo::System::new();
    let sys_pid = Pid::from_u32(pid);
    let start_time = Instant::now();
    let _msg_handle = start_message_timeout();

    loop {
        if Instant::now() > start_time + WAIT_TIMEOUT {
            anyhow::bail!(
                "Could not connect within {} timeout.",
                WAIT_TIMEOUT.as_secs()
            );
        }

        sys.refresh_processes(sysinfo::ProcessesToUpdate::Some(&[sys_pid]), true);

        let status = sys.process(sys_pid);

        // dont endlessly try to connect, if the server exited / crashed
        if status.is_none() || status.is_some_and(|v| v.status() == ProcessStatus::Zombie) {
            anyhow::bail!("Process {pid} exited before being able to connect!");
        }

        match MusicPlayerClient::connect(addr.clone()).await {
            Err(err) => {
                // downcast "tonic::transport::Error" to a "std::io::Error"(kind: Os)
                if let Some(os_err) = find_source::<std::io::Error>(&err) {
                    if os_err.kind() == std::io::ErrorKind::ConnectionRefused {
                        debug!("Connection refused found!");
                        tokio::time::sleep(WAIT_INTERVAL).await;
                        continue;
                    }
                    if os_err.kind() == std::io::ErrorKind::NotFound {
                        debug!("Socket File not found!");
                        tokio::time::sleep(WAIT_INTERVAL).await;
                        continue;
                    }
                }

                // return the error and stop if it is anything other than "Connection Refused"
                return Err(anyhow::anyhow!(err).context(addr));
            }
            Ok(client) => return Ok((client, addr)),
        }
    }
}

/// Start a task to print a message if the connection time is longer than [`WAIT_MESSAGE_TIME`].
///
/// And cancel this task if the handle is dropped. This makes it easy to not print once the TUI is up and
/// we dont need to explicitly abort it in each return case.
fn start_message_timeout() -> AbortOnDropHandle {
    // This is a task so that if somehow connecting is blocking and not doing the loop, we can still print the message

    let handle = tokio::spawn(async {
        tokio::time::sleep(WAIT_MESSAGE_TIME).await;
        eprintln!("Connecting is taking more time than expected...");
    });

    AbortOnDropHandle(handle.abort_handle())
}

/// As the name implies, calls [`AbortHandle::abort`] on [`drop`].
#[derive(Debug)]
struct AbortOnDropHandle(AbortHandle);

impl Drop for AbortOnDropHandle {
    fn drop(&mut self) {
        self.0.abort();
    }
}

/// Find a specific error in the [`Error::source`] chain
fn find_source<E: Error + 'static>(err: &dyn Error) -> Option<&E> {
    let mut err = err.source();
    while let Some(cause) = err {
        if let Some(typed) = cause.downcast_ref() {
            return Some(typed);
        }
        err = cause.source();
    }

    None
}

fn get_config(args: &cli::Args) -> Result<CombinedSettings> {
    let config_server = ServerConfigVersionedDefaulted::from_config_path()?.into_settings();

    let max_depth = args.max_depth.map(ScanDepth::Limited);

    let music_dir = if let Some(ref dir) = args.music_directory {
        Some(get_path(dir).context("resolving cli music-dir")?)
    } else {
        None
    };

    let overlay_server = ServerOverlay {
        settings: config_server,
        music_dir_overwrite: music_dir,
        disable_discord_status: args.disable_discord,
        metadata_scan_depth: max_depth,
    };

    let config_tui = TuiConfigVersionedDefaulted::from_config_path()?.into_settings();

    let coverart_hidden_overwrite = if args.hide_cover { Some(true) } else { None };

    let overlay_tui = TuiOverlay {
        settings: config_tui,
        coverart_hidden_overwrite,
        cover_features: !args.disable_cover,
    };

    Ok(CombinedSettings {
        server: new_shared_server_settings(overlay_server),
        tui: new_shared_tui_settings(overlay_tui),
    })
}

fn get_path(dir: &Path) -> Result<PathBuf> {
    let mut path = dir.to_path_buf();

    if path.exists() {
        if !path.has_root() {
            if let Ok(p_base) = std::env::current_dir() {
                path = p_base.join(path);
            }
        }

        if let Ok(p_canonical) = path.canonicalize() {
            path = p_canonical;
        }

        return Ok(path);
    }

    bail!("Error: non-existing directory '{}'", dir.display());
}

async fn execute_action(action: cli::Action, config: &CombinedSettings) -> Result<()> {
    match action {
        cli::Action::Import { file } => {
            println!("need to import from file {}", file.display());

            let path = get_path(&file).context("import cli file-path")?;
            let config_dir_path =
                utils::get_app_config_path().context("getting app-config-path")?;

            // to not hold a mutexguard across await points
            let config_c = config.server.read().settings.podcast.clone();

            podcast::import_from_opml(&config_dir_path, &config_c, &path)
                .await
                .context("import opml")?;
        }
        cli::Action::Export { file } => {
            println!("need to export to file {}", file.display());
            let path = utils::absolute_path(&file)?;
            let config_dir_path =
                utils::get_app_config_path().context("getting app-config-path")?;
            podcast::export_to_opml(&config_dir_path, &path).context("export opml")?;
        }
    }

    Ok(())
}

/// Determines if the CTRL+C Handler may need to clean-up the terminal mode
static TERMINAL_ALTERNATE_MODE: AtomicBool = AtomicBool::new(false);

/// This might seem useless, but the CTRL+C handler can be invoked when the TUI is not yet or not anymore listening to key events
/// This can happened for example when viuer is in a loop expecting a response or just before or after the TUI is fully started.
///
/// Setup is to press twice within 2 seconds to force a exit.
fn ctrl_c_handler() -> Result<()> {
    // needs to be defined outside, as otherwise a new mutex will get created each time the ctrlc handler is called
    let last_ctrlc = Mutex::new(None);

    ctrlc::set_handler(move || {
        warn!("CTRL+C handler invoked! TUI key-handling not started or overwritten?");

        let mut lock = last_ctrlc.lock();
        let Some(val) = lock.as_ref() else {
            *lock = Some(Instant::now());
            return;
        };

        if val.elapsed() > Duration::from_secs(2) {
            *lock = Some(Instant::now());
            return;
        }

        error!("Exiting because of CTRL+C!");

        // Reset the terminal mode so that the user does not have to use "reset"
        if TERMINAL_ALTERNATE_MODE.load(Ordering::SeqCst) {
            ui::model::Model::hook_reset_terminal();
        }

        std::process::exit(-1);
    })?;

    Ok(())
}
