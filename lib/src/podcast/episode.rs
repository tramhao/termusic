use std::path::PathBuf;

use chrono::{DateTime, Utc};

use crate::utils::StringUtils;

use super::{EPISODE_DURATION_LENGTH, EPISODE_PUBDATE_LENGTH, Menuable};

/// Struct holding data about an individual podcast episode. Most of this
/// is metadata, but if the episode has been downloaded to the local
/// machine, the filepath will be included here as well. `played`
/// indicates whether the podcast has been marked as played or unplayed.
#[derive(Debug, Clone, Default)]
pub struct Episode {
    pub id: i64,
    pub pod_id: i64,
    pub title: String,
    pub url: String,
    pub guid: String,
    pub description: String,
    pub pubdate: Option<DateTime<Utc>>,
    pub duration: Option<i64>,
    pub path: Option<PathBuf>,
    pub played: bool,
    pub last_position: Option<i64>,
    pub image_url: Option<String>,
}

impl Episode {
    /// Formats the duration in seconds into an HH:MM:SS format.
    #[must_use]
    pub fn format_duration(&self) -> String {
        match self.duration {
            Some(dur) => {
                let mut seconds = dur;
                let hours = seconds / 3600;
                seconds -= hours * 3600;
                let minutes = seconds / 60;
                seconds -= minutes * 60;
                format!("{hours:02}:{minutes:02}:{seconds:02}")
            }
            None => "--:--:--".to_string(),
        }
    }
}

impl Menuable for Episode {
    /// Returns the database ID for the episode.
    fn get_id(&self) -> i64 {
        self.id
    }

    /// Returns the title for the episode, up to length characters.
    fn get_title(&self, length: usize) -> String {
        let out = match self.path {
            Some(_) => {
                let title = self.title.substr(0, length - 4);
                format!("[D] {title}")
            }
            None => self.title.substr(0, length).to_string(),
        };
        if length > EPISODE_PUBDATE_LENGTH {
            let dur = self.format_duration();
            let meta_dur = format!("[{dur}]");

            if let Some(pubdate) = self.pubdate {
                // print pubdate and duration
                let pd = pubdate.format("%F");
                let meta_str = format!("({pd}) {meta_dur}");
                let added_len = meta_str.chars().count();

                let out_added = out.substr(0, length - added_len - 3);
                format!(
                    " {out_added} {meta_str:>width$} ",
                    width = length - out_added.grapheme_len() - 3
                )
            } else {
                // just print duration
                let out_added = out.substr(0, length - meta_dur.chars().count() - 3);
                format!(
                    " {out_added} {meta_dur:>width$} ",
                    width = length - out_added.grapheme_len() - 3
                )
            }
        } else if length > EPISODE_DURATION_LENGTH {
            let dur = self.format_duration();
            let meta_dur = format!("[{dur}]");
            let out_added = out.substr(0, length - meta_dur.chars().count() - 3);
            format!(
                " {out_added} {meta_dur:>width$} ",
                width = length - out_added.grapheme_len() - 3
            )
        } else {
            format!(" {} ", out.substr(0, length - 2))
        }
    }

    fn is_played(&self) -> bool {
        self.played
    }
}

/// Struct holding data about an individual podcast episode, before it
/// has been inserted into the database.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct EpisodeNoId {
    pub title: String,
    pub url: String,
    pub guid: String,
    pub description: String,
    pub pubdate: Option<DateTime<Utc>>,
    pub duration: Option<i64>,
    pub image_url: Option<String>,
}
