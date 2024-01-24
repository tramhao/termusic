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
use termusicplayback::{PlayerCmd, PlayerCmdSender};
use tonic::{Request, Response, Status};

#[derive(Debug)]
pub struct MusicPlayerService {
    cmd_tx: PlayerCmdSender,
    pub progress: Arc<Mutex<GetProgressResponse>>,
}

impl MusicPlayerService {
    pub fn new(cmd_tx: PlayerCmdSender) -> Self {
        let progress = GetProgressResponse {
            position: 0,
            duration: 60,
            current_track_index: 0,
            status: 1,
            volume: 50,
            speed: 10,
            gapless: true,
            current_track_updated: false,
            radio_title: String::new(),
        };
        let progress = Arc::new(Mutex::new(progress));

        Self { cmd_tx, progress }
    }
}

impl MusicPlayerService {
    fn command(&self, cmd: &PlayerCmd) {
        if let Err(e) = self.cmd_tx.lock().send(cmd.clone()) {
            error!("error {cmd:?}: {e}");
        }
    }
}

#[tonic::async_trait]
impl MusicPlayer for MusicPlayerService {
    async fn cycle_loop(
        &self,
        _request: Request<CycleLoopRequest>,
    ) -> Result<Response<CycleLoopReply>, Status> {
        let reply = CycleLoopReply {};
        self.command(&PlayerCmd::CycleLoop);

        Ok(Response::new(reply))
    }
    async fn get_progress(
        &self,
        _request: Request<GetProgressRequest>,
    ) -> Result<Response<GetProgressResponse>, Status> {
        let mut r = self.progress.lock();
        let reply = r.clone();
        if r.current_track_updated {
            r.current_track_updated = false;
        }

        Ok(Response::new(reply))
    }

    async fn play_selected(
        &self,
        _request: Request<PlaySelectedRequest>,
    ) -> Result<Response<EmptyReply>, Status> {
        let reply = EmptyReply {};
        self.command(&PlayerCmd::PlaySelected);

        Ok(Response::new(reply))
    }

    async fn reload_config(
        &self,
        _request: Request<ReloadConfigRequest>,
    ) -> Result<Response<EmptyReply>, Status> {
        let reply = EmptyReply {};
        self.command(&PlayerCmd::ReloadConfig);

        Ok(Response::new(reply))
    }

    async fn reload_playlist(
        &self,
        _request: Request<ReloadPlaylistRequest>,
    ) -> Result<Response<EmptyReply>, Status> {
        let reply = EmptyReply {};
        self.command(&PlayerCmd::ReloadPlaylist);

        Ok(Response::new(reply))
    }

    async fn seek_backward(
        &self,
        _request: Request<SeekBackwardRequest>,
    ) -> Result<Response<SeekReply>, Status> {
        self.command(&PlayerCmd::SeekBackward);
        // This is to let the player update volume within loop
        std::thread::sleep(std::time::Duration::from_millis(20));
        let s = self.progress.lock();
        let reply = SeekReply {
            position: s.position,
            duration: s.duration,
        };

        Ok(Response::new(reply))
    }

    async fn seek_forward(
        &self,
        _request: Request<SeekForwardRequest>,
    ) -> Result<Response<SeekReply>, Status> {
        self.command(&PlayerCmd::SeekForward);
        // This is to let the player update volume within loop
        std::thread::sleep(std::time::Duration::from_millis(20));
        let s = self.progress.lock();

        let reply = SeekReply {
            position: s.position,
            duration: s.duration,
        };

        Ok(Response::new(reply))
    }

    async fn skip_next(
        &self,
        _request: Request<SkipNextRequest>,
    ) -> Result<Response<SkipNextResponse>, Status> {
        let reply = SkipNextResponse {};
        self.command(&PlayerCmd::SkipNext);

        Ok(Response::new(reply))
    }
    async fn skip_previous(
        &self,
        _request: Request<SkipPreviousRequest>,
    ) -> Result<Response<EmptyReply>, Status> {
        let reply = EmptyReply {};
        self.command(&PlayerCmd::SkipPrevious);

        Ok(Response::new(reply))
    }

    async fn speed_down(
        &self,
        _request: Request<SpeedDownRequest>,
    ) -> Result<Response<SpeedReply>, Status> {
        self.command(&PlayerCmd::SpeedDown);
        // This is to let the player update volume within loop
        std::thread::sleep(std::time::Duration::from_millis(20));
        let s = self.progress.lock();
        let reply = SpeedReply { speed: s.speed };

        Ok(Response::new(reply))
    }

    async fn speed_up(
        &self,
        _request: Request<SpeedUpRequest>,
    ) -> Result<Response<SpeedReply>, Status> {
        self.command(&PlayerCmd::SpeedUp);
        // This is to let the player update volume within loop
        std::thread::sleep(std::time::Duration::from_millis(20));
        let s = self.progress.lock();
        let reply = SpeedReply { speed: s.speed };

        Ok(Response::new(reply))
    }

    async fn toggle_gapless(
        &self,
        _request: Request<ToggleGaplessRequest>,
    ) -> Result<Response<ToggleGaplessReply>, Status> {
        self.command(&PlayerCmd::ToggleGapless);
        // This is to let the player update volume within loop
        std::thread::sleep(std::time::Duration::from_millis(20));
        let r = self.progress.lock();
        let reply = ToggleGaplessReply { gapless: r.gapless };

        Ok(Response::new(reply))
    }

    async fn toggle_pause(
        &self,
        _request: Request<TogglePauseRequest>,
    ) -> Result<Response<TogglePauseResponse>, Status> {
        self.command(&PlayerCmd::TogglePause);
        std::thread::sleep(std::time::Duration::from_millis(20));
        let r = self.progress.lock();
        let reply = TogglePauseResponse { status: r.status };

        Ok(Response::new(reply))
    }

    async fn volume_down(
        &self,
        _request: Request<VolumeDownRequest>,
    ) -> Result<Response<VolumeReply>, Status> {
        self.command(&PlayerCmd::VolumeDown);
        // This is to let the player update volume within loop
        std::thread::sleep(std::time::Duration::from_millis(20));
        let r = self.progress.lock();
        let reply = VolumeReply { volume: r.volume };

        Ok(Response::new(reply))
    }

    async fn volume_up(
        &self,
        _request: Request<VolumeUpRequest>,
    ) -> Result<Response<VolumeReply>, Status> {
        self.command(&PlayerCmd::VolumeUp);
        // This is to let the player update volume within loop
        std::thread::sleep(std::time::Duration::from_millis(20));
        let r = self.progress.lock();
        let reply = VolumeReply { volume: r.volume };

        Ok(Response::new(reply))
    }
}
