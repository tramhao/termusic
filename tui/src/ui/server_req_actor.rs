use anyhow::Result;
use termusiclib::player::playlist_helpers::PlaylistRemoveTrackType;
use tokio::sync::mpsc::UnboundedReceiver;

use crate::ui::{
    model::TxToMain,
    msg::{Msg, ServerReqResponse},
    music_player_client::Playback,
    tui_cmd::{PlaylistCmd, TuiCmd},
};

/// Actor that handles all requests to the Server via GRPC.
///
/// This actor can be given commands via [`TuiCmd`] and responds on [`TxToMain`].
pub struct ServerRequestActor {
    client_handle: Playback,
    rx_cmd: UnboundedReceiver<TuiCmd>,
    tx_main: TxToMain,
}

impl ServerRequestActor {
    /// Create and start a new [`ServerRequestActor`].
    ///
    /// To shutdown this actor, close `rx_cmd` channel.
    pub fn start_actor(
        client_handle: Playback,
        rx_cmd: UnboundedReceiver<TuiCmd>,
        tx_main: TxToMain,
    ) {
        let obj = Self {
            client_handle,
            rx_cmd,
            tx_main,
        };

        // clippy suggests manually dropping instead of "let _ =" on a future, which "JoinHandle" is
        let jh = tokio::spawn(Self::run(obj));
        drop(jh);
    }

    /// The actor loop.
    async fn run(mut actor: Self) {
        while let Some(cmd) = actor.rx_cmd.recv().await {
            if let Err(err) = actor.handle_cmd(cmd).await {
                error!("Error processing command to server: {err:#?}");
            }
        }
    }

    /// Handle all commands to the server and their responses.
    async fn handle_cmd(&mut self, cmd: TuiCmd) -> Result<()> {
        match cmd {
            TuiCmd::TogglePause => {
                // result will be populated back via UpdateStream
                let _ = self.client_handle.toggle_pause().await?;
            }
            TuiCmd::SeekForward => {
                // result will be populated back via UpdateStream
                let _ = self.client_handle.seek_forward().await?;
            }
            TuiCmd::SeekBackward => {
                // result will be populated back via UpdateStream
                let _ = self.client_handle.seek_backward().await?;
            }
            TuiCmd::VolumeUp => {
                // result will be populated back via UpdateStream
                let _ = self.client_handle.volume_up().await?;
            }
            TuiCmd::VolumeDown => {
                // result will be populated back via UpdateStream
                let _ = self.client_handle.volume_down().await?;
            }
            TuiCmd::SpeedUp => {
                // result will be populated back via UpdateStream
                let _ = self.client_handle.speed_up().await?;
            }
            TuiCmd::SpeedDown => {
                // result will be populated back via UpdateStream
                let _ = self.client_handle.speed_down().await?;
            }
            TuiCmd::SkipNext => {
                // result will be populated back via UpdateStream
                self.client_handle.skip_next().await?;
            }
            TuiCmd::SkipPrevious => {
                // result will be populated back via UpdateStream
                self.client_handle.skip_previous().await?;
            }
            TuiCmd::ToggleGapless => {
                // result will be populated back via UpdateStream
                let _ = self.client_handle.toggle_gapless().await?;
            }
            TuiCmd::CycleLoop => {
                // result will be populated back via UpdateStream
                let _ = self.client_handle.cycle_loop().await?;
            }
            TuiCmd::GetProgress => {
                let res = self.client_handle.get_progress().await?;

                self.send_response(Msg::ServerReqResponse(ServerReqResponse::GetProgress(res)));
            }
            TuiCmd::ReloadConfig => {
                self.client_handle.reload_config().await?;
            }
            TuiCmd::Playlist(playlist_cmd) => self.handle_playlist_cmd(playlist_cmd).await?,
        }

        Ok(())
    }

    /// Handle Playlist requests.
    async fn handle_playlist_cmd(&mut self, cmd: PlaylistCmd) -> Result<()> {
        match cmd {
            PlaylistCmd::PlaySpecific(playlist_play_specific) => {
                // result will be populated back via UpdateStream
                self.client_handle
                    .play_specific(playlist_play_specific)
                    .await?;
            }
            PlaylistCmd::AddTrack(playlist_add_track) => {
                // result will be populated back via UpdateStream
                self.client_handle
                    .add_to_playlist(playlist_add_track)
                    .await?;
            }
            PlaylistCmd::RemoveTrack(playlist_remove_track_indexed) => {
                // result will be populated back via UpdateStream
                self.client_handle
                    .remove_from_playlist(PlaylistRemoveTrackType::Indexed(
                        playlist_remove_track_indexed,
                    ))
                    .await?;
            }
            PlaylistCmd::Clear => {
                // result will be populated back via UpdateStream
                self.client_handle
                    .remove_from_playlist(PlaylistRemoveTrackType::Clear)
                    .await?;
            }
            PlaylistCmd::SwapTrack(playlist_swap_track) => {
                // result will be populated back via UpdateStream
                self.client_handle.swap_tracks(playlist_swap_track).await?;
            }
            PlaylistCmd::Shuffle => {
                // result will be populated back via UpdateStream
                self.client_handle.shuffle_playlist().await?;
            }
            PlaylistCmd::RemoveDeletedItems => {
                // result will be populated back via UpdateStream
                self.client_handle.remove_deleted_tracks().await?;
            }
            PlaylistCmd::SelfReloadPlaylist => {
                let tracks = self.client_handle.get_playlist().await?;

                self.send_response(Msg::ServerReqResponse(ServerReqResponse::FullPlaylist(
                    tracks,
                )));
            }
        }

        Ok(())
    }

    #[inline]
    fn send_response(&self, msg: Msg) {
        let _ = self.tx_main.send(msg);
    }
}
