use anyhow::Result;
use std::sync::{Arc, Mutex};
use termusicplayback::player::music_player_server::MusicPlayer;
use termusicplayback::player::{
    GetProgressRequest, GetProgressResponse, SkipNextRequest, SkipNextResponse, TogglePauseRequest,
    TogglePauseResponse,
};
use termusicplayback::PlayerCmd;
use tokio::sync::mpsc::UnboundedSender;
use tonic::{Request, Response, Status};

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

    async fn skip_next(
        &self,
        request: Request<SkipNextRequest>,
    ) -> Result<Response<SkipNextResponse>, Status> {
        println!("got a request: {:?}", request);
        let reply = SkipNextResponse {};
        if let Ok(tx) = self.cmd_tx.lock() {
            tx.send(PlayerCmd::Skip).ok();
            info!("PlayerCmd Skip sent");
        }
        Ok(Response::new(reply))
    }

    async fn get_progress(
        &self,
        request: Request<GetProgressRequest>,
    ) -> Result<Response<GetProgressResponse>, Status> {
        println!("got a request: {:?}", request);
        let reply = GetProgressResponse {
            position: 25,
            duration: 100,
            current_track_index: 0,
        };
        // if let Ok(tx) = self.cmd_tx.lock() {
        //     tx.send(PlayerCmd::Skip).ok();
        //     info!("PlayerCmd Skip sent");
        // }
        Ok(Response::new(reply))
    }
}

impl MusicPlayerService {
    pub fn new(cmd_tx: Arc<Mutex<UnboundedSender<PlayerCmd>>>) -> Self {
        Self { cmd_tx }
    }
}
