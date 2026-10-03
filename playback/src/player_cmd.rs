use termusiclib::player::{
    ChangeLoopMode, ChangeSpeed, ChangeVolume, SeekReq,
    playlist_helpers::{
        PlaylistAddTrack, PlaylistPlaySpecific, PlaylistRemoveTrackIndexed, PlaylistSwapTrack,
    },
    protobuf::queue::{SortCriterion, SortDirection},
};
use tokio::sync::{
    mpsc::{UnboundedReceiver, UnboundedSender, error::SendError},
    oneshot,
};

use crate::Volume;

pub type PlayerCmdCallback = oneshot::Receiver<()>;
pub type PlayerCmdReciever = UnboundedReceiver<(PlayerCmd, PlayerCmdCallbackSender)>;

/// Wrapper around the potential oneshot sender to implement convenience functions.
#[derive(Debug)]
pub struct PlayerCmdCallbackSender(Option<oneshot::Sender<()>>);

impl PlayerCmdCallbackSender {
    /// Create a new instance
    pub(crate) fn new(tx: Option<oneshot::Sender<()>>) -> Self {
        Self(tx)
    }

    /// Send on the oneshot, if there is any.
    pub fn call(self) {
        let Some(sender) = self.0 else {
            return;
        };
        let _ = sender.send(());
    }
}

/// Wrapper for the actual sender, to make it easier to implement new functions.
#[derive(Debug, Clone)]
pub struct PlayerCmdSender(UnboundedSender<(PlayerCmd, PlayerCmdCallbackSender)>);

impl PlayerCmdSender {
    /// Send a given [`PlayerCmd`] without any callback.
    ///
    /// # Errors
    /// Also see [`oneshot::Sender::send`].
    pub fn send(
        &self,
        cmd: PlayerCmd,
    ) -> Result<(), SendError<(PlayerCmd, PlayerCmdCallbackSender)>> {
        self.0.send((cmd, PlayerCmdCallbackSender::new(None)))
    }

    /// Send a given [`PlayerCmd`] with a callback, returning the receiver.
    ///
    /// # Errors
    /// Also see [`oneshot::Sender::send`].
    pub fn send_cb(
        &self,
        cmd: PlayerCmd,
    ) -> Result<PlayerCmdCallback, SendError<(PlayerCmd, PlayerCmdCallbackSender)>> {
        let (tx, rx) = oneshot::channel();
        self.0.send((cmd, PlayerCmdCallbackSender::new(Some(tx))))?;
        Ok(rx)
    }

    #[must_use]
    pub fn new(tx: UnboundedSender<(PlayerCmd, PlayerCmdCallbackSender)>) -> Self {
        Self(tx)
    }
}

#[derive(Clone, Debug, Copy, PartialEq)]
pub enum PlayerErrorType {
    /// The error happened for the currently playing track.
    Current,
    /// The error happened for the track that was tried to be enqueued.
    Enqueue,
}

#[derive(Clone, Debug)]
pub enum PlayerCmd {
    // Mainly called from the backends
    /// The Backend indicates the current track is about to end.
    AboutToFinish,
    /// The Backend indicates that the current track has ended.
    Eos,
    /// The Backend indicates new metadata is available.
    MetadataChanged,
    /// A Error happened in the backend (for example `NotFound`) that makes it unrecoverable to continue to play the current track.
    /// This will basically be treated as a [`Eos`](PlayerCmd::Eos), with some extra handling.
    ///
    /// This should **not** be used if the whole backend is unrecoverable.
    Error(PlayerErrorType),

    // Internal only
    /// Update information on a interval.
    Tick,

    // Mainly called from outside sources (client, mpris)
    /// Change the loop mode to the one specified.
    ChangeLoopMode(ChangeLoopMode),
    /// Skip to the previous track, if there is one.
    SkipPrevious,
    /// Skip to the next track, if there is one.
    SkipNext,
    /// Pause the playback, if not already (does nothing if `Stopped`)
    Pause,
    /// Resume the playback from `Paused` and `Stopped`.
    Play,
    /// Toggle the playback between `Playing` and `Paused`.
    /// Starts playing if the state was `Stopped`.
    TogglePause,
    /// Quit the server process. Includes the source triggering the quit.
    Quit(&'static str),
    /// Force reload the config.
    ReloadConfig,
    /// Force reload the playlist.
    // TODO: This is not necessary anymore and should be removed.
    ReloadPlaylist,
    /// Seek with the provided parameters.
    Seek(SeekReq),
    /// Change the speed by the provided parameters.
    ChangeSpeed(ChangeSpeed),
    /// Change the volume by the provided parameters.
    ChangeVolume(ChangeVolume),
    /// Change the volume to this specific value.
    // TODO: Combine with [`ChangeVolume`]?
    VolumeSet(Volume),
    /// Toggle the gapless state.
    ToggleGapless,

    /// Play a specific playlist track.
    PlaylistPlaySpecific(PlaylistPlaySpecific),
    /// Add tracks to the playlist.
    PlaylistAddTrack(PlaylistAddTrack),
    /// Remove tracks from the playlist.
    PlaylistRemoveTrack(PlaylistRemoveTrackIndexed),
    /// Clear the playlist.
    PlaylistClear,
    /// Swap tracks in the playlist.
    PlaylistSwapTrack(PlaylistSwapTrack),
    /// Shuffle / Randomize the playlist.
    PlaylistShuffle,
    /// Sort the playlist with the provided parameters.
    PlaylistSort(SortCriterion, SortDirection),
    /// Remove tracks from the playlist which dont exist on disk anymore.
    PlaylistRemoveDeletedTracks,
}
