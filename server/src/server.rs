mod cli;
mod logger;
mod music_player_service;

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use anyhow::{bail, Context, Result};
use clap::Parser;
use music_player_service::MusicPlayerService;
use parking_lot::Mutex;
use termusiclib::config::v2::server::config_extra::ServerConfigVersionedDefaulted;
use termusiclib::config::v2::server::ScanDepth;
use termusiclib::config::ServerOverlay;
use termusiclib::track::MediaType;
use termusiclib::{podcast, utils};
use termusicplayback::player::music_player_server::MusicPlayerServer;
use termusicplayback::player::{GetProgressResponse, PlayerTime};
use termusicplayback::{
    Backend, BackendSelect, GeneralPlayer, PlayerCmd, PlayerCmdReciever, PlayerCmdSender,
    PlayerProgress, PlayerTrait, SpeedSigned, Status, VolumeSigned,
};
use tokio::runtime::Handle;
use tokio::sync::oneshot;
use tonic::transport::server::TcpIncoming;
use tonic::transport::Server;

#[macro_use]
extern crate log;

pub const MAX_DEPTH: usize = 4;
pub const VOLUME_STEP: VolumeSigned = 5;
pub const SPEED_STEP: SpeedSigned = 1;

/// Stats for the music player responses
#[derive(Debug, Clone, PartialEq)]
struct PlayerStats {
    pub progress: PlayerProgress,
    pub current_track_index: u32,
    pub status: u32,
    pub volume: u16,
    pub speed: i32,
    pub gapless: bool,
    pub current_track_updated: bool,
    pub radio_title: String,
}

impl PlayerStats {
    pub fn new() -> Self {
        Self {
            progress: PlayerProgress {
                position: None,
                total_duration: None,
            },
            current_track_index: 0,
            status: 1,
            volume: 0,
            speed: 10,
            gapless: true,
            current_track_updated: false,
            radio_title: String::new(),
        }
    }

    pub fn as_getprogress_response(&self) -> GetProgressResponse {
        GetProgressResponse {
            progress: Some(self.as_playertime()),
            current_track_index: self.current_track_index,
            status: self.status,
            volume: u32::from(self.volume),
            speed: self.speed,
            gapless: self.gapless,
            current_track_updated: self.current_track_updated,
            radio_title: self.radio_title.clone(),
        }
    }

    pub fn as_playertime(&self) -> PlayerTime {
        self.progress.into()
    }
}

fn main() -> Result<()> {
    // print error to the log and then throw it
    if let Err(err) = actual_main() {
        error!("Error: {:?}", err);
        return Err(err);
    }

    Ok(())
}

#[tokio::main]
async fn actual_main() -> Result<()> {
    let args = cli::Args::parse();
    let _ = logger::setup(&args);
    let config = get_config(&args)?;

    if let Some(action) = args.action {
        return execute_action(action, &config);
    }

    info!("Server starting...");
    let (cmd_tx, cmd_rx) = tokio::sync::mpsc::unbounded_channel();

    let music_player_service: MusicPlayerService = MusicPlayerService::new(cmd_tx.clone());
    let playerstats = music_player_service.player_stats.clone();

    let cmd_tx_ctrlc = cmd_tx.clone();
    let cmd_tx_ticker = cmd_tx.clone();

    ctrlc::set_handler(move || {
        cmd_tx_ctrlc
            .send(PlayerCmd::Quit)
            .expect("Could not send signal on channel.");
    })
    .expect("Error setting Ctrl-C handler");

    let addr = std::net::SocketAddr::from(config.settings.com);

    // workaround to print address once sever "actually" is started and address is known
    // see https://github.com/hyperium/tonic/issues/351
    let tcp_listener = tokio::net::TcpListener::bind(addr)
        .await
        .with_context(|| format!("Error binding address: {}", addr))?;
    info!("Server listening on {}", tcp_listener.local_addr().unwrap());
    let tcp_stream =
        TcpIncoming::from_listener(tcp_listener, true, None).map_err(|e| anyhow::anyhow!(e))?;

    let tokio_handle = Handle::current();
    let (player_handle_os_tx, player_handle_os_rx) = oneshot::channel();
    let player_handle = std::thread::Builder::new()
        .name("main player loop".into())
        .spawn(move || {
            let _guard = tokio_handle.enter();
            let res = player_loop(args.backend.into(), cmd_tx, cmd_rx, config, playerstats);
            let _ = player_handle_os_tx.send(res);
        })?;

    ticker_thread(cmd_tx_ticker)?;

    tokio::spawn(
        Server::builder()
            .add_service(MusicPlayerServer::new(music_player_service))
            .serve_with_incoming(tcp_stream),
    );

    info!("Server started and listening on {}", addr);

    // await the oneshot completing in a async fashion
    player_handle_os_rx.await??;
    // do this *after* the oneshot, because this is a blocking operation
    // and by doing this after the oneshot we can be sure the thread is actually exited, or exiting
    let _ = player_handle.join();

    Ok(())
}

/// The main player loop where we handle all events
fn player_loop(
    backend: BackendSelect,
    cmd_tx: PlayerCmdSender,
    mut cmd_rx: PlayerCmdReciever,
    config: ServerOverlay,
    playerstats: Arc<Mutex<PlayerStats>>,
) -> Result<()> {
    let mut player = GeneralPlayer::new_backend(backend, config, cmd_tx)?;
    while let Some(cmd) = cmd_rx.blocking_recv() {
        #[allow(unreachable_patterns)]
        match cmd {
            PlayerCmd::AboutToFinish => {
                info!("about to finish signal received");
                if !player.playlist.is_empty()
                    && !player.playlist.has_next_track()
                    && player.config.read().settings.player.gapless
                {
                    player.enqueue_next_from_playlist();
                }
            }
            PlayerCmd::Quit => {
                info!("PlayerCmd::Quit received");
                player.player_save_last_position();
                if let Err(e) = player.playlist.save() {
                    error!("error when saving playlist: {e}");
                };
                if let Err(e) =
                    ServerConfigVersionedDefaulted::save_config_path(&player.config.read().settings)
                {
                    error!("error when saving config: {e}");
                };
                std::process::exit(0);
            }
            PlayerCmd::CycleLoop => {
                player.config.write().settings.player.loop_mode = player.playlist.cycle_loop_mode();
            }
            PlayerCmd::Eos => {
                info!("Eos received");
                if player.playlist.is_empty() {
                    player.stop();
                    continue;
                }
                debug!(
                    "current track index: {:?}",
                    player.playlist.get_current_track_index()
                );
                player.playlist.clear_current_track();
                player.start_play();
                debug!(
                    "playing index is: {}",
                    player.playlist.get_current_track_index()
                );
            }
            PlayerCmd::GetProgress | PlayerCmd::ProcessID => {}
            PlayerCmd::PlaySelected => {
                info!("play selected");
                player.player_save_last_position();
                player.playlist.proceed_false();
                player.next();
            }
            PlayerCmd::SkipPrevious => {
                info!("skip to previous track");
                player.player_save_last_position();
                player.previous();
            }
            PlayerCmd::ReloadConfig => {
                if let Err(err) = player.reload_config() {
                    error!("Reloading config failed, using old: {:#?}", err);
                }
            }
            PlayerCmd::ReloadPlaylist => {
                player.playlist.reload_tracks().ok();
            }
            PlayerCmd::SeekBackward => {
                player.seek_relative(false);
                let mut p_tick = playerstats.lock();
                if let Some(progress) = player.get_progress() {
                    p_tick.progress = progress
                }
            }
            PlayerCmd::SeekForward => {
                player.seek_relative(true);
                let mut p_tick = playerstats.lock();
                if let Some(progress) = player.get_progress() {
                    p_tick.progress = progress
                }
            }
            PlayerCmd::SkipNext => {
                info!("skip to next track.");
                player.player_save_last_position();
                player.next();
            }
            PlayerCmd::SpeedDown => {
                let new_speed = player.add_speed(-SPEED_STEP);
                info!("after speed down: {}", new_speed);
                player.config.write().settings.player.speed = new_speed;
                let mut p_tick = playerstats.lock();
                p_tick.speed = new_speed;
            }

            PlayerCmd::SpeedUp => {
                let new_speed = player.add_speed(SPEED_STEP);
                info!("after speed up: {}", new_speed);
                player.config.write().settings.player.speed = new_speed;
                let mut p_tick = playerstats.lock();
                p_tick.speed = new_speed;
            }
            PlayerCmd::Tick => {
                // info!("tick received");
                player.mpris_handle_events();
                let mut p_tick = playerstats.lock();
                p_tick.status = player.playlist.status().as_u32();
                // branch to auto-start playing if status is "stopped"(not paused) and playlist is not empty anymore
                if player.playlist.status() == Status::Stopped {
                    if player.playlist.is_empty() {
                        continue;
                    }
                    debug!(
                        "current track index: {:?}",
                        player.playlist.get_current_track_index()
                    );
                    player.playlist.clear_current_track();
                    player.playlist.proceed_false();
                    player.start_play();
                    continue;
                }
                if let Some(progress) = player.get_progress() {
                    p_tick.progress = progress;
                    player.mpris_update_progress(&p_tick.progress);
                }
                if player.current_track_updated {
                    p_tick.current_track_index = player.playlist.get_current_track_index() as u32;
                    p_tick.current_track_updated = player.current_track_updated;
                    player.current_track_updated = false;
                }
                if let Some(track) = player.playlist.current_track() {
                    // if only one backend is enabled, rust will complain that it is the only thing that happens
                    #[allow(irrefutable_let_patterns)]
                    if MediaType::LiveRadio == track.media_type {
                        // TODO: consider changing "radio_title" and "media_title" to be consistent
                        p_tick.radio_title = player.media_info().media_title.unwrap_or_default();

                        if let Backend::Rusty(ref mut backend) = player.backend {
                            p_tick.progress.total_duration = Some(Duration::from_secs(
                                ((*backend.radio_downloaded.lock() as f32 * 44100.0
                                    / 1000000.0
                                    / 1024.0)
                                    * (backend.speed() as f32 / 10.0))
                                    as u64,
                            ));
                        }

                        #[cfg(feature = "gst")]
                        if let Backend::GStreamer(_) = player.backend {
                            // p_tick.duration = player.backend.get_buffer_duration();
                            // error!("buffer duration: {}", p_tick.duration);
                            p_tick.progress.total_duration = Some(
                                p_tick.progress.position.unwrap_or_default()
                                    + Duration::from_secs(20),
                            );
                        }
                    }
                }
            }
            PlayerCmd::ToggleGapless => {
                let new_gapless = player.toggle_gapless();
                let mut p_tick = playerstats.lock();
                p_tick.gapless = new_gapless;
            }
            PlayerCmd::TogglePause => {
                info!("player toggled pause");
                player.toggle_pause();
                let mut p_tick = playerstats.lock();
                p_tick.status = player.playlist.status().as_u32();
            }
            PlayerCmd::VolumeDown => {
                info!("before volumedown: {}", player.volume());
                let new_volume = player.add_volume(-VOLUME_STEP);
                player.config.write().settings.player.volume = new_volume;
                info!("after volumedown: {}", new_volume);
                let mut p_tick = playerstats.lock();
                p_tick.volume = new_volume;
                player.mpris_volume_update();
            }
            PlayerCmd::VolumeUp => {
                info!("before volumeup: {}", player.volume());
                let new_volume = player.add_volume(VOLUME_STEP);
                player.config.write().settings.player.volume = new_volume;
                info!("after volumeup: {}", new_volume);
                let mut p_tick = playerstats.lock();
                p_tick.volume = new_volume;
                player.mpris_volume_update();
            }
            PlayerCmd::Pause => {
                player.pause();
            }
            PlayerCmd::Play => {
                player.resume();
            }
        }
    }

    Ok(())
}

/// Spawn the thread that periodically sends [`PlayerCmd::Tick`]
fn ticker_thread(cmd_tx: PlayerCmdSender) -> Result<()> {
    std::thread::Builder::new()
        .name("ticker".into())
        .spawn(move || {
            while cmd_tx.send(PlayerCmd::Tick).is_ok() {
                std::thread::sleep(std::time::Duration::from_millis(500));
            }
        })?;

    Ok(())
}

fn get_config(args: &cli::Args) -> Result<ServerOverlay> {
    let config = ServerConfigVersionedDefaulted::from_config_path()?.into_settings();

    let max_depth = args.max_depth.map(ScanDepth::Limited);

    let music_dir = if let Some(ref dir) = args.music_directory {
        Some(get_path(dir).context("cli provided music-dir")?)
    } else {
        None
    };

    let overlay = ServerOverlay {
        settings: config,
        music_dir_overwrite: music_dir,
        disable_discord_status: args.disable_discord,
        library_scan_depth: max_depth,
    };

    Ok(overlay)
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

fn execute_action(action: cli::Action, config: &ServerOverlay) -> Result<()> {
    match action {
        cli::Action::Import { file } => {
            println!("need to import from file {}", file.display());

            let path = get_path(&file).context("import cli file-path")?;
            let config_dir_path =
                utils::get_app_config_path().context("getting app-config-path")?;

            podcast::import_from_opml(&config_dir_path, &config.settings.podcast, &path)
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
