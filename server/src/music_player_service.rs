use anyhow::Result;
use parking_lot::Mutex;
use std::pin::Pin;
use std::sync::Arc;
use termusiclib::config::SharedServerSettings;
use termusiclib::player::music_player_server::MusicPlayer;
use termusiclib::player::playlist_helpers::{PlaylistPlaySpecific, PlaylistRemoveTrackType};
use termusiclib::player::{
    self, Empty, GaplessState, GetProgressResponse, PlayState, PlayerTime, PlaylistLoopMode,
    PlaylistSwapTracks, PlaylistTracks, PlaylistTracksToAdd, PlaylistTracksToRemove, SpeedReply,
    StreamUpdates, UpdateMissedEvents, VolumeReply, stream_updates,
};
use termusicplayback::{PlayerCmd, PlayerCmdCallback, PlayerCmdSender, SharedPlaylist, StreamTX};
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::wrappers::errors::BroadcastStreamRecvError;
use tokio_stream::{Stream, StreamExt};
use tonic::{Request, Response, Status};

use crate::PlayerStats;

#[derive(Debug)]
pub struct MusicPlayerService {
    cmd_tx: PlayerCmdSender,
    stream_tx: StreamTX,
    config: SharedServerSettings,
    playlist: SharedPlaylist,
    pub(crate) player_stats: Arc<Mutex<PlayerStats>>,
}

impl MusicPlayerService {
    pub fn new(
        cmd_tx: PlayerCmdSender,
        stream_tx: StreamTX,
        config: SharedServerSettings,
        playlist: SharedPlaylist,
    ) -> Self {
        let mut player_stats = PlayerStats::new();
        let config_read = config.read();
        player_stats.volume = config_read.settings.player.volume;
        player_stats.gapless = config_read.settings.player.gapless;
        player_stats.speed = config_read.settings.player.speed;
        drop(config_read);

        let player_stats = Arc::new(Mutex::new(player_stats));

        Self {
            cmd_tx,
            player_stats,
            stream_tx,
            playlist,
            config,
        }
    }
}

impl MusicPlayerService {
    fn command(&self, cmd: PlayerCmd) {
        if let Err(e) = self.cmd_tx.send(cmd.clone()) {
            error!("error {cmd:?}: {e}");
        }
    }

    #[expect(clippy::result_large_err)] // for now we dont care about that here, also see https://github.com/hyperium/tonic/issues/2253
    fn command_cb(&self, cmd: PlayerCmd) -> Result<PlayerCmdCallback, Status> {
        let rx = self.cmd_tx.send_cb(cmd.clone()).map_err(|err| {
            error!("error {cmd:?}: {err}");
            Status::from_error(err.into())
        })?;

        Ok(rx)
    }
}

#[tonic::async_trait]
impl MusicPlayer for MusicPlayerService {
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
    async fn get_progress(
        &self,
        _request: Request<Empty>,
    ) -> Result<Response<GetProgressResponse>, Status> {
        let r = self.player_stats.lock();
        let reply = r.as_getprogress_response(self.playlist.read().status());

        Ok(Response::new(reply))
    }

    async fn play_specific(
        &self,
        request: Request<player::PlaylistPlaySpecific>,
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

    async fn reload_config(&self, _request: Request<Empty>) -> Result<Response<Empty>, Status> {
        let reply = Empty {};
        self.command(PlayerCmd::ReloadConfig);

        Ok(Response::new(reply))
    }

    async fn seek_backward(
        &self,
        _request: Request<Empty>,
    ) -> Result<Response<PlayerTime>, Status> {
        let rx = self.command_cb(PlayerCmd::SeekBackward)?;
        // wait until the event was processed
        let _ = rx.await;
        let s = self.player_stats.lock();
        let reply = s.as_playertime();

        Ok(Response::new(reply))
    }

    async fn seek_forward(&self, _request: Request<Empty>) -> Result<Response<PlayerTime>, Status> {
        let rx = self.command_cb(PlayerCmd::SeekForward)?;
        // wait until the event was processed
        let _ = rx.await;
        let s = self.player_stats.lock();

        let reply = s.as_playertime();

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

    async fn speed_down(&self, _request: Request<Empty>) -> Result<Response<SpeedReply>, Status> {
        let rx = self.command_cb(PlayerCmd::SpeedDown)?;
        // wait until the event was processed
        let _ = rx.await;
        let s = self.player_stats.lock();
        let reply = SpeedReply { speed: s.speed };

        Ok(Response::new(reply))
    }

    async fn speed_up(&self, _request: Request<Empty>) -> Result<Response<SpeedReply>, Status> {
        let rx = self.command_cb(PlayerCmd::SpeedUp)?;
        // wait until the event was processed
        let _ = rx.await;
        let s = self.player_stats.lock();
        let reply = SpeedReply { speed: s.speed };

        Ok(Response::new(reply))
    }

    async fn toggle_gapless(
        &self,
        _request: Request<Empty>,
    ) -> Result<Response<GaplessState>, Status> {
        let rx = self.command_cb(PlayerCmd::ToggleGapless)?;
        // wait until the event was processed
        let _ = rx.await;
        let r = self.player_stats.lock();
        let reply = GaplessState { gapless: r.gapless };

        Ok(Response::new(reply))
    }

    async fn toggle_pause(&self, _request: Request<Empty>) -> Result<Response<PlayState>, Status> {
        let rx = self.command_cb(PlayerCmd::TogglePause)?;
        // wait until the event was processed
        let _ = rx.await;
        let reply = PlayState {
            status: self.playlist.read().status().as_u32(),
        };

        Ok(Response::new(reply))
    }

    async fn volume_down(&self, _request: Request<Empty>) -> Result<Response<VolumeReply>, Status> {
        let rx = self.command_cb(PlayerCmd::VolumeDown)?;
        // wait until the event was processed
        let _ = rx.await;
        let r = self.player_stats.lock();
        let reply = VolumeReply {
            volume: u32::from(r.volume),
        };

        Ok(Response::new(reply))
    }

    async fn volume_up(&self, _request: Request<Empty>) -> Result<Response<VolumeReply>, Status> {
        let rx = self.command_cb(PlayerCmd::VolumeUp)?;
        // wait until the event was processed
        let _ = rx.await;
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
        _: Request<Empty>,
    ) -> Result<Response<Self::SubscribeServerUpdatesStream>, Status> {
        let rx = self.stream_tx.subscribe();

        // map to the grpc types
        let receiver_stream = BroadcastStream::new(rx).map(|res| match res {
            Ok(ev) => Ok(ev.into()),
            Err(err) => {
                let BroadcastStreamRecvError::Lagged(amount) = err;
                Ok(StreamUpdates {
                    r#type: Some(stream_updates::Type::MissedEvents(UpdateMissedEvents {
                        amount,
                    })),
                })

                // else case if ever necessary
                // Err(Status::from_error(Box::new(err)))
            }
        });
        Ok(Response::new(Box::pin(receiver_stream)))
    }

    async fn add_to_playlist(
        &self,
        request: Request<PlaylistTracksToAdd>,
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
        request: Request<PlaylistTracksToRemove>,
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
        request: Request<PlaylistSwapTracks>,
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

    async fn get_playlist(&self, _: Request<Empty>) -> Result<Response<PlaylistTracks>, Status> {
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

    async fn remove_deleted_tracks(&self, _: Request<Empty>) -> Result<Response<Empty>, Status> {
        let rx = self.command_cb(PlayerCmd::PlaylistRemoveDeletedTracks)?;
        // wait until the event was processed
        let _ = rx.await;
        let reply = Empty {};

        Ok(Response::new(reply))
    }
}
