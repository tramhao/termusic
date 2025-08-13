use std::sync::Arc;

use crate::ids::{IdConfigEditor, IdKeyGlobal, IdKeyOther};
use crate::podcast::{EpData, PodcastFeed, PodcastNoId};
use crate::songtag::SongTag;
use image::DynamicImage;

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

/// Constant strings for Unknown values
pub mod const_unknown {
    use crate::const_str;

    const_str! {
        UNKNOWN_ARTIST "Unknown Artist",
        UNKNOWN_TITLE "Unknown Title",
        UNKNOWN_ALBUM "Unknown Album",
        UNKNOWN_FILE "Unknown File",
    }
}
