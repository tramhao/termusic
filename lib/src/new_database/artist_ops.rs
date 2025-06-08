use anyhow::{Result, bail};
use indoc::{formatdoc, indoc};
use rusqlite::{Connection, OptionalExtension, Row, named_params};

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
    let stmt = formatdoc! {"
        SELECT artists.id AS artist_id, artists.artist
        FROM artists
        ORDER BY {};
        ",
        order.as_sql()
    };
    let mut stmt = conn.prepare(&stmt)?;

    let result: Vec<ArtistRead> = stmt
        .query_map(named_params! {}, |row| {
            let artist_read = common_row_to_artistread(row);

            Ok(artist_read)
        })?
        .collect::<Result<Vec<_>, rusqlite::Error>>()?;

    Ok(result)
}

/// Get all the Artists that match `like`.
///
/// # Panics
///
/// If the database schema does not match what is expected.
pub fn get_all_artists_like(
    conn: &Connection,
    like: &str,
    order: RowOrdering,
) -> Result<Vec<ArtistRead>> {
    let stmt = formatdoc! {"
        SELECT artists.id AS artist_id, artists.artist
        FROM artists
        WHERE artists.artist LIKE :like
        ORDER BY {};
        ",
        order.as_sql()
    };
    let mut stmt = conn.prepare(&stmt)?;

    let result: Vec<ArtistRead> = stmt
        .query_map(named_params! {":like": like}, |row| {
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

    let mut stmt = conn.prepare(indoc! {"
        SELECT artists.id AS artist_id, artists.artist
        FROM artists
        WHERE artists.artist=:artist_name;
    "})?;

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

    let mut stmt = conn.prepare(indoc! {"
        SELECT artists.id
        FROM artists
        WHERE artists.artist=:artist_name;
    "})?;

    let exists = stmt.exists(named_params! {":artist_name": artist})?;

    Ok(exists)
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

/// Remove all artists that are unreferenced.
///
/// Returns the number of deleted rows. Will return `Ok(0)` if the query did not do anything.
///
/// # Panics
///
/// If the database schema does not match what is expected.
pub fn delete_all_unreferenced_artists(conn: &Connection) -> Result<usize> {
    // the following query is likely very slow compared to other queries
    let mut stmt = conn.prepare_cached(indoc! {"
        DELETE FROM artists
        WHERE artists.id NOT IN (
            SELECT tracks_artists.artist FROM tracks_artists
            UNION
            SELECT albums_artists.artist FROM albums_artists
        );
    "})?;

    let affected = stmt
        .execute(named_params! {})
        .optional()?
        .unwrap_or_default();

    Ok(affected)
}

#[cfg(test)]
mod tests {
    use std::{path::Path, time::Duration};

    use pretty_assertions::assert_eq;

    use crate::{
        new_database::{
            album_ops::delete_all_unreferenced_albums,
            artist_insert::ArtistInsertable,
            artist_ops::{
                RowOrdering, artist_exists, count_all_artists, delete_all_unreferenced_artists,
                get_all_artists, get_all_artists_like, get_artist,
            },
            test_utils::{gen_database, test_path},
            track_insert::TrackInsertable,
        },
        track::TrackMetadata,
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
    fn all_artists_like() {
        let db = gen_database();

        let artist = ArtistInsertable { artist: "ArtistA" };
        let _artist_id = artist.try_insert_or_update(&db.get_connection()).unwrap();

        let artist = ArtistInsertable { artist: "ArtistB" };
        let _artist_id = artist.try_insert_or_update(&db.get_connection()).unwrap();

        let artists: Vec<String> =
            get_all_artists_like(&db.get_connection(), "%artista%", RowOrdering::IdAsc)
                .unwrap()
                .into_iter()
                .map(|v| v.name)
                .collect();

        assert_eq!(artists, &["ArtistA"]);
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

    #[test]
    fn delete_unreferenced_albums() {
        let db = gen_database();

        let metadata = TrackMetadata {
            album: Some("AlbumA".to_string()),
            album_artist: Some("ArtistA".to_string()),
            album_artists: Some(vec!["ArtistA".to_string()]),
            artist: Some("ArtistA".to_string()),
            artists: Some(vec!["ArtistA".to_string()]),
            title: Some("FileA1".to_string()),
            duration: Some(Duration::from_secs(10)),
            ..Default::default()
        };
        let path = &test_path(Path::new("/somewhere/fileA1.ext"));
        let insertable = TrackInsertable::try_from_track(path, &metadata).unwrap();
        let _ = insertable
            .try_insert_or_update(&db.get_connection())
            .unwrap();

        let metadata = TrackMetadata {
            album: Some("AlbumB".to_string()),
            album_artist: Some("ArtistB".to_string()),
            album_artists: Some(vec!["ArtistB".to_string()]),
            artist: Some("ArtistB".to_string()),
            artists: Some(vec!["ArtistB".to_string()]),
            title: Some("FileB1".to_string()),
            duration: Some(Duration::from_secs(10)),
            ..Default::default()
        };
        let path = &test_path(Path::new("/somewhere/fileB1.ext"));
        let insertable = TrackInsertable::try_from_track(path, &metadata).unwrap();
        let _ = insertable
            .try_insert_or_update(&db.get_connection())
            .unwrap();

        let artists = count_all_artists(&db.get_connection()).unwrap();

        assert_eq!(artists, 2);

        let affected = delete_all_unreferenced_artists(&db.get_connection()).unwrap();

        assert_eq!(affected, 0);

        let metadata = TrackMetadata {
            album: Some("AlbumA".to_string()),
            album_artist: Some("ArtistA".to_string()),
            album_artists: Some(vec!["ArtistA".to_string()]),
            artist: Some("ArtistA".to_string()),
            artists: Some(vec!["ArtistA".to_string()]),
            title: Some("FileB1".to_string()),
            duration: Some(Duration::from_secs(10)),
            ..Default::default()
        };
        let path = &test_path(Path::new("/somewhere/fileB1.ext"));
        let insertable = TrackInsertable::try_from_track(path, &metadata).unwrap();
        let _ = insertable
            .try_insert_or_update(&db.get_connection())
            .unwrap();

        let artists = count_all_artists(&db.get_connection()).unwrap();

        assert_eq!(artists, 2);

        let affected = delete_all_unreferenced_artists(&db.get_connection()).unwrap();

        assert_eq!(affected, 0);

        // albums are unique based on title+artist_display, which changed, so a new album is made and the old one still referenced the artist
        // and so would otherwise not be deleted, as can be seen by the assert just above
        let _ = delete_all_unreferenced_albums(&db.get_connection()).unwrap();

        let affected = delete_all_unreferenced_artists(&db.get_connection()).unwrap();

        assert_eq!(affected, 1);

        let artists = count_all_artists(&db.get_connection()).unwrap();

        assert_eq!(artists, 1);
    }
}
