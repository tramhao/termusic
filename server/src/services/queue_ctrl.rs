use anyhow::Result;
use termusiclib::config::SharedServerSettings;
use termusiclib::player::playlist_helpers::{PlaylistPlaySpecific, PlaylistRemoveTrackType};
use termusiclib::player::protobuf::common::Empty;
use termusiclib::player::protobuf::queue::queue_control_server::QueueControl;
use termusiclib::player::protobuf::queue::{
    PlaylistLoopMode, PlaylistPlaySpecificRequest, PlaylistState, PlaylistSwapTracksRequest,
    PlaylistTracksToAddRequest, PlaylistTracksToRemoveRequest, SortCriterion, SortDirection,
    SortPlaylistRequest,
};
use termusicplayback::{PlayerCmd, PlayerCmdCallback, PlayerCmdSender, SharedPlaylist};
use tonic::{Request, Response, Status};

#[derive(Debug)]
pub struct QueueControlService {
    cmd_tx: PlayerCmdSender,
    config: SharedServerSettings,
    playlist: SharedPlaylist,
}

impl QueueControlService {
    pub fn new(
        cmd_tx: PlayerCmdSender,
        config: SharedServerSettings,
        playlist: SharedPlaylist,
    ) -> Self {
        Self {
            cmd_tx,
            playlist,
            config,
        }
    }
}

impl QueueControlService {
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
impl QueueControl for QueueControlService {
    async fn cycle_loop(
        &self,
        _request: Request<Empty>,
    ) -> Result<Response<PlaylistLoopMode>, Status> {
        let rx = self.command_cb(PlayerCmd::CycleLoop)?;
        // wait until the event was processed
        let _ = rx.await;
        let config = self.config.read();

        let reply = PlaylistLoopMode {
            mode: u32::from(config.settings.player.loop_mode.discriminant()),
        };

        Ok(Response::new(reply))
    }

    async fn play_specific(
        &self,
        request: Request<PlaylistPlaySpecificRequest>,
    ) -> Result<Response<Empty>, Status> {
        let converted: PlaylistPlaySpecific = request
            .into_inner()
            .try_into()
            .map_err(|err: anyhow::Error| Status::from_error(err.into()))?;
        let rx = self.command_cb(PlayerCmd::PlaylistPlaySpecific(converted))?;

        // wait until the event was processed
        let _ = rx.await;

        let reply = Empty {};

        Ok(Response::new(reply))
    }

    async fn add_to_playlist(
        &self,
        request: Request<PlaylistTracksToAddRequest>,
    ) -> Result<Response<Empty>, Status> {
        let converted = request
            .into_inner()
            .try_into()
            .map_err(|err: anyhow::Error| Status::from_error(err.into()))?;
        let rx = self.command_cb(PlayerCmd::PlaylistAddTrack(converted))?;
        // wait until the event was processed
        let _ = rx.await;
        let reply = Empty {};

        Ok(Response::new(reply))
    }

    async fn remove_from_playlist(
        &self,
        request: Request<PlaylistTracksToRemoveRequest>,
    ) -> Result<Response<Empty>, Status> {
        let converted: PlaylistRemoveTrackType = request
            .into_inner()
            .try_into()
            .map_err(|err: anyhow::Error| Status::from_error(err.into()))?;

        let ev = match converted {
            PlaylistRemoveTrackType::Indexed(v) => PlayerCmd::PlaylistRemoveTrack(v),
            PlaylistRemoveTrackType::Clear => PlayerCmd::PlaylistClear,
        };

        let rx = self.command_cb(ev)?;
        // wait until the event was processed
        let _ = rx.await;
        let reply = Empty {};

        Ok(Response::new(reply))
    }

    async fn swap_tracks(
        &self,
        request: Request<PlaylistSwapTracksRequest>,
    ) -> Result<Response<Empty>, Status> {
        let converted = request
            .into_inner()
            .try_into()
            .map_err(|err: anyhow::Error| Status::from_error(err.into()))?;

        let rx = self.command_cb(PlayerCmd::PlaylistSwapTrack(converted))?;
        // wait until the event was processed
        let _ = rx.await;
        let reply = Empty {};

        Ok(Response::new(reply))
    }

    async fn get_playlist(&self, _: Request<Empty>) -> Result<Response<PlaylistState>, Status> {
        let playlist = self.playlist.read();
        let reply = playlist.as_grpc_playlist_tracks().unwrap();

        Ok(Response::new(reply))
    }

    async fn shuffle_playlist(&self, _: Request<Empty>) -> Result<Response<Empty>, Status> {
        // execute shuffle in the player thread instead of the service thread
        // this does not necessarily need to be done, but its better to have the service read-only
        let rx = self.command_cb(PlayerCmd::PlaylistShuffle)?;
        // wait until the event was processed
        let _ = rx.await;

        let reply = Empty {};

        Ok(Response::new(reply))
    }

    async fn sort_playlist(
        &self,
        request: Request<SortPlaylistRequest>,
    ) -> Result<Response<Empty>, Status> {
        let req = request.into_inner();
        let criterion = SortCriterion::try_from(req.criterion)
            .map_err(|e| Status::invalid_argument(format!("unknown sort criterion: {e}")))?;
        let direction = SortDirection::try_from(req.direction)
            .map_err(|e| Status::invalid_argument(format!("unknown sort direction: {e}")))?;
        info!("Sort playlist: {criterion:?} {direction:?}");
        let rx = self.command_cb(PlayerCmd::PlaylistSort(criterion, direction))?;
        let _ = rx.await;

        let reply = Empty {};

        Ok(Response::new(reply))
    }

    async fn remove_deleted_tracks(&self, _: Request<Empty>) -> Result<Response<Empty>, Status> {
        let rx = self.command_cb(PlayerCmd::PlaylistRemoveDeletedTracks)?;
        // wait until the event was processed
        let _ = rx.await;
        let reply = Empty {};

        Ok(Response::new(reply))
    }
}
