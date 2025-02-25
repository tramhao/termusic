use anyhow::Result;
use parking_lot::Mutex;
use std::pin::Pin;
use std::sync::Arc;
use termusiclib::config::SharedServerSettings;
use termusiclib::player::music_player_server::MusicPlayer;
use termusiclib::player::playlist_helpers::{
    PlaylistPlaySpecific, PlaylistRemoveTrackType, PlaylistTrackSource,
};
use termusiclib::player::{
    self, stream_updates, Empty, GaplessState, GetProgressResponse, PlayState, PlayerTime,
    PlaylistAddTrack, PlaylistLoopMode, PlaylistSwapTracks, PlaylistTracks, PlaylistTracksToAdd,
    PlaylistTracksToRemove, SpeedReply, StreamUpdates, UpdateMissedEvents, VolumeReply,
};
use termusicplayback::{
    PlayerCmd, PlayerCmdCallback, PlayerCmdSender, Playlist, SharedPlaylist, StreamTX,
};
use tokio_stream::wrappers::errors::BroadcastStreamRecvError;
use tokio_stream::wrappers::BroadcastStream;
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
        let player_stats = Arc::new(Mutex::new(PlayerStats::new()));

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
        let mut r = self.player_stats.lock();
        let reply = r.as_getprogress_response();
        if r.current_track_updated {
            r.current_track_updated = false;
        }

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
        let r = self.player_stats.lock();
        let reply = PlayState { status: r.status };

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
        let reply = playlist_to_grpc_tracks(&playlist);

        Ok(Response::new(reply))
    }

    async fn shuffle_playlist(
        &self,
        _: Request<Empty>,
    ) -> Result<Response<PlaylistTracks>, Status> {
        // execute shuffle in the player thread instead of the service thread
        // this does not necessarily need to be done, but its better to have the service read-only
        let rx = self.command_cb(PlayerCmd::PlaylistShuffle)?;
        // wait until the event was processed
        let _ = rx.await;

        let playlist = self.playlist.read();
        let reply = playlist_to_grpc_tracks(&playlist);

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

/// Common function to map [`Playlist`] tracks to the GRPC message types
fn playlist_to_grpc_tracks(playlist: &Playlist) -> PlaylistTracks {
    let tracks = playlist
        .tracks()
        .iter()
        .enumerate()
        .filter_map(|(idx, track)| {
            let at_index = u64::try_from(idx).unwrap();
            // TODO: refactor Track::file to be always existing
            let Some(file) = track.file() else {
                error!("Track did not have a file(id), skipping!");
                return None;
            };
            // TODO: this should likely be a function on "Track"
            let id = match track.media_type {
                termusiclib::track::MediaType::Music => PlaylistTrackSource::Path(file.to_string()),
                termusiclib::track::MediaType::Podcast => {
                    PlaylistTrackSource::PodcastUrl(file.to_string())
                }
                termusiclib::track::MediaType::LiveRadio => {
                    PlaylistTrackSource::Url(file.to_string())
                }
            };
            Some(PlaylistAddTrack {
                at_index,
                duration: Some(track.duration().into()),
                id: Some(id.into()),
                optional_title: None,
            })
        })
        .collect();

    PlaylistTracks {
        current_track_index: u64::try_from(playlist.get_current_track_index()).unwrap(),
        tracks,
    }
}
