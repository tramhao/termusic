// database
use crate::config::{get_app_config_path, Termusic};
use crate::track::Track;
use crate::ui::model::Model;
use rusqlite::{params, Connection, Result};
use std::path::PathBuf;
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

        // conn.execute(
        //     "create table if not exists directory(
        //      id integer primary key
        //      ,name text not null
        //  )",
        //     [],
        // )
        // .expect("creat table directory failed");
        // ,directory_id integer not null references directory(id)

        // ,album    TEXT NOT NULL
        //        ,title    TEXT NOT NULL
        //        ,file     TEXT UNIQUE
        //        ,duration DOUBLE
        //        ,name     TEXT
        //        ,ext     TEXT
        //        ,directory_id TEXT
        //        ,last_modified TEXT
        conn.execute(
            "create table if not exists track(
             id integer primary key,
             artist TEXT NOT NULL,
             title TEXT NOT NULL,
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

    fn add_record(&mut self, track: &Track) -> Result<()> {
        // self.conn.execute(
        //     "insert into directory (name) values (?1)",
        //     &[&track.directory()],
        // )?;
        // let last_id: String = self.conn.last_insert_rowid().to_string();
        self.conn.execute(
            // "insert into track (name, file, directory_id,last_modified) values (?1, ?2, ?3, ?4)",
            // "insert into track (name, file, directory, last_modified) values (?1, ?2, ?3, ?4)",
            "INSERT INTO track (artist, title, file, duration, name, ext, directory, last_modified) 
            values (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                track.artist().unwrap_or("Unknown Artist").to_string(),
                track.title().unwrap_or("Unknown Title").to_string(),
                track.file().unwrap_or("Unknown File").to_string(),
                track.duration().as_secs(),
                track.name().unwrap_or_default().to_string(),
                track.ext().unwrap_or_default().to_string(),
                // &track.file().unwrap_or_default().to_string(),
                // // &last_id,
                track.directory().unwrap_or_default().to_string(),
                track
                    .last_modified
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs()
                    .to_string(),
            ],
        )?;

        Ok(())
    }

    pub fn need_update(&self, track: &Track) -> Result<bool> {
        let mut stmt = self.conn.prepare("SELECT file, last_modified FROM track")?;
        // let tracks = stmt.query_map([], |row| {
        let rows = stmt.query_map([], |row| {
            // let path_str: String = row.get(0)?;
            // let track = Track::read_from_path(path_str);
            // track.name = row.get(0)?;
            // track.directory = row.get(2)?;
            let file: String = row.get(0)?;
            let last_modified: String = row.get(1)?;

            Ok((file, last_modified))
            // Ok(track)
        })?;

        for r in rows {
            eprintln!("Found my track {:?}", r);
        }

        Ok(true)
    }

    pub fn sync_database(&mut self) {
        let all_items = walkdir::WalkDir::new(self.path.as_path()).follow_links(true);
        for record in all_items.into_iter().filter_map(std::result::Result::ok) {
            let track = Track::read_from_path(record.path()).unwrap();
            if let Ok(true) = self.need_update(&track) {
                self.add_record(&track).expect("add record error");
            }
        }
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

        for cat in cats {
            eprintln!("Found my track {:?}", cat);
        }
        Ok(())
    }
}
