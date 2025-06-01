use std::{
    borrow::Cow,
    cell::RefCell,
    fmt::Display,
    fs::File,
    io::BufReader,
    num::NonZeroUsize,
    path::{Path, PathBuf},
    str::FromStr,
    sync::Arc,
    time::{Duration, SystemTime},
};

use anyhow::{anyhow, bail, Context, Result};
use id3::frame::Lyrics as Id3Lyrics;
use lofty::{
    config::ParseOptions,
    file::{AudioFile, FileType, TaggedFileExt},
    picture::{Picture, PictureType},
    probe::Probe,
    tag::{Accessor, ItemKey, ItemValue, Tag as LoftyTag},
};
use lru::LruCache;

use crate::{
    player::playlist_helpers::PlaylistTrackSource, podcast::episode::Episode, songtag::lrc::Lyric,
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

        let track_data = TrackData {
            path,
            album: metadata.album,
            file_type: metadata.file_type,
        };

        Ok(Self {
            inner: MediaTypes::Track(track_data),
            duration: metadata.duration,
            title: metadata.title,
            artist: metadata.artist,
        })
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
                PlaylistTrackSource::Url(radio_track_data.url.to_string())
            }
            MediaTypes::Podcast(podcast_track_data) => {
                PlaylistTrackSource::PodcastUrl(podcast_track_data.url.to_string())
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

/// Try to get a [`Picture`] for a given music track.
///
/// # Errors
///
/// - if reading the file fails
/// - if parsing the file fails
/// - also see [`find_folder_picture`]
fn get_picture_for_music_track(track_path: &Path) -> Result<Option<Picture>> {
    let result = parse_metadata_from_file(
        track_path,
        MetadataOptions {
            cover: true,
            ..Default::default()
        },
    )?;

    let Some(picture) = result.cover else {
        let maybe_dir_pic = find_folder_picture(track_path)?;
        return Ok(maybe_dir_pic);
    };

    Ok(Some(picture))
}

/// Find a picture file and parse it in the parent directory of the given path.
///
/// # Errors
///
/// - if there is no parent in the given path
/// - reading the directory fails
/// - reading the file fails
/// - parsing the file as a picture fails
fn find_folder_picture(track_path: &Path) -> Result<Option<Picture>> {
    let Some(parent_folder) = track_path.parent() else {
        return Err(anyhow!("Track does not have a parent directory")
            .context(track_path.display().to_string()));
    };

    let files = std::fs::read_dir(parent_folder).context(parent_folder.display().to_string())?;

    for entry in files.flatten() {
        let path = entry.path();

        let Some(ext) = path.extension() else {
            continue;
        };

        let Some(name) = path.file_stem() else {
            continue;
        };

        // only take some picture files we can handle and are common
        if ext != "jpg" || ext != "png" {
            continue;
        }

        // skip "artist.EXT" files; those may exist for standalone tracks which are in the same directory as the artist info
        // for example this might exist when using jellyfin
        // and the artist cover is unlikely we want as a track picture
        if name.eq_ignore_ascii_case("artist") {
            continue;
        }

        let mut reader = BufReader::new(File::open(path)?);

        let picture = Picture::from_reader(&mut reader)?;

        return Ok(Some(picture));
    }

    Ok(None)
}

/// Format the given Duration in the following way via a `Display` impl:
///
/// ```txt
/// # if Hours > 0
/// 10:01:01
/// # if Hour == 0
/// 01:01
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DurationFmtShort(pub Duration);

impl Display for DurationFmtShort {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let d = self.0;
        let duration_hour = d.as_secs() / 3600;
        let duration_min = (d.as_secs() % 3600) / 60;
        let duration_secs = d.as_secs() % 60;

        if duration_hour == 0 {
            write!(f, "{duration_min:0>2}:{duration_secs:0>2}")
        } else {
            write!(f, "{duration_hour}:{duration_min:0>2}:{duration_secs:0>2}")
        }
    }
}

/// See [`TrackMetadata`] for explanation of values.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
#[allow(clippy::struct_excessive_bools)] // configuration, this is not a state machine
pub struct MetadataOptions {
    pub album: bool,
    pub album_artist: bool,
    pub album_artists: bool,
    pub artist: bool,
    pub artists: bool,
    pub title: bool,
    pub duration: bool,
    pub genre: bool,
    pub cover: bool,
    pub lyrics: bool,
    pub file_times: bool,
}

impl MetadataOptions {
    /// Enable all options
    #[must_use]
    pub fn all() -> Self {
        Self {
            album: true,
            album_artist: true,
            album_artists: true,
            artist: true,
            artists: true,
            title: true,
            duration: true,
            genre: true,
            cover: true,
            lyrics: true,
            file_times: true,
        }
    }
}

/// For ID3v2 tags consult <https://exiftool.org/TagNames/ID3.html#v2_4>.
///
/// For common-usage consult <https://kodi.wiki/view/Music_tagging#Tags_Kodi_reads>.
/// For common `TXX` tags consult <https://picard-docs.musicbrainz.org/en/appendices/tag_mapping.html#artists>.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct TrackMetadata {
    /// ID3v2 tag `TALB` or equivalent
    pub album: Option<String>,
    /// ID3v2 tag `TPE2` or equivalent
    pub album_artist: Option<String>,
    /// ID3v2 tag `TXX:ALBUMARTISTS` <https://kodi.wiki/view/Music_tagging#Tags_Kodi_reads>
    pub album_artists: Option<Vec<String>>,
    /// ID3v2 tag `TPE1` or equivalent
    pub artist: Option<String>,
    /// ID3v2 tag `TXX:ARTISTS` <https://picard-docs.musicbrainz.org/en/appendices/tag_mapping.html>
    pub artists: Option<Vec<String>>,
    /// ID3v2 tag `TIT2` or equivalent
    pub title: Option<String>,
    /// Total duration, this may or may not come from a tag
    pub duration: Option<Duration>,
    /// ID3v2 tag `TCON` or equivalent
    pub genre: Option<String>,
    /// ID3v2 tag `APIC` or equivalent
    pub cover: Option<Picture>,
    /// ID3v2 tags `USLT` or equivalent
    pub lyric_frames: Option<Vec<Id3Lyrics>>,
    pub file_times: Option<FileTimes>,

    pub file_type: Option<FileType>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct FileTimes {
    pub modified: Option<SystemTime>,
    pub created: Option<SystemTime>,
}

/// Try to parse all specified metadata in the given `options`.
pub fn parse_metadata_from_file(path: &Path, options: MetadataOptions) -> Result<TrackMetadata> {
    let mut parse_options = ParseOptions::new();

    parse_options = parse_options.read_cover_art(options.cover);

    let probe = Probe::open(path)?.options(parse_options);

    let tagged_file = probe.read()?;

    let mut res = TrackMetadata::default();

    if options.duration {
        let properties = tagged_file.properties();
        res.duration = Some(properties.duration());
    }

    res.file_type = Some(tagged_file.file_type());

    if let Some(tag) = tagged_file.primary_tag() {
        handle_tag(tag, options, &mut res);
    } else if let Some(tag) = tagged_file.first_tag() {
        handle_tag(tag, options, &mut res);
    }

    if options.file_times {
        if let Ok(metadata) = std::fs::metadata(path) {
            let filetimes = FileTimes {
                modified: metadata.modified().ok(),
                created: metadata.created().ok(),
            };

            res.file_times = Some(filetimes);
        }
    }

    Ok(res)
}

/// The inner working to actually copy data from the given [`LoftyTag`] into the `res`ult
fn handle_tag(tag: &LoftyTag, options: MetadataOptions, res: &mut TrackMetadata) {
    if let Some(len_tag) = tag.get_string(&ItemKey::Length) {
        match len_tag.parse::<u64>() {
            Ok(v) => res.duration = Some(Duration::from_millis(v)),
            Err(_) => warn!(
                "Failed reading precise \"Length\", expected u64 parseable, got \"{len_tag:#?}\"",
            ),
        }
    }

    if options.artist {
        res.artist = tag.artist().map(Cow::into_owned);
    }
    if options.artists {
        res.artists = Some(
            tag.get_strings(&ItemKey::TrackArtists)
                .map(ToString::to_string)
                .collect(),
        );
    }
    if options.album {
        res.album = tag.album().map(Cow::into_owned);
    }
    if options.album_artist {
        res.album_artist = tag
            .get(&ItemKey::AlbumArtist)
            .and_then(|v| v.value().text())
            .map(ToString::to_string);
    }
    if options.album_artists {
        // manual implementation as it currently does not exist upstream
        // see https://github.com/Serial-ATA/lofty-rs/issues/522
        // res.album_artists = Some(tag.get_strings(&ItemKey::AlbumArtists).map(ToString::to_string).collect());
        // lofty already separates them from a "; "
        res.album_artists = Some(
            tag.get_strings(&ItemKey::Unknown("ALBUMARTISTS".to_string()))
                .map(ToString::to_string)
                .collect::<Vec<_>>(),
        );
    }
    if options.title {
        res.title = tag.title().map(Cow::into_owned);
    }
    if options.genre {
        res.genre = tag.genre().map(Cow::into_owned);
    }

    if options.cover {
        res.cover = tag
            .pictures()
            .iter()
            .find(|pic| pic.pic_type() == PictureType::CoverFront)
            .or_else(|| tag.pictures().first())
            .cloned();
    }

    if options.lyrics {
        let mut lyric_frames: Vec<Id3Lyrics> = Vec::new();
        get_lyrics_from_tags(tag, &mut lyric_frames);
        res.lyric_frames = Some(lyric_frames);
    }
}

/// Fetch all lyrics from the given Lofty tag into the given array.
fn get_lyrics_from_tags(tag: &LoftyTag, lyric_frames: &mut Vec<Id3Lyrics>) {
    let lyrics = tag.get_items(&ItemKey::Lyrics);
    for lyric in lyrics {
        if let ItemValue::Text(lyrics_text) = lyric.value() {
            lyric_frames.push(Id3Lyrics {
                lang: lyric.lang().escape_ascii().to_string(),
                description: lyric.description().to_string(),
                text: lyrics_text.to_string(),
            });
        }
    }

    lyric_frames.sort_by(|a, b| {
        a.description
            .to_lowercase()
            .cmp(&b.description.to_lowercase())
    });
}

#[cfg(test)]
mod tests {
    mod durationfmt {
        use std::time::Duration;

        use crate::track::DurationFmtShort;

        #[test]
        fn should_format_without_hours() {
            assert_eq!(
                DurationFmtShort(Duration::from_secs(61)).to_string(),
                "01:01"
            );
        }

        #[test]
        fn should_format_with_hours() {
            assert_eq!(
                DurationFmtShort(Duration::from_secs(60 * 61 + 1)).to_string(),
                "1:01:01"
            );
        }
    }
}
