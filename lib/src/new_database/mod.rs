#![allow(clippy::unnecessary_debug_formatting)] // for logging we want all paths's characters to be escaped

use std::{fmt::Debug, path::Path, sync::Arc};

use anyhow::{Context, Result};
use parking_lot::{Mutex, MutexGuard};
use rusqlite::Connection;
use tokio::{
    runtime::Handle,
    sync::{OwnedSemaphorePermit, Semaphore},
};
use track_insert::TrackInsertable;
use walkdir::DirEntry;

use crate::{
    config::{ServerOverlay, v2::server::ScanDepth},
    track::{MetadataOptions, parse_metadata_from_file},
    utils::{filetype_supported, get_app_new_database_path},
};

/// Sqlite / rusqlite integer type alias.
///
/// This alias exists to keep it in one place and because rusqlite does not export such a type.
pub type Integer = i64;

mod album_insert;
pub mod album_ops;
mod artist_insert;
pub mod artist_ops;
mod migrate;
mod track_insert;
pub mod track_ops;

#[allow(clippy::doc_markdown)]
/// The SQLite Database interface.
///
/// This *can* be shared between threads via `clone`, **but** only one operation may occur at a time.
#[derive(Clone)]
pub struct Database {
    conn: Arc<Mutex<Connection>>,
    /// Limit how many scanners are active at a time
    semaphore: Arc<Semaphore>,
}

impl Debug for Database {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DataBase")
            .field("conn", &"<unavailable>")
            .finish()
    }
}

impl Database {
    /// Create a new database at the given `path`, with all migrations applied
    ///
    /// # Panics
    ///
    /// - if database creation fails
    /// - if database migration fails
    pub fn new(path: &Path) -> Result<Self> {
        let conn = Connection::open(path).context("open/create database")?;

        Self::new_from_connection(conn)
    }

    /// Create a new database at the default app config path.
    ///
    /// # Panics
    ///
    /// see [`new`](Self::new).
    pub fn new_default_path() -> Result<Self> {
        Self::new(&get_app_new_database_path()?)
    }

    /// Get a lock to the underlying connection to start operations.
    pub fn get_connection(&self) -> MutexGuard<'_, Connection> {
        self.conn.lock()
    }

    /// Prepare the given Connection for usage.
    fn new_from_connection(conn: Connection) -> Result<Self> {
        migrate::migrate(&conn).context("Database migration")?;

        let conn = Arc::new(Mutex::new(conn));
        // for now limit to one worker at a time
        let semaphore = Arc::new(Semaphore::new(1));
        Ok(Self { conn, semaphore })
    }

    /// Scan the given path recursively, limited to [`ServerOverlay::get_library_scan_depth`].
    ///
    /// If `replace_metadata` is `false` then paths that already exist in the database will not be updated.
    ///
    /// Waits for a permit before starting another worker.
    pub fn scan_path(
        &self,
        path: &Path,
        config: &ServerOverlay,
        replace_metadata: bool,
    ) -> Result<()> {
        let path = path
            .canonicalize()
            .with_context(|| path.display().to_string())?;

        let walker = {
            let mut walker = walkdir::WalkDir::new(&path).follow_links(true);

            if let ScanDepth::Limited(limit) = config.get_metadata_scan_depth() {
                walker = walker.max_depth(usize::try_from(limit).unwrap_or(usize::MAX));
            }

            walker
                .into_iter()
                .filter_map(Result::ok)
                // only process files which we support
                .filter(|v| v.file_type().is_file())
                .filter(|v| filetype_supported(v.path()))
        };
        let db = self.clone();

        let handle = Handle::current();
        let handle_1 = handle.clone();
        let separators = config.settings.metadata.artist_separators.clone();

        // first spawn a task to acquire a permit, then spawn a blocking task as WalkDir and rusqlite are sync-only.
        handle.spawn(async move {
            let Ok(permit) = db.semaphore.clone().acquire_owned().await else {
                error!("Failed to acquite permit for scanner!");
                return;
            };

            handle_1.spawn_blocking(move || {
                let separators: Vec<&str> = separators.iter().map(String::as_str).collect();
                Self::process_iter(walker, permit, &db, &path, replace_metadata, &separators);
            });
        });

        Ok(())
    }

    /// The actual function to walk the iterator of files for [`Self::scan_path`].
    ///
    /// Expects `path` to be absolute.
    fn process_iter(
        walker: impl Iterator<Item = DirEntry>,
        permit: OwnedSemaphorePermit,
        db: &Self,
        path: &Path,
        replace_metadata: bool,
        separators: &[&str],
    ) {
        // keep the permit for the entirety of this function
        let _permit = permit;
        info!("Scanning {path:#?}");

        let mut created_updated: usize = 0;

        // assumptions in this function:
        // - "walker" iterator is already filtered to only contain files
        // - "walker" iterator is already filtered to only our supported file types
        for record in walker {
            let path = record.path();

            // skip existing paths, if no full scan is requested
            if !replace_metadata {
                match track_ops::track_exists(&db.conn.lock(), path) {
                    Ok(true) => continue,
                    Err(err) => {
                        warn!("Error checking if {path:#?} exists: {err:#?}");
                        continue;
                    }
                    Ok(false) => (),
                }
            }

            let track_metadata = match parse_metadata_from_file(
                path,
                MetadataOptions {
                    album: true,
                    album_artist: true,
                    album_artists: true,
                    artist: true,
                    artists: true,
                    artist_separators: separators,
                    title: true,
                    duration: true,
                    genre: true,
                    ..Default::default()
                },
            ) {
                Ok(v) => v,
                Err(err) => {
                    warn!("Error scanning path {path:#?}: {err:#?}");
                    continue;
                }
            };

            let db_track = match TrackInsertable::try_from_track(path, &track_metadata) {
                Ok(v) => v,
                Err(err) => {
                    warn!("Error converting to database track {path:#?}: {err:#?}");
                    continue;
                }
            };

            let _id = match db_track.try_insert_or_update(&db.conn.lock()) {
                Ok(v) => v,
                Err(err) => {
                    warn!("Error inserting or updating {path:#?}: {err:#?}");
                    continue;
                }
            };

            created_updated += 1;
        }

        info!("Finished Scanning {path:#?} with {created_updated} created or updated");
    }
}

#[cfg(test)]
mod test_utils {
    use std::path::{Path, PathBuf};

    use rusqlite::Connection;

    use super::Database;

    /// Open a new In-Memory sqlite database
    pub fn gen_database_raw() -> Connection {
        Connection::open_in_memory().expect("open db failed")
    }

    /// Open a new In-Memory database that is fully prepared
    pub fn gen_database() -> Database {
        Database::new_from_connection(gen_database_raw()).expect("db creation failed")
    }

    /// Unix / DOS path handling, because depending on the system paths would otherwise not be absolute
    pub fn test_path(path: &Path) -> PathBuf {
        if cfg!(windows) {
            let mut pathbuf = PathBuf::from("C:\\");
            pathbuf.push(path);

            pathbuf
        } else {
            path.to_path_buf()
        }
    }

    #[test]
    #[cfg(unix)]
    fn test_path_absolute_unix() {
        let path = test_path(Path::new("/somewhere/else"));
        assert!(path.is_absolute());

        assert_eq!(path, Path::new("/somewhere/else"));
    }

    #[test]
    #[cfg(windows)]
    fn test_path_absolute_windows() {
        let path = test_path(Path::new("/somewhere/else"));
        assert!(path.is_absolute());

        assert_eq!(path, Path::new("C:\\somewhere\\else"));
    }
}
