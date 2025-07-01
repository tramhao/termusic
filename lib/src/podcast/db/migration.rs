use anyhow::{Context, Result, bail};
use indoc::indoc;
use rusqlite::{Connection, params};
use semver::Version;

/// The Current Database schema version this application is meant to run against
pub(super) const DB_VERSION: u32 = 1;

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

    update_version_col(conn)?;

    Ok(())
}

/// The Function that actually applies creations / migrations, checks / preparation is done in the function above
///
/// Migrates from `0` to [`DB_VERSION`]
#[allow(unused_assignments)] // for future possible migrations
fn apply_migrations(conn: &Connection, mut user_version: u32) -> Result<()> {
    // do all migrations in steps, this way everyone is in the same state and had the same things applied, even for new things
    if user_version == 0 {
        // Version 1 is the base version, so there are basically no migrations, only creations
        conn.execute_batch(include_str!("./migrations/001.sql"))
            .context("PodcastDatabase version 1 could not be created")?;
        user_version = set_user_version(conn, 1)?;
    }

    Ok(())
}

/// Update the last used application version in the database `version`
fn update_version_col(conn: &Connection) -> Result<()> {
    // NOTE: for future updates use the "user_version" instead

    // get version number stored in database
    let mut stmt = conn.prepare("SELECT version FROM version WHERE id = 1;")?;
    let vstr: Result<String, rusqlite::Error> = stmt.query_row([], |row| row.get("version"));

    // compare to current app version
    let curr_ver = Version::parse(crate::VERSION)?;

    match vstr {
        Ok(vstr) => {
            let db_version = Version::parse(&vstr)?;
            if db_version < curr_ver {
                // any version checks for DB migrations should
                // go here first, before we update the version

                // adding a column to capture episode guids
                // if db_version <= Version::parse("1.2.1")? {
                //     conn.execute("ALTER TABLE episodes ADD COLUMN guid TEXT;", params![])
                //         .expect("Could not run database migrations.");
                // }

                // conn.update_version(&curr_ver, true)?;
            }
        }
        Err(_) => update_version_exec(conn, &curr_ver, false)?,
    }

    Ok(())
}

/// Insert / Update a entry in the `version` table
fn update_version_exec(conn: &Connection, current_version: &Version, update: bool) -> Result<()> {
    if update {
        conn.execute(
            indoc! {"
            UPDATE version SET version = ?
            WHERE id = ?;
            "},
            params![current_version.to_string(), 1],
        )?;
    } else {
        conn.execute(
            indoc! {"
            INSERT INTO version (id, version)
            VALUES (?, ?);
            "},
            params![1, current_version.to_string()],
        )?;
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
        assert_eq!(1, get_user_version(&conn).unwrap());

        let all_tracks: Vec<String> = {
            let mut prep = conn.prepare("SELECT name FROM sqlite_schema WHERE type ='table' AND name NOT LIKE 'sqlite_%';").unwrap();
            prep.query_map([], |r| r.get(0))
                .unwrap()
                .flatten()
                .collect()
        };

        assert_eq!(&all_tracks, &["podcasts", "episodes", "files", "version"]);
    }
}
