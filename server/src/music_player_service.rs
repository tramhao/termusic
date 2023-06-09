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
    pub progress: Arc<Mutex<GetProgressResponse>>,
}

#[tonic::async_trait]
impl MusicPlayer for MusicPlayerService {
    async fn get_progress(
        &self,
        request: Request<GetProgressRequest>,
    ) -> Result<Response<GetProgressResponse>, Status> {
        println!("got a request: {:?}", request);
        let mut reply = GetProgressResponse {
            position: 25,
            duration: 120,
            current_track_index: 0,
            status: 1,
        };
        if let Ok(r) = self.progress.lock() {
            reply.position = r.position;
            reply.duration = r.duration;
            reply.current_track_index = r.current_track_index;
            reply.status = r.status;
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
        let progress = GetProgressResponse {
            position: 0,
            duration: 60,
            current_track_index: 0,
            status: 1,
        };
        let progress = Arc::new(Mutex::new(progress));

        Self { cmd_tx, progress }
    }
}
