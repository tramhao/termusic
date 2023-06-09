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

    std::thread::spawn(move || {
        let mut cmd_rx = cmd_rx.lock().expect("lock cmd_rx failed");
        loop {
            if let Ok(cmd) = cmd_rx.try_recv() {
                match cmd {
                    PlayerCmd::AboutToFinish => todo!(),
                    PlayerCmd::CycleLoop => todo!(),
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
                    PlayerCmd::SpeedDown => todo!(),
                    PlayerCmd::SpeedUp => todo!(),
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
                        }
                    }
                    PlayerCmd::ToggleGapless => todo!(),
                    PlayerCmd::TogglePause => {
                        info!("player toggled pause");
                        player.toggle_pause();
                    }
                    PlayerCmd::VolumeDown => todo!(),
                    PlayerCmd::VolumeUp => todo!(),
                }
            }
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
    });

    Server::builder()
        .add_service(MusicPlayerServer::new(music_player_service))
        .serve(addr)
        .await?;

    Ok(())
}
