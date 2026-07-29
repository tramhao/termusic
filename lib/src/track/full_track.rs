use std::{
    borrow::Cow,
    cell::RefCell,
    num::NonZeroUsize,
    path::{Path, PathBuf},
    str::FromStr,
    sync::Arc,
    time::Duration,
};

use anyhow::{Result, bail};
use id3::frame::Lyrics as Id3Lyrics;
use lofty::{file::FileType, picture::Picture};
use lru::LruCache;

use crate::{
    player::playlist_helpers::PlaylistTrackSource,
    podcast::episode::Episode,
    songtag::lrc::Lyric,
    track::{
        DurationFmtShort, MetadataOptions, TrackMetadata, parse_metadata_from_file,
        read_metadata::get_picture_for_music_track,
    },
};

/// A simple no-value representation of [`MediaTypes`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MediaTypesSimple {
    Music,
    Podcast,
    LiveRadio,
}

#[derive(Debug, Clone)]
pub struct PodcastTrackData {
    /// The Podcast url, used as the sole identifier for equality
    url: String,

    localfile: Option<PathBuf>,
    image_url: Option<String>,
}

impl PartialEq for PodcastTrackData {
    fn eq(&self, other: &Self) -> bool {
        self.url == other.url
    }
}

impl PodcastTrackData {
    /// Get the Podcast URL identifier
    #[must_use]
    pub fn url(&self) -> &str {
        &self.url
    }

    /// Get the local file path for the downloaded podcast
    #[must_use]
    pub fn localfile(&self) -> Option<&Path> {
        self.localfile.as_deref()
    }

    /// Check if this track has a localfile attached
    #[must_use]
    pub fn has_localfile(&self) -> bool {
        self.localfile.is_some()
    }

    #[must_use]
    pub fn image_url(&self) -> Option<&str> {
        self.image_url.as_deref()
    }

    /// Create new [`PodcastTrackData`] with only the url.
    ///
    /// This should mainly be used for tests only.
    #[must_use]
    pub fn new(url: String) -> Self {
        Self {
            url,

            localfile: None,
            image_url: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RadioTrackData {
    /// The Radio url, used as the sole identifier for equality
    url: String,
}

impl RadioTrackData {
    /// Get the url for for the radio
    #[must_use]
    pub fn url(&self) -> &str {
        &self.url
    }

    /// Create new [`RadioTrackData`] with only the url.
    ///
    /// This should mainly be used for tests only.
    #[must_use]
    pub fn new(url: String) -> Self {
        Self { url }
    }
}

#[derive(Debug, Clone)]
pub struct TrackData {
    /// The Track file path, used as the sole identifier for equality
    path: PathBuf,

    album: Option<String>,

    file_type: Option<FileType>,
}

impl PartialEq for TrackData {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path
    }
}

impl TrackData {
    /// Get the path the track is stored at
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    #[must_use]
    pub fn album(&self) -> Option<&str> {
        self.album.as_deref()
    }

    /// The lofty File-Type; may not exist if lofty could not parse the file.
    ///
    /// Note that if lofty cannot parse the file, that **does not** mean that symphonia cannot play it.
    #[must_use]
    pub fn file_type(&self) -> Option<FileType> {
        self.file_type
    }

    /// Create new [`TrackData`] with only the path.
    ///
    /// This should mainly be used for tests only.
    #[must_use]
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            album: None,
            file_type: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum MediaTypes {
    Track(TrackData),
    Radio(RadioTrackData),
    Podcast(PodcastTrackData),
}

#[derive(Debug, Clone, PartialEq)]
pub struct LyricData {
    pub raw_lyrics: Vec<Id3Lyrics>,
    pub parsed_lyrics: Option<Lyric>,
}

type PictureCache = LruCache<PathBuf, Arc<Picture>>;
type LyricCache = LruCache<PathBuf, Arc<LyricData>>;

// NOTE: thread_locals are like "LazyLock"s, they only get initialized on first access.
std::thread_local! {
    static PICTURE_CACHE: RefCell<PictureCache> = RefCell::new(PictureCache::new(NonZeroUsize::new(5).unwrap()));
    static LYRIC_CACHE: RefCell<LyricCache> = RefCell::new(LyricCache::new(NonZeroUsize::new(5).unwrap()));
}

#[derive(Debug, Clone)]
pub struct Track {
    inner: MediaTypes,

    duration: Option<Duration>,
    title: Option<String>,
    artist: Option<String>,
}

impl PartialEq for Track {
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}

impl Track {
    /// Create a new Track instance from a Podcast Episode from the database
    #[must_use]
    pub fn from_podcast_episode(ep: &Episode) -> Self {
        let localfile = ep.path.as_ref().take_if(|v| v.exists()).cloned();

        let podcast_data = PodcastTrackData {
            url: ep.url.clone(),
            localfile,
            image_url: ep.image_url.clone(),
        };

        let duration = ep
            .duration
            .map(u64::try_from)
            .transpose()
            .ok()
            .flatten()
            .map(Duration::from_secs);

        Self {
            inner: MediaTypes::Podcast(podcast_data),
            duration,
            title: Some(ep.title.clone()),
            artist: None,
        }
    }

    /// Create a new Track from a radio url
    #[must_use]
    pub fn new_radio<U: Into<String>>(url: U) -> Self {
        let radio_data = RadioTrackData { url: url.into() };

        Self {
            inner: MediaTypes::Radio(radio_data),
            duration: None,
            // will be fetched later, maybe consider storing a cache in the database?
            title: None,
            artist: None,
        }
    }

    /// Create a new Track from a local file, populated with the most important tags
    pub fn read_track_from_path<P: Into<PathBuf>>(path: P) -> Result<Self> {
        let path: PathBuf = path.into();

        // for the case that we somehow get a path that is just ""(empty)
        if path.as_os_str().is_empty() {
            bail!("Given path is empty!");
        }

        let metadata = match parse_metadata_from_file(
            &path,
            MetadataOptions {
                album: true,
                artist: true,
                title: true,
                duration: true,
                ..Default::default()
            },
        ) {
            Ok(v) => v,
            Err(err) => {
                // not being able to read metadata is not fatal, we will just have less information about it
                warn!(
                    "Failed to read metadata from \"{}\": {}",
                    path.display(),
                    err
                );
                TrackMetadata::default()
            }
        };

        Ok(Self::from_track_metadata(path, metadata))
    }

    /// Create a Track instance from the given Track Metadata.
    ///
    /// This is mainly meant to be used for creating dummy tracks for testing.
    pub fn from_track_metadata<P: Into<PathBuf>>(path: P, metadata: TrackMetadata) -> Self {
        let track_data = TrackData {
            path: path.into(),
            album: metadata.album,
            file_type: metadata.file_type,
        };

        Self {
            inner: MediaTypes::Track(track_data),
            duration: metadata.duration,
            title: metadata.title,
            artist: metadata.artist,
        }
    }

    #[must_use]
    pub fn artist(&self) -> Option<&str> {
        self.artist.as_deref()
    }

    #[must_use]
    pub fn title(&self) -> Option<&str> {
        self.title.as_deref()
    }

    #[must_use]
    pub fn duration(&self) -> Option<Duration> {
        self.duration
    }

    /// Format the Track's duration to a short-form.
    ///
    /// see [`DurationFmtShort`] for formatting.
    #[must_use]
    pub fn duration_str_short(&self) -> Option<DurationFmtShort> {
        let dur = self.duration?;

        Some(DurationFmtShort(dur))
    }

    /// Get the main URL-identifier of the current track, if it is a type that has one.
    ///
    /// Only [`MediaTypes::Track`] does not have a URL at the moment.
    #[must_use]
    pub fn url(&self) -> Option<&str> {
        match &self.inner {
            MediaTypes::Track(_track_data) => None,
            MediaTypes::Radio(radio_track_data) => Some(radio_track_data.url()),
            MediaTypes::Podcast(podcast_track_data) => Some(podcast_track_data.url()),
        }
    }

    /// Get the main Path-identifier of the current track, if it is a type that has one.
    ///
    /// Only [`MediaTypes::Track`] currently has a main Path-identifier.
    #[must_use]
    pub fn path(&self) -> Option<&Path> {
        if let MediaTypes::Track(track_data) = &self.inner {
            Some(track_data.path())
        } else {
            None
        }
    }

    #[must_use]
    pub fn as_track(&self) -> Option<&TrackData> {
        if let MediaTypes::Track(track_data) = &self.inner {
            Some(track_data)
        } else {
            None
        }
    }

    #[must_use]
    pub fn as_radio(&self) -> Option<&RadioTrackData> {
        if let MediaTypes::Radio(radio_data) = &self.inner {
            Some(radio_data)
        } else {
            None
        }
    }

    #[must_use]
    pub fn as_podcast(&self) -> Option<&PodcastTrackData> {
        if let MediaTypes::Podcast(podcast_data) = &self.inner {
            Some(podcast_data)
        } else {
            None
        }
    }

    #[must_use]
    pub fn inner(&self) -> &MediaTypes {
        &self.inner
    }

    /// Get a Enum without values to check against types.
    ///
    /// Mainly for not having to change too many functions yet.
    #[must_use]
    pub fn media_type(&self) -> MediaTypesSimple {
        match &self.inner {
            MediaTypes::Track(_) => MediaTypesSimple::Music,
            MediaTypes::Radio(_) => MediaTypesSimple::LiveRadio,
            MediaTypes::Podcast(_) => MediaTypesSimple::Podcast,
        }
    }

    /// Create a [`PlaylistTrackSource`] from the current track identifier for GRPC.
    #[must_use]
    pub fn as_track_source(&self) -> PlaylistTrackSource {
        match &self.inner {
            MediaTypes::Track(track_data) => {
                PlaylistTrackSource::Path(track_data.path.to_string_lossy().to_string())
            }
            MediaTypes::Radio(radio_track_data) => {
                PlaylistTrackSource::Url(radio_track_data.url.clone())
            }
            MediaTypes::Podcast(podcast_track_data) => {
                PlaylistTrackSource::PodcastUrl(podcast_track_data.url.clone())
            }
        }
    }

    /// Get a cover / picture for the current track.
    ///
    /// Returns `Ok(None)` if there was no error, but also no picture could be found.
    ///
    /// This is currently **only** implemented for Music Tracks.
    ///
    /// # Errors
    ///
    /// - if reading the file fails
    /// - if parsing the file fails
    /// - if there is no parent in the given path
    /// - reading the directory fails
    /// - reading the file fails
    /// - parsing the file as a picture fails
    pub fn get_picture(&self) -> Result<Option<Arc<Picture>>> {
        match &self.inner {
            MediaTypes::Track(track_data) => {
                let path_key = track_data.path().to_owned();

                // TODO: option to disable getting with folder cover for tag editor?
                let res = PICTURE_CACHE.with_borrow_mut(|cache| {
                    cache
                        .try_get_or_insert(path_key, || {
                            let picture =
                                get_picture_for_music_track(track_data.path()).map_err(Some)?;

                            let Some(picture) = picture else {
                                return Err(None);
                            };

                            Ok(Arc::new(picture))
                        })
                        .cloned()
                });

                // this has to be done as LruCache::try_get_or_insert enforces that the Ok result is the value itself, no mapping can be done.
                match res {
                    Ok(v) => return Ok(Some(v)),
                    Err(None) => return Ok(None),
                    Err(Some(err)) => return Err(err),
                }
            }
            MediaTypes::Radio(_radio_track_data) => trace!("Unimplemented: radio picture"),
            MediaTypes::Podcast(_podcast_track_data) => trace!("Unimplemented: podcast picture"),
        }

        Ok(None)
    }

    /// Get a display-able identifier
    ///
    /// # Panics
    ///
    /// If somehow a [`MediaTypes::Track`] does not have a `file_name`.
    #[must_use]
    pub fn id_str(&self) -> Cow<'_, str> {
        match &self.inner {
            // A music track will always have a file_name (and not terminate in "..")
            MediaTypes::Track(track_data) => track_data
                .path()
                .file_name()
                .map(|v| v.to_string_lossy())
                .unwrap(),
            MediaTypes::Radio(radio_track_data) => radio_track_data.url().into(),
            MediaTypes::Podcast(podcast_track_data) => podcast_track_data.url().into(),
        }
    }

    /// Get the lyrics data for the current Track.
    ///
    /// Only works for Music Tracks.
    pub fn get_lyrics(&self) -> Result<Option<Arc<LyricData>>> {
        let Some(track_data) = self.as_track() else {
            bail!("Track is not a Music Track!");
        };

        let path_key = track_data.path().to_owned();

        let res = LYRIC_CACHE.with_borrow_mut(|cache| {
            cache
                .try_get_or_insert(path_key, || {
                    let result = parse_metadata_from_file(
                        track_data.path(),
                        MetadataOptions {
                            lyrics: true,
                            ..Default::default()
                        },
                    )?;
                    let lyric_frames = result.lyric_frames.unwrap_or_default();

                    let parsed_lyric = lyric_frames
                        .first()
                        .and_then(|frame| Lyric::from_str(&frame.text).ok());

                    Ok(Arc::new(LyricData {
                        raw_lyrics: lyric_frames,
                        parsed_lyrics: parsed_lyric,
                    }))
                })
                .cloned()
        });

        // this has to be done as LruCache::try_get_or_insert enforces that the Ok result is the value itself, no mapping can be done.
        match res {
            Ok(v) => Ok(Some(v)),
            Err(None) => Ok(None),
            Err(Some(err)) => Err(err),
        }
    }

    /// Remove the given path from the Lyric parse cache, forcing a reload upon next access.
    ///
    /// If the key does not exist, it will not fail.
    pub fn unset_cache_for_path(path: &Path) {
        LYRIC_CACHE.with_borrow_mut(|cache| {
            cache.pop(path);
        });
    }
}

impl PartialEq<PlaylistTrackSource> for &Track {
    fn eq(&self, other: &PlaylistTrackSource) -> bool {
        match other {
            PlaylistTrackSource::Path(path) => self
                .as_track()
                .is_some_and(|v| v.path().to_string_lossy() == path.as_str()),
            PlaylistTrackSource::Url(url) => self.as_radio().is_some_and(|v| v.url() == url),
            PlaylistTrackSource::PodcastUrl(url) => {
                self.as_podcast().is_some_and(|v| v.url() == url)
            }
        }
    }
}
