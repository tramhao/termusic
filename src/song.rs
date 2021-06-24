use crate::lyric::lrc::Lyric;
use anyhow::Result;
use humantime::format_duration;
use id3::frame::Lyrics;
use id3::frame::Picture;
use id3::{Tag, Version};
use std::fmt;
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
    pub file: String,
    /// Duration of the song
    pub duration: Duration,
    /// name of the song
    pub name: String,
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
        let mut id3_tag = Tag::read_from_path(self.file.as_str())?;
        id3_tag.set_artist(self.artist.as_ref().unwrap());
        id3_tag.set_title(self.title.as_ref().unwrap());
        id3_tag.set_album(self.album.as_ref().unwrap());
        id3_tag.remove_all_lyrics();
        for l in self.lyric_frames.iter() {
            id3_tag.add_lyrics(l.clone());
        }
        id3_tag.write_to_path(self.file.as_str(), Version::Id3v24)?;
        Ok(())
    }
}

impl fmt::Display for Song {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // write!(f, "{}-{}", self.file, self.file,)
        let duration = format_duration(Duration::from_secs(self.duration.as_secs()));
        write!(
            f,
            "[{:.8}] {:.12}《{:.12}》{:.10}",
            duration,
            self.artist().unwrap_or(self.name.as_ref()),
            self.title().unwrap_or("Unknown Title"),
            self.album().unwrap_or("Unknown Album"),
        )
    }
}
impl FromStr for Song {
    type Err = std::string::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let duration = match mp3_duration::from_path(s) {
            Ok(d) => d,
            Err(_) => Duration::from_secs(0),
        };

        let id3_tag = Tag::read_from_path(s).unwrap_or_default();
        // let artist: Option<String> = Some(String::from(id3_tag.artist().unwrap_or_default()));
        let artist: Option<String> = id3_tag.artist().and_then(|s| Some(String::from(s)));
        let album: Option<String> = id3_tag.album().and_then(|s| Some(String::from(s)));
        let title: Option<String> = id3_tag.title().and_then(|s| Some(String::from(s)));
        let p: &Path = Path::new(s);
        let name = String::from(p.file_name().unwrap().to_string_lossy());

        let mut lyrics: Vec<Lyrics> = Vec::new();
        for l in id3_tag.lyrics().cloned() {
            lyrics.push(l);
        }

        let parsed_lyric = match Lyric::from_str(lyrics[0].text.as_ref()) {
            Ok(l) => Some(l),
            Err(e) => {
                panic!("{}", e);
            }
        };
        let mut picture: Vec<Picture> = Vec::new();
        for p in id3_tag.pictures().cloned() {
            picture.push(p);
        }

        let file = String::from(s);
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
