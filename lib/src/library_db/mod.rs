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
// database
use crate::config::ServerOverlay;
use crate::track::Track;
use crate::utils::{filetype_supported, get_app_config_path, get_pin_yin};
use parking_lot::Mutex;
use rusqlite::{params, Connection, Error, Result};
use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, UNIX_EPOCH};
use track_db::TrackDBInsertable;

mod track_db;

pub use track_db::TrackDB;

const DB_VERSION: u32 = 2;

pub struct DataBase {
    conn: Arc<Mutex<Connection>>,
    max_depth: ScanDepth,
}

#[derive(PartialEq, Eq)]
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
    pub fn new(config: &ServerOverlay) -> Self {
        let mut db_path = get_app_config_path().expect("failed to get app configuration path");
        db_path.push("library.db");
        let conn = Connection::open(db_path).expect("open db failed");

        let user_version: u32 = conn
            .query_row("SELECT user_version FROM pragma_user_version", [], |r| {
                r.get(0)
            })
            .expect("get user_version error");
        if DB_VERSION != user_version {
            conn.execute("DROP TABLE tracks", []).ok();
            conn.pragma_update(None, "user_version", DB_VERSION)
                .expect("update user_version error");
        }

        conn.execute(include_str!("./migrations/002.sql"), [])
            .expect("Database version 2 could not be created");

        let max_depth = config.get_library_scan_depth();

        let conn = Arc::new(Mutex::new(conn));
        Self { conn, max_depth }
    }

    fn add_records(conn: &Arc<Mutex<Connection>>, tracks: Vec<Track>) -> Result<()> {
        let mut conn = conn.lock();
        let tx = conn.transaction()?;

        for track in tracks {
            TrackDBInsertable::from(&track).insert_track(&tx)?;
        }

        tx.commit()?;
        Ok(())
    }

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

    fn need_delete(conn: &Arc<Mutex<Connection>>) -> Result<Vec<String>> {
        let conn = conn.lock();
        let mut stmt = conn.prepare("SELECT * FROM tracks")?;
        let mut track_vec: Vec<String> = vec![];
        let vec: Vec<TrackDB> = stmt
            .query_map([], TrackDB::try_from_row_named)?
            .flatten()
            .collect();
        for record in vec {
            let path = Path::new(&record.file);
            if path.exists() {
                continue;
            }
            track_vec.push(record.file.clone());
        }
        Ok(track_vec)
    }

    fn delete_records(conn: &Arc<Mutex<Connection>>, tracks: Vec<String>) -> Result<()> {
        let mut conn = conn.lock();
        let tx = conn.transaction()?;

        for track in tracks {
            tx.execute("DELETE FROM tracks WHERE file = ?", params![track])?;
        }

        tx.commit()?;
        Ok(())
    }

    pub fn sync_database(&mut self, path: &Path) {
        // add updated records
        let conn = self.conn.clone();
        let mut track_vec: Vec<Track> = vec![];
        let all_items = {
            let mut walker = walkdir::WalkDir::new(path).follow_links(true);

            if let ScanDepth::Limited(limit) = self.max_depth {
                walker = walker.max_depth(usize::try_from(limit).unwrap_or(usize::MAX));
            }

            walker
        };

        std::thread::spawn(move || -> Result<()> {
            for record in all_items
                .into_iter()
                .filter_map(std::result::Result::ok)
                .filter(|f| f.file_type().is_file())
                .filter(|f| filetype_supported(&f.path().to_string_lossy()))
            {
                match Self::need_update(&conn, record.path()) {
                    Ok(true) => {
                        if let Ok(track) = Track::read_from_path(record.path(), true) {
                            track_vec.push(track);
                        }
                    }
                    Ok(false) => {}
                    Err(e) => {
                        error!("Error in need_update: {e}");
                    }
                }
            }
            if !track_vec.is_empty() {
                Self::add_records(&conn, track_vec)?;
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

    /// # Panics
    ///
    /// if the connection is unavailable
    pub fn get_all_records(&mut self) -> Result<Vec<TrackDB>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare("SELECT * FROM tracks")?;
        let vec: Vec<TrackDB> = stmt
            .query_map([], TrackDB::try_from_row_named)?
            .flatten()
            .collect();
        Ok(vec)
    }

    /// # Panics
    ///
    /// if the connection is unavailable
    pub fn get_record_by_criteria(
        &mut self,
        str: &str,
        cri: &SearchCriteria,
    ) -> Result<Vec<TrackDB>> {
        let search_str = format!("SELECT * FROM tracks WHERE {cri} = ?");
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(&search_str)?;

        let mut vec_records: Vec<TrackDB> = stmt
            .query_map([str], TrackDB::try_from_row_named)?
            .flatten()
            .collect();

        // Left for debug
        // error!("str: {}", str);
        // error!("cri: {}", cri);
        // error!("vec: {:?}", vec_records);

        vec_records.sort_by_cached_key(|k| get_pin_yin(&k.name));
        Ok(vec_records)
    }

    /// # Panics
    ///
    /// if the connection is unavailable
    pub fn get_criterias(&mut self, cri: &SearchCriteria) -> Result<Vec<String>> {
        let search_str = format!("SELECT DISTINCT {cri} FROM tracks");
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

    /// # Panics
    ///
    /// if the connection is unavailable
    pub fn get_last_position(&mut self, track: &Track) -> Result<Duration> {
        let query = "SELECT last_position FROM tracks WHERE name = ?1";

        let mut last_position: Duration = Duration::from_secs(0);
        let conn = self.conn.lock();
        conn.query_row(
            query,
            params![track.name().unwrap_or("Unknown File").to_string(),],
            |row| {
                let last_position_u64: u64 = row.get(0)?;
                // error!("last_position_u64 is {last_position_u64}");
                last_position = Duration::from_secs(last_position_u64);
                Ok(last_position)
            },
        )?;
        // .expect("get last position failed.");
        // error!("get last pos as {}", last_position.as_secs());
        Ok(last_position)
    }

    /// # Panics
    ///
    /// if the connection is unavailable
    pub fn set_last_position(&mut self, track: &Track, last_position: Duration) {
        let query = "UPDATE tracks SET last_position = ?1 WHERE name = ?2";
        let conn = self.conn.lock();
        conn.execute(
            query,
            params![
                last_position.as_secs(),
                track.name().unwrap_or("Unknown File Name").to_string(),
            ],
        )
        .expect("update last position failed.");
        // error!("set last position as {}", last_position.as_secs());
    }

    /// # Panics
    ///
    /// if the connection is unavailable
    pub fn get_record_by_path(&mut self, str: &str) -> Result<TrackDB> {
        let search_str = "SELECT * FROM tracks WHERE file = ?";
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(search_str)?;

        let vec_records: Vec<TrackDB> = stmt
            .query_map([str], TrackDB::try_from_row_named)?
            .flatten()
            .collect();

        // Left for debug
        // error!("str: {}", str);
        // error!("cri: {}", cri);
        if let Some(record) = vec_records.first() {
            return Ok(record.clone());
        }

        Err(Error::QueryReturnedNoRows)
    }
}
