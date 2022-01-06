// use crate::player::GStreamer;
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
 * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
 * SOFTWARE.
 */
use crate::songtag::lrc::Lyric;
use anyhow::{bail, Result};
use humantime::{format_duration, FormattedDuration};
use id3::frame::Lyrics;
use if_chain::if_chain;
use lofty::id3::v2::{Frame, FrameFlags, FrameValue, Id3v2Tag, LanguageFrame, TextEncoding};
use lofty::{
    mp3::Mp3File, Accessor, AudioFile, FileType, ItemKey, ItemValue, Picture, PictureType, TagItem,
    TagType,
};
use std::convert::From;
use std::ffi::OsStr;
use std::fs::rename;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::str::FromStr;
use std::time::Duration;

#[derive(Clone)]
pub struct Song {
    /// Artist of the song
    artist: Option<String>,
    /// Album of the song
    album: Option<String>,
    /// Title of the song
    title: Option<String>,
    /// File path to the song
    file: Option<String>,
    /// Duration of the song
    duration: Duration,
    /// Name of the song
    name: Option<String>,
    /// Extension of the song
    ext: Option<String>,
    /// USLT lyrics
    lyric_frames: Vec<Lyrics>,
    lyric_selected_index: usize,
    parsed_lyric: Option<Lyric>,
    picture: Option<Picture>,
    file_type: Option<FileType>,
}

impl Song {
    fn new(s: &str) -> Self {
        let p: &Path = Path::new(s);
        let ext = p.extension().and_then(OsStr::to_str).map(String::from);
        let artist = Some(String::from("Not Support?"));
        let album = Some(String::from("Not Support?"));
        let title = p.file_stem().and_then(OsStr::to_str).map(String::from);
        let file = Some(String::from(s));
        let duration = Duration::from_secs(0);
        let name = Some(String::from(""));
        let parsed_lyric: Option<Lyric> = None;
        let lyric_frames: Vec<Lyrics> = Vec::new();
        let picture: Option<Picture> = None;
        Self {
            ext,
            file_type: None,
            artist,
            album,
            title,
            file,
            duration,
            name,
            parsed_lyric,
            lyric_frames,
            lyric_selected_index: 0,
            picture,
        }
    }

    pub fn adjust_lyric_delay(&mut self, time_pos: i64, offset: i64) -> Result<()> {
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
            bail!("no lyrics embeded");
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
        match self.parsed_lyric.as_ref() {
            Some(pl) => Some(pl),
            None => None,
        }
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
        match self.picture.as_ref() {
            Some(picture) => Some(picture),
            None => None,
        }
    }

    /// Optionally return the artist of the song
    /// If `None` it wasn't able to read the tags
    pub fn artist(&self) -> Option<&str> {
        match self.artist.as_ref() {
            Some(artist) => Some(artist),
            None => None,
        }
    }

    pub fn set_artist(&mut self, a: &str) {
        self.artist = Some(a.to_string());
    }
    /// Optionally return the song's album
    /// If `None` failed to read the tags
    pub fn album(&self) -> Option<&str> {
        match self.album.as_ref() {
            Some(album) => Some(album),
            None => None,
        }
    }
    pub fn set_album(&mut self, album: &str) {
        self.album = Some(album.to_string());
    }
    /// Optionally return the title of the song
    /// If `None` it wasn't able to read the tags
    pub fn title(&self) -> Option<&str> {
        match self.title.as_ref() {
            Some(title) => Some(title),
            None => None,
        }
    }
    pub fn set_title(&mut self, title: &str) {
        self.title = Some(title.to_string());
    }

    pub fn file(&self) -> Option<&str> {
        match self.file.as_ref() {
            Some(file) => Some(file),
            None => None,
        }
    }

    fn ext(&self) -> Option<&str> {
        match self.ext.as_ref() {
            Some(ext) => Some(ext),
            None => None,
        }
    }

    pub const fn duration(&self) -> Duration {
        self.duration
    }

    pub fn duration_formatted(&self) -> FormattedDuration {
        format_duration(self.duration)
    }

    pub fn name(&self) -> Option<&str> {
        match self.name.as_ref() {
            Some(name) => Some(name),
            None => None,
        }
    }

    // update_duration is only used for mp3 and wav, as other formats don't have length or
    // duration tag
    // #[allow(clippy::cast_possible_truncation)]
    // pub fn update_duration(&self) -> Result<()> {
    //     let s = self.file().ok_or_else(|| anyhow!("no file found"))?;

    //     if let Some("mp3" | "wav") = self.ext() {
    //         let mut id3_tag = id3::Tag::new();
    //         if let Ok(t) = id3::Tag::read_from_path(s) {
    //             id3_tag = t;
    //         }

    //         let duration_player = GStreamer::duration(s);
    //         id3_tag.remove_duration();
    //         id3_tag.set_duration((duration_player.mseconds()) as u32);
    //         let _drop = id3_tag.write_to_path(s, id3::Version::Id3v24);
    //     }
    //     Ok(())
    // }

    pub fn save_tag(&mut self) -> Result<()> {
        match self.file_type {
            Some(FileType::MP3) => {
                // Some(FileType::MP3 | FileType::WAV) => {
                if let Some(file_path) = self.file() {
                    let mut tag = Id3v2Tag::default();

                    tag.set_artist(
                        self.artist()
                            .map_or_else(|| String::from("Unknown Artist"), str::to_string),
                    );
                    tag.set_title(
                        self.title()
                            .map_or_else(|| String::from("Unknown Title"), str::to_string),
                    );
                    tag.set_album(
                        self.album()
                            .map_or_else(|| String::from("Unknown Album"), str::to_string),
                    );

                    if !self.lyric_frames_is_empty() {
                        if let Some(lyric_frames) = self.lyric_frames() {
                            for l in lyric_frames {
                                if let Ok(l_frame) = Frame::new(
                                    "USLT",
                                    FrameValue::UnSyncText(LanguageFrame {
                                        encoding: TextEncoding::UTF8,
                                        language: l.lang,
                                        description: l.description,
                                        content: l.text,
                                    }),
                                    FrameFlags::default(),
                                ) {
                                    tag.insert(l_frame);
                                }
                            }
                        }
                    }

                    if let Some(any_picture) = self.picture().cloned() {
                        tag.insert_picture(any_picture);
                    }

                    tag.write_to_path(file_path)?;
                }
            }

            _ => {
                if let Some(file_path) = self.file() {
                    let target_tag_type = match self.file_type {
                        Some(FileType::AIFF | FileType::MP3 | FileType::WAV) => TagType::Id3v2,
                        Some(FileType::APE) => TagType::Ape,
                        Some(FileType::MP4) => TagType::Mp4Ilst,
                        Some(FileType::Opus | FileType::Vorbis | FileType::FLAC) => {
                            TagType::VorbisComments
                        }
                        None => return Ok(()),
                    };

                    let mut tag = lofty::Tag::new(target_tag_type);

                    tag.set_artist(
                        self.artist()
                            .map_or_else(|| String::from("Unknown Artist"), str::to_string),
                    );
                    tag.set_title(
                        self.title()
                            .map_or_else(|| String::from("Unknown Title"), str::to_string),
                    );
                    tag.set_album(
                        self.album()
                            .map_or_else(|| String::from("Unknown Album"), str::to_string),
                    );

                    if !self.lyric_frames_is_empty() {
                        if let Some(lyric_frames) = self.lyric_frames() {
                            for l in lyric_frames {
                                tag.insert_text(ItemKey::Lyrics, l.text);
                            }
                        }
                    }

                    if let Some(any_picture) = self.picture().cloned() {
                        tag.push_picture(any_picture);
                        // if let Some(front_cover) = tag.get_picture_type(PictureType::CoverFront).cloned() {
                        // tag.push_picture(front_cover);
                    }

                    tag.save_to_path(file_path)?;
                }
            }
        }

        self.rename_by_tag()?;
        Ok(())
    }

    fn rename_by_tag(&mut self) -> Result<()> {
        let new_name = format!(
            "{}-{}.{}",
            self.artist().unwrap_or("Unknown Artist"),
            self.title().unwrap_or("Unknown Title"),
            self.ext().unwrap_or("mp3"),
        );
        let new_name_path: &Path = Path::new(new_name.as_str());
        if let Some(file) = self.file() {
            let p_old: &Path = Path::new(file);
            if let Some(p_prefix) = p_old.parent() {
                let p_new = p_prefix.join(new_name_path);
                rename(p_old, &p_new)?;
                self.file = Some(String::from(p_new.to_string_lossy()));
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
}

// #[allow(clippy::single_match_else)]
impl FromStr for Song {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let p: &Path = Path::new(s);

        let tagged_file = lofty::Probe::open(p)?;
        let file_type = tagged_file.file_type();

        if_chain! {
            if let Ok(file) = tagged_file.read(true);
            if let Some(tag) = file.primary_tag();
            then {
                let lyric_frames: Vec<Lyrics> = match file_type {
                    Some(FileType::MP3) => {
                        let mut reader = BufReader::new(File::open(p)?);
                        let file = Mp3File::read_from(&mut reader, false)?;

                        let mut l = vec![];
                        if let Some(id3v2_tag) = file.id3v2_tag() {
                            for lyrics_frame in id3v2_tag.unsync_text() {
                                l.push(Lyrics{
                                    lang: lyrics_frame.language.clone(),
                                    description: lyrics_frame.description.clone(),
                                    text: lyrics_frame.content.clone(),
                                });
                            }
                        }
                        l
                    }
                    _ =>
                    tag.items().iter().filter_map(create_lyrics).collect(),
                };
                let parsed_lyric = lyric_frames.first().map(|lf| Lyric::from_str(&lf.text).ok()).and_then(|pl| pl);
                let mut picture = tag.pictures().iter().find(|pic|pic.pic_type() == PictureType::CoverFront).cloned();
                if picture.is_none() {
                    picture = tag.pictures().first().cloned();
                }

                let properties = file.properties();
                let duration= if let Some(len_tag) = tag.get_string(&ItemKey::Length) {
                    Duration::from_millis(len_tag.parse::<u64>()?)
                } else {
                    properties.duration()
                };

                return Ok(Self {
                    artist: tag.artist().map(str::to_string),
                    album: tag.album().map(str::to_string),
                    title: tag.title().map(str::to_string),
                    file: Some(String::from(s)),
                    duration,
                    ext: p.extension().and_then(OsStr::to_str).map(String::from),
                    lyric_frames,
                    name: p.file_name().and_then(OsStr::to_str).map(std::string::ToString::to_string),
                    lyric_selected_index: 0,
                    parsed_lyric,
                    picture,
                    file_type: Some(*file.file_type())
                })
            }
        }

        Ok(Self::new(s))
    }
}

fn create_lyrics(item: &TagItem) -> Option<Lyrics> {
    if_chain! {
        if item.key() == &ItemKey::Lyrics;
        if let ItemValue::Text(lyrics_text) = item.value();
        if lyrics_text.len() > 10;
        then {
            Some(Lyrics {
                lang: "eng".to_string(),
                description: "termusic".to_string(),
                text: lyrics_text.to_string(),
            })
        } else {
            None
        }
    }
}
