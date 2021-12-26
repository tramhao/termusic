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
use crate::player::GStreamer;
use crate::songtag::lrc::Lyric;
use anyhow::{anyhow, bail, Result};
use humantime::{format_duration, FormattedDuration};
use id3::frame::{Lyrics, Picture, PictureType};
use lofty::{Accessor, ItemKey, Probe, Tag};
use metaflac::Tag as FlacTag;
use mp4ameta::{Img, ImgFmt};
use std::convert::From;
use std::ffi::OsStr;
use std::fs::rename;
use std::path::Path;
use std::str::FromStr;
use std::time::Duration;

#[derive(Clone)]
pub enum AudioFormat {
    MP3,
    Wav,
    Ogg,
    Opus,
    Flac,
    M4A,
    Unsupported,
}

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
    // / uslt lyrics
    lyric_frames: Vec<Lyrics>,
    lyric_selected_index: usize,
    parsed_lyric: Option<Lyric>,
    picture: Option<Picture>,
    format: AudioFormat,
}

impl Song {
    pub fn adjust_lyric_delay(&mut self, time_pos: u64, offset: i64) -> Result<()> {
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
        match self.format {
            AudioFormat::MP3 => self.save_mp3_tag()?,
            AudioFormat::M4A => self.save_m4a_tag()?,
            AudioFormat::Flac => self.save_flac_tag()?,
            // AudioFormat::Ogg => self.save_ogg_tag()?,
            AudioFormat::Opus | AudioFormat::Ogg => self.save_opus_tag()?,
            AudioFormat::Wav => self.save_wav_tag()?,
            AudioFormat::Unsupported => return Ok(()),
        }

        self.rename_by_tag()?;

        Ok(())
    }

    fn save_mp3_tag(&self) -> Result<()> {
        let mut id3_tag = id3::Tag::default();
        if let Some(file) = self.file() {
            if let Ok(t) = id3::Tag::read_from_path(file) {
                id3_tag = t;
            }
        }

        id3_tag.set_artist(self.artist().unwrap_or(&String::from("Unknown Artist")));
        id3_tag.set_title(self.title().unwrap_or(&String::from("Unknown Title")));
        id3_tag.set_album(self.album().unwrap_or(&String::from("Unknown Album")));
        id3_tag.remove_all_lyrics();

        if !self.lyric_frames.is_empty() {
            let lyric_frames = self.lyric_frames.clone();
            for l in lyric_frames {
                id3_tag.add_lyrics(l);
            }
        }

        if let Some(p) = &self.picture {
            id3_tag.add_picture(p.clone());
        }

        if let Some(file) = self.file() {
            id3_tag
                .write_to_path(file, id3::Version::Id3v24)
                .map_err(|e| anyhow!("write mp3 tag error {:?}", e))?;
        }
        Ok(())
    }

    fn save_m4a_tag(&self) -> Result<()> {
        let mut m4a_tag = mp4ameta::Tag::default();
        if let Some(file) = self.file() {
            if let Ok(t) = mp4ameta::Tag::read_from_path(file) {
                m4a_tag = t;
            }
        }

        m4a_tag.set_artist(self.artist().unwrap_or(&String::from("Unknown Artist")));
        m4a_tag.set_title(self.title().unwrap_or(&String::from("Unknown Title")));
        m4a_tag.set_album(
            self.album
                .as_ref()
                .unwrap_or(&String::from("Unknown Album")),
        );
        m4a_tag.remove_lyrics();

        if !self.lyric_frames.is_empty() {
            let lyric_frames = self.lyric_frames.clone();
            for l in lyric_frames {
                m4a_tag.set_lyrics(l.text);
            }
        }

        if let Some(p) = &self.picture {
            let fmt = match p.mime_type.as_str() {
                "image/bmp" => ImgFmt::Bmp,
                "image/Png" => ImgFmt::Png,
                "image/jpeg" | &_ => ImgFmt::Jpeg,
            };

            let img = Img {
                data: p.data.clone(),
                fmt,
            };

            m4a_tag.set_artwork(img);
        }

        if let Some(file) = self.file() {
            m4a_tag
                .write_to_path(file)
                .map_err(|e| anyhow!("write m4a tag error {:?}", e))?;
        }
        Ok(())
    }

    fn save_flac_tag(&self) -> Result<()> {
        let mut flac_tag = FlacTag::default();
        if let Some(file) = self.file() {
            if let Ok(t) = FlacTag::read_from_path(file) {
                flac_tag = t;
            }
        }

        flac_tag.set_vorbis(
            "Artist",
            vec![self.artist().unwrap_or(&String::from("Unknown Artist"))],
        );
        flac_tag.set_vorbis(
            "Title",
            vec![self.title().unwrap_or(&String::from("Unknown Title"))],
        );
        flac_tag.set_vorbis(
            "Album",
            vec![self
                .album
                .as_ref()
                .unwrap_or(&String::from("Unknown Album"))],
        );
        flac_tag.remove_vorbis("Lyrics");

        if !self.lyric_frames.is_empty() {
            let lyric_frames = self.lyric_frames.clone();
            for l in lyric_frames {
                flac_tag.set_vorbis("Lyrics", vec![l.text]);
            }
        }

        if let Some(p) = &self.picture {
            flac_tag.add_picture(
                p.mime_type.clone(),
                metaflac::block::PictureType::Other,
                p.data.clone(),
            );
        }

        let file = self.file().ok_or_else(|| anyhow!("no file found"))?;
        flac_tag
            .write_to_path(file)
            .map_err(|e| anyhow!("write flac tag error {:?}", e))?;

        Ok(())
    }

    // fn save_ogg_tag(&self) -> Result<()> {
    //     //open files
    //     let file = self.file().ok_or_else(|| anyhow!("no file found"))?;
    //     let mut f_in_disk = File::open(file)?;
    //     let mut f_in_ram: Vec<u8> = vec![];
    //     std::io::copy(&mut f_in_disk, &mut f_in_ram)?;
    //     f_in_disk.read_to_end(&mut f_in_ram)?;

    //     let f_in = Cursor::new(&f_in_ram);
    //     let mut new_comment = CommentHeader::new();
    //     new_comment.set_vendor("Ogg");
    //     new_comment.add_tag_single("artist", self.artist().unwrap_or("Unknown Artist"));
    //     new_comment.add_tag_single("title", self.title().unwrap_or("Unknown Artist"));
    //     new_comment.add_tag_single("album", self.album().unwrap_or("Unknown Artist"));
    //     if !self.lyric_frames.is_empty() {
    //         let lyric_frames = self.lyric_frames.clone();
    //         for l in lyric_frames {
    //             new_comment.add_tag_single("lyrics", &l.text);
    //         }
    //     }
    //     if let Some(p) = &self.picture {
    //         let mime_type = match p.mime_type.as_str() {
    //             "image/bmp" => MimeType::Bmp,
    //             "image/png" => MimeType::Png,
    //             "image/jpeg" | &_ => MimeType::Jpeg,
    //         };
    //         let picture_ogg = ogg_picture::OggPicture::new(
    //             OggPictureType::CoverFront,
    //             mime_type,
    //             Some("some image".to_string()),
    //             (0, 0),
    //             0,
    //             0,
    //             p.data.clone(),
    //         );
    //         let picture_decoded = ogg_picture::OggPicture::as_apic_bytes(&picture_ogg);
    //         let picture_encoded = base64::encode(&picture_decoded);
    //         new_comment.add_tag_single("METADATA_BLOCK_PICTURE", &picture_encoded);
    //     }

    //     let mut f_out = replace_comment_header(f_in, &new_comment);
    //     let mut f_out_disk = File::create(file)?;
    //     std::io::copy(&mut f_out, &mut f_out_disk)?;

    //     Ok(())
    // }

    #[allow(clippy::option_if_let_else)]
    fn save_opus_tag(&self) -> Result<()> {
        //open files
        let file = self.file().ok_or_else(|| anyhow!("no file found"))?;

        let p = Path::new(file);
        let mut tagged_file = Probe::open(file)?.read(false)?;

        let tag = match tagged_file.primary_tag_mut() {
            Some(primary_tag) => primary_tag,
            None => {
                if let Some(first_tag) = tagged_file.first_tag_mut() {
                    first_tag
                } else {
                    let tag_type = tagged_file.primary_tag_type();

                    eprintln!(
                        "WARN: No tags found, creating a new tag of type `{:?}`",
                        tag_type
                    );
                    tagged_file.insert_tag(Tag::new(tag_type));

                    tagged_file.primary_tag_mut().unwrap()
                }
            }
        };

        tag.set_title(self.title().unwrap_or("Unknown title").to_string());
        tag.set_artist(self.artist().unwrap_or("Unknown artist").to_string());
        tag.set_album(self.album().unwrap_or("").to_string());

        if let Some(p) = &self.picture {
            let mime_type = lofty::MimeType::from_str(p.mime_type.as_str());
            let picture_opus = lofty::Picture::new_unchecked(
                lofty::PictureType::CoverFront,
                mime_type,
                Some(p.description.clone()),
                p.data.clone(),
            );
            tag.remove_picture_type(lofty::PictureType::CoverFront);
            tag.push_picture(picture_opus);
        }
        if !self.lyric_frames.is_empty() {
            let lyric_frames = self.lyric_frames.clone();
            for l in lyric_frames {
                tag.insert_text(ItemKey::Lyrics, l.text);
            }
        }
        tag.save_to_path(p)
            .map_err(|e| anyhow!("save opus file by lofty error: {}", e))?;
        Ok(())
    }

    fn save_wav_tag(&self) -> Result<()> {
        let mut id3_tag = id3::Tag::default();
        if let Some(file) = self.file() {
            if let Ok(t) = id3::Tag::read_from_path(file) {
                id3_tag = t;
            }
        }

        id3_tag.set_artist(self.artist().unwrap_or(&String::from("Unknown Artist")));
        id3_tag.set_title(self.title().unwrap_or(&String::from("Unknown Title")));
        id3_tag.set_album(self.album().unwrap_or(&String::from("Unknown Album")));
        id3_tag.remove_all_lyrics();

        if !self.lyric_frames.is_empty() {
            let lyric_frames = self.lyric_frames.clone();
            for l in lyric_frames {
                id3_tag.add_lyrics(l);
            }
        }

        if let Some(p) = &self.picture {
            id3_tag.add_picture(p.clone());
        }

        if let Some(file) = self.file() {
            id3_tag
                .write_to_path(file, id3::Version::Id3v24)
                .map_err(|e| anyhow!("write wav tag error {:?}", e))?;
        }
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
                self.name = p_new
                    .file_name()
                    .and_then(OsStr::to_str)
                    .map(std::string::ToString::to_string);
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

    #[allow(clippy::cast_possible_truncation)]
    fn from_mp3(s: &str) -> Self {
        let p: &Path = Path::new(s);
        let ext = p.extension().and_then(OsStr::to_str);
        let name = p
            .file_name()
            .and_then(OsStr::to_str)
            .map(std::string::ToString::to_string);

        let id3_tag = if let Ok(tag) = id3::Tag::read_from_path(s) {
            tag
        } else {
            let mut t = id3::Tag::new();
            let p_mp3: &Path = Path::new(s);
            if let Some(p_base) = p_mp3.file_stem() {
                t.set_title(p_base.to_string_lossy());
            }
            let _drop = t.write_to_path(p_mp3, id3::Version::Id3v24);
            t
        };

        let artist: Option<String> = id3_tag.artist().map(String::from);
        let album: Option<String> = id3_tag.album().map(String::from);
        let title: Option<String> = id3_tag.title().map(String::from);
        let mut lyrics: Vec<Lyrics> = Vec::new();
        for l in id3_tag.lyrics().cloned() {
            lyrics.push(l);
        }
        lyrics.sort_by_cached_key(|a| a.description.clone());

        let parsed_lyric = if lyrics.is_empty() {
            None
        } else {
            match Lyric::from_str(lyrics[0].text.as_ref()) {
                Ok(l) => Some(l),
                Err(_) => None,
            }
        };
        let mut picture: Option<Picture> = None;
        let mut p_iter = id3_tag.pictures();
        if let Some(p) = p_iter.next() {
            picture = Some(p.clone());
        }

        let mut duration = match id3_tag.duration() {
            Some(d) => Duration::from_millis(d.into()),
            None => Duration::from_secs(0),
        };

        let duration_player = GStreamer::duration(s);
        let mut id3_tag_duration = id3_tag.clone();
        let diff = duration.as_secs().checked_sub(duration_player.seconds());
        if let Some(d) = diff {
            if d > 1 {
                id3_tag_duration.remove_duration();
                id3_tag_duration.set_duration((duration_player.mseconds()) as u32);
                let _drop = id3_tag_duration.write_to_path(s, id3::Version::Id3v24);
                duration = Duration::from_millis(duration_player.mseconds());
            }
        } else {
            id3_tag_duration.remove_duration();
            id3_tag_duration.set_duration((duration_player.mseconds()) as u32);
            let _drop = id3_tag_duration.write_to_path(s, id3::Version::Id3v24);
            duration = Duration::from_millis(duration_player.mseconds());
        }

        let file = Some(String::from(s));

        Self {
            artist,
            album,
            title,
            file,
            duration,
            name,
            ext: ext.map(String::from),
            lyric_frames: lyrics,
            lyric_selected_index: 0,
            parsed_lyric,
            picture,
            format: AudioFormat::MP3,
        }
    }

    #[allow(clippy::cast_possible_truncation)]
    fn from_wav(s: &str) -> Self {
        let p: &Path = Path::new(s);
        let ext = p.extension().and_then(OsStr::to_str);
        let name = p
            .file_name()
            .and_then(OsStr::to_str)
            .map(std::string::ToString::to_string);

        let id3_tag = if let Ok(tag) = id3::Tag::read_from_path(s) {
            tag
        } else {
            let mut t = id3::Tag::new();
            let p_mp3: &Path = Path::new(s);
            if let Some(p_base) = p_mp3.file_stem() {
                t.set_title(p_base.to_string_lossy());
            }
            let _drop = t.write_to_path(p_mp3, id3::Version::Id3v24);
            t
        };

        let artist: Option<String> = id3_tag.artist().map(String::from);
        let album: Option<String> = id3_tag.album().map(String::from);
        let title: Option<String> = id3_tag.title().map(String::from);
        let mut lyrics: Vec<Lyrics> = Vec::new();
        for l in id3_tag.lyrics().cloned() {
            lyrics.push(l);
        }
        lyrics.sort_by_cached_key(|a| a.description.clone());

        let parsed_lyric = if lyrics.is_empty() {
            None
        } else {
            match Lyric::from_str(lyrics[0].text.as_ref()) {
                Ok(l) => Some(l),
                Err(_) => None,
            }
        };
        let mut picture: Option<Picture> = None;
        let mut p_iter = id3_tag.pictures();
        if let Some(p) = p_iter.next() {
            picture = Some(p.clone());
        }

        let mut duration = match id3_tag.duration() {
            Some(d) => Duration::from_millis(d.into()),
            None => Duration::from_secs(0),
        };

        let duration_player = GStreamer::duration(s);
        let mut id3_tag_duration = id3_tag.clone();
        let diff = duration.as_secs().checked_sub(duration_player.seconds());
        if let Some(d) = diff {
            if d > 1 {
                id3_tag_duration.remove_duration();
                id3_tag_duration.set_duration((duration_player.mseconds()) as u32);
                let _drop = id3_tag_duration.write_to_path(s, id3::Version::Id3v24);
                duration = Duration::from_millis(duration_player.mseconds());
            }
        } else {
            id3_tag_duration.remove_duration();
            id3_tag_duration.set_duration((duration_player.mseconds()) as u32);
            let _drop = id3_tag_duration.write_to_path(s, id3::Version::Id3v24);
            duration = Duration::from_millis(duration_player.mseconds());
        }

        let file = Some(String::from(s));

        Self {
            artist,
            album,
            title,
            file,
            duration,
            name,
            ext: ext.map(String::from),
            lyric_frames: lyrics,
            lyric_selected_index: 0,
            parsed_lyric,
            picture,
            format: AudioFormat::Wav,
        }
    }

    fn from_m4a(s: &str) -> Self {
        let p: &Path = Path::new(s);
        let ext = p.extension().and_then(OsStr::to_str);
        let name = p
            .file_name()
            .and_then(OsStr::to_str)
            .map(std::string::ToString::to_string);

        let m4a_tag = if let Ok(t) = mp4ameta::Tag::read_from_path(s) {
            t
        } else {
            let mut t = mp4ameta::Tag::default();
            let p_m4a: &Path = Path::new(s);
            if let Some(p_base) = p_m4a.file_stem() {
                t.set_title(p_base.to_string_lossy());
            }
            let _drop = t.write_to_path(p_m4a);
            t
        };

        let artist: Option<String> = m4a_tag.artist().map(String::from);
        let album: Option<String> = m4a_tag.album().map(String::from);
        let title: Option<String> = m4a_tag.title().map(String::from);

        let lyrics = m4a_tag.lyrics().map(String::from);
        let mut parsed_lyric: Option<Lyric> = None;
        if let Some(l) = &lyrics {
            parsed_lyric = match Lyric::from_str(l) {
                Ok(l) => Some(l),
                Err(_) => None,
            }
        }

        let mut lyric_frames: Vec<Lyrics> = Vec::new();
        if let Some(s) = lyrics {
            lyric_frames.push(Lyrics {
                lang: String::from("eng"),
                description: String::from("Termusic"),
                text: s,
            });
        };

        let mut picture: Option<Picture> = None;
        if let Some(artwork) = m4a_tag.artwork() {
            let fmt = match artwork.fmt {
                ImgFmt::Bmp => "image/bmp",
                ImgFmt::Jpeg => "image/jpeg",
                ImgFmt::Png => "image/png",
            };
            picture = Some(Picture {
                mime_type: fmt.to_string(),
                picture_type: PictureType::Other,
                description: "some image".to_string(),
                data: artwork.data.to_vec(),
            });
        }

        let duration = m4a_tag.duration().unwrap_or_else(|| Duration::from_secs(0));

        let file = Some(String::from(s));
        Self {
            artist,
            album,
            title,
            file,
            duration,
            name,
            ext: ext.map(String::from),
            lyric_frames,
            lyric_selected_index: 0,
            parsed_lyric,
            picture,
            format: AudioFormat::M4A,
        }
    }

    fn from_flac(s: &str) -> Self {
        let p: &Path = Path::new(s);
        let ext = p.extension().and_then(OsStr::to_str);
        let name = p
            .file_name()
            .and_then(OsStr::to_str)
            .map(std::string::ToString::to_string);

        let flac_tag = if let Ok(t) = FlacTag::read_from_path(s) {
            t
        } else {
            let mut t = FlacTag::default();
            let p_flac: &Path = Path::new(s);
            if let Some(p_base) = p_flac.file_stem() {
                t.set_vorbis("Title", vec![p_base.to_string_lossy()]);
            }
            let _drop = t.write_to_path(p_flac);
            t
        };

        let mut artist: Option<String> = None;
        let a_vec = flac_tag.get_vorbis("Artist");
        if let Some(a_vec) = a_vec {
            let mut a_string = String::new();
            for a in a_vec {
                a_string.push_str(a);
            }
            artist = Some(a_string);
        }

        let mut album: Option<String> = None;
        let album_vec = flac_tag.get_vorbis("Album");
        if let Some(album_vec) = album_vec {
            let mut album_string = String::new();
            for a in album_vec {
                album_string.push_str(a);
            }
            album = Some(album_string);
        }

        let mut title: Option<String> = None;
        let title_vec = flac_tag.get_vorbis("Title");
        if let Some(title_vec) = title_vec {
            let mut title_string = String::new();
            for t in title_vec {
                title_string.push_str(t);
            }
            title = Some(title_string);
        }

        let mut lyric_frames: Vec<Lyrics> = vec![];
        let lyric_vec = flac_tag.get_vorbis("Lyrics");
        if let Some(l_vec) = lyric_vec {
            for l in l_vec {
                lyric_frames.push(Lyrics {
                    lang: "eng".to_string(),
                    description: "termusic".to_string(),
                    text: l.to_string(),
                });
            }
        }

        let mut parsed_lyric: Option<Lyric> = None;
        if let Some(l) = lyric_frames.get(0) {
            parsed_lyric = match Lyric::from_str(&l.text) {
                Ok(l) => Some(l),
                Err(_) => None,
            }
        }

        let mut picture: Option<Picture> = None;
        let mut picture_iter = flac_tag.pictures();
        if let Some(p) = picture_iter.next() {
            picture = Some(Picture {
                mime_type: p.mime_type.clone(),
                picture_type: PictureType::Other,
                description: "some image".to_string(),
                data: p.data.clone(),
            });
        }

        let mut duration = Duration::from_secs(0);
        let stream_info = flac_tag.get_streaminfo();
        if let Some(s) = stream_info {
            let secs = s.total_samples.checked_div(u64::from(s.sample_rate));
            if let Some(s) = secs {
                duration = Duration::from_secs(s);
            }
        }

        let file = Some(String::from(s));
        Self {
            artist,
            album,
            title,
            file,
            duration,
            name,
            ext: ext.map(String::from),
            lyric_frames,
            lyric_selected_index: 0,
            parsed_lyric,
            picture,
            format: AudioFormat::Flac,
        }
    }
    // fn from_ogg(s: &str) -> Self {
    //     let p: &Path = Path::new(s);
    //     let ext = p.extension().and_then(OsStr::to_str);

    //     let name = p
    //         .file_name()
    //         .and_then(OsStr::to_str)
    //         .map(std::string::ToString::to_string);
    //     let file = Some(String::from(s));

    //     let mut title = "Unknown Title".to_string();
    //     let mut album = " ".to_string();
    //     let mut artist = "Unknown Artist".to_string();
    //     let mut lyrics_text = "".to_string();
    //     let mut picture_encoded = "".to_string();

    //     //get the title, album, and artist of the song
    //     if let Ok(song_file) = File::open(s) {
    //         if let Ok(song) = lewton::inside_ogg::OggStreamReader::new(song_file) {
    //             for comment in song.comment_hdr.comment_list {
    //                 match comment.0.as_str() {
    //                     "TITLE" | "title" => title = comment.1,
    //                     "ALBUM" | "album" => album = comment.1,
    //                     "ARTIST" | "artist" => artist = comment.1,
    //                     "LYRICS" | "lyrics" => lyrics_text = comment.1,
    //                     "METADATA_BLOCK_PICTURE" | "metadata_block_picture" => {
    //                         picture_encoded = comment.1;
    //                     }
    //                     _ => {}
    //                 }
    //             }
    //         }
    //     }
    //     let mut picture: Option<Picture> = None;
    //     if let Ok(picture_decoded) = base64::decode(picture_encoded) {
    //         if let Ok(p) = ogg_picture::OggPicture::from_apic_bytes(&picture_decoded) {
    //             let mime_type = String::from(p.mime_type);
    //             let p_id3 = Picture {
    //                 mime_type,
    //                 picture_type: PictureType::CoverFront,
    //                 description: "some image".to_string(),
    //                 data: p.data.to_vec(),
    //             };
    //             picture = Some(p_id3);
    //         }
    //     }

    //     let mut lyric_frames: Vec<Lyrics> = Vec::new();
    //     let mut parsed_lyric: Option<Lyric> = None;
    //     if lyrics_text.len() > 10 {
    //         let lyrics = Lyrics {
    //             lang: "eng".to_string(),
    //             description: "termusic".to_string(),
    //             text: lyrics_text,
    //         };
    //         lyric_frames = vec![lyrics];
    //         if let Some(l) = lyric_frames.get(0) {
    //             parsed_lyric = match Lyric::from_str(&l.text) {
    //                 Ok(l) => Some(l),
    //                 Err(_) => None,
    //             }
    //         }
    //     }

    //     //get the song duration
    //     let duration = GStreamer::duration(s).into();

    //     Self {
    //         artist: Some(artist),
    //         album: Some(album),
    //         title: Some(title),
    //         file,
    //         duration,
    //         name,
    //         ext: ext.map(String::from),
    //         lyric_frames,
    //         lyric_selected_index: 0,
    //         parsed_lyric,
    //         picture,
    //         format: AudioFormat::Ogg,
    //     }
    // }
    #[allow(clippy::option_if_let_else)]
    fn from_opus(s: &str) -> Result<Self> {
        let p: &Path = Path::new(s);
        let ext = p.extension().and_then(OsStr::to_str);

        let name = p
            .file_name()
            .and_then(OsStr::to_str)
            .map(std::string::ToString::to_string);
        let file = Some(String::from(s));

        let mut tagged_file = Probe::open(p)
            .map_err(|e| anyhow!("Error: Bad path provided: {}", e))?
            .read(true)
            .map_err(|e| anyhow!("Error: Failed to read file: {}", e))?;

        let tag = match tagged_file.primary_tag_mut() {
            Some(primary_tag) => primary_tag,
            None => {
                if let Some(first_tag) = tagged_file.first_tag_mut() {
                    first_tag
                } else {
                    let tag_type = tagged_file.primary_tag_type();

                    // eprintln!(
                    //     "WARN: No tags found, creating a new tag of type `{:?}`",
                    //     tag_type
                    // );
                    tagged_file.insert_tag(Tag::new(tag_type));

                    tagged_file.primary_tag_mut().unwrap()
                }
            }
        };

        //get the title, album, and artist of the song
        let title = tag.title().unwrap_or("Unknown title").to_string();
        let album = tag.album().unwrap_or("").to_string();
        let artist = tag.artist().unwrap_or("Unknown Artist").to_string();
        let lyrics_text = tag.get_string(&ItemKey::Lyrics).unwrap_or("").to_string();

        let picture_lofty = tag.pictures().first();
        let mut picture: Option<Picture> = None;
        if let Some(p) = picture_lofty {
            let mime_type = p.mime_type().as_str().to_string();
            let description = p.description().unwrap_or("some image").to_string();
            let p_id3 = Picture {
                mime_type,
                picture_type: PictureType::CoverFront,
                description,
                data: p.data().to_vec(),
            };
            picture = Some(p_id3);
        }
        let mut lyric_frames: Vec<Lyrics> = Vec::new();
        let mut parsed_lyric: Option<Lyric> = None;
        if lyrics_text.len() > 10 {
            let lyrics = Lyrics {
                lang: "eng".to_string(),
                description: "termusic".to_string(),
                text: lyrics_text,
            };
            lyric_frames = vec![lyrics];
            if let Some(l) = lyric_frames.get(0) {
                parsed_lyric = match Lyric::from_str(&l.text) {
                    Ok(l) => Some(l),
                    Err(_) => None,
                }
            }
        }

        //get the song duration
        // let duration = GStreamer::duration(s).into();
        let properties = tagged_file.properties();
        let duration = properties.duration();
        let format = match ext {
            Some("ogg") => AudioFormat::Ogg,
            Some("opus") => AudioFormat::Opus,
            _ => AudioFormat::Unsupported,
        };

        Ok(Self {
            artist: Some(artist),
            album: Some(album),
            title: Some(title),
            file,
            duration,
            name,
            ext: ext.map(String::from),
            lyric_frames,
            lyric_selected_index: 0,
            parsed_lyric,
            picture,
            format,
        })
    }
}

impl FromStr for Song {
    type Err = anyhow::Error;
    // type Err = std::string::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let p: &Path = Path::new(s);
        let ext = p.extension().and_then(OsStr::to_str);

        match ext {
            Some("mp3") => Ok(Self::from_mp3(s)),
            Some("m4a") => Ok(Self::from_m4a(s)),
            Some("flac") => Ok(Self::from_flac(s)),
            // Some("ogg") => Ok(Self::from_ogg(s)),
            Some("opus" | "ogg") => Self::from_opus(s),
            Some("wav") => Ok(Self::from_wav(s)),
            _ => {
                let artist = Some(String::from("Not Support?"));
                let album = Some(String::from("Not Support?"));
                let title = Some(String::from(s));
                let file = Some(String::from(s));
                let duration = Duration::from_secs(0);
                let name = Some(String::from(""));
                let parsed_lyric: Option<Lyric> = None;
                let lyric_frames: Vec<Lyrics> = Vec::new();
                let picture: Option<Picture> = None;
                Ok(Self {
                    artist,
                    album,
                    title,
                    file,
                    duration,
                    name,
                    ext: ext.map(String::from),
                    lyric_frames,
                    lyric_selected_index: 0,
                    parsed_lyric,
                    picture,
                    format: AudioFormat::Unsupported,
                })
            }
        }
    }
}
