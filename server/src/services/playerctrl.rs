use std::sync::Arc;

use anyhow::Result;
use parking_lot::Mutex;
use termusiclib::config::SharedServerSettings;
use termusiclib::player::ChangeRunningState;
use termusiclib::player::protobuf::common::Empty;
use termusiclib::player::protobuf::player::player_control_server::PlayerControl;
use termusiclib::player::protobuf::player::{
    ChangeRunningStateRequest, ChangeSpeedRequest, ChangeVolumeRequest, GaplessState,
    GetProgressResponse, PlayState, SeekRequest, SpeedReply, VolumeReply,
};
use termusicplayback::{PlayerCmd, PlayerCmdCallback, PlayerCmdSender, SharedRunInfo};
use tonic::{Request, Response, Status};

use crate::PlayerStats;

#[derive(Debug)]
pub struct PlayerControlService {
    cmd_tx: PlayerCmdSender,
    config: SharedServerSettings,
    run_info: SharedRunInfo,
    pub(crate) player_stats: Arc<Mutex<PlayerStats>>,
}

impl PlayerControlService {
    pub fn new(
        cmd_tx: PlayerCmdSender,
        config: SharedServerSettings,
        run_info: SharedRunInfo,
    ) -> Self {
        let player_stats = PlayerStats::new();

        let player_stats = Arc::new(Mutex::new(player_stats));

        Self {
            cmd_tx,
            player_stats,
            config,
            run_info,
        }
    }
}

impl PlayerControlService {
    /// Send a command without a callback.
    fn command(&self, cmd: PlayerCmd) {
        if let Err(e) = self.cmd_tx.send(cmd.clone()) {
            error!("error {cmd:?}: {e}");
        }
    }

    /// Send a command with a callback that can be waited for.
    fn command_cb(&self, cmd: PlayerCmd) -> Result<PlayerCmdCallback, Status> {
        let rx = self.cmd_tx.send_cb(cmd.clone()).map_err(|err| {
            error!("error {cmd:?}: {err}");
            Status::from_error(err.into())
        })?;

        Ok(rx)
    }
}

#[tonic::async_trait]
impl PlayerControl for PlayerControlService {
    async fn get_progress(
        &self,
        _request: Request<Empty>,
    ) -> Result<Response<GetProgressResponse>, Status> {
        let stats = self.player_stats.lock();
        let reply =
            stats.as_getprogress_response(self.run_info.read().status(), &self.config.read());

        Ok(Response::new(reply))
    }

    async fn seek(&self, request: Request<SeekRequest>) -> Result<Response<Empty>, Status> {
        let ev = request
            .into_inner()
            .try_into()
            .map_err(|err: anyhow::Error| {
                error!("error {err}");
                Status::from_error(err.into())
            })?;
        let rx = self.command_cb(PlayerCmd::Seek(ev))?;
        // wait until the event was processed
        let _ = rx.await;
        let reply = Empty {};

        Ok(Response::new(reply))
    }

    async fn skip_next(&self, _request: Request<Empty>) -> Result<Response<Empty>, Status> {
        let reply = Empty {};
        self.command(PlayerCmd::SkipNext);

        Ok(Response::new(reply))
    }
    async fn skip_previous(&self, _request: Request<Empty>) -> Result<Response<Empty>, Status> {
        let reply = Empty {};
        self.command(PlayerCmd::SkipPrevious);

        Ok(Response::new(reply))
    }

    async fn change_speed(
        &self,
        request: Request<ChangeSpeedRequest>,
    ) -> Result<Response<SpeedReply>, Status> {
        let ev = request
            .into_inner()
            .try_into()
            .map_err(|err: anyhow::Error| {
                error!("error {err}");
                Status::from_error(err.into())
            })?;
        let rx = self.command_cb(PlayerCmd::ChangeSpeed(ev))?;
        // wait until the event was processed
        let _ = rx.await;
        let reply = SpeedReply {
            speed: self.config.read().settings.player.speed,
        };

        Ok(Response::new(reply))
    }

    async fn toggle_gapless(
        &self,
        _request: Request<Empty>,
    ) -> Result<Response<GaplessState>, Status> {
        let rx = self.command_cb(PlayerCmd::ToggleGapless)?;
        // wait until the event was processed
        let _ = rx.await;
        let reply = GaplessState {
            gapless: self.config.read().settings.player.gapless,
        };

        Ok(Response::new(reply))
    }

    async fn change_running_state(
        &self,
        request: Request<ChangeRunningStateRequest>,
    ) -> Result<Response<PlayState>, Status> {
        let ev: ChangeRunningState =
            request
                .into_inner()
                .try_into()
                .map_err(|err: anyhow::Error| {
                    error!("error {err}");
                    Status::from_error(err.into())
                })?;
        let ev = match ev {
            ChangeRunningState::Toggle => PlayerCmd::TogglePause,
            ChangeRunningState::Pause => PlayerCmd::Pause,
            ChangeRunningState::Resume => PlayerCmd::Play,
        };
        let rx = self.command_cb(ev)?;
        // wait until the event was processed
        let _ = rx.await;
        let reply = PlayState {
            status: self.run_info.read().status().as_u32(),
        };

        Ok(Response::new(reply))
    }

    async fn change_volume(
        &self,
        request: Request<ChangeVolumeRequest>,
    ) -> Result<Response<VolumeReply>, Status> {
        let ev = request
            .into_inner()
            .try_into()
            .map_err(|err: anyhow::Error| {
                error!("error {err}");
                Status::from_error(err.into())
            })?;
        let rx = self.command_cb(PlayerCmd::ChangeVolume(ev))?;
        // wait until the event was processed
        let _ = rx.await;
        let reply = VolumeReply {
            volume: u32::from(self.config.read().settings.player.volume),
        };

        Ok(Response::new(reply))
    }
}
