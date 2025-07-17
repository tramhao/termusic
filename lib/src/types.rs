use std::path::PathBuf;
use std::sync::Arc;

use crate::config::v2::tui::{keys::KeyBinding, theme::styles::ColorTermusic};
use crate::ids::{IdConfigEditor, IdKey};
use crate::invidious::{Instance, YoutubeVideo};
use crate::library_db::SearchCriteria;
use crate::podcast::{EpData, PodcastFeed, PodcastNoId};
use crate::songtag::SongTag;
use anyhow::{Result, anyhow};
use image::DynamicImage;
use crate::dlna::models::DlnaDevice;

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Msg {
    // AppClose,
    ConfigEditor(ConfigEditorMsg),
    DataBase(DBMsg),
    DeleteConfirmCloseCancel,
    DeleteConfirmCloseOk,
    DeleteConfirmShow,
    DlnaServer(DSMsg),
    Download(DLMsg),
    ErrorPopupClose,
    GeneralSearch(GSMsg),
    HelpPopupShow,
    HelpPopupClose,
    Layout(MainLayoutMsg),
    Library(LIMsg),
    LyricMessage(LyricMsg),
    LyricCycle,
    LyricAdjustDelay(i64),
    Player(PlayerMsg),
    Playlist(PLMsg),
    Podcast(PCMsg),
    /// Closes the Quit Popup, if it was shown without quitting.
    QuitPopupCloseCancel,
    /// Always will directly quit.
    QuitPopupCloseOk,
    /// Either shows the Quit Dialog if enabled, or if dialog is disabled, directly quits
    QuitPopupShow,
    SavePlaylist(SavePlaylistMsg),
    TagEditor(TEMsg),
    UpdatePhoto,
    YoutubeSearch(YSMsg),
    Xywh(XYWHMsg),

    /// Force a redraw because of some change.
    ///
    /// This is necessary as `Components` do not have access to `Model.redraw`.
    ///
    /// For example pushing ARROW DOWN to change the selection in a table.
    ForceRedraw,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MainLayoutMsg {
    /// Switch to the Music library view
    TreeView,
    /// Switch to the Database view
    DataBase,
    /// Switch to the Podcast view
    Podcast,
    /// Switch to the DLNA server view
    DlnaServer,
}

/// Player relates messages
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerMsg {
    ToggleGapless,
    TogglePause,
    VolumeUp,
    VolumeDown,
    SpeedUp,
    SpeedDown,
    SeekForward,
    SeekBackward,
}

/// Save Playlist Popup related messages
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SavePlaylistMsg {
    PopupShow,
    PopupCloseCancel,
    PopupUpdate(String),
    PopupCloseOk(String),
    ConfirmCloseCancel,
    ConfirmCloseOk(String),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum XYWHMsg {
    /// Toggle the hidden / shown status of the displayed image.
    ToggleHidden,
    MoveLeft,
    MoveRight,
    MoveUp,
    MoveDown,
    ZoomIn,
    ZoomOut,
}

pub type DLMsgURL = Arc<str>;

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum DLMsg {
    /// Indicates a Start of a download.
    ///
    /// `(Url, Title)`
    DownloadRunning(DLMsgURL, String),
    /// Indicates the Download was a Success, though termusic post-processing is not done yet.
    ///
    /// `(Url)`
    DownloadSuccess(DLMsgURL),
    /// Indicates the Download thread finished in both Success or Error.
    ///
    /// `(Url, Filename)`
    DownloadCompleted(DLMsgURL, Option<String>),
    /// Indicates that the Download has Errored and has been aborted.
    ///
    /// `(Url, Title, ErrorAsString)`
    DownloadErrDownload(DLMsgURL, String, String),
    /// Indicates that the Download was a Success, but termusic post-processing failed.
    /// Like re-saving tags after editing.
    ///
    /// `(Url, Title)`
    DownloadErrEmbedData(DLMsgURL, String),

    // TODO: The Following 2 things have absolutely nothing to-do with Download
    /// Show a status message in the TUI.
    ///
    /// `((Title, Text))`
    MessageShow((String, String)),
    /// Hide a status message in the TUI.
    ///
    /// `((Title, Text))`
    MessageHide((String, String)),

    // TODO: The Following 2 things have absolutely nothing to-do with Download
    /// Fetching & loading the image was a success, with the image.
    FetchPhotoSuccess(ImageWrapper),
    /// Fetching & loading the image has failed, with error message.
    /// `(ErrorAsString)`
    FetchPhotoErr(String),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LyricMsg {
    LyricTextAreaBlurUp,
    LyricTextAreaBlurDown,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum ConfigEditorMsg {
    PodcastDirBlurDown,
    PodcastDirBlurUp,
    PodcastSimulDownloadBlurDown,
    PodcastSimulDownloadBlurUp,
    PodcastMaxRetriesBlurDown,
    PodcastMaxRetriesBlurUp,

    AlbumPhotoAlignBlurDown,
    AlbumPhotoAlignBlurUp,
    ChangeLayout,
    CloseCancel,
    CloseOk,
    ColorChanged(IdConfigEditor, ColorTermusic),
    SymbolChanged(IdConfigEditor, String),
    ConfigChanged,
    ConfigSaveOk,
    ConfigSaveCancel,
    ExitConfirmationBlurDown,
    ExitConfirmationBlurUp,
    ExtraYtdlpArgsBlurDown,
    ExtraYtdlpArgsBlurUp,
    Open,
    KeyFocus(KFMsg),
    MusicDirBlurDown,
    MusicDirBlurUp,

    PlaylistDisplaySymbolBlurDown,
    PlaylistDisplaySymbolBlurUp,
    PlaylistRandomTrackBlurDown,
    PlaylistRandomTrackBlurUp,
    PlaylistRandomAlbumBlurDown,
    PlaylistRandomAlbumBlurUp,

    LibraryForegroundBlurDown,
    LibraryForegroundBlurUp,
    LibraryBackgroundBlurDown,
    LibraryBackgroundBlurUp,
    LibraryBorderBlurDown,
    LibraryBorderBlurUp,
    LibraryHighlightBlurDown,
    LibraryHighlightBlurUp,
    LibraryHighlightSymbolBlurDown,
    LibraryHighlightSymbolBlurUp,

    PlaylistForegroundBlurDown,
    PlaylistForegroundBlurUp,
    PlaylistBackgroundBlurDown,
    PlaylistBackgroundBlurUp,
    PlaylistBorderBlurDown,
    PlaylistBorderBlurUp,
    PlaylistHighlightBlurDown,
    PlaylistHighlightBlurUp,
    PlaylistHighlightSymbolBlurDown,
    PlaylistHighlightSymbolBlurUp,

    ProgressForegroundBlurDown,
    ProgressForegroundBlurUp,
    ProgressBackgroundBlurDown,
    ProgressBackgroundBlurUp,
    ProgressBorderBlurDown,
    ProgressBorderBlurUp,

    LyricForegroundBlurDown,
    LyricForegroundBlurUp,
    LyricBackgroundBlurDown,
    LyricBackgroundBlurUp,
    LyricBorderBlurDown,
    LyricBorderBlurUp,

    ThemeSelectBlurDown,
    ThemeSelectBlurUp,
    ThemeSelectLoad(usize),

    KeyChange(IdKey, KeyBinding),
    SaveLastPositionBlurDown,
    SaveLastPosotionBlurUp,
    SeekStepBlurDown,
    SeekStepBlurUp,
    KillDaemonBlurDown,
    KillDaemonBlurUp,

    PlayerUseMprisBlurDown,
    PlayerUseMprisBlurUp,
    PlayerUseDiscordBlurDown,
    PlayerUseDiscordBlurUp,
    PlayerPortBlurDown,
    PlayerPortBlurUp,

    CurrentlyPlayingTrackSymbolBlurDown,
    CurrentlyPlayingTrackSymbolBlurUp,

    ImportantPopupForegroundBlurDown,
    ImportantPopupForegroundBlurUp,
    ImportantPopupBackgroundBlurDown,
    ImportantPopupBackgroundBlurUp,
    ImportantPopupBorderBlurDown,
    ImportantPopupBorderBlurUp,

    FallbackForegroundBlurDown,
    FallbackForegroundBlurUp,
    FallbackBackgroundBlurDown,
    FallbackBackgroundBlurUp,
    FallbackBorderBlurDown,
    FallbackBorderBlurUp,
    FallbackHighlightBlurDown,
    FallbackHighlightBlurUp,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KFMsg {
    DatabaseAddAllBlurDown,
    DatabaseAddAllBlurUp,
    DatabaseAddSelectedBlurDown,
    DatabaseAddSelectedBlurUp,
    GlobalConfigBlurDown,
    GlobalConfigBlurUp,
    GlobalDownBlurDown,
    GlobalDownBlurUp,
    GlobalGotoBottomBlurDown,
    GlobalGotoBottomBlurUp,
    GlobalGotoTopBlurDown,
    GlobalGotoTopBlurUp,
    GlobalHelpBlurDown,
    GlobalHelpBlurUp,
    GlobalLayoutTreeviewBlurDown,
    GlobalLayoutTreeviewBlurUp,
    GlobalLayoutDatabaseBlurDown,
    GlobalLayoutDatabaseBlurUp,
    GlobalLeftBlurDown,
    GlobalLeftBlurUp,
    GlobalLyricAdjustForwardBlurDown,
    GlobalLyricAdjustForwardBlurUp,
    GlobalLyricAdjustBackwardBlurDown,
    GlobalLyricAdjustBackwardBlurUp,
    GlobalLyricCycleBlurDown,
    GlobalLyricCycleBlurUp,
    GlobalPlayerNextBlurDown,
    GlobalPlayerNextBlurUp,
    GlobalPlayerPreviousBlurDown,
    GlobalPlayerPreviousBlurUp,
    GlobalPlayerSeekForwardBlurDown,
    GlobalPlayerSeekForwardBlurUp,
    GlobalPlayerSeekBackwardBlurDown,
    GlobalPlayerSeekBackwardBlurUp,
    GlobalPlayerSpeedUpBlurDown,
    GlobalPlayerSpeedUpBlurUp,
    GlobalPlayerSpeedDownBlurDown,
    GlobalPlayerSpeedDownBlurUp,
    GlobalPlayerToggleGaplessBlurDown,
    GlobalPlayerToggleGaplessBlurUp,
    GlobalPlayerTogglePauseBlurDown,
    GlobalPlayerTogglePauseBlurUp,
    GlobalQuitBlurDown,
    GlobalQuitBlurUp,
    GlobalRightBlurDown,
    GlobalRightBlurUp,
    GlobalUpBlurDown,
    GlobalUpBlurUp,
    GlobalVolumeDownBlurDown,
    GlobalVolumeDownBlurUp,
    GlobalVolumeUpBlurDown,
    GlobalVolumeUpBlurUp,
    GlobalSavePlaylistBlurDown,
    GlobalSavePlaylistBlurUp,
    LibraryDeleteBlurDown,
    LibraryDeleteBlurUp,
    LibraryLoadDirBlurDown,
    LibraryLoadDirBlurUp,
    LibraryPasteBlurDown,
    LibraryPasteBlurUp,
    LibrarySearchBlurDown,
    LibrarySearchBlurUp,
    LibrarySearchYoutubeBlurDown,
    LibrarySearchYoutubeBlurUp,
    LibraryTagEditorBlurDown,
    LibraryTagEditorBlurUp,
    LibraryYankBlurDown,
    LibraryYankBlurUp,
    PlaylistDeleteBlurDown,
    PlaylistDeleteBlurUp,
    PlaylistDeleteAllBlurDown,
    PlaylistDeleteAllBlurUp,
    PlaylistShuffleBlurDown,
    PlaylistShuffleBlurUp,
    PlaylistModeCycleBlurDown,
    PlaylistModeCycleBlurUp,
    PlaylistPlaySelectedBlurDown,
    PlaylistPlaySelectedBlurUp,
    PlaylistSearchBlurDown,
    PlaylistSearchBlurUp,
    PlaylistSwapDownBlurDown,
    PlaylistSwapDownBlurUp,
    PlaylistSwapUpBlurDown,
    PlaylistSwapUpBlurUp,
    PlaylistAddRandomAlbumBlurDown,
    PlaylistAddRandomAlbumBlurUp,
    PlaylistAddRandomTracksBlurDown,
    PlaylistAddRandomTracksBlurUp,
    LibrarySwitchRootBlurDown,
    LibrarySwitchRootBlurUp,
    LibraryAddRootBlurDown,
    LibraryAddRootBlurUp,
    LibraryRemoveRootBlurDown,
    LibraryRemoveRootBlurUp,
    GlobalLayoutPodcastBlurDown,
    GlobalLayoutPodcastBlurUp,
    GlobalXywhMoveLeftBlurDown,
    GlobalXywhMoveLeftBlurUp,
    GlobalXywhMoveRightBlurDown,
    GlobalXywhMoveRightBlurUp,
    GlobalXywhMoveUpBlurDown,
    GlobalXywhMoveUpBlurUp,
    GlobalXywhMoveDownBlurDown,
    GlobalXywhMoveDownBlurUp,
    GlobalXywhZoomInBlurDown,
    GlobalXywhZoomInBlurUp,
    GlobalXywhZoomOutBlurDown,
    GlobalXywhZoomOutBlurUp,
    GlobalXywhHideBlurDown,
    GlobalXywhHideBlurUp,
    PodcastMarkPlayedBlurDown,
    PodcastMarkPlayedBlurUp,
    PodcastMarkAllPlayedBlurDown,
    PodcastMarkAllPlayedBlurUp,
    PodcastEpDownloadBlurDown,
    PodcastEpDownloadBlurUp,
    PodcastEpDeleteFileBlurDown,
    PodcastEpDeleteFileBlurUp,
    PodcastDeleteFeedBlurDown,
    PodcastDeleteFeedBlurUp,
    PodcastDeleteAllFeedsBlurDown,
    PodcastDeleteAllFeedsBlurUp,
    PodcastSearchAddFeedBlurDown,
    PodcastSearchAddFeedBlurUp,
    PodcastRefreshFeedBlurDown,
    PodcastRefreshFeedBlurUp,
    PodcastRefreshAllFeedsBlurDown,
    PodcastRefreshAllFeedsBlurUp,
}

/// Basically a Tree Node, but without having to include `tui-realm-treeview` as another dependency for lib
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecVec<T, V> {
    pub id: T,
    pub value: V,
    pub children: Vec<RecVec<T, V>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LIMsg {
    TreeStepInto(String),
    TreeStepOut,
    TreeBlur,
    Yank,
    Paste,
    SwitchRoot,
    AddRoot,
    RemoveRoot,

    /// A requested node is ready from loading.
    /// `(Tree, FocusNode)`
    TreeNodeReady(RecVec<PathBuf, String>, Option<String>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DSMsg {
    //TreeStepInto(String),
    TreeNodeReady(RecVec<String, String>, Option<String>),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DBMsg {
    /// Add all Track Results (from view `Tracks`) to the playlist
    AddAllToPlaylist,
    /// Add a single Track Result (from view `Tracks`) to the playlist
    AddPlaylist(usize),
    /// Add all Results (from view `Result`) to the playlist
    AddAllResultsToPlaylist,
    /// Add a single result (from view `Result`) to the playlist
    AddResultToPlaylist(usize),
    CriteriaBlurDown,
    CriteriaBlurUp,
    /// Search Results (for view `Result`) from a `Database`(view) index
    SearchResult(SearchCriteria),
    SearchResultBlurDown,
    SearchResultBlurUp,
    /// Serarch Tracks (for view `Tracks`) from a `Result`(view) index
    SearchTrack(usize),
    SearchTracksBlurDown,
    SearchTracksBlurUp,

    AddAllResultsConfirmShow,
    AddAllResultsConfirmCancel,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PCMsg {
    PodcastBlurDown,
    PodcastBlurUp,
    EpisodeBlurDown,
    EpisodeBlurUp,
    PodcastAddPopupShow,
    PodcastAddPopupCloseOk(String),
    PodcastAddPopupCloseCancel,
    SyncData((i64, PodcastNoId)),
    NewData(PodcastNoId),
    Error(PodcastFeed),
    PodcastSelected(usize),
    DescriptionUpdate,
    EpisodeAdd(usize),
    EpisodeMarkPlayed(usize),
    EpisodeMarkAllPlayed,
    PodcastRefreshOne(usize),
    PodcastRefreshAll,
    FetchPodcastStart(String),
    EpisodeDownload(usize),
    DLStart(EpData),
    DLComplete(EpData),
    DLResponseError(EpData),
    DLFileCreateError(EpData),
    DLFileWriteError(EpData),
    EpisodeDeleteFile(usize),
    FeedDeleteShow,
    FeedDeleteCloseOk,
    FeedDeleteCloseCancel,
    FeedsDeleteShow,
    FeedsDeleteCloseOk,
    FeedsDeleteCloseCancel,
    SearchItunesCloseCancel,
    SearchItunesCloseOk(usize),
    SearchSuccess(Vec<PodcastFeed>),
    SearchError(String),
}

/// Playlist Library View messages
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PLMsg {
    NextSong,
    PrevSong,
    /// Change focus to the next view
    PlaylistTableBlurDown,
    /// Change focus to the previous view
    PlaylistTableBlurUp,
    /// Add a directory / file to the playlist
    Add(PathBuf),
    /// Remove INDEX from playlist
    Delete(usize),
    /// Clear the Playlist
    DeleteAll,
    /// Select the next mode in the list
    ///
    /// see `termusicplayback::playlist::Loop` for all modes
    LoopModeCycle,
    /// Play a specific index
    PlaySelected(usize),
    /// Shuffle the current items in the playlist
    Shuffle,
    /// Swap a entry at INDEX with +1 (down)
    SwapDown(usize),
    /// Swap a entry at INDEX with -1 (up)
    SwapUp(usize),
    /// Start choosing random albums to be added to the playlist
    AddRandomAlbum,
    /// Start choosing random tracks to be added to the playlist
    AddRandomTracks,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GSMsg {
    PopupShowDatabase,
    PopupShowLibrary,
    PopupShowPlaylist,
    PopupShowEpisode,
    PopupShowPodcast,
    PopupShowDlnaServer,
    PopupCloseCancel,
    InputBlur,
    PopupUpdateDatabase(String),
    PopupUpdateLibrary(String),
    PopupUpdatePlaylist(String),
    PopupUpdateEpisode(String),
    PopupUpdatePodcast(String),
    TableBlur,
    PopupCloseEpisodeAddPlaylist,
    PopupCloseDatabaseAddPlaylist,
    PopupCloseLibraryAddPlaylist,
    PopupCloseOkLibraryLocate,
    PopupClosePlaylistPlaySelected,
    PopupCloseOkPlaylistLocate,
    PopupCloseOkEpisodeLocate,
    PopupCloseOkPodcastLocate,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum YSMsg {
    InputPopupShow,
    InputPopupCloseCancel,
    InputPopupCloseOk(String),
    TablePopupNext,
    TablePopupPrevious,
    TablePopupCloseCancel,
    TablePopupCloseOk(usize),

    /// The youtube search was a success, with all values.
    YoutubeSearchSuccess(YoutubeOptions),
    /// Indicates that the youtube search has failed, with error message.
    ///
    /// `(ErrorAsString)`
    YoutubeSearchFail(String),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TEMsg {
    TagEditorRun(String),
    TagEditorClose,
    TECounterDeleteOk,
    TEDownload(usize),
    TEEmbed(usize),
    TEFocus(TFMsg),
    TERename,
    TESearch,
    TESelectLyricOk(usize),

    TESearchLyricResult(SongTagRecordingResult),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TFMsg {
    CounterDeleteBlurDown,
    CounterDeleteBlurUp,
    InputArtistBlurDown,
    InputArtistBlurUp,
    InputTitleBlurDown,
    InputTitleBlurUp,
    InputAlbumBlurDown,
    InputAlbumBlurUp,
    InputGenreBlurDown,
    InputGenreBlurUp,
    SelectLyricBlurDown,
    SelectLyricBlurUp,
    TableLyricOptionsBlurDown,
    TableLyricOptionsBlurUp,
    TextareaLyricBlurDown,
    TextareaLyricBlurUp,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SongTagRecordingResult {
    Finish(Vec<SongTag>),
}

#[derive(Clone, PartialEq, Debug)]
pub struct ImageWrapper {
    pub data: DynamicImage,
}
impl Eq for ImageWrapper {}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct YoutubeOptions {
    pub items: Vec<YoutubeVideo>,
    pub page: u32,
    pub invidious_instance: Instance,
}

impl Default for YoutubeOptions {
    fn default() -> Self {
        Self {
            items: Vec::new(),
            page: 1,
            invidious_instance: crate::invidious::Instance::default(),
        }
    }
}

impl YoutubeOptions {
    pub fn get_by_index(&self, index: usize) -> Result<&YoutubeVideo> {
        if let Some(item) = self.items.get(index) {
            return Ok(item);
        }
        Err(anyhow!("index not found"))
    }

    pub async fn prev_page(&mut self) -> Result<()> {
        if self.page > 1 {
            self.page -= 1;
            self.items = self.invidious_instance.get_search_query(self.page).await?;
        }
        Ok(())
    }

    pub async fn next_page(&mut self) -> Result<()> {
        self.page += 1;
        self.items = self.invidious_instance.get_search_query(self.page).await?;
        Ok(())
    }

    #[must_use]
    pub const fn page(&self) -> u32 {
        self.page
    }
}
