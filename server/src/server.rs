mod cli;
mod logger;
mod music_player_service;

use std::path::Path;

use anyhow::{Context, Result};
use clap::Parser;
use music_player_service::MusicPlayerService;
use parking_lot::Mutex;
use std::sync::Arc;
use termusiclib::config::Settings;
use termusiclib::track::MediaType;
use termusicplayback::player::music_player_server::MusicPlayerServer;
use termusicplayback::{Backend, GeneralPlayer, PlayerCmd, PlayerTrait, Status};
use tonic::transport::server::TcpIncoming;
use tonic::transport::Server;

#[macro_use]
extern crate log;

pub const MAX_DEPTH: usize = 4;

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
    info!("background thread start");

    let (cmd_tx, cmd_rx) = tokio::sync::mpsc::unbounded_channel();
    let cmd_tx = Arc::new(Mutex::new(cmd_tx));
    let cmd_rx = Arc::new(Mutex::new(cmd_rx));

    let music_player_service: MusicPlayerService = MusicPlayerService::new(cmd_tx.clone());
    let mut config = get_config(&args)?;
    let progress_tick = music_player_service.progress.clone();

    let cmd_tx_ctrlc = cmd_tx.clone();

    ctrlc::set_handler(move || {
        cmd_tx_ctrlc
            .lock()
            .send(PlayerCmd::Quit)
            .expect("Could not send signal on channel.");
    })
    .expect("Error setting Ctrl-C handler");

    let addr: std::net::SocketAddr = format!("[::]:{}", config.player_port).parse()?;

    // workaround to print address once sever "actually" is started and address is known
    // see https://github.com/hyperium/tonic/issues/351
    let tcp_listener = tokio::net::TcpListener::bind(addr)
        .await
        .with_context(|| format!("Error binding address: {}", addr))?;
    info!("Server listening on {}", tcp_listener.local_addr().unwrap());
    let tcp_stream =
        TcpIncoming::from_listener(tcp_listener, true, None).map_err(|e| anyhow::anyhow!(e))?;

    let player_handle = tokio::task::spawn_blocking(move || -> Result<()> {
        let mut player = GeneralPlayer::new_backend(
            args.backend.into(),
            &config,
            cmd_tx.clone(),
            cmd_rx.clone(),
        )?;
        loop {
            {
                let mut cmd_rx = cmd_rx.lock();
                if let Some(cmd) = cmd_rx.blocking_recv() {
                    #[allow(unreachable_patterns)]
                    match cmd {
                        PlayerCmd::AboutToFinish => {
                            info!("about to finish signal received");
                            if !player.playlist.is_empty()
                                && !player.playlist.has_next_track()
                                && player.config.player_gapless
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
                            if let Err(e) = config.save() {
                                error!("error when saving config: {e}");
                            };
                            std::process::exit(0);
                        }
                        PlayerCmd::CycleLoop => {
                            config.player_loop_mode = player.playlist.cycle_loop_mode();
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
                            config.load()?;
                            info!("config reloaded");
                            player.config = config.clone();
                        }
                        PlayerCmd::ReloadPlaylist => {
                            player.playlist.reload_tracks().ok();
                        }
                        PlayerCmd::SeekBackward => {
                            player.seek_relative(false);
                            let mut p_tick = progress_tick.lock();
                            if let Ok((position, _duration)) = player.get_progress() {
                                p_tick.position = position as u32;
                            }
                        }
                        PlayerCmd::SeekForward => {
                            player.seek_relative(true);
                            let mut p_tick = progress_tick.lock();
                            if let Ok((position, _duration)) = player.get_progress() {
                                p_tick.position = position as u32;
                            }
                        }
                        PlayerCmd::SkipNext => {
                            info!("skip to next track.");
                            player.player_save_last_position();
                            player.next();
                        }
                        PlayerCmd::SpeedDown => {
                            player.speed_down();
                            info!("after speed down: {}", player.speed());
                            config.player_speed = player.speed();
                            let mut p_tick = progress_tick.lock();
                            p_tick.speed = config.player_speed;
                        }

                        PlayerCmd::SpeedUp => {
                            player.speed_up();
                            info!("after speed up: {}", player.speed());
                            config.player_speed = player.speed();
                            let mut p_tick = progress_tick.lock();
                            p_tick.speed = config.player_speed;
                        }
                        PlayerCmd::Tick => {
                            // info!("tick received");
                            if config.player_use_mpris {
                                player.update_mpris();
                            }
                            let mut p_tick = progress_tick.lock();
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
                            if let Ok((position, duration)) = player.get_progress() {
                                p_tick.position = position as u32;
                                p_tick.duration = duration as u32;
                                if player.current_track_updated {
                                    p_tick.current_track_index =
                                        player.playlist.get_current_track_index() as u32;
                                    p_tick.current_track_updated = player.current_track_updated;
                                    player.current_track_updated = false;
                                }
                                if let Some(track) = player.playlist.current_track() {
                                    if let Some(MediaType::LiveRadio) = &track.media_type {
                                        // TODO: consider changing "radio_title" and "media_title" to be consistent
                                        match player.backend {
                                            #[cfg(feature = "mpv")]
                                            Backend::Mpv(ref mut backend) => {
                                                p_tick.radio_title =
                                                    backend.media_title.lock().clone();
                                            }
                                            #[cfg(feature = "rusty")]
                                            Backend::Rusty(ref mut backend) => {
                                                p_tick.radio_title =
                                                    backend.radio_title.lock().clone();
                                                p_tick.duration = ((*backend.radio_downloaded.lock()
                                                    as f32
                                                    * 44100.0
                                                    / 1000000.0
                                                    / 1024.0)
                                                    * (backend.speed() as f32 / 10.0))
                                                    as u32;
                                            }
                                            #[cfg(feature = "gst")]
                                            Backend::GStreamer(ref mut backend) => {
                                                // p_tick.duration = player.backend.get_buffer_duration();
                                                // eprintln!("buffer duration: {}", p_tick.duration);
                                                p_tick.duration = position as u32 + 20;
                                                p_tick.radio_title =
                                                    backend.radio_title.lock().clone();
                                                // eprintln!("radio title: {}", p_tick.radio_title);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        PlayerCmd::ToggleGapless => {
                            config.player_gapless = player.toggle_gapless();
                            let mut p_tick = progress_tick.lock();
                            p_tick.gapless = config.player_gapless;
                        }
                        PlayerCmd::TogglePause => {
                            info!("player toggled pause");
                            player.toggle_pause();
                            let mut p_tick = progress_tick.lock();
                            p_tick.status = player.playlist.status().as_u32();
                        }
                        PlayerCmd::VolumeDown => {
                            info!("before volumedown: {}", player.volume());
                            player.volume_down();
                            config.player_volume = player.volume();
                            info!("after volumedown: {}", player.volume());
                            let mut p_tick = progress_tick.lock();
                            p_tick.volume = config.player_volume;
                        }
                        PlayerCmd::VolumeUp => {
                            info!("before volumeup: {}", player.volume());
                            player.volume_up();
                            config.player_volume = player.volume();
                            info!("after volumeup: {}", player.volume());
                            let mut p_tick = progress_tick.lock();
                            p_tick.volume = config.player_volume;
                        } // _ => {}
                        PlayerCmd::Pause => {
                            player.pause();
                        }
                        PlayerCmd::Play => {
                            player.resume();
                        }
                    }
                }
            }
        }
    });

    Server::builder()
        .add_service(MusicPlayerServer::new(music_player_service))
        .serve_with_incoming(tcp_stream)
        .await?;

    let _drop = player_handle.await?;

    Ok(())
}

fn get_config(args: &cli::Args) -> Result<Settings> {
    let mut config = Settings::default();
    config.load()?;

    config.disable_album_art_from_cli = args.disable_cover;
    config.disable_discord_rpc_from_cli = args.disable_discord;

    if let Some(dir) = &args.music_directory {
        config.music_dir_from_cli = get_path(dir);
    }

    config.max_depth_cli = match args.max_depth {
        Some(d) => d,
        None => MAX_DEPTH,
    };

    Ok(config)
}

fn get_path(dir: &str) -> Option<String> {
    let music_dir: Option<String>;
    let mut path = Path::new(&dir).to_path_buf();

    if path.exists() {
        if !path.has_root() {
            if let Ok(p_base) = std::env::current_dir() {
                path = p_base.join(path);
            }
        }

        if let Ok(p_canonical) = path.canonicalize() {
            path = p_canonical;
        }

        music_dir = Some(path.to_string_lossy().to_string());
    } else {
        eprintln!("Error: unknown directory '{dir}'");
        std::process::exit(0);
    }
    music_dir
}
