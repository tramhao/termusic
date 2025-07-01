use std::cmp::Ordering;

use chrono::{DateTime, Utc};

use crate::utils::StringUtils;

use super::{
    Menuable, PODCAST_UNPLAYED_TOTALS_LENGTH,
    episode::{Episode, EpisodeNoId},
};

/// Struct holding data about an individual podcast feed. This includes a
/// (possibly empty) vector of episodes.
#[derive(Debug, Clone)]
pub struct Podcast {
    pub id: i64,
    pub title: String,
    pub sort_title: String,
    pub url: String,
    pub description: Option<String>,
    pub author: Option<String>,
    pub explicit: Option<bool>,
    pub last_checked: DateTime<Utc>,
    pub episodes: Vec<Episode>,
    pub image_url: Option<String>,
}

impl Podcast {
    // Counts and returns the number of unplayed episodes in the podcast.
    #[must_use]
    pub fn num_unplayed(&self) -> usize {
        self.episodes
            .iter()
            .map(|ep| usize::from(!ep.is_played()))
            .sum()
    }
}

impl Menuable for Podcast {
    /// Returns the database ID for the podcast.
    fn get_id(&self) -> i64 {
        self.id
    }

    /// Returns the title for the podcast, up to length characters.
    fn get_title(&self, length: usize) -> String {
        let mut title_length = length;

        // if the size available is big enough, we add the unplayed data
        // to the end
        if length > PODCAST_UNPLAYED_TOTALS_LENGTH {
            let meta_str = format!("({}/{})", self.num_unplayed(), self.episodes.len());
            title_length = length - meta_str.chars().count() - 3;

            let out = self.title.substr(0, title_length);

            format!(
                " {out} {meta_str:>width$} ",
                width = length - out.grapheme_len() - 3
            ) // this pads spaces between title and totals
        } else {
            format!(" {} ", self.title.substr(0, title_length - 2))
        }
    }

    fn is_played(&self) -> bool {
        self.num_unplayed() == 0
    }
}

impl PartialEq for Podcast {
    fn eq(&self, other: &Self) -> bool {
        self.sort_title == other.sort_title
    }
}
impl Eq for Podcast {}

impl PartialOrd for Podcast {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Podcast {
    fn cmp(&self, other: &Self) -> Ordering {
        self.sort_title.cmp(&other.sort_title)
    }
}

/// Struct holding data about an individual podcast feed, before it has
/// been inserted into the database. This includes a
/// (possibly empty) vector of episodes.
#[derive(Debug, Clone, Eq, PartialEq)]
#[allow(clippy::module_name_repetitions)]
pub struct PodcastNoId {
    pub title: String,
    pub url: String,
    pub description: Option<String>,
    pub author: Option<String>,
    pub explicit: Option<bool>,
    pub last_checked: DateTime<Utc>,
    pub episodes: Vec<EpisodeNoId>,
    pub image_url: Option<String>,
}
