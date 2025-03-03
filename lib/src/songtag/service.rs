use std::fmt::Display;

use lofty::picture::Picture;

use super::SongTag;

#[allow(dead_code)] // TODO: change this to "expected" if MSRV is 1.81
#[derive(Debug, Clone, Copy)]
pub enum SongTagServiceErrorWhere {
    SearchRecording,
    GetLyrics,
    GetPicture,
    DownloadRecording,
}

impl Display for SongTagServiceErrorWhere {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let res = match self {
            Self::SearchRecording => "search_recording",
            Self::GetLyrics => "get_lyrics",
            Self::GetPicture => "get_picture",
            Self::DownloadRecording => "download_recording",
        };

        write!(f, "{res}")
    }
}

/// General [`SongTag`] Error with common variants
#[derive(Debug, thiserror::Error)]
pub enum SongTagServiceError<T> {
    /// Indicate a method is not supported to be called
    ///
    /// (which function, service name)
    #[error("Function \"{0}\" is not supported by service \"{1}\"")]
    NotSupported(SongTagServiceErrorWhere, &'static str),
    /// Indicate that a given [`SongTag`] was for a different service
    ///
    /// (file was for, actual service)
    #[error("Given song was for service \"{0}\", but given to \"{1}\"")]
    IncorrectService(String, &'static str),

    /// Any other error like broken connection
    #[error(transparent)]
    Other(#[from] T),
}

#[allow(clippy::module_name_repetitions)]
pub trait SongTagService {
    type Error;

    /// The Display name to use for this service
    fn display_name() -> &'static str
    where
        Self: Sized;

    /// Try searching for a recording(song / track) with the keywords at offset & limit
    async fn search_recording(
        &self,
        keywords: &str,
        offset: u32,
        limit: u32,
    ) -> Result<Vec<SongTag>, SongTagServiceError<Self::Error>>;
    // /// Try searching for a album with the keywords at offset & limit
    // async fn search_album(&self, keywords: &str, offset: u32, limit: u32) -> Result<Vec<()>, Self::Error>;

    /// Try to get lyrics associated with the given songtag
    async fn get_lyrics(&self, song: &SongTag) -> Result<String, SongTagServiceError<Self::Error>>;

    /// Try to get a picture associated with the given songtag
    async fn get_picture(
        &self,
        song: &SongTag,
    ) -> Result<Picture, SongTagServiceError<Self::Error>>;

    /// Try to get a URL for downloading the whole song
    async fn download_recording(
        &self,
        song: &SongTag,
    ) -> Result<String, SongTagServiceError<Self::Error>>;
}
