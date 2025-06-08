use anyhow::{Result, bail};
use indoc::{formatdoc, indoc};
use rusqlite::{Connection, OptionalExtension, Row, named_params};

use crate::new_database::{
    Integer,
    artist_ops::{ArtistRead, common_row_to_artistread},
};

/// Count all rows currently in the `albums` database
pub fn count_all_albums(conn: &Connection) -> Result<Integer> {
    let count = conn.query_row("SELECT COUNT(id) FROM albums;", [], |v| v.get(0))?;

    Ok(count)
}

/// Count all rows currently in the `albums_artists` database
#[cfg(test)]
pub(super) fn count_all_albums_artist_mapping(conn: &Connection) -> Result<Integer> {
    let count = conn.query_row("SELECT COUNT(album) FROM albums_artists;", [], |v| v.get(0))?;

    Ok(count)
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlbumRead {
    pub id: Integer,

    pub title: String,
    pub artist_display: String,

    pub artists: Vec<ArtistRead>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RowOrdering {
    IdAsc,
    IdDesc,
}

impl RowOrdering {
    /// Represent it as the data for a `ORDER BY` clause.
    fn as_sql(self) -> &'static str {
        match self {
            RowOrdering::IdAsc => "albums.id ASC",
            RowOrdering::IdDesc => "albums.id DESC",
        }
    }
}

/// Get all the Albums currently stored in the database with all the important data.
///
/// # Panics
///
/// If the database schema does not match what is expected.
pub fn get_all_albums(conn: &Connection, order: RowOrdering) -> Result<Vec<AlbumRead>> {
    let stmt = formatdoc! {"
        SELECT albums.id as album_id, albums.title, albums.artist_display
        FROM albums
        ORDER BY {};
        ",
        order.as_sql()
    };
    let mut stmt = conn.prepare(&stmt)?;

    let result: Vec<AlbumRead> = stmt
        .query_map(named_params! {}, |row| {
            let album_read = common_row_to_album(conn, row);

            Ok(album_read)
        })?
        .collect::<Result<Vec<_>, rusqlite::Error>>()?;

    Ok(result)
}

/// Get all the Albums that match `like`.
///
/// # Panics
///
/// If the database schema does not match what is expected.
pub fn get_all_albums_like(
    conn: &Connection,
    like: &str,
    order: RowOrdering,
) -> Result<Vec<AlbumRead>> {
    let stmt = formatdoc! {"
        SELECT albums.id as album_id, albums.title, albums.artist_display
        FROM albums
        WHERE albums.title LIKE :like
        ORDER BY {};
        ",
        order.as_sql()
    };
    let mut stmt = conn.prepare(&stmt)?;

    let result: Vec<AlbumRead> = stmt
        .query_map(named_params! {":like": like}, |row| {
            let album_read = common_row_to_album(conn, row);

            Ok(album_read)
        })?
        .collect::<Result<Vec<_>, rusqlite::Error>>()?;

    Ok(result)
}

/// Get all the artists for a given album.
///
/// # Panics
///
/// If the database schema does not match what is expected.
// maybe this should be in "artist_ops" instead?
pub fn get_all_artists_for_album(conn: &Connection, album_id: Integer) -> Result<Vec<ArtistRead>> {
    let mut stmt = conn.prepare(indoc! {"
        SELECT artists.id AS artist_id, artists.artist FROM artists
        INNER JOIN albums_artists ON albums_artists.album=(:album_id)
        WHERE artists.id=albums_artists.artist;
    "})?;

    let result: Vec<ArtistRead> = stmt
        .query_map(named_params! {":album_id": album_id}, |row| {
            let artist_read = common_row_to_artistread(row);

            Ok(artist_read)
        })?
        .collect::<Result<Vec<_>, rusqlite::Error>>()?;

    Ok(result)
}

/// Check if a entry for the given `album` exists.
///
/// # Panics
///
/// If sqlite somehow does not return what is expected.
pub fn album_exists(conn: &Connection, album: &str) -> Result<bool> {
    if album.trim().is_empty() {
        bail!("Given album is empty!");
    }

    let mut stmt = conn.prepare(indoc! {"
        SELECT albums.id
        FROM albums
        WHERE albums.title=:album_name;
    "})?;

    let exists = stmt.exists(named_params! {":album_name": album})?;

    Ok(exists)
}

/// Common function that converts a well-known named row to a [`AlbumRead`].
///
/// For row names look at [`get_all_albums`].
fn common_row_to_album(conn: &Connection, row: &Row<'_>) -> AlbumRead {
    let id = row.get("album_id").unwrap();
    let title = row.get("title").unwrap();
    let artist_display = row.get("artist_display").unwrap();

    let artists = match get_all_artists_for_album(conn, id) {
        Ok(v) => v,
        Err(err) => {
            warn!("Error resolving artists for a album: {err:#?}");
            Vec::new()
        }
    };

    AlbumRead {
        id,
        title,
        artist_display,
        artists,
    }
}

/// Remove all albums-artists mappings for the given album.
///
/// Returns the number of deleted rows. Will return `Ok(0)` if the query did not do anything.
///
/// # Panics
///
/// If the database schema does not match what is expected.
pub fn delete_albums_artist_mapping_for(conn: &Connection, album_id: Integer) -> Result<usize> {
    let mut stmt = conn.prepare_cached(indoc! {"
        DELETE FROM albums_artists
        WHERE albums_artists.album=:album_id;
    "})?;

    let affected = stmt
        .execute(named_params! {":album_id": album_id})
        .optional()?
        .unwrap_or_default();

    Ok(affected)
}

/// Remove all albums that are unreferenced from `tracks`.
///
/// Returns the number of deleted rows. Will return `Ok(0)` if the query did not do anything.
///
/// # Panics
///
/// If the database schema does not match what is expected.
pub fn delete_all_unreferenced_albums(conn: &Connection) -> Result<usize> {
    // the following query is likely very slow compared to other queries
    let mut stmt = conn.prepare_cached(indoc! {"
        DELETE FROM albums
        WHERE albums.id NOT IN (
            SELECT tracks.album FROM tracks
            WHERE tracks.album IS NOT NULL
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

    use either::Either;

    use crate::{
        new_database::{
            album_insert::AlbumInsertable,
            album_ops::{
                AlbumRead, RowOrdering, album_exists, count_all_albums,
                count_all_albums_artist_mapping, delete_albums_artist_mapping_for,
                delete_all_unreferenced_albums, get_all_albums, get_all_albums_like,
                get_all_artists_for_album,
            },
            artist_insert::ArtistInsertable,
            artist_ops::ArtistRead,
            test_utils::{gen_database, test_path},
            track_insert::TrackInsertable,
        },
        track::TrackMetadata,
    };

    #[test]
    fn artists_for_album() {
        let db = gen_database();

        let album = AlbumInsertable {
            title: "AlbumA",
            artist_display: "ArtistA",
            artists: vec![
                Either::Left(ArtistInsertable { artist: "ArtistA" }.into()),
                Either::Left(ArtistInsertable { artist: "ArtistB" }.into()),
            ],
        };
        let album_id = album.try_insert_or_update(&db.get_connection()).unwrap();

        let mut all_artists: Vec<String> =
            get_all_artists_for_album(&db.get_connection(), album_id)
                .unwrap()
                .into_iter()
                .map(|v| v.name)
                .collect();
        // just making sure they are consistently ordered
        all_artists.sort();

        assert_eq!(all_artists, &["ArtistA", "ArtistB"]);
    }

    #[test]
    fn all_albums() {
        let db = gen_database();

        let album = AlbumInsertable {
            title: "AlbumA",
            artist_display: "ArtistA",
            artists: vec![Either::Left(ArtistInsertable { artist: "ArtistA" }.into())],
        };
        let _album_id = album.try_insert_or_update(&db.get_connection()).unwrap();

        let album = AlbumInsertable {
            title: "AlbumB",
            artist_display: "ArtistB",
            artists: vec![Either::Left(ArtistInsertable { artist: "ArtistB" }.into())],
        };
        let _album_id = album.try_insert_or_update(&db.get_connection()).unwrap();

        let all_albums = get_all_albums(&db.get_connection(), RowOrdering::IdAsc).unwrap();

        assert_eq!(
            all_albums,
            &[
                AlbumRead {
                    id: 1,
                    title: "AlbumA".to_string(),
                    artist_display: "ArtistA".to_string(),
                    artists: vec![ArtistRead {
                        id: 1,
                        name: "ArtistA".to_string()
                    }]
                },
                AlbumRead {
                    id: 2,
                    title: "AlbumB".to_string(),
                    artist_display: "ArtistB".to_string(),
                    artists: vec![ArtistRead {
                        id: 2,
                        name: "ArtistB".to_string()
                    }]
                },
            ]
        );
    }

    #[test]
    fn all_albums_like() {
        let db = gen_database();

        let album = AlbumInsertable {
            title: "AlbumA",
            artist_display: "ArtistA",
            artists: vec![Either::Left(ArtistInsertable { artist: "ArtistA" }.into())],
        };
        let _album_id = album.try_insert_or_update(&db.get_connection()).unwrap();

        let album = AlbumInsertable {
            title: "AlbumB",
            artist_display: "ArtistB",
            artists: vec![Either::Left(ArtistInsertable { artist: "ArtistB" }.into())],
        };
        let _album_id = album.try_insert_or_update(&db.get_connection()).unwrap();

        let all_albums =
            get_all_albums_like(&db.get_connection(), "%albuma%", RowOrdering::IdAsc).unwrap();

        assert_eq!(
            all_albums,
            &[AlbumRead {
                id: 1,
                title: "AlbumA".to_string(),
                artist_display: "ArtistA".to_string(),
                artists: vec![ArtistRead {
                    id: 1,
                    name: "ArtistA".to_string()
                }]
            },]
        );
    }

    #[test]
    fn exists() {
        let db = gen_database();

        let album = AlbumInsertable {
            title: "AlbumA",
            artist_display: "ArtistA",
            artists: vec![Either::Left(ArtistInsertable { artist: "ArtistA" }.into())],
        };
        let _album_id = album.try_insert_or_update(&db.get_connection()).unwrap();

        let res = album_exists(&db.get_connection(), "AlbumA").unwrap();

        assert!(res);

        let res = album_exists(&db.get_connection(), "AlbumB").unwrap();

        assert!(!res);
    }

    #[test]
    fn delete_albums_artists_mapping() {
        let db = gen_database();

        let album = AlbumInsertable {
            title: "AlbumA",
            artist_display: "ArtistA feat. ArtistB",
            artists: vec![
                Either::Left(ArtistInsertable { artist: "ArtistA" }.into()),
                Either::Left(ArtistInsertable { artist: "ArtistB" }.into()),
            ],
        };
        let album_id_a = album.try_insert_or_update(&db.get_connection()).unwrap();

        let album = AlbumInsertable {
            title: "AlbumB",
            artist_display: "ArtistA feat. ArtistB",
            artists: vec![
                Either::Left(ArtistInsertable { artist: "ArtistA" }.into()),
                Either::Left(ArtistInsertable { artist: "ArtistB" }.into()),
            ],
        };
        let _album_id = album.try_insert_or_update(&db.get_connection()).unwrap();

        let mapping_counts = count_all_albums_artist_mapping(&db.get_connection()).unwrap();

        assert_eq!(mapping_counts, 4);

        let affected = delete_albums_artist_mapping_for(&db.get_connection(), album_id_a).unwrap();

        assert_eq!(affected, 2);

        let mapping_counts = count_all_albums_artist_mapping(&db.get_connection()).unwrap();

        assert_eq!(mapping_counts, 2);
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

        let albums = count_all_albums(&db.get_connection()).unwrap();

        assert_eq!(albums, 2);

        let affected = delete_all_unreferenced_albums(&db.get_connection()).unwrap();

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

        let albums = count_all_albums(&db.get_connection()).unwrap();

        assert_eq!(albums, 2);

        let affected = delete_all_unreferenced_albums(&db.get_connection()).unwrap();

        assert_eq!(affected, 1);

        let albums = count_all_albums(&db.get_connection()).unwrap();

        assert_eq!(albums, 1);
    }
}
