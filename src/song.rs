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
use crate::lyric::lrc::Lyric;
use crate::player::gst::GSTPlayer;
use anyhow::Result;
use humantime::{format_duration, FormattedDuration};
use id3::frame::Lyrics;
use id3::frame::Picture;
use id3::{Tag, Version};
use std::fs::rename;
use std::path::Path;
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

    pub fn duration(&self) -> FormattedDuration {
        match self.duration.as_ref() {
            Some(d) => format_duration(Duration::from_secs(d.as_secs())),
            None => format_duration(Duration::from_secs(0)),
        }
    }

    pub fn save(&self) -> Result<()> {
        let mut id3_tag = Tag::read_from_path(self.file.as_ref().unwrap())?;
        id3_tag.set_artist(
            self.artist
                .as_ref()
                .unwrap_or(&String::from("Unknown Artist")),
        );
        id3_tag.set_title(
            self.title
                .as_ref()
                .unwrap_or(&String::from("Unknown Title")),
        );
        id3_tag.set_album(
            self.album
                .as_ref()
                .unwrap_or(&String::from("Unknown Album")),
        );
        id3_tag.remove_all_lyrics();

        if let Some(mut lyric) = self.parsed_lyric.clone() {
            if let Some(text) = lyric.as_lrc() {
                let lyric_frame: Lyrics = Lyrics {
                    lang: String::from("chi"),
                    description: String::from("saved by termusic."),
                    text,
                };
                id3_tag.add_lyrics(lyric_frame);
            }
        }

        if let Some(file) = self.file.as_ref() {
            id3_tag.write_to_path(file, Version::Id3v24)?;
        }

        Ok(())
    }

    pub fn rename_by_tag(&mut self) -> Result<()> {
        let new_name = format!(
            "{}-{}.mp3",
            self.artist.as_ref().unwrap(),
            self.title.as_ref().unwrap()
        );
        let new_name_path: &Path = Path::new(new_name.as_str());
        let p_old: &Path = Path::new(self.file.as_ref().unwrap());
        let p_prefix = p_old.parent().unwrap();
        let p_new = p_prefix.join(new_name_path);
        rename(p_old, p_new.clone())?;
        self.file = Some(String::from(p_new.to_string_lossy()));
        self.name = Some(String::from(p_new.file_name().unwrap().to_string_lossy()));
        Ok(())
    }
}

impl FromStr for Song {
    type Err = std::string::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let p: &Path = Path::new(s);
        let ext = String::from(p.extension().unwrap().to_string_lossy());
        match ext.as_str() {
            "mp3" => {
                let name = Some(String::from(p.file_name().unwrap().to_string_lossy()));
                let duration: Option<Duration> = match mp3_duration::from_path(s) {
                    Ok(d) => Some(d),
                    Err(_) => Some(Duration::from_secs(0)),
                };

                let id3_tag = match Tag::read_from_path(s) {
                    Ok(tag) => tag,
                    Err(_) => {
                        let mut t = Tag::new();
                        let p: &Path = Path::new(s);
                        if let Some(p_base) = p.file_stem() {
                            t.set_title(p_base.to_string_lossy());
                        }
                        match t.write_to_path(p, Version::Id3v24) {
                            Ok(_) => t,
                            Err(_) => t,
                        }
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
                    ext: Some(ext),
                    lyric_frames: lyrics,
                    parsed_lyric,
                    picture,
                })
            }
            "m4a" => {
                let name = Some(String::from(p.file_name().unwrap().to_string_lossy()));
                // let duration: Option<Duration> = match mp3_duration::from_path(s) {
                //     Ok(d) => Some(d),
                //     Err(_) => Some(Duration::from_secs(10)),
                // };

                let duration_u64 = GSTPlayer::duration_m4a(s);
                let duration = Some(Duration::from_secs(duration_u64));
                // let tag = mp4ameta::Tag::read_from_path(s).unwrap();

                // println!("{}", tag.artist().unwrap());
                let id3_tag = match Tag::read_from_path(s) {
                    Ok(tag) => tag,
                    Err(_) => {
                        let mut t = Tag::new();
                        let p: &Path = Path::new(s);
                        if let Some(p_base) = p.file_stem() {
                            t.set_title(p_base.to_string_lossy());
                        }
                        match t.write_to_path(p, Version::Id3v24) {
                            Ok(_) => t,
                            Err(_) => t,
                        }
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
                    ext: Some(ext),
                    lyric_frames: lyrics,
                    parsed_lyric,
                    picture,
                })
            }
            _ => {
                let artist = Some(String::from(""));
                let album = Some(String::from(""));
                let title = Some(String::from(s));
                let file = Some(String::from(s));
                let duration = Some(Duration::from_secs(0));
                let name = Some(String::from(""));
                let parsed_lyric: Option<Lyric> = None;
                let lyrics: Vec<Lyrics> = Vec::new();
                let picture: Vec<Picture> = Vec::new();
                Ok(Self {
                    artist,
                    album,
                    title,
                    file,
                    duration,
                    name,
                    ext: Some(ext),
                    lyric_frames: lyrics,
                    parsed_lyric,
                    picture,
                })
            }
        }
    }
}
