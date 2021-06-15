use anyhow::Result;
use humantime::format_duration;
use std::fmt;
use std::time::Duration;

#[derive(Clone)]
pub struct Song {
    pub file: String,
    pub duration: Duration,
}

impl Song {
    pub fn load(file: String) -> Result<Self> {
        let duration = match mp3_duration::from_path(&file) {
            Ok(d) => d,
            Err(_) => Duration::from_secs(0),
        };

        Ok(Self { file, duration })
    }
}

impl fmt::Display for Song {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // write!(f, "{}-{}", self.file, self.file,)
        let duration = format_duration(Duration::from_secs(self.duration.as_secs()));
        write!(
            f,
            "{} - {}",
            // "{} - {} - {} ({})",
            // self.artist().unwrap_or("Unknown Artist"),
            // self.album().unwrap_or("Unknown Album"),
            // self.title().unwrap_or("Unknown Title"),
            self.file,
            duration,
        )
    }
}
