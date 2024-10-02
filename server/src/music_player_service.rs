use anyhow::Result;
use parking_lot::Mutex;
use std::pin::Pin;
use std::sync::Arc;
use termusiclib::player::music_player_server::MusicPlayer;
use termusiclib::player::{
    CycleLoopReply, CycleLoopRequest, EmptyReply, GetProgressRequest, GetProgressResponse,
    PlaySelectedRequest, PlayerTime, ReloadConfigRequest, ReloadPlaylistRequest,
    SeekBackwardRequest, SeekForwardRequest, SkipNextRequest, SkipNextResponse,
    SkipPreviousRequest, SpeedDownRequest, SpeedReply, SpeedUpRequest, ToggleGaplessReply,
    ToggleGaplessRequest, TogglePauseRequest, TogglePauseResponse, VolumeDownRequest, VolumeReply,
    VolumeUpRequest,
};
use termusicplayback::{PlayerCmd, PlayerCmdSender, StreamTX};
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::{Stream, StreamExt};
use tonic::{Request, Response, Status};

use crate::PlayerStats;

#[derive(Debug)]
pub struct MusicPlayerService {
    cmd_tx: PlayerCmdSender,
    stream_tx: StreamTX,
    pub(crate) player_stats: Arc<Mutex<PlayerStats>>,
}

impl MusicPlayerService {
    pub fn new(cmd_tx: PlayerCmdSender, stream_tx: StreamTX) -> Self {
        let player_stats = Arc::new(Mutex::new(PlayerStats::new()));

        Self {
            cmd_tx,
            player_stats,
            stream_tx,
        }
    }
}

impl MusicPlayerService {
    fn command(&self, cmd: &PlayerCmd) {
        if let Err(e) = self.cmd_tx.send(cmd.clone()) {
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
        let mut r = self.player_stats.lock();
        let reply = r.as_getprogress_response();
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
    ) -> Result<Response<PlayerTime>, Status> {
        self.command(&PlayerCmd::SeekBackward);
        // This is to let the player update volume within loop
        std::thread::sleep(std::time::Duration::from_millis(20));
        let s = self.player_stats.lock();
        let reply = s.as_playertime();

        Ok(Response::new(reply))
    }

    async fn seek_forward(
        &self,
        _request: Request<SeekForwardRequest>,
    ) -> Result<Response<PlayerTime>, Status> {
        self.command(&PlayerCmd::SeekForward);
        // This is to let the player update volume within loop
        std::thread::sleep(std::time::Duration::from_millis(20));
        let s = self.player_stats.lock();

        let reply = s.as_playertime();

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
        let s = self.player_stats.lock();
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
        let s = self.player_stats.lock();
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
        let r = self.player_stats.lock();
        let reply = ToggleGaplessReply { gapless: r.gapless };

        Ok(Response::new(reply))
    }

    async fn toggle_pause(
        &self,
        _request: Request<TogglePauseRequest>,
    ) -> Result<Response<TogglePauseResponse>, Status> {
        self.command(&PlayerCmd::TogglePause);
        std::thread::sleep(std::time::Duration::from_millis(20));
        let r = self.player_stats.lock();
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
        let r = self.player_stats.lock();
        let reply = VolumeReply {
            volume: u32::from(r.volume),
        };

        Ok(Response::new(reply))
    }

    async fn volume_up(
        &self,
        _request: Request<VolumeUpRequest>,
    ) -> Result<Response<VolumeReply>, Status> {
        self.command(&PlayerCmd::VolumeUp);
        // This is to let the player update volume within loop
        std::thread::sleep(std::time::Duration::from_millis(20));
        let r = self.player_stats.lock();
        let reply = VolumeReply {
            volume: u32::from(r.volume),
        };

        Ok(Response::new(reply))
    }

    type SubscribeServerUpdatesStream =
        Pin<Box<dyn Stream<Item = Result<termusiclib::player::StreamUpdates, Status>> + Send>>;
    async fn subscribe_server_updates(
        &self,
        _: Request<EmptyReply>,
    ) -> Result<Response<Self::SubscribeServerUpdatesStream>, Status> {
        let rx = self.stream_tx.subscribe();

        // map to the grpc types
        let receiver_stream = BroadcastStream::new(rx).map(|res| match res {
            Ok(ev) => Ok(ev.into()),
            Err(err) => Err(Status::from_error(Box::new(err))),
        });
        Ok(Response::new(Box::pin(receiver_stream)))
    }
}
