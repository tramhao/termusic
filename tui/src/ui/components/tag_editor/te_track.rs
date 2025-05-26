use std::{
    borrow::Cow,
    path::{Path, PathBuf},
    str::FromStr,
    time::Duration,
};

use anyhow::{bail, Result};
use id3::frame::Lyrics;
use lofty::{
    config::WriteOptions,
    file::FileType,
    id3::v2::{Frame, Id3v2Tag, UnsynchronizedTextFrame},
    picture::Picture,
    tag::{Accessor, ItemKey, ItemValue, Tag, TagExt, TagItem},
};
use termusiclib::{
    songtag::lrc::Lyric,
    track::{parse_metadata_from_file, LyricData, MetadataOptions, Track},
};

use crate::ui::model::ExtraLyricData;

/// Track data for the Tag-Editor with helper functions
#[derive(Debug, Clone, PartialEq)]
pub struct TETrack {
    path: PathBuf,

    artist: Option<String>,
    title: Option<String>,
    album: Option<String>,
    genre: Option<String>,

    picture: Option<Picture>,

    lyric_selected_idx: usize,
    lyric_frames: Vec<Lyrics>,
    lyric_parsed: Option<Lyric>,

    file_type: FileType,
}

impl TryFrom<&Track> for TETrack {
    type Error = anyhow::Error;

    fn try_from(value: &Track) -> Result<Self, Self::Error> {
        let Some(track_data) = value.as_track() else {
            bail!("Track is not a Music Track!");
        };

        let Some(file_type) = track_data.file_type() else {
            bail!("Track does not have a lofty FileType, cannot process with lofty!");
        };

        Ok(Self {
            path: track_data.path().to_owned(),
            artist: value.artist().map(|v| v.to_string()),
            title: value.title().map(|v| v.to_string()),
            album: track_data.album().map(|v| v.to_string()),
            // TODO: init genre
            genre: None,
            picture: None,
            lyric_selected_idx: 0,
            lyric_frames: Vec::new(),
            lyric_parsed: None,

            file_type,
        })
    }
}

impl TETrack {
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
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
    pub fn album(&self) -> Option<&str> {
        self.album.as_deref()
    }

    #[must_use]
    pub fn genre(&self) -> Option<&str> {
        self.genre.as_deref()
    }

    pub fn set_artist<S: Into<String>>(&mut self, value: S) {
        self.artist = Some(value.into());
    }

    pub fn set_title<S: Into<String>>(&mut self, value: S) {
        self.title = Some(value.into());
    }

    pub fn set_album<S: Into<String>>(&mut self, value: S) {
        self.album = Some(value.into());
    }

    pub fn set_genre<S: Into<String>>(&mut self, value: S) {
        self.genre = Some(value.into());
    }

    pub fn set_picture(&mut self, value: Picture) {
        self.picture = Some(value);
    }

    /// Set the current selected lyric with the given data, or add one with the given data if there is none.
    pub fn set_lyric<S: Into<String>, L: Into<String>, D: Into<String>>(
        &mut self,
        content: S,
        lang: L,
        description: Option<D>,
    ) {
        if let Some(frame) = self.lyric_frames.get_mut(self.lyric_selected_idx) {
            let description =
                description.map_or_else(|| std::mem::take(&mut frame.description), |v| v.into());
            *frame = Lyrics {
                text: content.into(),
                lang: lang.into(),
                description,
            };
        } else {
            let lang = lang.into();
            let description = description.map_or_else(|| lang.clone(), |v| v.into());
            self.lyric_frames.push(Lyrics {
                text: content.into(),
                lang,
                description,
            });
        }
    }

    pub fn set_lyric_selected_index(&mut self, index: usize) {
        self.lyric_selected_idx = index;
    }

    pub fn lyric_frames_remove_selected(&mut self) {
        self.lyric_frames.remove(self.lyric_selected_idx);
    }

    #[must_use]
    pub fn lyric_frames(&self) -> &[Lyrics] {
        &self.lyric_frames
    }

    #[must_use]
    pub fn lyric_selected_index(&self) -> usize {
        self.lyric_selected_idx
    }

    #[must_use]
    pub fn lyric_selected(&self) -> Option<&Lyrics> {
        self.lyric_frames.get(self.lyric_selected_idx)
    }

    pub fn set_parsed_lyrics(&mut self, parsed: Option<Lyric>) {
        self.lyric_parsed = parsed;
    }

    /// Apply Lyric data from [`ExtraLyricData`]. But only if it is `Some` and for the same track-path.
    pub fn lyric_set_with_extra(&mut self, extra_lyric: Option<&ExtraLyricData>) -> Option<()> {
        let extra_lyric = extra_lyric?;
        // dont apply lyric data for a different track
        if extra_lyric.for_track != self.path {
            return None;
        }

        self.lyric_selected_idx = extra_lyric.selected_idx;
        self.lyric_frames.clone_from(&extra_lyric.data.raw_lyrics);
        self.lyric_parsed
            .clone_from(&extra_lyric.data.parsed_lyrics);

        Some(())
    }

    /// Adjust the lyric delay at `time_pos` by `offset`.
    ///
    /// See [`Lyric::adjust_offset`].
    pub fn lyric_adjust_delay(&mut self, time_pos: Duration, offset: i64) {
        if let Some(lyric) = self.lyric_parsed.as_mut() {
            lyric.adjust_offset(time_pos, offset);
            let raw = self.lyric_frames.get(self.lyric_selected_idx);
            let lang = raw.map_or_else(|| "eng".to_string(), |v| v.lang.clone());
            let description = raw.map_or_else(
                || "Adjusted".to_string(),
                |v| {
                    if v.description.starts_with("Adjusted") {
                        v.description.clone()
                    } else {
                        format!("Adjusted {}", v.description)
                    }
                },
            );
            let content = lyric.as_lrc_text();
            self.set_lyric(content, lang, Some(description));
        }
    }

    /// Convert the current instance to only [`ExtraLyricData`].
    pub fn into_extra_lyric_data(self) -> ExtraLyricData {
        ExtraLyricData {
            for_track: self.path,
            data: LyricData {
                raw_lyrics: self.lyric_frames,
                parsed_lyrics: self.lyric_parsed,
            },
            selected_idx: self.lyric_selected_idx,
        }
    }

    /// Save the current tag data to the given path.
    pub fn save_tag(&mut self) -> Result<()> {
        match self.file_type {
            FileType::Mpeg => self.save_tag_mpeg(),
            _ => self.save_tag_generic(),
        }
    }

    /// Save the tag in a lofty handled generic way
    fn save_tag_generic(&mut self) -> Result<()> {
        let tag_type = self.file_type.primary_tag_type();

        let mut tag = Tag::new(tag_type);
        self.set_data_on_tag(&mut tag);

        if let Some(picture) = self.picture.clone() {
            tag.push_picture(picture);
        }

        if !self.lyric_frames.is_empty() {
            for lyric in &self.lyric_frames {
                let mut tag_item =
                    TagItem::new(ItemKey::Lyrics, ItemValue::Text(lyric.text.clone()));
                tag_item.set_description(lyric.description.clone());
                if lyric.lang.len() == 3 {
                    let lang = lyric.lang.as_bytes()[0..3].try_into().unwrap();
                    tag_item.set_lang(lang);
                }
                tag.push(tag_item);
            }
        }

        tag.save_to_path(self.path(), WriteOptions::new())?;

        Ok(())
    }

    /// Handle saving mpeg tags some there is some problem saving it via the generic way
    ///
    /// [From the discussion](https://github.com/tramhao/termusic/commit/5f4276c012fa7dff90fa9cbb8bde823df5387ce8):
    /// - Recently I downloaded some tracks from youtube, and they have several languages of lyrics. When I delete one language, the other were saved wrong. The language and description of uslt frame cannot be preserved in lofty tags.
    /// - Theoretically, it should just work in the other way since lofty 0.20.0, see Serial-ATA/lofty-rs#392 (we currently use lofty 0.22.x)
    /// - Tried with set_lang and set_description of lofty tag but not working. The tag item was modified but after push, lang is set to XXX and description is empty. I'll keep the separate handling of writing for now.
    fn save_tag_mpeg(&mut self) -> Result<()> {
        let mut tag = Id3v2Tag::default();
        self.set_data_on_tag(&mut tag);

        if let Some(picture) = self.picture.clone() {
            tag.insert_picture(picture);
        }

        if !self.lyric_frames.is_empty() {
            for lyric in &self.lyric_frames {
                let lang = if lyric.lang.len() == 3 {
                    let lang = lyric.lang.as_bytes()[0..3].try_into().unwrap();
                    lang
                } else {
                    *b"eng"
                };
                let frame = Frame::UnsynchronizedText(UnsynchronizedTextFrame::new(
                    lofty::TextEncoding::UTF8,
                    lang,
                    lyric.description.clone(),
                    lyric.text.clone(),
                ));
                tag.insert(frame);
            }
        }

        tag.save_to_path(self.path(), WriteOptions::new())?;

        Ok(())
    }

    /// Helper function to set common tags from `Self` to `T`
    fn set_data_on_tag<T: Accessor>(&self, tag: &mut T) {
        if let Some(artist) = self.artist.clone() {
            tag.set_artist(artist);
        }
        if let Some(title) = self.title.clone() {
            tag.set_title(title);
        }
        if let Some(album) = self.album.clone() {
            tag.set_album(album);
        }
        if let Some(genre) = self.genre.clone() {
            tag.set_genre(genre);
        }
    }

    /// Get the path id as a string
    pub fn path_as_id_str(&self) -> Cow<'_, str> {
        self.path.to_string_lossy()
    }

    /// Read metadata from a file with all the metadata that can be handled.
    ///
    /// Note that this completely bypasses any [`Track`] functions and caching (both in getting and in setting).
    pub fn read_metadata_from_file<P: Into<PathBuf>>(path: P) -> Result<Self> {
        let path = path.into();
        let metadata = parse_metadata_from_file(
            &path,
            MetadataOptions {
                album: true,
                artist: true,
                title: true,
                genre: true,
                cover: true,
                lyrics: true,
                ..Default::default()
            },
        )?;

        let Some(file_type) = metadata.file_type else {
            bail!("Unable to create TETrack due to missing file_type");
        };

        let lyric_frames = metadata.lyric_frames.unwrap_or_default();

        let lyric_parsed = lyric_frames
            .first()
            .and_then(|v| Lyric::from_str(&v.text).ok());

        let res = Self {
            path,
            artist: metadata.artist,
            title: metadata.title,
            album: metadata.album,
            genre: metadata.genre,
            picture: metadata.cover,
            lyric_selected_idx: 0,
            lyric_frames,
            lyric_parsed,
            file_type,
        };

        Ok(res)
    }
}
