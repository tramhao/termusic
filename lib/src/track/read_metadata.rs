use std::{
    borrow::Cow,
    fs::File,
    io::BufReader,
    path::Path,
    time::{Duration, SystemTime},
};

use anyhow::{Context, Result, anyhow};
use id3::frame::Lyrics as Id3Lyrics;
use lofty::{
    config::ParseOptions,
    file::{AudioFile, FileType, TaggedFileExt},
    picture::{Picture, PictureType},
    probe::Probe,
    tag::{Accessor, ItemKey, ItemValue, Tag as LoftyTag},
};

use crate::utils::SplitArrayIter;

/// Try to get a [`Picture`] for a given music track.
///
/// # Errors
///
/// - if reading the file fails
/// - if parsing the file fails
/// - also see [`find_folder_picture`]
pub(super) fn get_picture_for_music_track(track_path: &Path) -> Result<Option<Picture>> {
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

/// All extension we support to find in a parent folder of a given track to use as a cover image.
///
/// The matching is done via lowercase EQ.
const SUPPORTED_IMG_EXTENSIONS: &[&str] = &["jpg", "jpeg", "png"];

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
        if !SUPPORTED_IMG_EXTENSIONS
            .iter()
            .any(|v| ext.eq_ignore_ascii_case(v))
        {
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

/// See [`TrackMetadata`] for explanation of values.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
#[allow(clippy::struct_excessive_bools)] // configuration, this is not a state machine
pub struct MetadataOptions<'a> {
    pub album: bool,
    pub album_artist: bool,
    pub album_artists: bool,
    pub artist: bool,
    pub artists: bool,
    /// Separators for fallback parsing of a single `artist` value into multiple `artists`.
    ///
    /// See [`DEFAULT_ARTIST_SEPARATORS`].
    pub artist_separators: &'a [&'a str],
    pub title: bool,
    pub duration: bool,
    pub genre: bool,
    pub cover: bool,
    pub lyrics: bool,
    pub file_times: bool,
}

impl MetadataOptions<'_> {
    /// Enable all options
    #[must_use]
    pub fn all() -> Self {
        Self {
            album: true,
            album_artist: true,
            album_artists: true,
            artist: true,
            artists: true,
            artist_separators: &[],
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
pub fn parse_metadata_from_file(
    path: &Path,
    options: MetadataOptions<'_>,
) -> Result<TrackMetadata> {
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

    if options.file_times
        && let Ok(metadata) = std::fs::metadata(path)
    {
        let filetimes = FileTimes {
            modified: metadata.modified().ok(),
            created: metadata.created().ok(),
        };

        res.file_times = Some(filetimes);
    }

    Ok(res)
}

/// The inner working to actually copy data from the given [`LoftyTag`] into the `res`ult
fn handle_tag(tag: &LoftyTag, options: MetadataOptions<'_>, res: &mut TrackMetadata) {
    if let Some(len_tag) = tag.get_string(ItemKey::Length) {
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
        let mut artists: Vec<String> = tag
            .get_strings(ItemKey::TrackArtists)
            .map(ToString::to_string)
            .collect();

        if artists.is_empty()
            && !options.artist_separators.is_empty()
            && let Some(artist) = tag.artist()
        {
            let artists_iter = split_artists(&artist, options);
            artists.extend(artists_iter);
        }

        res.artists = Some(artists);
    }
    if options.album {
        res.album = tag.album().map(Cow::into_owned);
    }
    if options.album_artist {
        res.album_artist = tag
            .get(ItemKey::AlbumArtist)
            .and_then(|v| v.value().text())
            .map(ToString::to_string);
    }
    if options.album_artists {
        let mut album_artists: Vec<String> = tag
            .get_strings(ItemKey::AlbumArtists)
            .map(ToString::to_string)
            .collect();

        if album_artists.is_empty()
            && !options.artist_separators.is_empty()
            && let Some(album_artist) = tag.get(ItemKey::AlbumArtist).and_then(|v| v.value().text())
        {
            let artists_iter = split_artists(album_artist, options);
            album_artists.extend(artists_iter);
        }

        res.album_artists = Some(album_artists);
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

/// Create a iterator which separates `artist` with options from `options`
#[inline]
fn split_artists<'a>(
    artist: &'a str,
    options: MetadataOptions<'a>,
) -> impl Iterator<Item = String> + 'a {
    SplitArrayIter::new(artist, options.artist_separators)
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(ToString::to_string)
}

/// Fetch all lyrics from the given Lofty tag into the given array.
fn get_lyrics_from_tags(tag: &LoftyTag, lyric_frames: &mut Vec<Id3Lyrics>) {
    let lyrics = tag.get_items(ItemKey::Lyrics);
    for lyric in lyrics {
        if let ItemValue::Text(lyrics_text) = lyric.value() {
            lyric_frames.push(Id3Lyrics {
                lang: lyric.lang().escape_ascii().to_string(),
                description: lyric.description().to_string(),
                text: lyrics_text.clone(),
            });
        }
    }

    lyric_frames.sort_by(|a, b| {
        a.description
            .to_lowercase()
            .cmp(&b.description.to_lowercase())
    });
}
