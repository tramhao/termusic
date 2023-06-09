use anyhow::Result;
use std::sync::{Arc, Mutex};
use termusicplayback::player::music_player_server::MusicPlayer;
use termusicplayback::player::{
    GetProgressRequest, GetProgressResponse, SkipNextRequest, SkipNextResponse, TogglePauseRequest,
    TogglePauseResponse, VolumeDownRequest, VolumeReply, VolumeUpRequest,
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
        _request: Request<GetProgressRequest>,
    ) -> Result<Response<GetProgressResponse>, Status> {
        // println!("got a request: {:?}", request);
        let mut reply = GetProgressResponse {
            position: 25,
            duration: 120,
            current_track_index: 0,
            status: 1,
            volume: 50,
            speed: 10,
            gapless: true,
        };
        if let Ok(r) = self.progress.lock() {
            reply.position = r.position;
            reply.duration = r.duration;
            reply.current_track_index = r.current_track_index;
            reply.status = r.status;
            reply.volume = r.volume;
            reply.speed = r.speed;
            reply.gapless = r.gapless;
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

    async fn volume_up(
        &self,
        request: Request<VolumeUpRequest>,
    ) -> Result<Response<VolumeReply>, Status> {
        println!("got a request: {:?}", request);
        if let Ok(tx) = self.cmd_tx.lock() {
            tx.send(PlayerCmd::VolumeUp).ok();
            info!("PlayerCmd VolumeUp sent");
        }
        // This is to let the player update volume within loop
        std::thread::sleep(std::time::Duration::from_millis(50));
        let mut reply = VolumeReply { volume: 50 };
        if let Ok(r) = self.progress.lock() {
            reply.volume = r.volume;
            info!("volume returned is: {}", r.volume);
        }
        Ok(Response::new(reply))
    }

    async fn volume_down(
        &self,
        request: Request<VolumeDownRequest>,
    ) -> Result<Response<VolumeReply>, Status> {
        println!("got a request: {:?}", request);
        if let Ok(tx) = self.cmd_tx.lock() {
            tx.send(PlayerCmd::VolumeDown).ok();
            info!("PlayerCmd VolumeUp sent");
        }
        // This is to let the player update volume within loop
        std::thread::sleep(std::time::Duration::from_millis(50));
        let mut reply = VolumeReply { volume: 50 };
        if let Ok(r) = self.progress.lock() {
            reply.volume = r.volume;
            info!("volume returned is: {}", r.volume);
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
            volume: 50,
            speed: 10,
            gapless: true,
        };
        let progress = Arc::new(Mutex::new(progress));

        Self { cmd_tx, progress }
    }
}
