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
use id3::frame::Lyrics;
use lofty::id3::v2::{Frame, FrameFlags, FrameValue, Id3v2Tag, LanguageFrame, TextEncoding};
use lofty::{
    mp3::Mp3File, Accessor, AudioFile, FileType, ItemKey, ItemValue, Picture, PictureType, TagExt,
    TagItem,
};
use std::convert::From;
use std::ffi::OsStr;
use std::fs::rename;
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::time::Duration;

#[derive(Clone)]
pub struct Track {
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
    directory: Option<String>,
    /// USLT lyrics
    lyric_frames: Vec<Lyrics>,
    lyric_selected_index: usize,
    parsed_lyric: Option<Lyric>,
    picture: Option<Picture>,
    album_photo: Option<String>,
    file_type: Option<FileType>,
}

impl Track {
    pub fn read_from_path<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();

        let probe = lofty::Probe::open(path)?;
        let file_type = probe.file_type();

        let mut song = Self::new(path);
        if let Ok(mut tagged_file) = probe.read(true) {
            // We can at most get the duration and file type at this point
            let properties = tagged_file.properties();
            song.duration = properties.duration();
            song.file_type = Some(tagged_file.file_type());

            if let Some(tag) = tagged_file.primary_tag_mut() {
                // Check for a length tag (Ex. TLEN in ID3v2)
                if let Some(len_tag) = tag.get_string(&ItemKey::Length) {
                    song.duration = Duration::from_millis(len_tag.parse::<u64>()?);
                }

                song.artist = tag.artist().map(str::to_string);
                song.album = tag.album().map(str::to_string);
                song.title = tag.title().map(str::to_string);

                // Get all of the lyrics tags
                let mut lyric_frames: Vec<Lyrics> = Vec::new();
                match file_type {
                    Some(FileType::MP3) => {
                        let mut reader = BufReader::new(File::open(path)?);
                        let file = Mp3File::read_from(&mut reader, false)?;

                        if let Some(id3v2_tag) = file.id3v2_tag() {
                            for lyrics_frame in id3v2_tag.unsync_text() {
                                lyric_frames.push(Lyrics {
                                    lang: lyrics_frame.language.clone(),
                                    description: lyrics_frame.description.clone(),
                                    text: lyrics_frame.content.clone(),
                                });
                            }
                        }
                    }
                    _ => {
                        create_lyrics(tag, &mut lyric_frames);
                    }
                };
                song.parsed_lyric = lyric_frames
                    .first()
                    .map(|lf| Lyric::from_str(&lf.text).ok())
                    .and_then(|pl| pl);
                song.lyric_frames = lyric_frames;

                // Get the picture (not necessarily the front cover)
                let mut picture = tag
                    .pictures()
                    .iter()
                    .find(|pic| pic.pic_type() == PictureType::CoverFront)
                    .cloned();
                if picture.is_none() {
                    picture = tag.pictures().first().cloned();
                }

                song.picture = picture;
            }
        }

        let mut parent_folder: PathBuf = PathBuf::new();

        if path.is_dir() {
            parent_folder = path.to_path_buf();
        } else if let Some(parent) = path.parent() {
            parent_folder = parent.to_path_buf();
        };

        if let Ok(files) = std::fs::read_dir(&parent_folder) {
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

    fn new<P: AsRef<Path>>(path: P) -> Self {
        let p = path.as_ref();
        let directory = Some(p.parent().unwrap().to_string_lossy().into_owned());
        let ext = p.extension().and_then(OsStr::to_str).map(String::from);
        let artist = Some(String::from("Unsupported?"));
        let album = Some(String::from("Unsupported?"));
        let title = p.file_stem().and_then(OsStr::to_str).map(String::from);
        let file = Some(p.to_string_lossy().into_owned());
        let duration = Duration::from_secs(0);
        let name = p
            .file_name()
            .and_then(OsStr::to_str)
            .map(std::string::ToString::to_string);
        let parsed_lyric: Option<Lyric> = None;
        let lyric_frames: Vec<Lyrics> = Vec::new();
        let picture: Option<Picture> = None;
        let album_photo: Option<String> = None;
        Self {
            ext,
            file_type: None,
            artist,
            album,
            title,
            file,
            directory,
            duration,
            name,
            parsed_lyric,
            lyric_frames,
            lyric_selected_index: 0,
            picture,
            album_photo,
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
    pub fn album_photo(&self) -> Option<&str> {
        match self.album_photo.as_ref() {
            Some(a) => Some(a),
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

    pub fn directory(&self) -> Option<&str> {
        match self.directory.as_ref() {
            Some(dir) => Some(dir),
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

    pub fn duration_formatted(&self) -> String {
        Self::duration_formatted_short(&self.duration)
    }

    pub fn duration_formatted_short(d: &Duration) -> String {
        let duration_hour = d.as_secs() / 3600;
        let duration_min = (d.as_secs() % 3600) / 60;
        let duration_secs = d.as_secs() % 60;

        let duration_string = if duration_hour == 0 {
            format!("{:0>2}:{:0>2}", duration_min, duration_secs)
        } else {
            format!(
                "{}:{:0>2}:{:0>2}",
                duration_hour, duration_min, duration_secs
            )
        };
        duration_string
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
                if let Some(file_path) = self.file() {
                    let mut tag = Id3v2Tag::default();
                    self.update_tag(&mut tag);

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

                    tag.save_to_path(file_path)?;
                }
            }

            _ => {
                if let Some(file_path) = self.file() {
                    let tag_type = match self.file_type {
                        Some(file_type) => file_type.primary_tag_type(),
                        None => return Ok(()),
                    };

                    let mut tag = lofty::Tag::new(tag_type);
                    self.update_tag(&mut tag);

                    if !self.lyric_frames_is_empty() {
                        if let Some(lyric_frames) = self.lyric_frames() {
                            for l in lyric_frames {
                                tag.push_item(TagItem::new(
                                    ItemKey::Lyrics,
                                    ItemValue::Text(l.text),
                                ));
                            }
                        }
                    }

                    if let Some(any_picture) = self.picture().cloned() {
                        tag.push_picture(any_picture);
                    }

                    tag.save_to_path(file_path)?;
                }
            }
        }

        self.rename_by_tag()?;
        Ok(())
    }

    fn rename_by_tag(&mut self) -> Result<()> {
        if let Some(ext) = self.ext() {
            let new_name = format!(
                "{}-{}.{}",
                self.artist().unwrap_or("Unknown Artist"),
                self.title().unwrap_or("Unknown Title"),
                ext,
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
    }
}

fn create_lyrics(tag: &mut lofty::Tag, lyric_frames: &mut Vec<Lyrics>) {
    let lyrics = tag.take(&ItemKey::Lyrics);
    for lyric in lyrics {
        if let ItemValue::Text(lyrics_text) = lyric.value() {
            lyric_frames.push(Lyrics {
                lang: "eng".to_string(),
                description: String::new(),
                text: lyrics_text.to_string(),
            });
        }
    }
}
