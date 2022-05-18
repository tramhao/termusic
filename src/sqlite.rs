// database
use crate::config::{get_app_config_path, Termusic};
use crate::track::Track;
use crate::ui::model::Model;
use rusqlite::{params, Connection, Error, Result, Row};
use std::path::{Path, PathBuf};
use std::time::{Duration, UNIX_EPOCH};

#[allow(unused)]
pub struct DataBase {
    conn: Connection,
    path: PathBuf,
}

#[derive(Debug)]
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
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
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

        Self { conn, path }
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
        if !track_vec.is_empty() {
            self.add_records(track_vec).expect("add record error");
        }
        // if let Ok(test1) = self.get_record_by_criteria("陈工", &SearchCriteria::Artist) {
        //     eprintln!("{:?}", test1);
        // };
        // if let Ok(test2) = self.get_record_by_criteria("夏天的风", &SearchCriteria::Title) {
        //     eprintln!("{:?}", test2);
        // };
        // if let Ok(test3) =
        //     self.get_record_by_criteria("/home/tramhao/Music/mp3/tmp", &SearchCriteria::Directory)
        // {
        //     eprintln!("{:?}", test3);
        // };
    }

    pub fn get_record_by_criteria(
        &mut self,
        str: &str,
        cri: &SearchCriteria,
    ) -> Result<Vec<TrackForDB>> {
        let search_str = format!("SELECT * FROM track WHERE {} = ?", cri);
        let mut stmt = self.conn.prepare(&search_str)?;

        // stmt.query_map(params, |row| Ok(song(row)))
        //     .unwrap()
        //     .flatten()
        //     .collect()
        let vec_records = stmt
            .query_map([str], |row| Ok(Self::track_db(row)))?
            .flatten()
            .collect();

        Ok(vec_records)
        // for r in cats.flatten() {
        //     eprintln!("Found my track {:?}", r);
        // }
        // Ok(())
    }

    fn track_db(row: &Row) -> TrackForDB {
        let d_f64: f64 = row.get(6).unwrap();
        TrackForDB {
            id: row.get(0).unwrap(),
            artist: row.get(1).unwrap(),
            title: row.get(2).unwrap(),
            album: row.get(3).unwrap(),
            genre: row.get(4).unwrap(),
            file: row.get(5).unwrap(),
            duration: Duration::from_secs_f64(d_f64),
            name: row.get(7).unwrap(),
            ext: row.get(8).unwrap(),
            directory: row.get(9).unwrap(),
            last_modified: row.get(10).unwrap(),
        }
    }

    pub fn get_criterias(&mut self, cri: usize) -> Vec<String> {
        let crit = SearchCriteria::from(cri);
        // eprintln!("{}", crit);
        let search_str = format!("SELECT DISTINCT {} FROM track", crit);
        // let cri_str2 = cri_str.clone();
        let mut stmt = self
            .conn
            // .prepare("SELECT DISTINCT ?1 FROM track ORDER BY ?2 COLLATE NOCASE")
            .prepare(&search_str)
            .unwrap();

        stmt.query_map([], |row| {
            let criteria: String = row.get(0).unwrap();
            Ok(criteria)
        })
        .unwrap()
        .flatten()
        .collect()
    }
}
