#![forbid(unsafe_code)]
#![recursion_limit = "2048"]
#![warn(clippy::all, clippy::correctness)]
#![warn(rust_2018_idioms)]
#![warn(clippy::pedantic)]
/**
 * MIT License
 *
 * termusic - Copyright (c) 2021 Larry Hao
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy
 * of this software and associated documentation files (the "Software"), to deal
 * in the Software without restriction, including without limitation the rights
 * to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 * copies of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in all
 * copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 * AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
 * SOFTWARE.
 */
mod cli;
mod logger;
mod ui;

use anyhow::{bail, Context, Result};
use clap::Parser;
use flexi_logger::LogSpecification;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::time::{Duration, Instant};
use std::{error::Error, path::Path};
use termusiclib::config::v2::server::config_extra::ServerConfigVersionedDefaulted;
use termusiclib::config::v2::server::ScanDepth;
use termusiclib::config::v2::tui::config_extra::TuiConfigVersionedDefaulted;
use termusiclib::config::{
    new_shared_server_settings, new_shared_tui_settings, ServerOverlay, SharedServerSettings,
    SharedTuiSettings, TuiOverlay,
};
use termusiclib::player::music_player_client::MusicPlayerClient;

use sysinfo::{Pid, ProcessStatus, System};
use termusiclib::{podcast, utils};
use ui::UI;

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
        error!("Error: {:?}", err);
        return Err(err);
    }

    Ok(())
}

/// Handles CLI args, potentially starts termusic-server, then runs UI loop
#[tokio::main]
async fn actual_main() -> Result<()> {
    let args = cli::Args::parse();
    let mut logger_handle = logger::setup(&args);
    let config = get_config(&args)?;

    if let Some(action) = args.action {
        return execute_action(action, &config);
    }

    // launch the daemon if it isn't already
    let mut termusic_server_prog = std::path::PathBuf::from("termusic-server");

    let mut system = System::new();
    system.refresh_all();
    let mut launch_daemon = true;
    let mut pid = 0;
    for (id, proc) in system.processes() {
        let Some(exe) = proc.exe().map(|v| v.display().to_string()) else {
            continue;
        };
        if exe.contains("termusic-server") {
            pid = id.as_u32();
            launch_daemon = false;
            break;
        }
    }

    // try to find the server binary adjacent to the currently executing binary path
    let potential_server_exe = {
        let mut exe = std::env::current_exe()?;
        exe.pop();
        exe.join(&termusic_server_prog)
    };
    if potential_server_exe.exists() {
        termusic_server_prog = potential_server_exe;
    }

    if launch_daemon {
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

        let backend = args.backend.to_string();
        server_args.push("--backend");
        server_args.push(&backend);

        let proc = utils::spawn_process(&termusic_server_prog, false, false, &server_args)
            .unwrap_or_else(|_| panic!("Could not find {} binary", termusic_server_prog.display()));

        pid = proc.id();
    }

    println!("Server process ID: {pid}");

    // this is a bad implementation, but there is no way to currently only shut off stderr / stdout
    // see https://github.com/emabee/flexi_logger/issues/142
    if !args.log_options.log_to_file {
        logger_handle.set_new_spec(LogSpecification::off());
    } else if let Err(err) =
        logger_handle.adapt_duplication_to_stderr(flexi_logger::Duplicate::None)
    {
        warn!("flexi_logger error: {}", err);
    }

    info!("Waiting until connected");

    let client = {
        let addr = {
            let config_read = config.tui.read();
            SocketAddr::from(*config_read.settings.get_com().ok_or(anyhow::anyhow!(
                "Expected tui-com settings to be resolved at this point"
            ))?)
        };

        wait_till_connected(addr, pid).await?
    };
    info!("Connected!");

    let mut ui = UI::new(config, client).await?;
    ui.run().await?;

    Ok(())
}

/// Timeout to give up connecting
const WAIT_TIMEOUT: Duration = Duration::from_secs(5);
/// Time to sleep
const WAIT_INTERVAL: Duration = Duration::from_millis(100);

/// Wait until tonic is connected, or:
/// - tonic errors anything other than `ConnectionRefused`
/// - given PID does not exist anymore
/// - timeout of [`WAIT_TIMEOUT`] reached
async fn wait_till_connected(
    socket: SocketAddr,
    pid: u32,
) -> Result<MusicPlayerClient<tonic::transport::Channel>> {
    let mut sys = sysinfo::System::new();
    let sys_pid = Pid::from_u32(pid);
    let start_time = Instant::now();
    loop {
        if Instant::now() > start_time + WAIT_TIMEOUT {
            anyhow::bail!(
                "Could not connect within {} timeout.",
                WAIT_TIMEOUT.as_secs()
            );
        }

        sys.refresh_processes(sysinfo::ProcessesToUpdate::Some(&[sys_pid]));

        let status = sys.process(sys_pid);

        // dont endlessly try to connect, if the server exited / crashed
        if status.is_none() || status.map_or(false, |v| v.status() == ProcessStatus::Zombie) {
            anyhow::bail!("Process {pid} exited before being able to connect!");
        }

        match MusicPlayerClient::connect(format!("http://{socket}")).await {
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
            Ok(client) => return Ok(client),
        }
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
        library_scan_depth: max_depth,
    };

    let config_tui = TuiConfigVersionedDefaulted::from_config_path()?.into_settings();

    let coverart_hidden_overwrite = if args.disable_cover { Some(true) } else { None };

    let overlay_tui = TuiOverlay {
        settings: config_tui,
        coverart_hidden_overwrite,
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

fn execute_action(action: cli::Action, config: &CombinedSettings) -> Result<()> {
    match action {
        cli::Action::Import { file } => {
            println!("need to import from file {}", file.display());

            let path = get_path(&file).context("import cli file-path")?;
            let config_dir_path =
                utils::get_app_config_path().context("getting app-config-path")?;

            podcast::import_from_opml(
                &config_dir_path,
                &config.server.read().settings.podcast,
                &path,
            )
            .context("import opml")?;
        }
        cli::Action::Export { file } => {
            println!("need to export to file {}", file.display());
            let path = utils::absolute_path(&file)?;
            let config_dir_path =
                utils::get_app_config_path().context("getting app-config-path")?;
            podcast::export_to_opml(&config_dir_path, &path).context("export opml")?;
        }
    };

    Ok(())
}
