mod music_player_service;
use anyhow::Result;
use music_player_service::MusicPlayerService;
use std::sync::{Arc, Mutex};
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

    let addr = "[::1]:50051".parse()?;

    let (cmd_tx, cmd_rx) = tokio::sync::mpsc::unbounded_channel();
    let cmd_tx = Arc::new(Mutex::new(cmd_tx));
    let cmd_rx = Arc::new(Mutex::new(cmd_rx));

    let music_player_service: MusicPlayerService = MusicPlayerService::new(Arc::clone(&cmd_tx));
    let mut config = Settings::default();
    config.load()?;
    let mut player = GeneralPlayer::new(&config, Arc::clone(&cmd_tx), Arc::clone(&cmd_rx));
    let progress_tick = music_player_service.progress.clone();

    std::thread::spawn(move || {
        let mut cmd_rx = cmd_rx.lock().expect("lock cmd_rx failed");
        loop {
            if let Ok(cmd) = cmd_rx.try_recv() {
                match cmd {
                    PlayerCmd::AboutToFinish => todo!(),
                    PlayerCmd::CycleLoop => {
                        config.player_loop_mode = player.playlist.cycle_loop_mode();
                    }
                    PlayerCmd::DurationNext(_) => todo!(),
                    PlayerCmd::Eos => {
                        info!("Eos received");
                        if player.playlist.is_empty() {
                            player.stop();
                            continue;
                        }
                        debug!(
                            "current track index: {}",
                            player.playlist.get_current_track_index()
                        );
                        player.playlist.clear_current_track();
                        player.start_play();
                    }
                    PlayerCmd::FetchStatus => todo!(),
                    PlayerCmd::GetProgress => todo!(),
                    PlayerCmd::PlaySelected => todo!(),
                    PlayerCmd::Previous => todo!(),
                    PlayerCmd::ProcessID => todo!(),
                    PlayerCmd::ReloadConfig => todo!(),
                    PlayerCmd::ReloadPlaylist => todo!(),
                    PlayerCmd::SeekBackward => todo!(),
                    PlayerCmd::SeekForward => todo!(),
                    PlayerCmd::Skip => {
                        info!("skip to next track");
                        player.player_save_last_position();
                        player.next();
                    }
                    PlayerCmd::SpeedDown => {
                        player.speed_down();
                        info!("after speed down: {}", player.speed());
                        if let Ok(mut p_tick) = progress_tick.lock() {
                            p_tick.speed = player.speed();
                        }
                    }

                    PlayerCmd::SpeedUp => {
                        player.speed_up();
                        info!("after speed up: {}", player.speed());
                        if let Ok(mut p_tick) = progress_tick.lock() {
                            p_tick.speed = player.speed();
                        }
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
                                "current track index: {}",
                                player.playlist.get_current_track_index()
                            );
                            player.playlist.clear_current_track();
                            player.need_proceed_to_next = false;
                            player.start_play();
                            continue;
                        }
                        if let Ok(mut p_tick) = progress_tick.lock() {
                            if let Ok((position, duration)) = player.get_progress() {
                                let currnet_track_index = player.playlist.get_current_track_index();
                                p_tick.position = position as u32;
                                p_tick.duration = duration as u32;
                                p_tick.current_track_index = currnet_track_index as u32;
                                p_tick.status = player.playlist.status().as_u32();
                                // p_tick.volume = player.volume();
                                // p_tick.speed = player.speed();
                                // p_tick.gapless = player.config.player_gapless;
                            }
                        }
                    }
                    PlayerCmd::ToggleGapless => {
                        config.player_gapless = player.toggle_gapless();
                        if let Ok(mut p_tick) = progress_tick.lock() {
                            p_tick.gapless = config.player_gapless;
                        }
                    }
                    PlayerCmd::TogglePause => {
                        info!("player toggled pause");
                        player.toggle_pause();
                        if let Ok(mut p_tick) = progress_tick.lock() {
                            p_tick.status = player.playlist.status().as_u32();
                        }
                    }
                    PlayerCmd::VolumeDown => {
                        info!("before volumedown: {}", player.volume());
                        player.volume_down();
                        info!("after volumedown: {}", player.volume());
                        if let Ok(mut p_tick) = progress_tick.lock() {
                            p_tick.volume = player.volume();
                        }
                    }
                    PlayerCmd::VolumeUp => {
                        info!("before volumeup: {}", player.volume());
                        player.volume_up();
                        info!("after volumeup: {}", player.volume());
                        if let Ok(mut p_tick) = progress_tick.lock() {
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

    Ok(())
}
