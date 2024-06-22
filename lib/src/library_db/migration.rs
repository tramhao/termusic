use anyhow::{bail, Context, Result};
use rusqlite::Connection;

/// The Current Database schema version this application is meant to run against
pub(super) const DB_VERSION: u32 = 2;
/// The Lowest Database schema version this application supports migration up against
///
/// Expection being "0" as that indicates a fresh database
const LOWEST_MIGRATEABLE_VERSION: u32 = 2;

/// Helper function to get the `user_version` with a single function call
#[inline]
fn get_user_version(conn: &Connection) -> Result<u32> {
    conn.query_row("SELECT user_version FROM pragma_user_version", [], |r| {
        r.get(0)
    })
    .context("get pragma \"user_version\"")
}

/// Helper function to set the `user_version` with a single function call
///
/// Returns the passed version for re-use
#[inline]
fn set_user_version(conn: &Connection, version: u32) -> Result<u32> {
    conn.pragma_update(None, "user_version", version)
        .context("update user_version error")?;

    Ok(version)
}

/// Create / Migrate everything in the database, if necessary
pub(super) fn migrate(conn: &Connection) -> Result<()> {
    let mut user_version: u32 = get_user_version(conn)?;

    if user_version > DB_VERSION {
        bail!(
            "Expected Database version to be lower or equal to {DB_VERSION}, found {user_version}!"
        );
    }

    if user_version < LOWEST_MIGRATEABLE_VERSION && user_version != 0 {
        // TODO: maybe we should just error out or have the whole file deleted instead of just resetting parts
        warn!("Found Database, but had lower than lowest migrateable version, resetting! Version: {user_version}");

        conn.execute("DROP TABLE tracks", [])
            .context("Dropping \"tracks\" table")?;
        user_version = set_user_version(conn, 0)?;
    }

    // only execute migrations if not already done so
    if user_version != DB_VERSION {
        apply_migrations(conn, user_version)?;
    }

    Ok(())
}

/// The Function that actually applies creations / migrations, checks / preparation is done in the function above
///
/// Migrates from [`LOWEST_MIGRATEABLE_VERSION`] / `0` to [`DB_VERSION`]
#[allow(unused_assignments)] // for future possible migrations
fn apply_migrations(conn: &Connection, mut user_version: u32) -> Result<()> {
    // do all migrations in steps, this way everyone is in the same state and had the same things applied, even for new things
    if user_version == 0 {
        // Version 2 is the base version, so there are basically no migrations, only creations
        conn.execute_batch(include_str!("./migrations/002.sql"))
            .context("Database version 2 could not be created")?;
        user_version = set_user_version(conn, 2)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::super::test_utils::gen_database;
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn should_create_from_fresh() {
        let conn = gen_database();

        assert_eq!(0, get_user_version(&conn).unwrap());
        migrate(&conn).unwrap();
        assert_eq!(2, get_user_version(&conn).unwrap());

        let all_tracks: Vec<String> = {
            let mut prep = conn.prepare("SELECT name FROM sqlite_schema WHERE type ='table' AND name NOT LIKE 'sqlite_%';").unwrap();
            prep.query_map([], |r| r.get(0))
                .unwrap()
                .flatten()
                .collect()
        };

        assert_eq!(&all_tracks, &["tracks"]);
    }
}
