use termusiclib::player::playlist_helpers::PlaylistAddTrack;

#[allow(clippy::doc_link_with_quotes)]
/// Enum for Commands to send to the [`MusicPlayerClient` "Actor"](crate::ui::music_player_client).
// This is completely different from playback's PlayerCmd, as the tui may need to handle stuff differently and not need all variants
#[derive(Clone, Debug)]
pub enum TuiCmd {
    TogglePause,
    // Play,
    // Pause,
    SeekForward,
    SeekBackward,
    VolumeUp,
    VolumeDown,
    SpeedUp,
    SpeedDown,
    SkipNext,
    SkipPrevious,
    ToggleGapless,
    CycleLoop,

    GetProgress,
    ReloadConfig,
    ReloadPlaylist,
    /// Play the selected track in the playlist (`current_track_index`)
    PlaySelected,

    Playlist(PlaylistCmd),
}

/// Enum for Commands to send specificly for Playlist
#[derive(Clone, Debug)]
pub enum PlaylistCmd {
    AddTrack(PlaylistAddTrack),
}
