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
use rand::seq::SliceRandom;
use rusqlite::{params, Connection, Error, Result, Row};
use std::path::Path;
use std::time::{Duration, UNIX_EPOCH};

const DB_VERSION: u32 = 1;

#[allow(unused)]
pub struct DataBase {
    conn: Connection,
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

#[allow(unused)]
impl DataBase {
    pub fn new(config: &Settings) -> Self {
        let mut db_path = get_app_config_path().expect("failed to get app configuration path");
        db_path.push("library.db");
        let conn = Connection::open(db_path).expect("open db failed");
        // let conn = Connection::open_in_memory().expect("open db failed");

        let user_version: u32 = conn
            .query_row("SELECT user_version FROM pragma_user_version", [], |r| {
                r.get(0)
            })
            .expect("get user_version error");
        if DB_VERSION > user_version {
            conn.execute("DROP TABLE track", []).ok();
            conn.pragma_update(None, "user_version", DB_VERSION)
                .expect("update user_version error");
        }

        conn.execute(
            "create table if not exists track(
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
             last_modified TEXT
            )",
            [],
        )
        .expect("create table track failed");

        let max_depth = config.max_depth_cli;

        Self { conn, max_depth }
    }

    fn add_records(&mut self, tracks: Vec<Track>) -> Result<()> {
        let tx = self.conn.transaction()?;

        for track in tracks {
            tx.execute(
            "INSERT INTO track (artist, title, album, genre,  file, duration, name, ext, directory, last_modified) 
            values (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
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
            ],
        )?;
        }

        tx.commit();
        Ok(())
    }

    pub fn need_update(&self, path: &Path) -> Result<bool> {
        // let name = track
        //     .name()
        //     .ok_or_else(|| Error::InvalidParameterName("file name missing".to_string()))?
        //     .to_string();
        let name = path
            .file_name()
            .ok_or_else(|| Error::InvalidParameterName("file name missing".to_string()))?
            .to_string_lossy();
        // .to_string();
        let mut stmt = self
            .conn
            .prepare("SELECT last_modified FROM track WHERE name = ? ")?;
        let rows = stmt.query_map([name], |row| {
            let last_modified: String = row.get(0)?;

            Ok(last_modified)
        })?;

        for r in rows.flatten() {
            let r_u64: u64 = r.parse().unwrap();
            // let file = track.file().unwrap();
            // let path = Path::new(file);
            let timestamp = path.metadata().unwrap().modified().unwrap();
            let timestamp_u64 = timestamp.duration_since(UNIX_EPOCH).unwrap().as_secs();
            if timestamp_u64 <= r_u64 {
                return Ok(false);
            }
        }

        Ok(true)
    }

    fn delete_records(&mut self, tracks: Vec<String>) -> Result<()> {
        let tx = self.conn.transaction()?;

        for track in tracks {
            tx.execute("DELETE FROM track WHERE file = ?", params![track])?;
        }

        tx.commit();
        Ok(())
    }

    pub fn sync_database(&mut self, path: &Path) {
        // add updated records
        let mut track_vec: Vec<Track> = vec![];
        let all_items = walkdir::WalkDir::new(path)
            .follow_links(true)
            .max_depth(self.max_depth);
        for record in all_items
            .into_iter()
            .filter_map(std::result::Result::ok)
            .filter(|f| f.file_type().is_file())
            .filter(|f| filetype_supported(&f.path().to_string_lossy()))
        {
            // if let Ok(track) = Track::read_from_path(record.path()) {
            match self.need_update(record.path()) {
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
            // }
        }
        if !track_vec.is_empty() {
            self.add_records(track_vec).expect("add record error");
        }

        // delete records where local file are missing
        let mut track_vec2: Vec<String> = vec![];
        if let Ok(vec) = self.get_all_records() {
            for record in vec {
                let path = Path::new(&record.file);
                if path.exists() {
                    continue;
                }
                track_vec2.push(record.file.clone());
            }

            if !track_vec2.is_empty() {
                self.delete_records(track_vec2)
                    .expect("delete record error");
            }
        }
    }

    pub fn get_all_records(&mut self) -> Result<Vec<TrackForDB>> {
        let mut stmt = self.conn.prepare("SELECT * FROM track")?;
        let vec: Vec<TrackForDB> = stmt
            .query_map([], |row| Ok(Self::track_db(row)))?
            .flatten()
            .collect();
        Ok(vec)
    }

    pub fn get_records_for_cmus_tqueue(&mut self, quantity: u32) -> Vec<TrackForDB> {
        let mut result = vec![];
        if let Ok(vec) = self.get_all_records() {
            let mut i = 0;
            loop {
                if let Some(record) = vec.choose(&mut rand::thread_rng()) {
                    if record.title.contains("Unknown Title") {
                        continue;
                    }
                    if filetype_supported(&record.file) {
                        result.push(record.clone());
                        i += 1;
                        if i > quantity - 1 {
                            break;
                        }
                    }
                }
            }
        }
        result
    }

    pub fn get_records_for_cmus_lqueue(&mut self, quantity: u32) -> Vec<TrackForDB> {
        let mut result = vec![];
        if let Ok(vec) = self.get_all_records() {
            let mut i = 0;
            loop {
                if let Some(v) = vec.choose(&mut rand::thread_rng()) {
                    if v.album.contains("empty") {
                        continue;
                    }
                    if let Ok(mut vec2) =
                        self.get_record_by_criteria(&v.album, &SearchCriteria::Album)
                    {
                        result.append(&mut vec2);
                        i += 1;
                        if i > quantity - 1 {
                            break;
                        }
                    }
                }
            }
        }
        result
    }

    pub fn get_record_by_criteria(
        &mut self,
        str: &str,
        cri: &SearchCriteria,
    ) -> Result<Vec<TrackForDB>> {
        let search_str = format!("SELECT * FROM track WHERE {} = ?", cri);
        let mut stmt = self.conn.prepare(&search_str)?;

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
        TrackForDB {
            id: row.get(0).unwrap(),
            artist: row.get(1).unwrap(),
            title: row.get(2).unwrap(),
            album: row.get(3).unwrap(),
            genre: row.get(4).unwrap(),
            file: row.get(5).unwrap(),
            duration: Duration::from_secs(d_u64),
            name: row.get(7).unwrap(),
            ext: row.get(8).unwrap(),
            directory: row.get(9).unwrap(),
            last_modified: row.get(10).unwrap(),
        }
    }

    pub fn get_criterias(&mut self, cri: &SearchCriteria) -> Vec<String> {
        let search_str = format!("SELECT DISTINCT {} FROM track", cri);
        let mut stmt = self.conn.prepare(&search_str).unwrap();

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
}
