use anyhow::{Context, Result, bail};
use rusqlite::{Connection, named_params};

/// The Current Database schema version this application is meant to run against
pub(super) const DB_VERSION: u32 = 1;

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
pub(super) fn migrate(conn: &Connection) -> Result<()> {
    let user_version: u32 = get_user_version(conn)?;

    if user_version > DB_VERSION {
        bail!(
            "Expected Database version to be lower or equal to {DB_VERSION}, found {user_version}!"
        );
    }

    // only execute migrations if not already done so
    if user_version != DB_VERSION {
        apply_migrations(conn, user_version)?;
    }

    Ok(())
}

/// Apply migrations to be at [`DB_VERSION`].
#[allow(unused_assignments)] // for future possible migrations
fn apply_migrations(conn: &Connection, mut user_version: u32) -> Result<()> {
    if user_version == 0 {
        // Version 2 is the base version, so there are basically no migrations, only creations
        conn.execute_batch(include_str!("./migrations/001.sql"))
            .context("Database version 1 could not be created")?;
        user_version = set_user_version(conn, 1)?;

        set_db_created_at(conn)?;
        set_db_created_with(conn)?;
    }

    set_last_updated_at(conn)?;

    Ok(())
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
    use pretty_assertions::assert_eq;

    use crate::new_database::migrate::{DB_VERSION, get_user_version, migrate};

    use super::super::test_utils::gen_database_raw;

    #[test]
    fn should_create_from_fresh() {
        let conn = gen_database_raw();

        // verify the created database is at 0
        assert_eq!(0, get_user_version(&conn).unwrap());
        migrate(&conn).unwrap();
        // verify the migrated database is at the highest version we want to work with
        assert_eq!(DB_VERSION, get_user_version(&conn).unwrap());

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
    }
}
