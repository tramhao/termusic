//! Module containing all TUI Component Identifiers

#[derive(Debug, Eq, PartialEq, Clone, Copy, Hash)]
pub enum Id {
    ConfigEditor(IdConfigEditor),
    DBListCriteria,
    DBListSearchResult,
    DBListSearchTracks,
    DeleteConfirmRadioPopup,
    DeleteConfirmInputPopup,
    DownloadSpinner,
    Episode,
    ErrorPopup,
    GeneralSearchInput,
    GeneralSearchTable,
    GlobalListener,
    HelpPopup,
    Label,
    Library,
    Lyric,
    MessagePopup,
    Playlist,
    Podcast,
    PodcastAddPopup,
    PodcastSearchTablePopup,
    FeedDeleteConfirmRadioPopup,
    FeedDeleteConfirmInputPopup,
    Progress,
    QuitPopup,
    SavePlaylistPopup,
    SavePlaylistLabel,
    SavePlaylistConfirm,
    TagEditor(IdTagEditor),
    YoutubeSearchInputPopup,
    YoutubeSearchTablePopup,
    DatabaseAddConfirmPopup,
}

#[derive(Debug, Eq, PartialEq, Clone, Copy, Hash)]
pub enum IdTagEditor {
    CounterDelete,
    LabelHint,
    InputArtist,
    InputTitle,
    InputAlbum,
    InputGenre,
    SelectLyric,
    TableLyricOptions,
    TextareaLyric,
}

#[derive(Debug, Eq, PartialEq, Clone, Copy, Hash)]
pub enum IdConfigEditor {
    AlbumPhotoAlign,
    CEThemeSelect,
    ConfigSavePopup,
    ExitConfirmation,
    ExtraYtdlpArgs,
    Footer,
    Header,
    KeyGlobal(IdKeyGlobal),
    KeyOther(IdKeyOther),
    KillDamon,

    LibraryBackground,
    LibraryBorder,
    LibraryForeground,
    LibraryHighlight,
    LibraryHighlightSymbol,
    LibraryLabel,

    LyricBackground,
    LyricBorder,
    LyricForeground,
    LyricLabel,

    MusicDir,
    PlayerPort,
    PlayerUseDiscord,
    PlayerUseMpris,

    PlaylistBackground,
    PlaylistBorder,
    PlaylistDisplaySymbol,
    PlaylistForeground,
    PlaylistHighlight,
    PlaylistHighlightSymbol,
    PlaylistLabel,
    PlaylistRandomAlbum,
    PlaylistRandomTrack,

    CurrentlyPlayingTrackSymbol,

    PodcastDir,
    PodcastMaxRetries,
    PodcastSimulDownload,

    ProgressBackground,
    ProgressBorder,
    ProgressForeground,
    ProgressLabel,

    SaveLastPosition,
    SeekStep,

    ImportantPopupLabel,
    ImportantPopupBackground,
    ImportantPopupBorder,
    ImportantPopupForeground,

    FallbackBackground,
    FallbackBorder,
    FallbackForeground,
    FallbackHighlight,
    FallbackLabel,
}

#[derive(Debug, Eq, PartialEq, Clone, Copy, Hash)]
pub enum IdKeyGlobal {
    LayoutTreeview,
    LayoutDatabase,
    LayoutPodcast,

    Quit,
    Config,
    Help,
    SavePlaylist,

    Up,
    Down,
    Left,
    Right,
    GotoBottom,
    GotoTop,

    PlayerToggleGapless,
    PlayerTogglePause,
    PlayerNext,
    PlayerPrevious,
    PlayerSeekForward,
    PlayerSeekBackward,
    PlayerSpeedUp,
    PlayerSpeedDown,
    PlayerVolumeUp,
    PlayerVolumeDown,

    LyricAdjustForward,
    LyricAdjustBackward,
    LyricCycle,

    XywhMoveUp,
    XywhMoveDown,
    XywhMoveLeft,
    XywhMoveRight,
    XywhZoomIn,
    XywhZoomOut,
    XywhHide,
}

#[derive(Debug, Eq, PartialEq, Clone, Copy, Hash)]
pub enum IdKeyOther {
    LibraryAddRoot,
    LibraryRemoveRoot,
    LibrarySwitchRoot,
    LibraryDelete,
    LibraryLoadDir,
    LibraryYank,
    LibraryPaste,
    LibrarySearch,
    LibrarySearchYoutube,
    LibraryTagEditor,

    PlaylistShuffle,
    PlaylistModeCycle,
    PlaylistPlaySelected,
    PlaylistSearch,
    PlaylistSwapUp,
    PlaylistSwapDown,
    PlaylistDelete,
    PlaylistDeleteAll,
    PlaylistAddRandomAlbum,
    PlaylistAddRandomTracks,

    DatabaseAddAll,
    DatabaseAddSelected,

    PodcastSearchAddFeed,
    PodcastMarkPlayed,
    PodcastMarkAllPlayed,
    PodcastEpDownload,
    PodcastEpDeleteFile,
    PodcastDeleteFeed,
    PodcastDeleteAllFeeds,
    PodcastRefreshFeed,
    PodcastRefreshAllFeeds,
}
