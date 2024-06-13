use std::time::Duration;

use rusqlite::Row;

/// A struct representing a [`Track`](crate::track::Track) in the database
#[derive(Clone, Debug)]
pub struct TrackDB {
    pub id: u64,
    pub artist: String,
    pub title: String,
    pub album: String,
    pub genre: String,
    pub file: String,
    pub duration: Duration,
    pub name: String,
    pub ext: String,
    pub directory: String,
    pub last_modified: String,
    pub last_position: Duration,
}

impl TrackDB {
    /// Try to convert a given row to a [`TrackDB`] instance, expecting correct row order
    pub fn try_from_row(row: &Row<'_>) -> Result<Self, rusqlite::Error> {
        let d_u64: u64 = row.get(6)?;
        let last_position_u64: u64 = row.get(11)?;
        Ok(TrackDB {
            id: row.get(0)?,
            artist: row.get(1)?,
            title: row.get(2)?,
            album: row.get(3)?,
            genre: row.get(4)?,
            file: row.get(5)?,
            duration: Duration::from_secs(d_u64),
            name: row.get(7)?,
            ext: row.get(8)?,
            directory: row.get(9)?,
            last_modified: row.get(10)?,
            last_position: Duration::from_secs(last_position_u64),
        })
    }
}
