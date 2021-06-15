use anyhow::Result;
use humantime::format_duration;
use id3::Tag;
use std::fmt;
use std::path::Path;
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
}

impl Song {
    pub fn load(file: String) -> Result<Self> {
        let duration = match mp3_duration::from_path(&file) {
            Ok(d) => d,
            Err(_) => Duration::from_secs(0),
        };

        // let id3_tag = if let Ok(tag) = ::id3::Tag::read_from_path(&file) {
        //     tag
        // } else {
        //     Tag::new()
        // };

        let id3_tag = Tag::read_from_path(&file).unwrap_or_default();
        // let artist: Option<String> = Some(String::from(id3_tag.artist().unwrap_or_default()));
        let artist: Option<String> = id3_tag.artist().and_then(|s| Some(String::from(s)));
        let album: Option<String> = id3_tag.album().and_then(|s| Some(String::from(s)));
        let title: Option<String> = id3_tag.title().and_then(|s| Some(String::from(s)));
        let p: &Path = Path::new(file.as_str());
        let name = String::from(p.file_name().unwrap().to_string_lossy());
        Ok(Self {
            artist,
            album,
            title,
            file,
            duration,
            name,
        })
    }
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
}

impl fmt::Display for Song {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // write!(f, "{}-{}", self.file, self.file,)
        let duration = format_duration(Duration::from_secs(self.duration.as_secs()));
        write!(
            f,
            "{:<10} | {:<10} | {:<10} ({})",
            self.artist().unwrap_or(self.name.as_ref()),
            self.title().unwrap_or("Unknown Title"),
            self.album().unwrap_or("Unknown Album"),
            duration,
        )
    }
}
