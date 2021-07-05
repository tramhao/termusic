use crate::lyric::lrc::Lyric;
use anyhow::Result;
use humantime::{format_duration, FormattedDuration};
use id3::frame::Lyrics;
use id3::frame::Picture;
use id3::{Tag, Version};
use std::fmt;
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
    /// name of the song
    pub name: Option<String>,
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

        id3_tag.write_to_path(self.file.as_ref().unwrap(), Version::Id3v24)?;
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

impl fmt::Display for Song {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // write!(f, "{}-{}", self.file, self.file,)
        let mut duration_display: FormattedDuration = format_duration(Duration::from_secs(0));
        if let Some(duration) = self.duration {
            duration_display = format_duration(Duration::from_secs(duration.as_secs()));
        };
        write!(
            f,
            "[{:.8}] {:.12}《{:.12}》{:.10}",
            duration_display,
            self.artist().unwrap_or_else(|| self.name.as_ref().unwrap()),
            self.title().unwrap_or("Unknown Title"),
            self.album().unwrap_or("Unknown Album"),
        )
    }
}
impl FromStr for Song {
    type Err = std::string::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let duration: Option<Duration> = match mp3_duration::from_path(s) {
            Ok(d) => Some(d),
            Err(_) => Some(Duration::from_secs(0)),
        };

        let id3_tag = Tag::read_from_path(s).unwrap_or_default();
        let artist: Option<String> = id3_tag.artist().map(String::from);
        let album: Option<String> = id3_tag.album().map(String::from);
        let title: Option<String> = id3_tag.title().map(String::from);
        let p: &Path = Path::new(s);
        let name = Some(String::from(p.file_name().unwrap().to_string_lossy()));

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
            lyric_frames: lyrics,
            parsed_lyric,
            picture,
        })
    }
}
