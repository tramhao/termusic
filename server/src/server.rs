mod music_player_service;
use anyhow::Result;
use music_player_service::MusicPlayerService;
use parking_lot::Mutex;
use std::sync::Arc;
use termusiclib::config::Settings;
use termusicplayback::player::music_player_server::MusicPlayerServer;
use termusicplayback::{GeneralPlayer, PlayerCmd, PlayerTrait, Status};
use tonic::transport::Server;

#[macro_use]
extern crate log;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    lovely_env_logger::init_default();
    info!("background thread start");

    let (cmd_tx, cmd_rx) = tokio::sync::mpsc::unbounded_channel();
    let cmd_tx = Arc::new(Mutex::new(cmd_tx));
    let cmd_rx = Arc::new(Mutex::new(cmd_rx));

    let music_player_service: MusicPlayerService = MusicPlayerService::new(cmd_tx.clone());
    let mut config = Settings::default();
    config.load()?;
    let progress_tick = music_player_service.progress.clone();

    let addr = format!("[::1]:{}", config.player_port).parse()?;
    let player_handle = tokio::task::spawn_blocking(move || -> Result<()> {
        let mut player = GeneralPlayer::new(&config, cmd_tx.clone(), cmd_rx.clone());
        loop {
            {
                let mut cmd_rx = cmd_rx.lock();
                if let Ok(cmd) = cmd_rx.try_recv() {
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
                        }
                        PlayerCmd::GetProgress | PlayerCmd::ProcessID => {}
                        PlayerCmd::PlaySelected => {
                            info!("play selected");
                            player.player_save_last_position();
                            player.need_proceed_to_next = false;
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
                            player.need_proceed_to_next = false;
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
                            info!("skip to next track");
                            player.player_save_last_position();
                            player.next();
                        }
                        PlayerCmd::SpeedDown => {
                            player.speed_down();
                            info!("after speed down: {}", player.speed());
                            let mut p_tick = progress_tick.lock();
                            p_tick.speed = player.speed();
                        }

                        PlayerCmd::SpeedUp => {
                            player.speed_up();
                            info!("after speed up: {}", player.speed());
                            let mut p_tick = progress_tick.lock();
                            p_tick.speed = player.speed();
                        }
                        PlayerCmd::Tick => {
                            // info!("tick received");
                            if config.player_use_mpris {
                                player.update_mpris();
                            }
                            if player.playlist.status() == Status::Stopped {
                                if player.playlist.is_empty() {
                                    continue;
                                }
                                debug!(
                                    "current track index: {:?}",
                                    player.playlist.get_current_track_index()
                                );
                                player.playlist.clear_current_track();
                                player.need_proceed_to_next = false;
                                player.start_play();
                                continue;
                            }
                            let mut p_tick = progress_tick.lock();
                            if let Ok((position, duration)) = player.get_progress() {
                                if let Some(currnet_track_index) =
                                    player.playlist.get_current_track_index()
                                {
                                    p_tick.current_track_index = currnet_track_index as u32;
                                }
                                p_tick.position = position as u32;
                                p_tick.duration = duration as u32;
                                p_tick.status = player.playlist.status().as_u32();
                                // p_tick.volume = player.volume();
                                // p_tick.speed = player.speed();
                                // p_tick.gapless = player.config.player_gapless;
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
                            info!("after volumedown: {}", player.volume());
                            let mut p_tick = progress_tick.lock();
                            p_tick.volume = player.volume();
                        }
                        PlayerCmd::VolumeUp => {
                            info!("before volumeup: {}", player.volume());
                            player.volume_up();
                            info!("after volumeup: {}", player.volume());
                            let mut p_tick = progress_tick.lock();
                            p_tick.volume = player.volume();
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
