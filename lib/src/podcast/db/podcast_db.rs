use chrono::{DateTime, Utc};
use rusqlite::Row;

use super::convert_date;

/// A struct representing a podcast feed in the database
#[derive(Debug, Clone)]
pub struct PodcastDB {
    pub id: i64,
    pub title: String,
    pub url: String,
    pub description: Option<String>,
    pub author: Option<String>,
    pub explicit: Option<bool>,
    pub last_checked: DateTime<Utc>,
    pub image_url: Option<String>,
}

impl PodcastDB {
    /// Try to convert a given row to a [`PodcastDB`] instance, using column names to resolve the values
    pub fn try_from_row_named(row: &Row<'_>) -> Result<Self, rusqlite::Error> {
        // NOTE: all the names in "get" below are the *column names* as defined in migrations/001.sql#table_podcasts (pseudo link)
        let last_checked =
            convert_date(&row.get("last_checked")).ok_or(rusqlite::Error::InvalidQuery)?;
        Ok(PodcastDB {
            id: row.get("id")?,
            title: row.get("title")?,
            url: row.get("url")?,
            description: row.get("description")?,
            author: row.get("author")?,
            explicit: row.get("explicit")?,
            last_checked,
            image_url: row.get("image_url")?,
        })
    }
}
