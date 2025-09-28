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
    ConfigSavePopup,

    Header,
    Footer,

    General(IdCEGeneral),
    Theme(IdCETheme),
    KeyGlobal(IdKeyGlobal),
    KeyOther(IdKeyOther),
}

#[derive(Debug, Eq, PartialEq, Clone, Copy, Hash)]
pub enum IdCETheme {
    ThemeSelectTable,

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

    PlaylistBackground,
    PlaylistBorder,
    PlaylistForeground,
    PlaylistHighlight,
    PlaylistHighlightSymbol,
    PlaylistLabel,

    CurrentlyPlayingTrackSymbol,

    ProgressBackground,
    ProgressBorder,
    ProgressForeground,
    ProgressLabel,

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

impl From<IdCETheme> for IdConfigEditor {
    fn from(value: IdCETheme) -> Self {
        IdConfigEditor::Theme(value)
    }
}

impl From<&IdCETheme> for IdConfigEditor {
    fn from(value: &IdCETheme) -> Self {
        IdConfigEditor::Theme(*value)
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Copy, Hash)]
pub enum IdCEGeneral {
    MusicDir,
    ExitConfirmation,
    AlbumPhotoAlign,
    ExtraYtdlpArgs,
    SaveLastPosition,
    SeekStep,

    PlayerPort,
    PlayerAddress,
    PlayerProtocol,
    PlayerUDSPath,
    PlayerUseDiscord,
    PlayerUseMpris,

    PodcastDir,
    PodcastMaxRetries,
    PodcastSimulDownload,

    PlaylistRandomAlbum,
    PlaylistRandomTrack,
    PlaylistDisplaySymbol,

    KillDamon,
}

impl From<IdCEGeneral> for IdConfigEditor {
    fn from(value: IdCEGeneral) -> Self {
        IdConfigEditor::General(value)
    }
}

impl From<&IdCEGeneral> for IdConfigEditor {
    fn from(value: &IdCEGeneral) -> Self {
        IdConfigEditor::General(*value)
    }
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

#[derive(Debug, Eq, PartialEq, Clone, Copy, Hash)]
pub enum IdKey {
    Global(IdKeyGlobal),
    Other(IdKeyOther),
}

impl From<&IdKey> for IdConfigEditor {
    fn from(value: &IdKey) -> Self {
        match *value {
            IdKey::Global(id_key_global) => IdConfigEditor::KeyGlobal(id_key_global),
            IdKey::Other(id_key_other) => IdConfigEditor::KeyOther(id_key_other),
        }
    }
}

impl From<IdKey> for IdConfigEditor {
    fn from(value: IdKey) -> Self {
        match value {
            IdKey::Global(id_key_global) => IdConfigEditor::KeyGlobal(id_key_global),
            IdKey::Other(id_key_other) => IdConfigEditor::KeyOther(id_key_other),
        }
    }
}
