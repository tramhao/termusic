use crate::config::v2::server::ScanDepth;
/**
 * MIT License
 *
 * termusic - Copyright (c) 2021 Larry Hao
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy
 * of this software and associated documentation files (the "Software"), to deal
 * in the Software without restriction, including without limitation the rights
 * to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 * copies of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in all
 * copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 * AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
 * SOFTWARE.
 */
use crate::config::ServerOverlay;
use crate::track::Track;
use crate::utils::{filetype_supported, get_app_config_path, get_pin_yin};
use anyhow::Context;
use parking_lot::Mutex;
use rusqlite::{params, Connection, Error, Result};
use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, UNIX_EPOCH};
use track_db::TrackDBInsertable;

mod migration;
mod track_db;

pub use track_db::TrackDB;

pub struct DataBase {
    conn: Arc<Mutex<Connection>>,
    max_depth: ScanDepth,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchCriteria {
    Artist,
    Album,

    // TODO: the values below are current unused
    Genre,
    Directory,
    Playlist,
}

impl From<usize> for SearchCriteria {
    fn from(u_index: usize) -> Self {
        match u_index {
            1 => Self::Album,
            2 => Self::Genre,
            3 => Self::Directory,
            4 => Self::Playlist,
            /* 0 | */ _ => Self::Artist,
        }
    }
}

impl std::fmt::Display for SearchCriteria {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Artist => write!(f, "artist"),
            Self::Album => write!(f, "album"),
            Self::Genre => write!(f, "genre"),
            Self::Directory => write!(f, "directory"),
            Self::Playlist => write!(f, "playlist"),
        }
    }
}

impl DataBase {
    /// # Panics
    ///
    /// - if app config path creation fails
    /// - if any required database operation fails
    pub fn new(config: &ServerOverlay) -> anyhow::Result<Self> {
        let mut db_path = get_app_config_path().context("failed to get app configuration path")?;
        db_path.push("library.db");
        let conn = Connection::open(db_path).context("open/create database")?;

        migration::migrate(&conn).context("Database creation / migration")?;

        let max_depth = config.get_library_scan_depth();

        let conn = Arc::new(Mutex::new(conn));
        Ok(Self { conn, max_depth })
    }

    /// Insert multiple tracks into the database
    fn add_records(conn: &Arc<Mutex<Connection>>, tracks: Vec<Track>) -> Result<()> {
        let mut conn = conn.lock();
        let tx = conn.transaction()?;

        for track in tracks {
            TrackDBInsertable::from(&track).insert_track(&tx)?;
        }

        tx.commit()?;
        Ok(())
    }

    /// Check if the given path's track needs to be updated in the database by comparing `last_modified` times
    fn need_update(conn: &Arc<Mutex<Connection>>, path: &Path) -> Result<bool> {
        let conn = conn.lock();
        let filename = path
            .file_name()
            .ok_or_else(|| Error::InvalidParameterName("file name missing".to_string()))?
            .to_string_lossy();
        let mut stmt = conn.prepare("SELECT last_modified FROM tracks WHERE name = ?")?;
        let rows = stmt.query_map([filename], |row| {
            let last_modified: String = row.get(0)?;

            Ok(last_modified)
        })?;

        for r in rows.flatten() {
            let r_u64: u64 = r.parse().unwrap();
            let timestamp = path.metadata().unwrap().modified().unwrap();
            let timestamp_u64 = timestamp.duration_since(UNIX_EPOCH).unwrap().as_secs();
            if timestamp_u64 <= r_u64 {
                return Ok(false);
            }
        }

        Ok(true)
    }

    /// Get all Track Paths from the database which dont exist on disk anymore
    fn need_delete(conn: &Arc<Mutex<Connection>>) -> Result<Vec<String>> {
        let conn = conn.lock();
        let mut stmt = conn.prepare("SELECT * FROM tracks")?;

        let track_vec: Vec<String> = stmt
            .query_map([], TrackDB::try_from_row_named)?
            .flatten()
            .filter_map(|record| {
                let path = Path::new(&record.file);
                if path.exists() {
                    None
                } else {
                    Some(record.file)
                }
            })
            .collect();

        Ok(track_vec)
    }

    /// Delete Tracks from the database by the full file path
    fn delete_records(conn: &Arc<Mutex<Connection>>, tracks: Vec<String>) -> Result<()> {
        let mut conn = conn.lock();
        let tx = conn.transaction()?;

        for track in tracks {
            tx.execute("DELETE FROM tracks WHERE file = ?", params![track])?;
        }

        tx.commit()?;
        Ok(())
    }

    /// Synchronize the database with the on-disk paths (insert, update, remove), limited to `path` root
    pub fn sync_database(&mut self, path: &Path) {
        // add updated records
        let conn = self.conn.clone();
        let all_items = {
            let mut walker = walkdir::WalkDir::new(path).follow_links(true);

            if let ScanDepth::Limited(limit) = self.max_depth {
                walker = walker.max_depth(usize::try_from(limit).unwrap_or(usize::MAX));
            }

            walker
        };

        std::thread::spawn(move || -> Result<()> {
            let mut need_updates: Vec<Track> = vec![];

            for record in all_items
                .into_iter()
                .filter_map(std::result::Result::ok)
                .filter(|f| f.file_type().is_file())
                .filter(|f| filetype_supported(&f.path().to_string_lossy()))
            {
                match Self::need_update(&conn, record.path()) {
                    Ok(true) => {
                        if let Ok(track) = Track::read_from_path(record.path(), true) {
                            need_updates.push(track);
                        }
                    }
                    Ok(false) => {}
                    Err(e) => {
                        error!("Error in need_update: {e}");
                    }
                }
            }
            if !need_updates.is_empty() {
                Self::add_records(&conn, need_updates)?;
            }

            // delete records where local file are missing

            match Self::need_delete(&conn) {
                Ok(string_vec) => {
                    if !string_vec.is_empty() {
                        Self::delete_records(&conn, string_vec)?;
                    }
                }
                Err(e) => {
                    error!("Error in need_delete: {e}");
                }
            }

            Ok(())
        });
    }

    /// Get all Tracks in the database at once
    pub fn get_all_records(&mut self) -> Result<Vec<TrackDB>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare("SELECT * FROM tracks")?;
        let vec: Vec<TrackDB> = stmt
            .query_map([], TrackDB::try_from_row_named)?
            .flatten()
            .collect();
        Ok(vec)
    }

    /// Get Tracks by [`SearchCriteria`]
    pub fn get_record_by_criteria(
        &mut self,
        criteria_val: &str,
        criteria: &SearchCriteria,
    ) -> Result<Vec<TrackDB>> {
        let search_str = format!("SELECT * FROM tracks WHERE {criteria} = ?");
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(&search_str)?;

        let mut vec_records: Vec<TrackDB> = stmt
            .query_map([criteria_val], TrackDB::try_from_row_named)?
            .flatten()
            .collect();

        // Left for debug
        // error!("criteria_val: {}", criteria_val);
        // error!("criteria: {}", criteria);
        // error!("vec: {:?}", vec_records);

        vec_records.sort_by_cached_key(|k| get_pin_yin(&k.name));
        Ok(vec_records)
    }

    /// Get a list of available distinct [`SearchCriteria`] (ie get Artist names deduplicated)
    pub fn get_criterias(&mut self, criteria: &SearchCriteria) -> Result<Vec<String>> {
        let search_str = format!("SELECT DISTINCT {criteria} FROM tracks");
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(&search_str)?;

        let mut vec: Vec<String> = stmt
            .query_map([], |row| {
                let criteria: String = row.get(0)?;
                Ok(criteria)
            })?
            .flatten()
            .collect();

        vec.sort_by_cached_key(|k| get_pin_yin(k));
        Ok(vec)
    }

    /// Get the stored `last_position` of a given track
    pub fn get_last_position(&mut self, track: &Track) -> Result<Duration> {
        let filename = track
            .name()
            .ok_or_else(|| Error::InvalidParameterName("file name missing".to_string()))?;
        let query = "SELECT last_position FROM tracks WHERE name = ?1";

        let mut last_position: Duration = Duration::from_secs(0);
        let conn = self.conn.lock();
        conn.query_row(query, params![filename], |row| {
            let last_position_u64: u64 = row.get(0)?;
            // error!("last_position_u64 is {last_position_u64}");
            last_position = Duration::from_secs(last_position_u64);
            Ok(last_position)
        })?;
        // error!("get last pos as {}", last_position.as_secs());
        Ok(last_position)
    }

    /// Set the stored `last_position` of a given track
    pub fn set_last_position(&mut self, track: &Track, last_position: Duration) -> Result<()> {
        let filename = track
            .name()
            .ok_or_else(|| Error::InvalidParameterName("file name missing".to_string()))?;
        let query = "UPDATE tracks SET last_position = ?1 WHERE name = ?2";
        let conn = self.conn.lock();
        conn.execute(query, params![last_position.as_secs(), filename,])?;
        // error!("set last position as {}", last_position.as_secs());
        Ok(())
    }

    /// Get a Track by the given full file path
    pub fn get_record_by_path(&mut self, file_path: &str) -> Result<TrackDB> {
        let search_str = "SELECT * FROM tracks WHERE file = ?";
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(search_str)?;

        let vec_records: Vec<TrackDB> = stmt
            .query_map([file_path], TrackDB::try_from_row_named)?
            .flatten()
            .collect();

        if let Some(record) = vec_records.first() {
            return Ok(record.clone());
        }

        Err(Error::QueryReturnedNoRows)
    }
}

#[cfg(test)]
mod test_utils {
    use rusqlite::Connection;

    /// Open a new In-Memory sqlite database
    pub fn gen_database() -> Connection {
        Connection::open_in_memory().expect("open db failed")
    }
}
