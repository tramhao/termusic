use anyhow::{Context, Result, bail};
use rusqlite::{Connection, TransactionBehavior, named_params};

/// The Current Database schema version this application is meant to run against
pub(super) const DB_VERSION: u32 = 2;

/// Helper function to get the `user_version` with a single function call.
#[inline]
fn get_user_version(conn: &Connection) -> Result<u32> {
    conn.query_row("SELECT user_version FROM pragma_user_version", [], |r| {
        r.get(0)
    })
    .context("get pragma \"user_version\"")
}

/// Helper function to set the `user_version` with a single function call.
///
/// Returns the passed version for re-use.
#[inline]
fn set_user_version(conn: &Connection, version: u32) -> Result<u32> {
    conn.pragma_update(None, "user_version", version)
        .context("update user_version error")?;

    Ok(version)
}

/// Check and update the database to be at [`DB_VERSION`].
pub(super) fn migrate(conn: &mut Connection) -> Result<()> {
    // Acquire an exclusive lock before reading the version so competing
    // initializers decide which migrations remain only after the previous writer commits.
    let tx = conn
        .transaction_with_behavior(TransactionBehavior::Exclusive)
        .context("begin library database migration transaction")?;
    let user_version: u32 = get_user_version(&tx)?;

    if user_version > DB_VERSION {
        bail!(
            "Expected Database version to be lower or equal to {DB_VERSION}, found {user_version}!"
        );
    }

    // only execute migrations if not already done so
    if user_version != DB_VERSION {
        apply_migrations(&tx, user_version)?;
    }

    tx.commit()
        .context("commit library database migration transaction")?;

    Ok(())
}

/// Apply migrations to be at [`DB_VERSION`].
#[allow(unused_assignments)] // for future possible migrations
fn apply_migrations(conn: &Connection, mut user_version: u32) -> Result<()> {
    if user_version == 0 {
        user_version = apply_version_1(conn)?;
    }

    if user_version == 1 {
        user_version = apply_version_2(conn)?;
    }

    set_last_updated_at(conn)?;

    Ok(())
}

/// Create the version 1 schema on a fresh database.
fn apply_version_1(conn: &Connection) -> Result<u32> {
    // Version 1 is the first schema, so there are basically no migrations, only creations
    conn.execute_batch(include_str!("./migrations/001.sql"))
        .context("Database version 1 could not be created")?;
    let user_version = set_user_version(conn, 1)?;

    set_db_created_at(conn)?;
    set_db_created_with(conn)?;

    Ok(user_version)
}

/// Migrate the database from version 1 to version 2.
fn apply_version_2(conn: &Connection) -> Result<u32> {
    // Add total_play_count and last_played_at columns for sort support (MostPlayed, Recency, Frecency) plus `added_at` column type change
    conn.execute_batch(include_str!("./migrations/002.sql"))
        .context("Database version 2 migration failed")?;
    let user_version = set_user_version(conn, 2)?;

    Ok(user_version)
}

// the following are to set some values in table "config", values which could help debugging database issues.

/// Set database config value `last_migrated_at` to the current time.
#[inline]
fn set_last_updated_at(conn: &Connection) -> Result<()> {
    let now = chrono::Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO config(key, value) VALUES (\"last_migrated_at\", :value)
            ON CONFLICT(key) DO UPDATE SET value=excluded.value;",
        named_params! {":value": now},
    )?;

    Ok(())
}

/// Set database config value `db_created_at` to the current time.
#[inline]
fn set_db_created_at(conn: &Connection) -> Result<()> {
    let now = chrono::Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO config(key, value) VALUES (\"db_created_at\", :value) ON CONFLICT(key) DO NOTHING;",
        named_params! {":value": now},
    )?;

    Ok(())
}

/// Set database config value `db_created_with` to the current time.
#[inline]
fn set_db_created_with(conn: &Connection) -> Result<()> {
    let version = crate::VERSION;

    conn.execute(
        "INSERT INTO config(key, value) VALUES (\"db_created_with\", :value) ON CONFLICT(key) DO NOTHING;",
        named_params! {":value": version},
    )?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::{
        cell::RefCell,
        fs,
        path::PathBuf,
        sync::{Arc, Barrier, mpsc},
        thread,
        time::{Duration, Instant},
    };

    use pretty_assertions::assert_eq;
    use rusqlite::{Connection, ErrorCode, TransactionBehavior};

    use crate::new_database::{
        Database,
        migrate::{
            DB_VERSION, apply_version_1, apply_version_2, get_user_version, migrate,
            set_user_version,
        },
    };

    use super::super::test_utils::gen_database_raw;

    const WAIT_TIMEOUT: Duration = Duration::from_secs(10);
    const ADDED_AT: &str = "2024-01-02T03:04:05+00:00";

    struct TempDatabase(PathBuf);

    impl TempDatabase {
        fn new() -> Self {
            let dir = std::env::temp_dir().join(format!(
                "termusic-migrate-{}-{}",
                std::process::id(),
                rand::random::<u64>()
            ));
            fs::create_dir(&dir).unwrap();
            Self(dir)
        }

        fn path(&self) -> PathBuf {
            self.0.join("library.db")
        }
    }

    impl Drop for TempDatabase {
        fn drop(&mut self) {
            // Connections must be dropped first, including on Windows.
            let result = fs::remove_dir_all(&self.0);
            if !thread::panicking() {
                result.unwrap();
            }
        }
    }

    fn version_one(conn: &Connection) {
        apply_version_1(conn).unwrap();
        conn.execute(
            "INSERT INTO tracks(id, file_dir, file_stem, file_ext, added_at)
             VALUES (42, '/music', 'example', 'mp3', ?1)",
            [ADDED_AT],
        )
        .unwrap();
    }

    fn get_all_schemas(conn: &Connection) -> Vec<(String, String)> {
        conn.prepare(
            "SELECT name, sql FROM sqlite_schema
             WHERE name NOT LIKE 'sqlite_%' ORDER BY name",
        )
        .unwrap()
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))
        .unwrap()
        .collect::<rusqlite::Result<_>>()
        .unwrap()
    }

    fn get_all_config_data(conn: &Connection) -> Vec<(String, String)> {
        conn.prepare("SELECT key, value FROM config ORDER BY key")
            .unwrap()
            .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))
            .unwrap()
            .collect::<rusqlite::Result<_>>()
            .unwrap()
    }

    /// Asserts that all config key-value pairs from the migration exist and are correct for DB version 2.
    fn assert_creation_metadata(conn: &Connection) {
        let values = get_all_config_data(conn);
        assert_eq!(values.len(), 3);
        assert_eq!(values[0].0, "db_created_at");
        chrono::DateTime::parse_from_rfc3339(&values[0].1).unwrap();
        assert_eq!(values[1], ("db_created_with".into(), crate::VERSION.into()));
        assert_eq!(values[2].0, "last_migrated_at");
        chrono::DateTime::parse_from_rfc3339(&values[2].1).unwrap();
    }

    /// Assert that the track from [`version_one`] is correctly migrated to version 2.
    fn assert_migrated_track(conn: &Connection) {
        let track = conn
            .query_row(
                "SELECT id, file_stem, added_at, total_play_count, last_played_at
                 FROM tracks",
                [],
                |r| {
                    Ok((
                        r.get::<_, i64>(0)?,
                        r.get::<_, String>(1)?,
                        r.get::<_, i64>(2)?,
                        r.get::<_, i64>(3)?,
                        r.get::<_, Option<i64>>(4)?,
                    ))
                },
            )
            .unwrap();
        assert_eq!(track, (42, "example".into(), 1_704_164_645, 0, None));
    }

    fn assert_full_schema(conn: &Connection) {
        // verify the migrated database is at the highest version we want to work with
        assert_eq!(DB_VERSION, get_user_version(conn).unwrap());

        // verify it has all the tables we expect
        let mut all_tracks: Vec<String> = {
            let mut prep = conn.prepare("SELECT name FROM sqlite_schema WHERE type ='table' AND name NOT LIKE 'sqlite_%';").unwrap();
            prep.query_map([], |r| r.get(0))
                .unwrap()
                .flatten()
                .collect()
        };

        all_tracks.sort();

        let expected = {
            let mut orig = [
                "config",
                "tracks",
                "tracks_metadata",
                "artists",
                "tracks_artists",
                "albums",
                "albums_artists",
            ];

            #[allow(clippy::stable_sort_primitive)]
            orig.sort();
            orig
        };

        assert_eq!(&all_tracks, &expected);

        let columns: Vec<(String, String)> = conn
            .prepare("SELECT name, type FROM pragma_table_info('tracks')")
            .unwrap()
            .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))
            .unwrap()
            .collect::<rusqlite::Result<_>>()
            .unwrap();
        for name in ["total_play_count", "last_played_at", "added_at"] {
            assert_eq!(columns.iter().filter(|(col, _)| col == name).count(), 1);
            assert!(columns.contains(&(name.into(), "INTEGER".into())));
        }
        assert!(!columns.iter().any(|(name, _)| name == "added_at_new"));
    }

    #[test]
    fn should_create_from_fresh() {
        let mut conn = gen_database_raw();

        // verify the created database is at 0
        assert_eq!(0, get_user_version(&conn).unwrap());
        migrate(&mut conn).unwrap();
        assert_full_schema(&conn);
        assert_creation_metadata(&conn);

        conn.execute_batch(
            "INSERT INTO tracks(id, file_dir, file_stem, file_ext, added_at)
             VALUES (42, '/music', 'example', 'mp3', 1704164645);
             UPDATE config SET value = 'sentinel' WHERE key = 'last_migrated_at';",
        )
        .unwrap();
        let original_schema = get_all_schemas(&conn);
        let original_metadata = get_all_config_data(&conn);

        migrate(&mut conn).unwrap();
        assert_full_schema(&conn);
        assert_eq!(get_all_schemas(&conn), original_schema);
        assert_eq!(get_all_config_data(&conn), original_metadata);
        assert_migrated_track(&conn);
    }

    #[test]
    fn should_migrate_version_one() {
        let mut conn = gen_database_raw();
        version_one(&conn);

        migrate(&mut conn).unwrap();

        assert_full_schema(&conn);
        assert_migrated_track(&conn);
    }

    // rusqlite's busy handler takes a function pointer, so keep this test's
    // channels on the worker thread instead of sharing global mutable state.
    thread_local! {
        static BUSY_SIGNAL: RefCell<Option<(mpsc::Sender<()>, mpsc::Receiver<()>)>> =
            const { RefCell::new(None) };
    }

    fn wait_for_competing_migration(_: i32) -> bool {
        BUSY_SIGNAL.with(|signal| {
            let Some((locked, committed)) = signal.borrow_mut().take() else {
                return false;
            };
            locked.send(()).is_ok() && committed.recv_timeout(WAIT_TIMEOUT).is_ok()
        })
    }

    #[test]
    fn should_recheck_version_after_competing_migration() {
        let temp = TempDatabase::new();
        let mut conn = Connection::open(temp.path()).unwrap();
        version_one(&conn);
        let tx = conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .unwrap();

        let (locked_tx, locked_rx) = mpsc::channel();
        let (committed_tx, committed_rx) = mpsc::channel();
        let (done_tx, done_rx) = mpsc::channel();
        let path = temp.path();
        let worker = thread::spawn(move || {
            let mut conn = Connection::open(path).unwrap();
            BUSY_SIGNAL.with(|signal| *signal.borrow_mut() = Some((locked_tx, committed_rx)));
            conn.busy_handler(Some(wait_for_competing_migration))
                .unwrap();
            let result = migrate(&mut conn);
            done_tx.send((conn, result)).unwrap();
        });

        // B has encountered A's writer reservation. With the old migrator B
        // has already read version 1 here, whereas the exclusive transaction
        // waits before reading the version.
        locked_rx.recv_timeout(WAIT_TIMEOUT).unwrap();
        apply_version_2(&tx).unwrap();
        tx.commit().unwrap();
        committed_tx.send(()).unwrap();

        let (other, result) = done_rx.recv_timeout(WAIT_TIMEOUT).unwrap();
        worker.join().unwrap();
        result.unwrap();
        assert_full_schema(&other);
        assert_migrated_track(&other);
    }

    #[test]
    fn should_initialize_fresh_database_concurrently() {
        let temp = TempDatabase::new();
        assert!(!temp.path().exists());
        let barrier = Arc::new(Barrier::new(3));
        let (done_tx, done_rx) = mpsc::channel();
        let workers: Vec<_> = (0..2)
            .map(|_| {
                let path = temp.path();
                let barrier = Arc::clone(&barrier);
                let done_tx = done_tx.clone();
                thread::spawn(move || {
                    barrier.wait();
                    done_tx.send(Database::new(&path)).unwrap();
                })
            })
            .collect();
        drop(done_tx);
        barrier.wait();

        let first = done_rx.recv_timeout(WAIT_TIMEOUT).unwrap().unwrap();
        let second = done_rx.recv_timeout(WAIT_TIMEOUT).unwrap().unwrap();
        for worker in workers {
            worker.join().unwrap();
        }
        for db in [&first, &second] {
            let conn = db.get_connection();
            assert_full_schema(&conn);
            assert_creation_metadata(&conn);
        }
        assert_eq!(
            get_all_config_data(&first.get_connection()),
            get_all_config_data(&second.get_connection())
        );
    }

    #[test]
    fn should_roll_back_on_metadata_failure() {
        let mut conn = gen_database_raw();
        version_one(&conn);
        conn.execute_batch(
            "CREATE TRIGGER reject_migration_metadata BEFORE INSERT ON config
             WHEN NEW.key = 'last_migrated_at'
             BEGIN SELECT RAISE(ABORT, 'reject migration metadata'); END;",
        )
        .unwrap();
        let original_schema = get_all_schemas(&conn);
        let original_metadata = get_all_config_data(&conn);

        let error = migrate(&mut conn).unwrap_err();
        assert!(format!("{error:#}").contains("reject migration metadata"));
        assert!(conn.is_autocommit());
        assert_eq!(get_user_version(&conn).unwrap(), 1);
        assert_eq!(get_all_schemas(&conn), original_schema);
        assert_eq!(get_all_config_data(&conn), original_metadata);
        let track: (i64, String, String) = conn
            .query_row("SELECT id, file_stem, added_at FROM tracks", [], |r| {
                Ok((r.get(0)?, r.get(1)?, r.get(2)?))
            })
            .unwrap();
        assert_eq!(track, (42, "example".into(), ADDED_AT.into()));

        conn.execute_batch("DROP TRIGGER reject_migration_metadata")
            .unwrap();
        migrate(&mut conn).unwrap();
        assert_full_schema(&conn);
        assert_migrated_track(&conn);
    }

    #[test]
    fn should_reject_future_version_without_changes() {
        let mut conn = gen_database_raw();
        version_one(&conn);
        set_user_version(&conn, DB_VERSION + 1).unwrap();
        let original_schema = get_all_schemas(&conn);
        let original_metadata = get_all_config_data(&conn);

        let error = migrate(&mut conn).unwrap_err();
        assert_eq!(
            error.to_string(),
            format!(
                "Expected Database version to be lower or equal to {DB_VERSION}, found {}!",
                DB_VERSION + 1
            )
        );
        assert!(conn.is_autocommit());
        assert_eq!(get_user_version(&conn).unwrap(), DB_VERSION + 1);
        assert_eq!(get_all_schemas(&conn), original_schema);
        assert_eq!(get_all_config_data(&conn), original_metadata);
        let added_at: String = conn
            .query_row("SELECT added_at FROM tracks WHERE id = 42", [], |r| {
                r.get(0)
            })
            .unwrap();
        assert_eq!(added_at, ADDED_AT);
    }

    #[test]
    fn should_return_lock_error_and_allow_retry() {
        let temp = TempDatabase::new();
        let mut writer = Connection::open(temp.path()).unwrap();
        version_one(&writer);
        let mut conn = Connection::open(temp.path()).unwrap();
        conn.busy_timeout(Duration::from_millis(50)).unwrap();
        let original_schema = get_all_schemas(&conn);
        let tx = writer
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .unwrap();

        let start = Instant::now();
        let error = migrate(&mut conn).unwrap_err();
        assert!(start.elapsed() < WAIT_TIMEOUT);
        assert_eq!(
            error
                .downcast_ref::<rusqlite::Error>()
                .unwrap()
                .sqlite_error_code(),
            Some(ErrorCode::DatabaseBusy)
        );
        assert!(conn.is_autocommit());
        assert_eq!(get_user_version(&conn).unwrap(), 1);
        assert_eq!(get_all_schemas(&conn), original_schema);

        tx.rollback().unwrap();
        migrate(&mut conn).unwrap();
        assert_full_schema(&conn);
        assert_migrated_track(&conn);
    }
}
