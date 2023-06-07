use anyhow::Result;
use termusiclib::config::Settings;
use termusicplayback::player::music_player_server::{MusicPlayer, MusicPlayerServer};
use termusicplayback::player::{TogglePauseRequest, TogglePauseResponse};
use termusicplayback::GeneralPlayer;
use tonic::{transport::Server, Request, Response, Status};

#[macro_use]
extern crate log;

#[derive(Debug, Default)]
pub struct MusicPlayerService {}

#[tonic::async_trait]
impl MusicPlayer for MusicPlayerService {
    async fn toggle_pause(
        &self,
        request: Request<TogglePauseRequest>,
    ) -> Result<Response<TogglePauseResponse>, Status> {
        println!("got a request: {:?}", request);
        // let req = request.into_inner();
        let reply = TogglePauseResponse {};
        Ok(Response::new(reply))
    }
}

impl MusicPlayerService {
    pub fn new() -> Result<Self> {
        Ok(Self {})
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    lovely_env_logger::init_default();
    info!("background thread start");

    let addr = "[::1]:50051".parse()?;
    let music_player_service: MusicPlayerService = MusicPlayerService::default();

    let mut config = Settings::default();
    config.load()?;
    let mut player = GeneralPlayer::new(&config);

    player.start_play();

    Server::builder()
        .add_service(MusicPlayerServer::new(music_player_service))
        .serve(addr)
        .await?;

    Ok(())
}
