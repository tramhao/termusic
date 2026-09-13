//! Module for a Thin representation of a Track.

use std::{
    borrow::Cow,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail};
use reqwest::Url;
use termusiclib::{
    player::playlist_helpers::PlaylistTrackSource,
    track::{MediaTypesSimple, Track},
    utils::filetype_supported,
};

/// The Track's Id, stored in the playlist, which then can be used to fetch the full information
///
/// To get a Track's information, use [`TUITrackId::try_upgrade`] (or its specific forms).
#[derive(Debug, Clone, PartialEq, Hash, Eq, PartialOrd, Ord)]
pub enum TUITrackId {
    Track(PathBuf),
    Radio(String),
    Podcast(String),
}

impl TUITrackId {
    /// Get the main URL-identifier of the current track, if it is a type that has one.
    ///
    /// Only [`Track`](Self::Track) does not have a URL at the moment.
    #[must_use]
    pub fn url(&self) -> Option<&str> {
        match &self {
            Self::Track(_track_data) => None,
            Self::Radio(radio_track_data) => Some(radio_track_data.as_str()),
            Self::Podcast(podcast_track_data) => Some(podcast_track_data.as_str()),
        }
    }

    /// Get the main Path-identifier of the current track, if it is a type that has one.
    ///
    /// Only [`Track`](Self::Track) currently has a Path-identifier.
    #[must_use]
    pub fn path(&self) -> Option<&Path> {
        if let Self::Track(track_data) = &self {
            Some(track_data.as_path())
        } else {
            None
        }
    }

    /// Get a Enum without values to check against types.
    #[must_use]
    pub fn media_type(&self) -> MediaTypesSimple {
        match &self {
            Self::Track(_) => MediaTypesSimple::Music,
            Self::Radio(_) => MediaTypesSimple::LiveRadio,
            Self::Podcast(_) => MediaTypesSimple::Podcast,
        }
    }

    /// Create a [`PlaylistTrackSource`] from the current track identifier for GRPC.
    #[must_use]
    pub fn as_track_source(&self) -> PlaylistTrackSource {
        match &self {
            Self::Track(track_data) => {
                PlaylistTrackSource::Path(track_data.to_string_lossy().to_string())
            }
            Self::Radio(radio_track_data) => PlaylistTrackSource::Url(radio_track_data.clone()),
            Self::Podcast(podcast_track_data) => {
                PlaylistTrackSource::PodcastUrl(podcast_track_data.clone())
            }
        }
    }

    /// Try to parse a [`PlaylistTrackSource`] into a new instance, verifying the input is well-formed.
    pub fn from_track_source(source: PlaylistTrackSource) -> Result<Self> {
        match source {
            PlaylistTrackSource::Path(path) => {
                let path = PathBuf::from(path);
                if path.file_name().is_none() {
                    bail!("Expected Path source to have a filename component");
                }
                if !filetype_supported(&path) {
                    bail!("Unsupported File type for path!");
                }

                Ok(Self::Track(path))
            }
            PlaylistTrackSource::Url(url) => {
                let _ = Url::parse(&url).context("Expected Url source to be a valid url")?;
                Ok(Self::Radio(url))
            }
            PlaylistTrackSource::PodcastUrl(url) => {
                let _ = Url::parse(&url).context("Expected Podcast source to be a valid url")?;
                Ok(Self::Podcast(url))
            }
        }
    }

    /// Try to get a [`TUITrackId`] from a [`Track`].
    pub fn from_track(track: &Track) -> Result<Self> {
        Self::from_track_source(track.as_track_source())
    }

    /// Get a display-able identifier.
    ///
    /// # Panics
    ///
    /// If somehow a [`MediaTypesThin::Track`] does not have a `file_name`.
    #[must_use]
    pub fn id_str(&self) -> Cow<'_, str> {
        match &self {
            // A music track will always have a file_name (and not terminate in "..")
            Self::Track(track_data) => track_data.file_name().map(|v| v.to_string_lossy()).unwrap(),
            Self::Radio(radio_track_data) => radio_track_data.into(),
            Self::Podcast(podcast_track_data) => podcast_track_data.into(),
        }
    }

    /// Get the current ID to display during loads.
    pub fn display_short(&self) -> Cow<'_, str> {
        match self {
            TUITrackId::Track(path_buf) => path_buf
                .file_name()
                .expect("There should always be a filename!")
                .to_string_lossy(),
            // TODO: if parsed as a URL and stored that way, use only the hostname
            TUITrackId::Radio(url) | TUITrackId::Podcast(url) => url.into(),
        }
    }
}

impl PartialEq<PlaylistTrackSource> for &TUITrackId {
    fn eq(&self, other: &PlaylistTrackSource) -> bool {
        match (other, self) {
            (PlaylistTrackSource::Path(path1), TUITrackId::Track(path2)) => {
                *path1 == path2.to_string_lossy()
            }
            (PlaylistTrackSource::PodcastUrl(url1), TUITrackId::Podcast(url2))
            | (PlaylistTrackSource::Url(url1), TUITrackId::Radio(url2)) => url1 == url2,
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use pretty_assertions::assert_eq;
    use termusiclib::player::playlist_helpers::PlaylistTrackSource;

    use super::TUITrackId;

    #[test]
    fn should_allow_new_tracks() {
        assert_eq!(
            TUITrackId::from_track_source(PlaylistTrackSource::Path("/valid/path.mp3".to_string()))
                .unwrap(),
            TUITrackId::Track(PathBuf::from("/valid/path.mp3"))
        );
        assert_eq!(
            TUITrackId::from_track_source(PlaylistTrackSource::PodcastUrl(
                "http://somewhere.com/".to_string()
            ))
            .unwrap(),
            TUITrackId::Podcast("http://somewhere.com/".to_string())
        );
        assert_eq!(
            TUITrackId::from_track_source(PlaylistTrackSource::Url(
                "http://somewhere.com/".to_string()
            ))
            .unwrap(),
            TUITrackId::Radio("http://somewhere.com/".to_string())
        );
    }

    #[test]
    fn should_refuse_invalid_paths() {
        let err =
            TUITrackId::from_track_source(PlaylistTrackSource::Path("/invalid/..".to_string()))
                .unwrap_err();
        assert_eq!(
            err.to_string(),
            "Expected Path source to have a filename component"
        );
        let err = TUITrackId::from_track_source(PlaylistTrackSource::Path("/invalid/".to_string()))
            .unwrap_err();
        assert_eq!(err.to_string(), "Unsupported File type for path!");
        let err =
            TUITrackId::from_track_source(PlaylistTrackSource::Path("/invalid/no_ext".to_string()))
                .unwrap_err();
        assert_eq!(err.to_string(), "Unsupported File type for path!");
        let err = TUITrackId::from_track_source(PlaylistTrackSource::Path(
            "/invalid/bad.ext".to_string(),
        ))
        .unwrap_err();
        assert_eq!(err.to_string(), "Unsupported File type for path!");
    }

    #[test]
    fn should_refuse_invalid_urls() {
        let err =
            TUITrackId::from_track_source(PlaylistTrackSource::PodcastUrl("/nowhere".to_string()))
                .unwrap_err();
        assert_eq!(err.to_string(), "Expected Podcast source to be a valid url");
        let err = TUITrackId::from_track_source(PlaylistTrackSource::Url("/nowhere".to_string()))
            .unwrap_err();
        assert_eq!(err.to_string(), "Expected Url source to be a valid url");
    }
}
