use anyhow::Result;
use std::sync::{Arc, Mutex};
use termusiclib::config::Settings;
use termusicplayback::player::music_player_server::{MusicPlayer, MusicPlayerServer};
use termusicplayback::player::{TogglePauseRequest, TogglePauseResponse};
use termusicplayback::{GeneralPlayer, PlayerCmd};
use tokio::sync::mpsc::UnboundedSender;
use tonic::{transport::Server, Request, Response, Status};

#[macro_use]
extern crate log;

#[derive(Debug)]
pub struct MusicPlayerService {
    cmd_tx: Arc<Mutex<UnboundedSender<PlayerCmd>>>,
}

#[tonic::async_trait]
impl MusicPlayer for MusicPlayerService {
    async fn toggle_pause(
        &self,
        request: Request<TogglePauseRequest>,
    ) -> Result<Response<TogglePauseResponse>, Status> {
        println!("got a request: {:?}", request);
        // let req = request.into_inner();
        let reply = TogglePauseResponse {};
        if let Ok(tx) = self.cmd_tx.lock() {
            tx.send(PlayerCmd::TogglePause).ok();
            info!("PlayerCmd TogglePause sent");
        }
        Ok(Response::new(reply))
    }
}

impl MusicPlayerService {
    pub fn new(cmd_tx: Arc<Mutex<UnboundedSender<PlayerCmd>>>) -> Self {
        Self { cmd_tx }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    lovely_env_logger::init_default();
    info!("background thread start");

    let addr = "[::1]:50051".parse()?;

    let (cmd_tx, cmd_rx) = tokio::sync::mpsc::unbounded_channel();
    let cmd_tx = Arc::new(Mutex::new(cmd_tx));
    let cmd_rx = Arc::new(Mutex::new(cmd_rx));

    let music_player_service: MusicPlayerService = MusicPlayerService::new(cmd_tx.clone());
    let mut config = Settings::default();
    config.load()?;
    let mut player = GeneralPlayer::new(&config, cmd_tx.clone(), cmd_rx.clone());

    player.start_play();

    std::thread::spawn(move || {
        let mut cmd_rx = cmd_rx.lock().expect("lock cmd_rx failed");
        loop {
            if let Ok(cmd) = cmd_rx.try_recv() {
                match cmd {
                    PlayerCmd::AboutToFinish => todo!(),
                    PlayerCmd::CycleLoop => todo!(),
                    PlayerCmd::DurationNext(_) => todo!(),
                    PlayerCmd::Eos => todo!(),
                    PlayerCmd::FetchStatus => todo!(),
                    PlayerCmd::GetProgress => todo!(),
                    PlayerCmd::PlaySelected => todo!(),
                    PlayerCmd::Previous => todo!(),
                    PlayerCmd::ProcessID => todo!(),
                    PlayerCmd::ReloadConfig => todo!(),
                    PlayerCmd::ReloadPlaylist => todo!(),
                    PlayerCmd::SeekBackward => todo!(),
                    PlayerCmd::SeekForward => todo!(),
                    PlayerCmd::Skip => todo!(),
                    PlayerCmd::SpeedDown => todo!(),
                    PlayerCmd::SpeedUp => todo!(),
                    PlayerCmd::Tick => todo!(),
                    PlayerCmd::ToggleGapless => todo!(),
                    PlayerCmd::TogglePause => {
                        player.toggle_pause();
                        info!("player toggled pause");
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
