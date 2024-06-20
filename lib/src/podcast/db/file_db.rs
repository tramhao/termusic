use std::path::PathBuf;

use rusqlite::Row;

use super::PodcastDBId;

/// A struct representing a episode file (downloaded) in the database
#[derive(Debug, Clone)]
pub struct FileDB {
    pub id: PodcastDBId,
    pub episode_id: PodcastDBId,
    pub path: PathBuf,
}

impl FileDB {
    /// Try to convert a given row to a [`FileDB`] instance, using column names to resolve the values
    pub fn try_from_row_named(row: &Row<'_>) -> Result<Self, rusqlite::Error> {
        // NOTE: all the names in "get" below are the *column names* as defined in migrations/001.sql#table_files (pseudo link)
        let path = PathBuf::from(row.get::<_, String>("path")?);
        Ok(Self {
            id: row.get("id")?,
            episode_id: row.get("episode_id")?,
            path,
        })
    }

    /// Try to convert a given row to a [`FileDB`] instance, using column names to resolve the values (with renamed id because of conflicts)
    pub fn try_from_row_named_alias_id(row: &Row<'_>) -> Result<Self, rusqlite::Error> {
        // NOTE: all the names in "get" below are the *column names* as defined in migrations/001.sql#table_files (pseudo link)
        let path = PathBuf::from(row.get::<_, String>("path")?);
        Ok(Self {
            id: row.get("fileid")?,
            episode_id: row.get("episode_id")?,
            path,
        })
    }
}
