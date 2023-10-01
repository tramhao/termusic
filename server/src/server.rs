mod music_player_service;
use anyhow::Result;
use music_player_service::MusicPlayerService;
use termusiclib::config::Settings;
use termusiclib::track::MediaType;
use termusiclib::types::player::music_player_server::MusicPlayerServer;
use termusicplayback::{GeneralPlayer, PlayerCmd, PlayerTrait, Status};
use tonic::transport::Server;

#[macro_use]
extern crate log;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    lovely_env_logger::init_default();
    info!("background thread start");

    let (cmd_tx, mut cmd_rx) = tokio::sync::mpsc::unbounded_channel();

    let music_player_service: MusicPlayerService = MusicPlayerService::new(cmd_tx.clone());
    let mut config = Settings::default();
    config.load()?;
    let progress_tick = music_player_service.progress.clone();

    let cmd_tx_ctrlc = cmd_tx.clone();

    ctrlc::set_handler(move || {
        cmd_tx_ctrlc
            .send(PlayerCmd::Quit)
            .expect("Could not send signal on channel.");
    })
    .expect("Error setting Ctrl-C handler");

    let addr = format!("[::]:{}", config.player_port).parse()?;
    let player_handle = tokio::task::spawn_blocking(move || -> Result<()> {
        let mut player = GeneralPlayer::new(&config, cmd_tx.clone());
        loop {
            {
                if let Ok(cmd) = cmd_rx.try_recv() {
                    #[allow(unreachable_patterns)]
                    match cmd {
                        PlayerCmd::AboutToFinish => {
                            info!("about to finish signal received");
                            if !player.playlist.is_empty()
                                && !player.playlist.has_next_track()
                                && player.config.player_gapless
                            {
                                player.enqueue_next();
                            }
                        }
                        PlayerCmd::Quit => {
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
                        #[cfg(not(any(feature = "mpv", feature = "gst")))]
                        PlayerCmd::DurationNext(duration) => {
                            player
                                .playlist
                                .set_next_track_duration(std::time::Duration::from_secs(duration));
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
                                        #[cfg(not(any(feature = "mpv", feature = "gst")))]
                                        {
                                            p_tick.radio_title =
                                                player.backend.radio_title.lock().clone();
                                            p_tick.duration =
                                                ((*player.backend.radio_downloaded.lock() as f32
                                                    * 44100.0
                                                    / 1000000.0
                                                    / 1024.0)
                                                    * (player.speed() as f32 / 10.0))
                                                    as u32;
                                        }
                                        #[cfg(feature = "mpv")]
                                        {
                                            p_tick.radio_title =
                                                player.backend.media_title.lock().clone();
                                        }
                                        #[cfg(all(feature = "gst", not(feature = "mpv")))]
                                        {
                                            // p_tick.duration = player.backend.get_buffer_duration();
                                            // eprintln!("buffer duration: {}", p_tick.duration);
                                            p_tick.duration = position as u32 + 20;
                                            p_tick.radio_title =
                                                player.backend.radio_title.lock().clone();
                                            // eprintln!("radio title: {}", p_tick.radio_title);
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
                        PlayerCmd::SubscribeToUpdates(sender) => {
                            player.subscribers.push(sender);
                        }
                    }
                }
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    });

    Server::builder()
        .add_service(MusicPlayerServer::new(music_player_service))
        .serve(addr)
        .await?;

    let _drop = player_handle.await?;

    Ok(())
}
