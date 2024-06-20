use std::path::PathBuf;

use chrono::{DateTime, Utc};
use rusqlite::Row;

use super::{convert_date, PodcastDBId};

/// A struct representing a episode in a podcast in the database
#[derive(Debug, Clone)]
pub struct EpisodeDB {
    pub id: PodcastDBId,
    pub pod_id: PodcastDBId,
    pub title: String,
    pub url: String,
    pub guid: String,
    pub description: String,
    pub pubdate: Option<DateTime<Utc>>,
    pub duration: Option<i64>,
    pub played: bool,
    pub hidden: bool,
    pub last_position: Option<i64>,
    pub image_url: Option<String>,
}

impl EpisodeDB {
    /// Try to convert a given row to a [`EpisodeDB`] instance, using column names to resolve the values
    pub fn try_from_row_named(row: &Row<'_>) -> Result<Self, rusqlite::Error> {
        // NOTE: all the names in "get" below are the *column names* as defined in migrations/001.sql#table_episodes (pseudo link)
        Ok(Self {
            id: row.get("id")?,
            pod_id: row.get("podcast_id")?,
            title: row.get("title")?,
            url: row.get("url")?,
            guid: row.get::<_, Option<String>>("guid")?.unwrap_or_default(),
            description: row.get("description")?,
            pubdate: convert_date(&row.get("pubdate")),
            duration: row.get("duration")?,
            played: row.get("played")?,
            hidden: row.get("hidden")?,
            last_position: row.get("last_position")?,
            image_url: row.get("image_url")?,
        })
    }

    /// Try to convert a given row to a [`EpisodeDB`] instance, using column names to resolve the values (with renamed id because of conflicts)
    pub fn try_from_row_named_alias_id(row: &Row<'_>) -> Result<Self, rusqlite::Error> {
        // NOTE: all the names in "get" below are the *column names* as defined in migrations/001.sql#table_episodes (pseudo link)
        Ok(Self {
            id: row.get("epid")?,
            pod_id: row.get("podcast_id")?,
            title: row.get("title")?,
            url: row.get("url")?,
            guid: row.get::<_, Option<String>>("guid")?.unwrap_or_default(),
            description: row.get("description")?,
            pubdate: convert_date(&row.get("pubdate")),
            duration: row.get("duration")?,
            played: row.get("played")?,
            hidden: row.get("hidden")?,
            last_position: row.get("last_position")?,
            image_url: row.get("image_url")?,
        })
    }
}
