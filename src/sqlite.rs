// database
use crate::config::{get_app_config_path, Termusic};
use crate::track::Track;
use crate::ui::model::Model;
use rusqlite::{params, Connection, Error, Result};
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

#[allow(unused)]
pub struct DataBase {
    conn: Connection,
    path: PathBuf,
}

#[derive(Debug)]
struct TrackForDB {
    name: String,
    color: String,
}

#[allow(unused)]
impl DataBase {
    pub fn new(config: &Termusic) -> Self {
        let path = Model::get_full_path_from_config(config);
        let mut db_path = get_app_config_path().expect("failed to get app configuration path");
        db_path.push("library.db");
        let conn = Connection::open(db_path).expect("open db failed");

        // USLT lyrics
        // lyric_frames: Vec<Lyrics>,
        // lyric_selected_index: usize,
        // parsed_lyric: Option<Lyric>,
        // picture: Option<Picture>,
        // album_photo: Option<String>,
        // file_type: Option<FileType>,

        conn.execute(
            "create table if not exists track(
             id integer primary key,
             artist TEXT,
             title TEXT,
             file TEXT NOT NULL,
             duration DOUBLE,
             name TEXT,
             ext TEXT,
             directory TEXT,
             last_modified TEXT
            )",
            [],
        )
        .expect("create table track failed");

        Self { conn, path }
    }

    fn add_records(&mut self, tracks: Vec<Track>) -> Result<()> {
        let tx = self.conn.transaction()?;

        for track in tracks {
            tx.execute(
            "INSERT INTO track (artist, title, file, duration, name, ext, directory, last_modified) 
            values (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                track.artist().unwrap_or("Unknown Artist").to_string(),
                track.title().unwrap_or("Unknown Title").to_string(),
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

    pub fn need_update(&self, track: &Track) -> Result<bool> {
        let filename = track
            .file()
            .ok_or_else(|| Error::InvalidParameterName("file name missing".to_string()))?
            .to_string();
        let mut stmt = self
            .conn
            .prepare("SELECT last_modified FROM track WHERE file = ? ")?;
        let rows = stmt.query_map([filename], |row| {
            let last_modified: String = row.get(0)?;

            Ok(last_modified)
        })?;

        for r in rows.flatten() {
            let r_u64: u64 = r.parse().unwrap();
            let file = track.file().unwrap();
            let path = Path::new(file);
            let timestamp = path.metadata().unwrap().modified().unwrap();
            let timestamp_u64 = timestamp.duration_since(UNIX_EPOCH).unwrap().as_secs();
            if timestamp_u64 <= r_u64 {
                return Ok(false);
            }
            // These two lines are not printed, so the true return for new file is somewhere else.
            // eprintln!("last_modified from db: {}", r_u64);
            // eprintln!("timestamp from file: {}", timestamp_u64);
        }

        Ok(true)
    }

    pub fn sync_database(&mut self) {
        let mut track_vec: Vec<Track> = vec![];
        let all_items = walkdir::WalkDir::new(self.path.as_path()).follow_links(true);
        for record in all_items
            .into_iter()
            .filter_map(std::result::Result::ok)
            .filter(|f| f.file_type().is_file())
        {
            let track = Track::read_from_path(record.path()).unwrap();
            match self.need_update(&track) {
                Ok(true) => {
                    // eprintln!("Updating: {:?}", track.file());
                    track_vec.push(track);
                }
                Ok(false) => {
                    // eprintln!("Not adding: {:?}", track.file());
                }
                Err(e) => {
                    eprintln!("Error in need_update: {}", e);
                }
            }
        }
        self.add_records(track_vec).expect("add record error");
        self.get_record().expect("get record error");
    }

    pub fn get_record(&mut self) -> Result<()> {
        let mut stmt = self.conn.prepare("SELECT artist, file FROM track")?;
        // let tracks = stmt.query_map([], |row| {
        let cats = stmt.query_map([], |row| {
            // let path_str: String = row.get(0)?;
            // let track = Track::read_from_path(path_str);
            // track.name = row.get(0)?;
            // track.directory = row.get(2)?;

            Ok(TrackForDB {
                name: row.get(0)?,
                color: row.get(1)?,
            })
            // Ok(track)
        })?;

        for r in cats.flatten() {
            // eprintln!("Found my track {:?}", cat);
            // if let Ok(r) = cat {
            let name = r.name;
            let color = r.color;
            // }
        }
        Ok(())
    }
}
