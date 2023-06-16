use anyhow::Result;
use parking_lot::Mutex;
use std::sync::Arc;
use termusicplayback::player::music_player_server::MusicPlayer;
use termusicplayback::player::{
    CycleLoopReply, CycleLoopRequest, EmptyReply, GetProgressRequest, GetProgressResponse,
    PlaySelectedRequest, ReloadConfigRequest, ReloadPlaylistRequest, SeekBackwardRequest,
    SeekForwardRequest, SeekReply, SkipNextRequest, SkipNextResponse, SkipPreviousRequest,
    SpeedDownRequest, SpeedReply, SpeedUpRequest, ToggleGaplessReply, ToggleGaplessRequest,
    TogglePauseRequest, TogglePauseResponse, VolumeDownRequest, VolumeReply, VolumeUpRequest,
};
use termusicplayback::PlayerCmd;
use tokio::sync::mpsc::UnboundedSender;
use tonic::{Request, Response, Status};

#[derive(Debug)]
pub struct MusicPlayerService {
    cmd_tx: Arc<Mutex<UnboundedSender<PlayerCmd>>>,
    pub progress: Arc<Mutex<GetProgressResponse>>,
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

#[tonic::async_trait]
impl MusicPlayer for MusicPlayerService {
    async fn cycle_loop(
        &self,
        _request: Request<CycleLoopRequest>,
    ) -> Result<Response<CycleLoopReply>, Status> {
        let reply = CycleLoopReply {};
        if let Err(e) = self.cmd_tx.lock().send(PlayerCmd::CycleLoop) {
            error!("PlayerCmd Skip error: {e}");
        }
        Ok(Response::new(reply))
    }
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
        let r = self.progress.lock();
        reply.position = r.position;
        reply.duration = r.duration;
        reply.current_track_index = r.current_track_index;
        reply.status = r.status;
        reply.volume = r.volume;
        reply.speed = r.speed;
        reply.gapless = r.gapless;

        Ok(Response::new(reply))
    }

    async fn skip_next(
        &self,
        request: Request<SkipNextRequest>,
    ) -> Result<Response<SkipNextResponse>, Status> {
        println!("got a request: {:?}", request);
        let reply = SkipNextResponse {};
        if let Err(e) = self.cmd_tx.lock().send(PlayerCmd::SkipNext) {
            error!("PlayerCmd Skip error: {e}");
        }
        Ok(Response::new(reply))
    }

    async fn speed_down(
        &self,
        _request: Request<SpeedDownRequest>,
    ) -> Result<Response<SpeedReply>, Status> {
        if let Err(e) = self.cmd_tx.lock().send(PlayerCmd::SpeedDown) {
            error!("PlayerCmd SpeedDown error: {e}");
        }
        // This is to let the player update volume within loop
        std::thread::sleep(std::time::Duration::from_millis(20));
        let mut reply = SpeedReply { speed: 10 };
        let s = self.progress.lock();
        reply.speed = s.speed;
        Ok(Response::new(reply))
    }

    async fn speed_up(
        &self,
        _request: Request<SpeedUpRequest>,
    ) -> Result<Response<SpeedReply>, Status> {
        if let Err(e) = self.cmd_tx.lock().send(PlayerCmd::SpeedUp) {
            error!("PlayerCmd SpeedUp error: {e}");
        }
        // This is to let the player update volume within loop
        std::thread::sleep(std::time::Duration::from_millis(20));
        let mut reply = SpeedReply { speed: 10 };
        let s = self.progress.lock();
        reply.speed = s.speed;

        Ok(Response::new(reply))
    }

    async fn toggle_pause(
        &self,
        request: Request<TogglePauseRequest>,
    ) -> Result<Response<TogglePauseResponse>, Status> {
        println!("got a request: {:?}", request);
        // let req = request.into_inner();
        if let Err(e) = self.cmd_tx.lock().send(PlayerCmd::TogglePause) {
            error!("PlayerCmd TogglePause error: {e}");
        }

        std::thread::sleep(std::time::Duration::from_millis(20));
        let mut reply = TogglePauseResponse { status: 1 };
        let r = self.progress.lock();
        reply.status = r.status;
        info!("status returned is: {}", r.status);

        Ok(Response::new(reply))
    }

    async fn volume_down(
        &self,
        request: Request<VolumeDownRequest>,
    ) -> Result<Response<VolumeReply>, Status> {
        println!("got a request: {:?}", request);
        if let Err(e) = self.cmd_tx.lock().send(PlayerCmd::VolumeDown) {
            error!("PlayerCmd VolumeDown error: {e}");
        }
        // This is to let the player update volume within loop
        std::thread::sleep(std::time::Duration::from_millis(20));
        let mut reply = VolumeReply { volume: 50 };
        let r = self.progress.lock();
        reply.volume = r.volume;
        info!("volume returned is: {}", r.volume);

        Ok(Response::new(reply))
    }

    async fn volume_up(
        &self,
        request: Request<VolumeUpRequest>,
    ) -> Result<Response<VolumeReply>, Status> {
        println!("got a request: {:?}", request);
        if let Err(e) = self.cmd_tx.lock().send(PlayerCmd::VolumeUp) {
            error!("PlayerCmd VolumeUp error: {e}");
        }
        // This is to let the player update volume within loop
        std::thread::sleep(std::time::Duration::from_millis(20));
        let mut reply = VolumeReply { volume: 50 };
        let r = self.progress.lock();
        reply.volume = r.volume;
        info!("volume returned is: {}", r.volume);

        Ok(Response::new(reply))
    }
    async fn toggle_gapless(
        &self,
        request: Request<ToggleGaplessRequest>,
    ) -> Result<Response<ToggleGaplessReply>, Status> {
        println!("got a request: {:?}", request);
        if let Err(e) = self.cmd_tx.lock().send(PlayerCmd::ToggleGapless) {
            error!("PlayerCmd Toggle Gapless error: {e}");
        }
        // This is to let the player update volume within loop
        std::thread::sleep(std::time::Duration::from_millis(20));
        let mut reply = ToggleGaplessReply { gapless: true };
        let r = self.progress.lock();
        reply.gapless = r.gapless;
        info!("gapless returned is: {}", r.gapless);

        Ok(Response::new(reply))
    }

    async fn seek_forward(
        &self,
        _request: Request<SeekForwardRequest>,
    ) -> Result<Response<SeekReply>, Status> {
        if let Err(e) = self.cmd_tx.lock().send(PlayerCmd::SeekForward) {
            error!("PlayerCmd SeekForward error: {e}");
        }
        // This is to let the player update volume within loop
        std::thread::sleep(std::time::Duration::from_millis(20));
        let mut reply = SeekReply {
            position: 0,
            duration: 60,
        };
        let s = self.progress.lock();
        reply.position = s.position;
        reply.duration = s.duration;

        Ok(Response::new(reply))
    }

    async fn seek_backward(
        &self,
        _request: Request<SeekBackwardRequest>,
    ) -> Result<Response<SeekReply>, Status> {
        if let Err(e) = self.cmd_tx.lock().send(PlayerCmd::SeekBackward) {
            error!("PlayerCmd SeekBackward error: {e}");
        }
        // This is to let the player update volume within loop
        std::thread::sleep(std::time::Duration::from_millis(20));
        let mut reply = SeekReply {
            position: 0,
            duration: 60,
        };
        let s = self.progress.lock();
        reply.position = s.position;
        reply.duration = s.duration;

        Ok(Response::new(reply))
    }

    async fn reload_config(
        &self,
        _request: Request<ReloadConfigRequest>,
    ) -> Result<Response<EmptyReply>, Status> {
        let reply = EmptyReply {};
        if let Err(e) = self.cmd_tx.lock().send(PlayerCmd::ReloadConfig) {
            error!("PlayerCmd ReloadConfig error: {e}");
        }
        Ok(Response::new(reply))
    }

    async fn reload_playlist(
        &self,
        _request: Request<ReloadPlaylistRequest>,
    ) -> Result<Response<EmptyReply>, Status> {
        let reply = EmptyReply {};
        if let Err(e) = self.cmd_tx.lock().send(PlayerCmd::ReloadPlaylist) {
            error!("PlayerCmd ReloadPlaylist error: {e}");
        }
        Ok(Response::new(reply))
    }

    async fn play_selected(
        &self,
        _request: Request<PlaySelectedRequest>,
    ) -> Result<Response<EmptyReply>, Status> {
        let reply = EmptyReply {};
        if let Err(e) = self.cmd_tx.lock().send(PlayerCmd::PlaySelected) {
            error!("PlayerCmd PlaySelected error: {e}");
        }
        Ok(Response::new(reply))
    }

    async fn skip_previous(
        &self,
        _request: Request<SkipPreviousRequest>,
    ) -> Result<Response<EmptyReply>, Status> {
        let reply = EmptyReply {};
        if let Err(e) = self.cmd_tx.lock().send(PlayerCmd::SkipPrevious) {
            error!("PlayerCmd Previous error: {e}");
        }
        Ok(Response::new(reply))
    }
}
