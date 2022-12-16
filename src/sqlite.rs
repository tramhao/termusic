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
use crate::config::{get_app_config_path, Settings};
use crate::track::Track;
use crate::utils::{filetype_supported, get_pin_yin};
use rusqlite::{params, Connection, Error, Result, Row};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::{Duration, UNIX_EPOCH};

const DB_VERSION: u32 = 2;

pub struct DataBase {
    conn: Arc<Mutex<Connection>>,
    max_depth: usize,
}

#[derive(Clone, Debug)]
pub struct TrackForDB {
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

pub enum SearchCriteria {
    Artist,
    Album,
    Genre,
    Directory,
}

impl From<usize> for SearchCriteria {
    fn from(u_index: usize) -> Self {
        match u_index {
            1 => Self::Album,
            2 => Self::Genre,
            3 => Self::Directory,
            _ => Self::Artist,
            // 0 | _ => Self::Artist,
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
        }
    }
}

impl DataBase {
    pub fn new(config: &Settings) -> Self {
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

        conn.execute(
            "create table if not exists tracks(
             id integer primary key,
             artist TEXT,
             title TEXT,
             album TEXT,
             genre TEXT,
             file TEXT NOT NULL,
             duration INTERGER,
             name TEXT,
             ext TEXT,
             directory TEXT,
             last_modified TEXT,
             last_position INTERGER
            )",
            [],
        )
        .expect("create table tracks failed");

        let max_depth = config.max_depth_cli;

        let conn = Arc::new(Mutex::new(conn));
        Self { conn, max_depth }
    }

    fn add_records(conn: &Arc<Mutex<Connection>>, tracks: Vec<Track>) -> Result<()> {
        let mut conn = conn.lock().expect("conn is not available for add records");
        let tx = conn.transaction()?;

        for track in tracks {
            tx.execute(
            "INSERT INTO tracks (artist, title, album, genre,  file, duration, name, ext, directory, last_modified, last_position) 
            values (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                track.artist().unwrap_or("Unknown Artist").to_string(),
                track.title().unwrap_or("Unknown Title").to_string(),
                track.album().unwrap_or("empty").to_string(),
                track.genre().unwrap_or("no type").to_string(),
                track.file().unwrap_or("Unknown File").to_string(),
                track.duration().as_secs(),
                track.name().unwrap_or_default().to_string(),
                track.ext().unwrap_or_default().to_string(),
                track.directory().unwrap_or_default().to_string(),
                track
                    .last_modified
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs()
                    .to_string(),
                0,

            ],
        )?;
        }

        tx.commit()?;
        Ok(())
    }

    fn need_update(conn: &Arc<Mutex<Connection>>, path: &Path) -> Result<bool> {
        let conn = conn.lock().expect("conn is not available for need update.");
        let filename = path
            .file_name()
            .ok_or_else(|| Error::InvalidParameterName("file name missing".to_string()))?
            .to_string_lossy();
        let mut stmt = conn.prepare("SELECT last_modified FROM tracks WHERE name = ? ")?;
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
        let conn = conn.lock().expect("conn is not available for need delete");
        let mut stmt = conn.prepare("SELECT * FROM tracks")?;
        let mut track_vec: Vec<String> = vec![];
        let vec: Vec<TrackForDB> = stmt
            .query_map([], |row| Ok(Self::track_db(row)))?
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
        let mut conn = conn
            .lock()
            .expect("conn is not available for delete records");
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
        let all_items = walkdir::WalkDir::new(path)
            .follow_links(true)
            .max_depth(self.max_depth);

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
                        eprintln!("Error in need_update: {}", e);
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
                    eprintln!("Error in need_delete: {}", e);
                }
            }

            Ok(())
        });
    }

    pub fn get_all_records(&mut self) -> Result<Vec<TrackForDB>> {
        let conn = self
            .conn
            .lock()
            .expect("conn is not available for get all records.");
        let mut stmt = conn.prepare("SELECT * FROM tracks")?;
        let vec: Vec<TrackForDB> = stmt
            .query_map([], |row| Ok(Self::track_db(row)))?
            .flatten()
            .collect();
        Ok(vec)
    }

    pub fn get_record_by_criteria(
        &mut self,
        str: &str,
        cri: &SearchCriteria,
    ) -> Result<Vec<TrackForDB>> {
        let search_str = format!("SELECT * FROM tracks WHERE {} = ?", cri);
        let conn = self
            .conn
            .lock()
            .expect("conn is not available for get record by criteria.");
        let mut stmt = conn.prepare(&search_str)?;

        let mut vec_records: Vec<TrackForDB> = stmt
            .query_map([str], |row| Ok(Self::track_db(row)))?
            .flatten()
            .collect();

        // Left for debug
        // eprintln!("str: {}", str);
        // eprintln!("cri: {}", cri);
        // eprintln!("vec: {:?}", vec_records);

        vec_records.sort_by_cached_key(|k| get_pin_yin(&k.name));
        Ok(vec_records)
    }

    fn track_db(row: &Row<'_>) -> TrackForDB {
        let d_u64: u64 = row.get(6).unwrap();
        let last_position_u64: u64 = row.get(11).unwrap();
        TrackForDB {
            // id: row.get(0).unwrap(),
            id: row.get_unwrap(0),
            artist: row.get_unwrap(1),
            title: row.get_unwrap(2),
            album: row.get(3).unwrap(),
            genre: row.get(4).unwrap(),
            file: row.get(5).unwrap(),
            duration: Duration::from_secs(d_u64),
            name: row.get(7).unwrap(),
            ext: row.get(8).unwrap(),
            directory: row.get(9).unwrap(),
            last_modified: row.get(10).unwrap(),
            last_position: Duration::from_secs(last_position_u64),
        }
    }

    pub fn get_criterias(&mut self, cri: &SearchCriteria) -> Vec<String> {
        let search_str = format!("SELECT DISTINCT {} FROM tracks", cri);
        let conn = self
            .conn
            .lock()
            .expect("conn is not available for get criterias.");
        let mut stmt = conn.prepare(&search_str).unwrap();

        let mut vec: Vec<String> = stmt
            .query_map([], |row| {
                let criteria: String = row.get(0).unwrap();
                Ok(criteria)
            })
            .unwrap()
            .flatten()
            .collect();

        vec.sort_by_cached_key(|k| get_pin_yin(k));
        vec
    }

    pub fn get_last_position(&mut self, track: &Track) -> Result<Duration> {
        let query = "SELECT last_position FROM tracks WHERE file = ?1";

        let mut last_position: Duration = Duration::from_secs(0);
        let conn = self
            .conn
            .lock()
            .expect("conn is not available for get last position.");
        conn.query_row(
            query,
            params![track.file().unwrap_or("Unknown File").to_string(),],
            |row| {
                let last_position_u64: u64 = row.get(0).unwrap();
                last_position = Duration::from_secs(last_position_u64);
                Ok(last_position)
            },
        )?;
        // .expect("get last position failed.");
        eprintln!("get last pos as {}", last_position.as_secs());
        Ok(last_position)
    }

    pub fn set_last_position(&mut self, track: &Track, last_position: Duration) {
        let query = "UPDATE tracks SET last_position = ?1 WHERE name = ?2";
        let conn = self
            .conn
            .lock()
            .expect("conn is not available for set last position.");
        conn.execute(
            query,
            params![
                last_position.as_secs(),
                track.name().unwrap_or("Unknown File Name").to_string(),
            ],
        )
        .expect("update last position failed.");
        eprintln!("set last position as {}", last_position.as_secs());
    }
}
