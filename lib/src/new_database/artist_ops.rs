use anyhow::{Result, bail};
use rusqlite::{Connection, Row, named_params};

use crate::new_database::Integer;

/// Count all rows currently in the `artists` database
pub fn count_all_artists(conn: &Connection) -> Result<Integer> {
    let count = conn.query_row("SELECT COUNT(id) FROM artists;", [], |v| v.get(0))?;

    Ok(count)
}

/// The most common data required for a artist read from the database
#[derive(Debug, Clone, PartialEq)]
pub struct ArtistRead {
    pub id: Integer,

    pub name: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RowOrdering {
    IdAsc,
    IdDesc,
    AddedAsc,
    AddedDesc,
}

impl RowOrdering {
    /// Represent it as the data for a `ORDER BY` clause.
    fn as_sql(self) -> &'static str {
        match self {
            RowOrdering::IdAsc => "artists.id ASC",
            RowOrdering::IdDesc => "artists.id DESC",
            RowOrdering::AddedAsc => "artists.added_at ASC",
            RowOrdering::AddedDesc => "artists.added_at DESC",
        }
    }
}

/// Get all the Artists currently stored in the database with all the important data.
///
/// # Panics
///
/// If the database schema does not match what is expected.
pub fn get_all_artists(conn: &Connection, order: RowOrdering) -> Result<Vec<ArtistRead>> {
    let stmt = format!(
        "SELECT artists.id AS artist_id, artists.artist
    FROM artists
    ORDER BY {};",
        order.as_sql()
    );
    let mut stmt = conn.prepare(&stmt)?;

    let result: Vec<ArtistRead> = stmt
        .query_map(named_params! {}, |row| {
            let artist_read = common_row_to_artistread(row);

            Ok(artist_read)
        })?
        .collect::<Result<Vec<_>, rusqlite::Error>>()?;

    Ok(result)
}

/// Get a specific artist.
///
/// # Panics
///
/// If the database schema does not match what is expected.
pub fn get_artist(conn: &Connection, artist: &str) -> Result<ArtistRead> {
    if artist.trim().is_empty() {
        bail!("Given artist is empty!");
    }

    let mut stmt = conn.prepare(
        "SELECT artists.id AS artist_id, artists.artist
            FROM artists
            WHERE artists.artist=:artist_name;",
    )?;

    let result: ArtistRead = stmt.query_row(named_params! {":artist_name": artist}, |row| {
        let artist_read = common_row_to_artistread(row);

        Ok(artist_read)
    })?;

    Ok(result)
}

/// Check if a entry for the given `artist` exists.
///
/// # Panics
///
/// If sqlite somehow does not return what is expected.
pub fn artist_exists(conn: &Connection, artist: &str) -> Result<bool> {
    if artist.trim().is_empty() {
        bail!("Given artist is empty!");
    }

    let mut stmt = conn.prepare(
        "SELECT COUNT(artists.id)
            FROM artists
            WHERE artists.artist=:artist_name;",
    )?;

    let count = stmt
        .query_row(named_params! {":artist_name": artist}, |row| {
            let count: Integer = row.get(0).unwrap();

            Ok(count.max(0))
        })
        .or_else(|v| {
            if v == rusqlite::Error::QueryReturnedNoRows {
                Ok(0)
            } else {
                Err(v)
            }
        })?;

    Ok(count != 0)
}

/// Common function that converts a well-known named row to a [`ArtistRead`].
///
/// For row names look at [`get_all_artists`].
pub(super) fn common_row_to_artistread(row: &Row<'_>) -> ArtistRead {
    let id = row.get("artist_id").unwrap();
    let artist_title = row.get("artist").unwrap();

    ArtistRead {
        id,
        name: artist_title,
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use crate::new_database::{
        artist_insert::ArtistInsertable,
        artist_ops::{RowOrdering, artist_exists, get_all_artists, get_artist},
        test_utils::gen_database,
    };

    #[test]
    fn all_artists() {
        let db = gen_database();

        let artist = ArtistInsertable { artist: "ArtistA" };
        let _artist_id = artist.try_insert_or_update(&db.get_connection()).unwrap();

        let artist = ArtistInsertable { artist: "ArtistB" };
        let _artist_id = artist.try_insert_or_update(&db.get_connection()).unwrap();

        let artists: Vec<String> = get_all_artists(&db.get_connection(), RowOrdering::IdAsc)
            .unwrap()
            .into_iter()
            .map(|v| v.name)
            .collect();

        assert_eq!(artists, &["ArtistA", "ArtistB"]);
    }

    #[test]
    fn single_artist() {
        let db = gen_database();

        let artist = ArtistInsertable { artist: "ArtistA" };
        let _artist_id = artist.try_insert_or_update(&db.get_connection()).unwrap();

        let artist = ArtistInsertable { artist: "ArtistB" };
        let _artist_id = artist.try_insert_or_update(&db.get_connection()).unwrap();

        let artist_a = get_artist(&db.get_connection(), "ArtistA").unwrap();
        assert_eq!(artist_a.name, "ArtistA");

        let artist_b = get_artist(&db.get_connection(), "ArtistB").unwrap();
        assert_eq!(artist_b.name, "ArtistB");

        let err = get_artist(&db.get_connection(), "ArtistC").unwrap_err();
        let err = err.downcast::<rusqlite::Error>().unwrap();
        assert_eq!(err, rusqlite::Error::QueryReturnedNoRows);
    }

    #[test]
    fn exists() {
        let db = gen_database();

        let artist = ArtistInsertable { artist: "ArtistA" };
        let _artist_id = artist.try_insert_or_update(&db.get_connection()).unwrap();

        let res = artist_exists(&db.get_connection(), "ArtistA").unwrap();

        assert!(res);

        let res = artist_exists(&db.get_connection(), "ArtistB").unwrap();

        assert!(!res);
    }
}
