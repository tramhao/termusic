use crate::library_db::const_unknown::{UNKNOWN_ARTIST, UNKNOWN_TITLE};
use crate::podcast::episode::Episode;
/**
 * MIT License
 *
 * termusic - Copyright (c) 2021 Larry Hao
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy
 * of this software and associated documentation files (the "Software"), to deal
 * in the Software without restriction, including without limitation the rights
 * to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 * copies of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in all
 * copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 * AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE US OR OTHER DEALINGS IN THE
 * SOFTWARE.
 */
use crate::songtag::lrc::Lyric;
use crate::utils::get_parent_folder;
use anyhow::{bail, Result};
use id3::frame::Lyrics;
use lofty::config::WriteOptions;
use lofty::picture::{Picture, PictureType};
use lofty::prelude::{Accessor, AudioFile, ItemKey, TagExt, TaggedFileExt};
use lofty::tag::{ItemValue, Tag as LoftyTag, TagItem};
use lofty::{file::FileType, probe::Probe};
use std::convert::From;
use std::ffi::OsStr;
use std::fs::rename;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::time::{Duration, SystemTime};

// TODO: use this for database migration
/// This is the old string that was used as default for "artist" & "album"
///
/// this value is currently unused, but this will stay here as a reminder until the database is migrated
#[allow(unused)]
pub const UNSUPPORTED: &str = "Unsupported?";

/// Location types for a Track, could be a local file with [`LocationType::Path`] or a remote URI with [`LocationType::Uri`]
#[derive(Clone, Debug, PartialEq)]
pub enum LocationType {
    /// A Local file, for use with [`MediaType::Music`]
    Path(PathBuf),
    /// A remote URI, for use with [`MediaType::LiveRadio`] and [`MediaType::Podcast`]
    Uri(String),
}

impl From<PathBuf> for LocationType {
    fn from(value: PathBuf) -> Self {
        Self::Path(value)
    }
}

#[derive(Clone, Debug)]
pub struct Track {
    /// The URI or the Path of the file
    location: LocationType,
    pub media_type: MediaType,

    /// Artist of the song
    artist: Option<String>,
    /// Album of the song
    album: Option<String>,
    /// Title of the song
    title: Option<String>,
    /// Duration of the song
    duration: Duration,
    pub last_modified: SystemTime,
    /// USLT lyrics
    lyric_frames: Vec<Lyrics>,
    lyric_selected_index: usize,
    parsed_lyric: Option<Lyric>,
    picture: Option<Picture>,
    album_photo: Option<String>,
    file_type: Option<FileType>,
    // Date
    // Track
    genre: Option<String>,
    // Composer
    // Performer
    // Disc
    // Comment
    pub podcast_localfile: Option<String>,
}

impl PartialEq for Track {
    fn eq(&self, other: &Self) -> bool {
        self.location == other.location
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MediaType {
    Music,
    Podcast,
    LiveRadio,
}

impl Track {
    /// Create a new [`MediaType::Podcast`] track
    #[allow(clippy::cast_sign_loss)]
    pub fn from_episode(ep: &Episode) -> Self {
        let lyric_frames: Vec<Lyrics> = Vec::new();
        let mut podcast_localfile: Option<String> = None;
        if let Some(path) = &ep.path {
            if path.exists() {
                podcast_localfile = Some(path.to_string_lossy().to_string());
            }
        }

        Self {
            artist: Some("Episode".to_string()),
            album: None,
            title: Some(ep.title.clone()),
            location: LocationType::Uri(ep.url.clone()),
            duration: Duration::from_secs(ep.duration.unwrap_or(0) as u64),
            last_modified: SystemTime::now(),
            lyric_frames,
            lyric_selected_index: 0,
            parsed_lyric: None,
            picture: None,
            album_photo: ep.image_url.clone(),
            file_type: None,
            genre: None,
            media_type: MediaType::Podcast,
            podcast_localfile,
        }
    }

    /// Create a new [`MediaType::Music`] track
    pub fn read_from_path<P: AsRef<Path>>(path: P, for_db: bool) -> Result<Self> {
        let path = path.as_ref();

        let probe = Probe::open(path)?;

        let mut song = Self::new(LocationType::Path(path.to_path_buf()), MediaType::Music);
        let tagged_file = match probe.read() {
            Ok(v) => Some(v),
            Err(err) => {
                warn!(
                    "Failed to read metadata from \"{}\": {}",
                    path.display(),
                    err
                );
                None
            }
        };

        if let Some(mut tagged_file) = tagged_file {
            // We can at most get the duration and file type at this point
            let properties = tagged_file.properties();
            song.duration = properties.duration();
            song.file_type = Some(tagged_file.file_type());

            if let Some(tag) = tagged_file.primary_tag_mut() {
                Self::process_tag(tag, &mut song, for_db)?;
            } else if let Some(tag) = tagged_file.first_tag_mut() {
                Self::process_tag(tag, &mut song, for_db)?;
            } else {
                warn!("File \"{}\" does not have any tags!", path.display());
            }
        }

        // exit early if its for db only as no cover is needed there
        if for_db {
            return Ok(song);
        }

        let parent_folder = get_parent_folder(path);

        if let Ok(files) = std::fs::read_dir(parent_folder) {
            for f in files.flatten() {
                let path = f.path();
                if let Some(extension) = path.extension() {
                    if extension == "jpg" || extension == "png" {
                        song.album_photo = Some(path.to_string_lossy().to_string());
                    }
                }
            }
        }

        Ok(song)
    }

    /// Process a given [`LoftyTag`] into the given `track`
    fn process_tag(tag: &mut LoftyTag, track: &mut Track, for_db: bool) -> Result<()> {
        // Check for a length tag (Ex. TLEN in ID3v2)
        if let Some(len_tag) = tag.get_string(&ItemKey::Length) {
            track.duration = Duration::from_millis(len_tag.parse::<u64>()?);
        }

        track.artist = tag.artist().map(std::borrow::Cow::into_owned);
        track.album = tag.album().map(std::borrow::Cow::into_owned);
        track.title = tag.title().map(std::borrow::Cow::into_owned);
        track.genre = tag.genre().map(std::borrow::Cow::into_owned);
        track.media_type = MediaType::Music;

        if for_db {
            return Ok(());
        }

        // Get all of the lyrics tags
        let mut lyric_frames: Vec<Lyrics> = Vec::new();
        create_lyrics(tag, &mut lyric_frames);

        track.parsed_lyric = lyric_frames
            .first()
            .map(|lf| Lyric::from_str(&lf.text).ok())
            .and_then(|pl| pl);
        track.lyric_frames = lyric_frames;

        // Get the picture (not necessarily the front cover)
        let mut picture = tag
            .pictures()
            .iter()
            .find(|pic| pic.pic_type() == PictureType::CoverFront)
            .cloned();
        if picture.is_none() {
            picture = tag.pictures().first().cloned();
        }

        track.picture = picture;

        Ok(())
    }

    /// Create a new [`MediaType::LiveRadio`] track
    pub fn new_radio(url: &str) -> Self {
        let mut track = Self::new(LocationType::Uri(url.to_string()), MediaType::LiveRadio);
        track.artist = Some("Radio".to_string());
        track.title = Some("Radio Station".to_string());
        track.album = Some("Live".to_string());
        track
    }

    fn new(location: LocationType, media_type: MediaType) -> Self {
        let duration = Duration::from_secs(0);
        let lyric_frames: Vec<Lyrics> = Vec::new();
        let mut last_modified = SystemTime::now();
        let mut title = None;

        if let LocationType::Path(path) = &location {
            if let Ok(meta) = path.metadata() {
                if let Ok(modified) = meta.modified() {
                    last_modified = modified;
                }
            }

            title = path.file_stem().and_then(OsStr::to_str).map(String::from);
        }

        Self {
            file_type: None,
            artist: None,
            album: None,
            title,
            duration,
            location,
            parsed_lyric: None,
            lyric_frames,
            lyric_selected_index: 0,
            picture: None,
            album_photo: None,
            last_modified,
            genre: None,
            media_type,
            podcast_localfile: None,
        }
    }

    pub fn adjust_lyric_delay(&mut self, time_pos: Duration, offset: i64) -> Result<()> {
        if let Some(lyric) = self.parsed_lyric.as_mut() {
            lyric.adjust_offset(time_pos, offset);
            let text = lyric.as_lrc_text();
            self.set_lyric(&text, "Adjusted");
            self.save_tag()?;
        }
        Ok(())
    }

    pub fn cycle_lyrics(&mut self) -> Result<&Lyrics> {
        if self.lyric_frames_is_empty() {
            bail!("no lyrics embedded");
        }

        self.lyric_selected_index += 1;
        if self.lyric_selected_index >= self.lyric_frames.len() {
            self.lyric_selected_index = 0;
        }

        if let Some(f) = self.lyric_frames.get(self.lyric_selected_index) {
            if let Ok(parsed_lyric) = Lyric::from_str(&f.text) {
                self.parsed_lyric = Some(parsed_lyric);
                return Ok(f);
            }
        }

        bail!("cycle lyrics error")
    }

    pub const fn parsed_lyric(&self) -> Option<&Lyric> {
        self.parsed_lyric.as_ref()
    }

    pub fn set_parsed_lyric(&mut self, pl: Option<Lyric>) {
        self.parsed_lyric = pl;
    }

    pub fn lyric_frames_remove_selected(&mut self) {
        self.lyric_frames.remove(self.lyric_selected_index);
    }

    pub fn set_lyric_selected_index(&mut self, index: usize) {
        self.lyric_selected_index = index;
    }

    pub const fn lyric_selected_index(&self) -> usize {
        self.lyric_selected_index
    }

    pub fn lyric_selected(&self) -> Option<&Lyrics> {
        if self.lyric_frames.is_empty() {
            return None;
        }
        if let Some(lf) = self.lyric_frames.get(self.lyric_selected_index) {
            return Some(lf);
        }
        None
    }

    pub fn lyric_frames_is_empty(&self) -> bool {
        self.lyric_frames.is_empty()
    }

    pub fn lyric_frames_len(&self) -> usize {
        if self.lyric_frames.is_empty() {
            return 0;
        }
        self.lyric_frames.len()
    }

    pub fn lyric_frames(&self) -> Option<Vec<Lyrics>> {
        if self.lyric_frames.is_empty() {
            return None;
        }
        Some(self.lyric_frames.clone())
    }

    pub const fn picture(&self) -> Option<&Picture> {
        self.picture.as_ref()
    }
    pub fn album_photo(&self) -> Option<&str> {
        self.album_photo.as_deref()
    }

    /// Optionally return the artist of the song
    /// If `None` it wasn't able to read the tags
    pub fn artist(&self) -> Option<&str> {
        self.artist.as_deref()
    }

    pub fn set_artist(&mut self, a: &str) {
        self.artist = Some(a.to_string());
    }

    /// Optionally return the song's album
    /// If `None` failed to read the tags
    pub fn album(&self) -> Option<&str> {
        self.album.as_deref()
    }

    pub fn set_album(&mut self, album: &str) {
        self.album = Some(album.to_string());
    }

    pub fn genre(&self) -> Option<&str> {
        self.genre.as_deref()
    }

    #[allow(unused)]
    pub fn set_genre(&mut self, genre: &str) {
        self.genre = Some(genre.to_string());
    }

    /// Optionally return the title of the song
    /// If `None` it wasn't able to read the tags
    pub fn title(&self) -> Option<&str> {
        self.title.as_deref()
    }

    pub fn set_title(&mut self, title: &str) {
        self.title = Some(title.to_string());
    }

    /// Get the full Path or URI of the track, if its a local file
    pub fn file(&self) -> Option<&str> {
        match &self.location {
            LocationType::Path(path_buf) => path_buf.to_str(),
            LocationType::Uri(uri) => Some(uri),
        }
    }

    /// Get the directory the track is in, if its a local file
    pub fn directory(&self) -> Option<&str> {
        if let LocationType::Path(path) = &self.location {
            // not using "utils::get_parent_directory" as if a track is "LocationType::Path", it should have a directory and a file in the path
            path.parent().and_then(Path::to_str)
        } else {
            None
        }
    }

    /// Get the extension of the track, if its a local file
    pub fn ext(&self) -> Option<&str> {
        if let LocationType::Path(path) = &self.location {
            path.extension().and_then(OsStr::to_str)
        } else {
            None
        }
    }

    pub const fn duration(&self) -> Duration {
        self.duration
    }

    pub fn duration_formatted(&self) -> String {
        Self::duration_formatted_short(&self.duration)
    }

    pub fn duration_formatted_short(d: &Duration) -> String {
        let duration_hour = d.as_secs() / 3600;
        let duration_min = (d.as_secs() % 3600) / 60;
        let duration_secs = d.as_secs() % 60;

        if duration_hour == 0 {
            format!("{duration_min:0>2}:{duration_secs:0>2}")
        } else {
            format!("{duration_hour}:{duration_min:0>2}:{duration_secs:0>2}")
        }
    }

    /// Get the `file_name` or the full URI of the current Track
    pub fn name(&self) -> Option<&str> {
        match &self.location {
            LocationType::Path(path) => path.file_name().and_then(OsStr::to_str),
            // TODO: should this really return the uri here instead of None?
            LocationType::Uri(uri) => Some(uri),
        }
    }

    pub fn save_tag(&mut self) -> Result<()> {
        if let Some(file_path) = self.file() {
            let tag_type = match self.file_type {
                Some(file_type) => file_type.primary_tag_type(),
                None => return Ok(()),
            };

            let mut tag = LoftyTag::new(tag_type);
            self.update_tag(&mut tag);

            if !self.lyric_frames_is_empty() {
                if let Some(lyric_frames) = self.lyric_frames() {
                    for l in lyric_frames {
                        tag.push(TagItem::new(ItemKey::Lyrics, ItemValue::Text(l.text)));
                    }
                }
            }

            if let Some(any_picture) = self.picture().cloned() {
                tag.push_picture(any_picture);
            }

            tag.save_to_path(file_path, WriteOptions::new())?;
        }

        self.rename_by_tag()?;
        Ok(())
    }

    fn rename_by_tag(&mut self) -> Result<()> {
        if let Some(ext) = self.ext() {
            let new_name = format!(
                "{}-{}.{}",
                self.artist().unwrap_or(UNKNOWN_ARTIST),
                self.title().unwrap_or(UNKNOWN_TITLE),
                ext,
            );

            let new_name_path: &Path = Path::new(new_name.as_str());
            if let Some(file) = self.file() {
                let p_old: &Path = Path::new(file);
                if let Some(p_prefix) = p_old.parent() {
                    let p_new = p_prefix.join(new_name_path);
                    rename(p_old, &p_new)?;
                    self.location = LocationType::Path(p_new);
                }
            }
        }

        Ok(())
    }

    pub fn set_lyric(&mut self, lyric_str: &str, lang_ext: &str) {
        let mut lyric_frames = self.lyric_frames.clone();
        match self.lyric_frames.get(self.lyric_selected_index) {
            Some(lyric_frame) => {
                lyric_frames.remove(self.lyric_selected_index);
                lyric_frames.insert(
                    self.lyric_selected_index,
                    Lyrics {
                        text: lyric_str.to_string(),
                        ..lyric_frame.clone()
                    },
                );
            }
            None => {
                lyric_frames.push(Lyrics {
                    lang: "eng".to_string(),
                    description: lang_ext.to_string(),
                    text: lyric_str.to_string(),
                });
            }
        }
        self.lyric_frames = lyric_frames;
    }

    pub fn set_photo(&mut self, picture: Picture) {
        self.picture = Some(picture);
    }

    fn update_tag<T: Accessor>(&self, tag: &mut T) {
        tag.set_artist(
            self.artist()
                .map_or_else(|| String::from(UNKNOWN_ARTIST), str::to_string),
        );

        tag.set_title(
            self.title()
                .map_or_else(|| String::from(UNKNOWN_TITLE), str::to_string),
        );

        tag.set_album(self.album().map_or_else(String::new, str::to_string));
        tag.set_genre(self.genre().map_or_else(String::new, str::to_string));
    }
}

fn create_lyrics(tag: &mut LoftyTag, lyric_frames: &mut Vec<Lyrics>) {
    let lyrics = tag.take(&ItemKey::Lyrics);
    for lyric in lyrics {
        if let ItemValue::Text(lyrics_text) = lyric.value() {
            lyric_frames.push(Lyrics {
                lang: lyric.lang().escape_ascii().to_string(),
                description: lyric.description().to_string(),
                text: lyrics_text.to_string(),
            });
        }
    }
}
