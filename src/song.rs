use crate::player::gst::GSTPlayer;
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
use anyhow::{anyhow, Result};
use humantime::{format_duration, FormattedDuration};
use id3::frame::Lyrics;
use id3::frame::{Picture, PictureType};
use mp4ameta::{Img, ImgFmt};
use std::ffi::OsStr;
use std::fs::rename;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::time::Duration;

#[derive(Clone)]
pub struct Song {
    /// Artist of the song
    pub artist: Option<String>,
    /// Album of the song
    pub album: Option<String>,
    /// Title of the song
    pub title: Option<String>,
    /// File path to the song
    pub file: Option<String>,
    /// Duration of the song
    pub duration: Option<Duration>,
    /// Name of the song
    pub name: Option<String>,
    /// Extension of the song
    pub ext: Option<String>,
    // / uslt lyrics
    pub lyric_frames: Vec<Lyrics>,
    pub lyric_selected: u32,
    pub parsed_lyric: Option<Lyric>,
    // pub lyrics: Option<String>,
    pub picture: Vec<Picture>,
}

impl Song {
    /// Optionally return the artist of the song
    /// If `None` it wasn't able to read the tags
    pub fn artist(&self) -> Option<&str> {
        match self.artist.as_ref() {
            Some(artist) => Some(artist),
            None => None,
        }
    }
    /// Optionally return the song's album
    /// If `None` failed to read the tags
    pub fn album(&self) -> Option<&str> {
        match self.album.as_ref() {
            Some(album) => Some(album),
            None => None,
        }
    }
    /// Optionally return the title of the song
    /// If `None` it wasn't able to read the tags
    pub fn title(&self) -> Option<&str> {
        match self.title.as_ref() {
            Some(title) => Some(title),
            None => None,
        }
    }

    pub fn file(&self) -> Option<&str> {
        match self.file.as_ref() {
            Some(file) => Some(file),
            None => None,
        }
    }

    pub fn duration(&self) -> FormattedDuration {
        match self.duration.as_ref() {
            Some(d) => format_duration(Duration::from_secs(d.as_secs())),
            None => format_duration(Duration::from_secs(0)),
        }
    }

    pub fn save(&mut self) -> Result<()> {
        if let Some(ext) = self.ext.as_ref() {
            match ext.as_str() {
                "mp3" => {
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

                    if let Some(lyric) = self.parsed_lyric.as_mut() {
                        if let Some(text) = lyric.as_lrc() {
                            let lyric_frame = Lyrics {
                                lang: String::from("chi"),
                                description: String::from("saved by termusic."),
                                text,
                            };
                            id3_tag.add_lyrics(lyric_frame);
                        }
                    }

                    for p in self.picture.iter() {
                        id3_tag.add_picture(p.to_owned());
                    }

                    if let Some(file) = self.file() {
                        id3_tag.write_to_path(file, id3::Version::Id3v24)?;
                    }
                }

                "m4a" => {
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

                    if let Some(lyric) = self.parsed_lyric.as_mut() {
                        if let Some(text) = lyric.as_lrc() {
                            m4a_tag.set_lyrics(text);
                        }
                    }

                    for p in self.picture.iter() {
                        let img = Img {
                            data: p.data.to_owned(),
                            fmt: ImgFmt::Jpeg,
                        };
                        m4a_tag.set_artwork(img);
                    }

                    if let Some(file) = self.file() {
                        let _ = m4a_tag
                            .write_to_path(file)
                            .map_err(|e| anyhow!("write m4a tag error {:?}", e))?;
                    }
                }

                &_ => {}
            }
        }

        Ok(())
    }

    pub fn rename_by_tag(&mut self) -> Result<()> {
        let new_name = format!(
            "{}-{}.{}",
            self.artist().unwrap_or(&"Unknown Artist".to_string()),
            self.title().unwrap_or(&"Unknown Title".to_string()),
            self.ext.as_ref().unwrap_or(&"mp3".to_string()),
        );
        let new_name_path: &Path = Path::new(new_name.as_str());
        if let Some(file) = self.file() {
            let p_old: &Path = Path::new(file);
            if let Some(p_prefix) = p_old.parent() {
                let p_new = p_prefix.join(new_name_path);
                rename(p_old, <PathBuf as AsRef<Path>>::as_ref(&p_new))
                    .map_err(|e| anyhow!("rename m4a file error {:?}", e))?;
                self.file = Some(String::from(p_new.to_string_lossy()));
                if let Some(name) = p_new.file_name() {
                    self.name = Some(String::from(name.to_string_lossy()));
                }
            }
        }
        Ok(())
    }

    pub fn set_lyric(&mut self, lyric_string: &str, lang_ext: String) {
        self.lyric_frames.clear();
        self.lyric_frames.push(Lyrics {
            lang: lang_ext,
            description: String::from("added by termusic."),
            text: lyric_string.to_string(),
        });

        let mut parsed_lyric: Option<Lyric> = None;
        self.parsed_lyric = parsed_lyric;

        if self.lyric_frames.is_empty() {
            return;
        }
        if let Some(lyric_frame) = self.lyric_frames.get(0) {
            parsed_lyric = match Lyric::from_str(lyric_frame.text.as_ref()) {
                Ok(l) => Some(l),
                Err(_) => None,
            };
            self.parsed_lyric = parsed_lyric;
        }
    }

    pub fn set_photo(&mut self, picture: Picture) {
        self.picture.clear();
        self.picture.push(picture);
    }
}

impl FromStr for Song {
    type Err = std::string::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let p: &Path = Path::new(s);
        let ext = p.extension().and_then(OsStr::to_str);

        match ext {
            Some("mp3") => {
                let name = p.file_name().and_then(OsStr::to_str).map(|x| x.to_string());
                let duration: Option<Duration> = match mp3_duration::from_path(s) {
                    Ok(d) => Some(d),
                    Err(_) => Some(Duration::from_secs(0)),
                };

                let id3_tag = match id3::Tag::read_from_path(s) {
                    Ok(tag) => tag,
                    Err(_) => {
                        let mut t = id3::Tag::new();
                        let p: &Path = Path::new(s);
                        if let Some(p_base) = p.file_stem() {
                            t.set_title(p_base.to_string_lossy());
                        }
                        let _ = t.write_to_path(p, id3::Version::Id3v24);
                        t
                    }
                };

                let artist: Option<String> = id3_tag.artist().map(String::from);
                let album: Option<String> = id3_tag.album().map(String::from);
                let title: Option<String> = id3_tag.title().map(String::from);
                let mut lyrics: Vec<Lyrics> = Vec::new();
                for l in id3_tag.lyrics().cloned() {
                    lyrics.push(l);
                }

                let mut parsed_lyric: Option<Lyric> = None;
                if !lyrics.is_empty() {
                    parsed_lyric = match Lyric::from_str(lyrics[0].text.as_ref()) {
                        Ok(l) => Some(l),
                        Err(_) => None,
                    };
                }

                let mut picture: Vec<Picture> = Vec::new();
                for p in id3_tag.pictures().cloned() {
                    picture.push(p);
                }

                let file = Some(String::from(s));
                Ok(Self {
                    artist,
                    album,
                    title,
                    file,
                    duration,
                    name,
                    ext: Some("mp3".to_string()),
                    lyric_frames: lyrics,
                    lyric_selected: 0,
                    parsed_lyric,
                    picture,
                })
            }
            Some("m4a") => {
                let name = p.file_name().and_then(OsStr::to_str).map(|x| x.to_string());

                let duration_u64 = GSTPlayer::duration_m4a(s);
                let duration = Some(Duration::from_secs(duration_u64));
                let m4a_tag = match mp4ameta::Tag::read_from_path(s) {
                    Ok(t) => t,
                    Err(_) => {
                        let mut t = mp4ameta::Tag::default();
                        let p: &Path = Path::new(s);
                        if let Some(p_base) = p.file_stem() {
                            t.set_title(p_base.to_string_lossy());
                        }
                        match t.write_to_path(p) {
                            Ok(_) => t,
                            Err(_) => t,
                        }
                    }
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
                        lang: String::from("chi"),
                        description: String::from("saved by termusic."),
                        text: s,
                    });
                };

                let mut picture: Vec<Picture> = Vec::new();
                if let Some(artwork) = m4a_tag.artwork() {
                    let fmt = match artwork.fmt {
                        ImgFmt::Bmp => "image/bmp",
                        ImgFmt::Jpeg => "image/jpeg",
                        ImgFmt::Png => "image/png",
                    };
                    picture.push(Picture {
                        mime_type: fmt.to_string(),
                        picture_type: PictureType::Other,
                        description: "some image".to_string(),
                        data: artwork.data.to_vec(),
                    });
                }

                let file = Some(String::from(s));
                Ok(Self {
                    artist,
                    album,
                    title,
                    file,
                    duration,
                    name,
                    ext: Some("m4a".to_string()),
                    lyric_frames,
                    lyric_selected: 0,
                    parsed_lyric,
                    picture,
                })
            }
            _ => {
                let artist = Some(String::from("Not Support?"));
                let album = Some(String::from("Not Support?"));
                let title = Some(String::from(s));
                let file = Some(String::from(s));
                let duration = Some(Duration::from_secs(0));
                let name = Some(String::from(""));
                let parsed_lyric: Option<Lyric> = None;
                let lyric_frames: Vec<Lyrics> = Vec::new();
                let picture: Vec<Picture> = Vec::new();
                Ok(Self {
                    artist,
                    album,
                    title,
                    file,
                    duration,
                    name,
                    ext: ext.map(|x| x.to_string()),
                    lyric_frames,
                    lyric_selected: 0,
                    parsed_lyric,
                    picture,
                })
            }
        }
    }
}
