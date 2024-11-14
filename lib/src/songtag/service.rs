use std::fmt::Display;

use lofty::picture::Picture;

use super::SongTag;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub enum SongTagServiceErrorWhere {
    SearchRecording,
    GetLyrics,
    GetPicture,
}

impl Display for SongTagServiceErrorWhere {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let res = match self {
            SongTagServiceErrorWhere::SearchRecording => "search_recording",
            SongTagServiceErrorWhere::GetLyrics => "get_lyrics",
            SongTagServiceErrorWhere::GetPicture => "get_picture",
        };

        write!(f, "{res}")
    }
}

// TODO: use thiserror
#[allow(dead_code)]
#[derive(Debug)]
pub enum SongTagServiceError<T> {
    /// Indicate a method is not supported to be called
    ///
    /// (which function, service name)
    NotSupported(SongTagServiceErrorWhere, &'static str),
    /// Indicate that a given [`SongTag`] was for a different service
    ///
    /// (file was for, actual service)
    IncorrectService(String, &'static str),

    /// Any other error like broken connection
    Other(T),
}

impl<T> Display for SongTagServiceError<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SongTagServiceError::NotSupported(error_where, service_name) => write!(
                f,
                "Function \"{error_where}\" is not supported by service \"{service_name}\""
            ),
            SongTagServiceError::IncorrectService(origin, service_name) => write!(
                f,
                "Given song was for service \"{origin}\", but given to \"{service_name}\""
            ),
            SongTagServiceError::Other(err) => Display::fmt(err, f),
        }
    }
}

impl<T> std::error::Error for SongTagServiceError<T> where T: std::error::Error {}

impl<T> From<T> for SongTagServiceError<T> {
    fn from(value: T) -> Self {
        Self::Other(value)
    }
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
