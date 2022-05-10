// database
use crate::config::{get_app_config_path, Termusic, MUSIC_DIR};
use crate::track::Track;
use crate::ui::model::Model;
use rusqlite::{Connection, Result};
// use rusqlite::Connection;
use glob::glob;
use std::path::PathBuf;

#[allow(unused)]
pub struct DataBase {
    conn: Connection,
    path: PathBuf,
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
            "create table if not exists directory(
             id integer primary key
             ,name text not null
         )",
            [],
        )
        .expect("creat table directory failed");
        conn.execute(
            "create table if not exists track(
             id integer primary key
             ,artist   TEXT NOT NULL
             ,album    TEXT NOT NULL
             ,title    TEXT NOT NULL
             ,file     TEXT NOT NULL UNIQUE
             ,duration DOUBLE NOT NULL
             ,name     TEXT NOT NULL
             ,ext     TEXT NOT NULL
             ,directory_id integer not null references directory(id)
         )",
            [],
        )
        .expect("create table track failed");
        Self { conn, path }
    }

    fn add_record(&mut self, track: &Track) -> Result<()> {
        self.conn.execute(
            "insert into directory (name) values (?1)",
            &[&track.directory()],
        )?;
        let last_id: String = self.conn.last_insert_rowid().to_string();
        self.conn.execute(
            "insert into track (name, directory_id) values (?1, ?2)",
            &[&track.name().unwrap_or_default().to_string(), &last_id],
        )?;
        Ok(())
    }

    pub fn need_update() -> bool {
        todo!()
    }

    pub fn sync_database(&mut self) -> Result<()> {
        let mut pattern = self.path.clone();
        pattern.push("**");
        pattern.push("*.*");
        let music_dir = format!("{}/**/*.*", MUSIC_DIR);
        for path in glob(pattern.to_str().unwrap_or(&music_dir))
            .unwrap()
            .filter_map(std::result::Result::ok)
        {
            println!("{:?}", path.display());
        } // if let Ok(paths) = std::fs::read_dir(self.path) {
          //     let mut paths: Vec<_> = paths
          //         .filter_map(std::result::Result::ok)
          //         .filter(|p| !p.file_name().into_string().unwrap().starts_with('.'))
          //         .collect();

        //     paths.sort_by_cached_key(|k| get_pin_yin(&k.file_name().to_string_lossy().to_string()));
        //     for p in paths {
        //         node.add_child(Self::library_dir_tree(p.path().as_path(), depth - 1));
        //     }
        // }

        Ok(())
    }
    pub fn get_record() {
        todo!()
    }
}
