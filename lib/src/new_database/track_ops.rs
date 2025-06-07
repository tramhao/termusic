use std::{
    ffi::OsString,
    path::{Path, PathBuf},
    time::Duration,
};

use anyhow::{Result, bail};
use indoc::{formatdoc, indoc};
use rusqlite::{Connection, Row, named_params};

use crate::new_database::{
    artist_ops::{ArtistRead, common_row_to_artistread},
    track_insert::{path_to_db_comp, validate_path},
};

use super::Integer;

/// Count all rows currently in the `tracks` database
pub fn count_all_tracks(conn: &Connection) -> Result<Integer> {
    let count = conn.query_row("SELECT COUNT(id) FROM tracks;", [], |v| v.get(0))?;

    Ok(count)
}

/// Count all rows currently in the `tracks_metadata` database
pub fn count_all_track_metadata(conn: &Connection) -> Result<Integer> {
    let count = conn.query_row("SELECT COUNT(track) FROM tracks_metadata;", [], |v| {
        v.get(0)
    })?;

    Ok(count)
}

/// Count all rows currently in the `tracks_artists` database
#[cfg(test)]
pub(super) fn count_all_track_artist_mapping(conn: &Connection) -> Result<Integer> {
    let count = conn.query_row("SELECT COUNT(track) FROM tracks_artists;", [], |v| v.get(0))?;

    Ok(count)
}

/// The lowest information required for a [`TrackRead`] to identify a Album.
#[derive(Debug, Clone, PartialEq)]
pub struct AlbumRead {
    pub id: Integer,

    pub title: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrackRead {
    pub id: Integer,

    // Track identifier
    pub file_dir: PathBuf,
    pub file_stem: OsString,
    pub file_ext: OsString,

    // Direct data on `tracks`
    pub duration: Option<Duration>,
    pub last_position: Option<Duration>,
    /// Either a reference to a insertable to look-up or a direct integer to use as reference into `albums`.
    pub album: Option<AlbumRead>,

    // Data on `tracks_metadata`
    pub title: Option<String>,
    pub genre: Option<String>,
    pub artist_display: Option<String>,

    // mapped metadata
    pub artists: Vec<ArtistRead>,
}

impl TrackRead {
    /// Convert all the `file_` values to a single path.
    #[must_use]
    pub fn as_pathbuf(&self) -> PathBuf {
        let mut path = self.file_dir.clone();
        let mut file_name = self.file_stem.clone();
        file_name.reserve_exact(self.file_ext.len() + 1);
        file_name.push(".");
        file_name.push(&self.file_ext);
        path.push(file_name);

        path
    }
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
            RowOrdering::IdAsc => "tracks.id ASC",
            RowOrdering::IdDesc => "tracks.id DESC",
            RowOrdering::AddedAsc => "tracks.added_at ASC",
            RowOrdering::AddedDesc => "tracks.added_at DESC",
        }
    }
}

/// Get all the Tracks currently stored in the database with all the important data.
///
/// # Panics
///
/// If the database schema does not match what is expected.
pub fn get_all_tracks(conn: &Connection, order: RowOrdering) -> Result<Vec<TrackRead>> {
    let stmt = formatdoc! {"
        SELECT 
            tracks.id AS track_id, tracks.file_dir, tracks.file_stem, tracks.file_ext, tracks.duration, tracks.last_position,
            tracks_metadata.title AS track_title, tracks_metadata.artist_display, tracks_metadata.genre,
            albums.id AS album_id, albums.title AS album_title
        FROM tracks
        LEFT JOIN tracks_metadata ON tracks.id = tracks_metadata.track
        LEFT JOIN albums ON tracks.album = albums.id
        ORDER BY {};
        ",
        order.as_sql()
    };
    let mut stmt = conn.prepare(&stmt)?;

    let result: Vec<TrackRead> = stmt
        .query_map(named_params! {}, |row| {
            let trackread = common_row_to_trackread(conn, row);

            Ok(trackread)
        })?
        .collect::<Result<Vec<_>, rusqlite::Error>>()?;

    Ok(result)
}

/// Get all the artists for a given track.
///
/// # Panics
///
/// If the database schema does not match what is expected.
// maybe this should be in "artist_ops" instead?
pub fn get_all_artists_for_track(conn: &Connection, track_id: Integer) -> Result<Vec<ArtistRead>> {
    let mut stmt = conn.prepare(indoc! {"
        SELECT artists.id AS artist_id, artists.artist FROM artists
        INNER JOIN tracks_artists ON tracks_artists.track=:track_id
        WHERE artists.id=tracks_artists.artist;
    "})?;

    let result: Vec<ArtistRead> = stmt
        .query_map(named_params! {":track_id": track_id}, |row| {
            let artist_read = common_row_to_artistread(row);

            Ok(artist_read)
        })?
        .collect::<Result<Vec<_>, rusqlite::Error>>()?;

    Ok(result)
}

/// Get the `last_position` for the given `track`.
///
/// # Panics
///
/// If the database schema does not match what is expected.
pub fn get_last_position(conn: &Connection, track: &Path) -> Result<Option<Duration>> {
    let (file_dir, file_stem, file_ext) = path_to_db_comp(track)?;
    let file_dir = file_dir.to_string_lossy();
    let file_stem = file_stem.to_string_lossy();
    let file_ext = file_ext.to_string_lossy();

    let mut stmt = conn.prepare_cached(indoc!{"
        SELECT last_position FROM tracks
        WHERE tracks.file_dir=:file_dir AND tracks.file_stem=:file_stem AND tracks.file_ext=:file_ext;
    "})?;

    let result: Option<Integer> = stmt.query_row(
        named_params! {":file_dir": file_dir, ":file_stem": file_stem, ":file_ext": file_ext},
        |row| row.get(0),
    )?;

    let last_position = result.map(|v: Integer| {
        let int = u64::try_from(v.max(0)).unwrap();
        Duration::from_secs(int)
    });

    Ok(last_position)
}

/// Set the `last_positon` for the given `track`.
pub fn set_last_position(conn: &Connection, track: &Path, to: Option<Duration>) -> Result<()> {
    let (file_dir, file_stem, file_ext) = path_to_db_comp(track)?;
    let file_dir = file_dir.to_string_lossy();
    let file_stem = file_stem.to_string_lossy();
    let file_ext = file_ext.to_string_lossy();

    let last_position = to.map(|v| v.as_secs());

    let mut stmt = conn.prepare_cached(indoc!{"
        UPDATE tracks SET last_position=:last_position
        WHERE tracks.file_dir=:file_dir AND tracks.file_stem=:file_stem AND tracks.file_ext=:file_ext;
    "})?;

    let affected = stmt.execute(named_params! {":file_dir": file_dir, ":file_stem": file_stem, ":file_ext": file_ext, ":last_position": last_position})?;

    // update would otherwise fail silently
    if affected == 0 {
        bail!("Track not found");
    }

    Ok(())
}

/// Get all tracks associated with the given album.
///
/// # Panics
///
/// If the database schema does not match what is expected.
pub fn get_tracks_from_album(
    conn: &Connection,
    album_title: &str,
    album_artist: &str,
    order: RowOrdering,
) -> Result<Vec<TrackRead>> {
    let stmt = formatdoc! {"
        SELECT 
            tracks.id AS track_id, tracks.file_dir, tracks.file_stem, tracks.file_ext, tracks.duration, tracks.last_position,
            tracks_metadata.title AS track_title, tracks_metadata.artist_display, tracks_metadata.genre,
            albums.id AS album_id, albums.title AS album_title
        FROM tracks
        LEFT JOIN tracks_metadata ON tracks.id=tracks_metadata.track
        LEFT JOIN albums ON tracks.album = albums.id
        WHERE albums.title=:album_title AND albums.artist_display=:album_artist
        ORDER BY {};
        ",
        order.as_sql()
    };
    let mut stmt = conn.prepare(&stmt)?;

    let result: Vec<TrackRead> = stmt
        .query_map(
            named_params! {":album_title": album_title, ":album_artist": album_artist},
            |row| {
                let trackread = common_row_to_trackread(conn, row);

                Ok(trackread)
            },
        )?
        .collect::<Result<Vec<_>, rusqlite::Error>>()?;

    Ok(result)
}

/// Get all tracks associated with the given artist.
///
/// # Panics
///
/// If the database schema does not match what is expected.
pub fn get_tracks_from_artist(
    conn: &Connection,
    artist: &str,
    order: RowOrdering,
) -> Result<Vec<TrackRead>> {
    let stmt = formatdoc! {"
        SELECT 
            tracks.id AS track_id, tracks.file_dir, tracks.file_stem, tracks.file_ext, tracks.duration, tracks.last_position,
            tracks_metadata.title AS track_title, tracks_metadata.artist_display, tracks_metadata.genre,
            albums.id AS album_id, albums.title AS album_title
        FROM tracks
        LEFT JOIN tracks_metadata ON tracks.id=tracks_metadata.track
        LEFT JOIN albums ON tracks.album = albums.id
        INNER JOIN tracks_artists ON tracks.id = tracks_artists.track
        INNER JOIN artists ON artists.id=tracks_artists.artist
        WHERE artists.artist=:artist
        ORDER BY {};
        ",
        order.as_sql()
    };
    let mut stmt = conn.prepare(&stmt)?;

    let result: Vec<TrackRead> = stmt
        .query_map(named_params! {":artist": artist}, |row| {
            let trackread = common_row_to_trackread(conn, row);

            Ok(trackread)
        })?
        .collect::<Result<Vec<_>, rusqlite::Error>>()?;

    Ok(result)
}

/// Get all tracks associated with a genre.
///
/// # Panics
///
/// If the database schema does not match what is expected.
pub fn get_tracks_from_genre(
    conn: &Connection,
    genre: &str,
    order: RowOrdering,
) -> Result<Vec<TrackRead>> {
    let stmt = formatdoc! {"
        SELECT
            tracks.id AS track_id, tracks.file_dir, tracks.file_stem, tracks.file_ext, tracks.duration, tracks.last_position,
            tracks_metadata.title AS track_title, tracks_metadata.artist_display, tracks_metadata.genre,
            albums.id AS album_id, albums.title AS album_title
        FROM tracks
        INNER JOIN tracks_metadata ON tracks.id=tracks_metadata.track
        LEFT JOIN albums ON tracks.album = albums.id
        WHERE tracks_metadata.genre=:genre
        ORDER BY {};
        ",
        order.as_sql()
    };
    let mut stmt = conn.prepare(&stmt)?;

    let result: Vec<TrackRead> = stmt
        .query_map(named_params! {":genre": genre}, |row| {
            let trackread = common_row_to_trackread(conn, row);

            Ok(trackread)
        })?
        .collect::<Result<Vec<_>, rusqlite::Error>>()?;

    Ok(result)
}

/// Get all tracks associated with the given directory.
///
/// # Panics
///
/// If the database schema does not match what is expected.
pub fn get_tracks_from_directory(
    conn: &Connection,
    dir: &Path,
    order: RowOrdering,
) -> Result<Vec<TrackRead>> {
    validate_path(dir)?;
    let dir = dir.to_string_lossy();

    let stmt = formatdoc! {"
        SELECT 
            tracks.id AS track_id, tracks.file_dir, tracks.file_stem, tracks.file_ext, tracks.duration, tracks.last_position,
            tracks_metadata.title AS track_title, tracks_metadata.artist_display, tracks_metadata.genre,
            albums.id AS album_id, albums.title AS album_title
        FROM tracks
        LEFT JOIN tracks_metadata ON tracks.id=tracks_metadata.track
        LEFT JOIN albums ON tracks.album = albums.id
        WHERE tracks.file_dir=:dir
        ORDER BY {};
        ",
        order.as_sql()
    };
    let mut stmt = conn.prepare(&stmt)?;

    let result: Vec<TrackRead> = stmt
        .query_map(named_params! {":dir": dir}, |row| {
            let trackread = common_row_to_trackread(conn, row);

            Ok(trackread)
        })?
        .collect::<Result<Vec<_>, rusqlite::Error>>()?;

    Ok(result)
}

/// Get all tracks associated with a genre.
///
/// # Panics
///
/// If the database schema does not match what is expected.
pub fn get_track_from_path(conn: &Connection, path: &Path) -> Result<TrackRead> {
    let (file_dir, file_stem, file_ext) = path_to_db_comp(path)?;
    let file_dir = file_dir.to_string_lossy();
    let file_stem = file_stem.to_string_lossy();
    let file_ext = file_ext.to_string_lossy();

    let mut stmt = conn.prepare(indoc! {"
        SELECT
            tracks.id AS track_id, tracks.file_dir, tracks.file_stem, tracks.file_ext, tracks.duration, tracks.last_position,
            tracks_metadata.title AS track_title, tracks_metadata.artist_display, tracks_metadata.genre,
            albums.id AS album_id, albums.title AS album_title
        FROM tracks
        INNER JOIN tracks_metadata ON tracks.id=tracks_metadata.track
        LEFT JOIN albums ON tracks.album = albums.id
        WHERE tracks.file_dir=:file_dir AND tracks.file_stem=:file_stem AND tracks.file_ext=:file_ext;
        ",
    })?;

    let result: TrackRead = stmt.query_row(
        named_params! {":file_dir": file_dir, ":file_stem": file_stem, ":file_ext": file_ext},
        |row| {
            let trackread = common_row_to_trackread(conn, row);

            Ok(trackread)
        },
    )?;

    Ok(result)
}

/// Common function that converts a well-known named row to a [`TrackRead`].
///
/// For row names look at [`get_all_tracks`].
fn common_row_to_trackread(conn: &Connection, row: &Row<'_>) -> TrackRead {
    let file_dir = row
        .get("file_dir")
        .map(|v: String| PathBuf::from(v))
        .unwrap();
    let file_stem = row
        .get("file_stem")
        .map(|v: String| OsString::from(v))
        .unwrap();
    let file_ext = row
        .get("file_ext")
        .map(|v: String| OsString::from(v))
        .unwrap();
    let id = row.get("track_id").unwrap();

    let duration = row.get("duration").ok().map(|v: Integer| {
        let int = u64::try_from(v.max(0)).unwrap();
        Duration::from_secs(int)
    });
    let last_position = row.get("last_position").ok().map(|v: Integer| {
        let int = u64::try_from(v.max(0)).unwrap();
        Duration::from_secs(int)
    });
    let title = row.get("track_title").unwrap_or_default();
    let genre = row.get("genre").unwrap_or_default();
    let artist_display = row.get("artist_display").unwrap_or_default();

    let album_id = row.get("album_id").ok();
    let album_title = row.get("album_title").ok();

    let album = if let (Some(album_id), Some(album_title)) = (album_id, album_title) {
        Some(AlbumRead {
            id: album_id,
            title: album_title,
        })
    } else {
        None
    };

    let artists = match get_all_artists_for_track(conn, id) {
        Ok(v) => v,
        Err(err) => {
            warn!("Error resolving artists for a track: {err:#?}");
            Vec::new()
        }
    };

    TrackRead {
        id,
        file_dir,
        file_stem,
        file_ext,
        duration,
        last_position,
        album,
        title,
        genre,
        artist_display,
        artists,
    }
}

/// Check if a entry for the given `track` exists.
///
/// # Panics
///
/// If sqlite somehow does not return what is expected.
pub fn track_exists(conn: &Connection, track: &Path) -> Result<bool> {
    let (file_dir, file_stem, file_ext) = path_to_db_comp(track)?;
    let file_dir = file_dir.to_string_lossy();
    let file_stem = file_stem.to_string_lossy();
    let file_ext = file_ext.to_string_lossy();

    let mut stmt = conn.prepare(indoc!{"
        SELECT tracks.id
        FROM tracks
        WHERE tracks.file_dir=:file_dir AND tracks.file_stem=:file_stem AND tracks.file_ext=:file_ext;
    "})?;

    let exists = stmt.exists(
        named_params! {":file_dir": file_dir, ":file_stem": file_stem, ":file_ext": file_ext},
    )?;

    Ok(exists)
}

/// Get all distinct directories.
///
/// # Panics
///
/// If sqlite somehow does not return what is expected.
pub fn all_distinct_directories(conn: &Connection) -> Result<Vec<String>> {
    let mut stmt = conn.prepare(&indoc! {"
        SELECT DISTINCT tracks.file_dir
        FROM tracks
        ",
    })?;

    let result: Vec<String> = stmt
        .query_map(named_params! {}, |row| {
            let res = row.get(0).unwrap();
            Ok(res)
        })?
        .collect::<Result<Vec<_>, rusqlite::Error>>()?;

    Ok(result)
}

#[cfg(test)]
mod tests {
    use std::{
        ffi::{OsStr, OsString},
        path::{Path, PathBuf},
        time::Duration,
    };

    use either::Either;
    use pretty_assertions::assert_eq;

    use crate::{
        new_database::{
            album_insert::AlbumInsertable,
            artist_insert::ArtistInsertable,
            test_utils::gen_database,
            track_insert::TrackInsertable,
            track_ops::{
                AlbumRead, ArtistRead, RowOrdering, TrackRead, all_distinct_directories,
                get_all_tracks, get_last_position, get_track_from_path, get_tracks_from_album,
                get_tracks_from_artist, get_tracks_from_directory, get_tracks_from_genre,
                set_last_position, track_exists,
            },
        },
        track::TrackMetadata,
    };

    use super::get_all_artists_for_track;

    #[test]
    fn artists_for_track() {
        let db = gen_database();

        let track = TrackInsertable {
            file_dir: Path::new("/somewhere"),
            file_stem: OsStr::new("file"),
            file_ext: OsStr::new("ext"),
            duration: Some(Duration::from_secs(10)),
            last_position: None,
            album: Some(Either::Left(
                AlbumInsertable {
                    title: "AlbumA",
                    artist_display: "ArtistA",
                    artists: vec![Either::Left(ArtistInsertable { artist: "ArtistA" }.into())],
                }
                .into(),
            )),
            title: Some("file test"),
            genre: None,
            artist_display: Some("ArtistA feat. ArtistB"),
            artists: vec![
                Either::Left(ArtistInsertable { artist: "ArtistA" }.into()),
                Either::Left(ArtistInsertable { artist: "ArtistB" }.into()),
            ],
        };
        let track_id = track.try_insert_or_update(&db.get_connection()).unwrap();

        let mut all_artists: Vec<String> =
            get_all_artists_for_track(&db.get_connection(), track_id)
                .unwrap()
                .into_iter()
                .map(|v| v.name)
                .collect();
        // just making sure they are consistently ordered
        all_artists.sort();

        assert_eq!(all_artists, &["ArtistA", "ArtistB"]);
    }

    #[test]
    fn all_tracks() {
        let db = gen_database();

        let track = TrackInsertable {
            file_dir: Path::new("/somewhere"),
            file_stem: OsStr::new("file"),
            file_ext: OsStr::new("ext"),
            duration: Some(Duration::from_secs(10)),
            last_position: None,
            album: Some(Either::Left(
                AlbumInsertable {
                    title: "AlbumA",
                    artist_display: "ArtistA",
                    artists: vec![Either::Left(ArtistInsertable { artist: "ArtistA" }.into())],
                }
                .into(),
            )),
            title: Some("file test"),
            genre: None,
            artist_display: Some("ArtistA"),
            artists: vec![Either::Left(ArtistInsertable { artist: "ArtistA" }.into())],
        };
        let _track_id = track.try_insert_or_update(&db.get_connection()).unwrap();

        let all_tracks = get_all_tracks(&db.get_connection(), RowOrdering::IdAsc).unwrap();

        assert_eq!(
            all_tracks,
            &[TrackRead {
                id: 1,
                file_dir: PathBuf::from("/somewhere"),
                file_stem: OsString::from("file"),
                file_ext: OsString::from("ext"),
                duration: Some(Duration::from_secs(10)),
                last_position: None,
                album: Some(AlbumRead {
                    id: 1,
                    title: "AlbumA".to_string()
                }),
                title: Some("file test".to_string()),
                genre: None,
                artist_display: Some("ArtistA".to_string()),
                artists: vec![ArtistRead {
                    id: 1,
                    name: "ArtistA".to_string()
                }]
            }]
        );
    }

    #[test]
    fn last_position_some() {
        let db = gen_database();

        let track = TrackInsertable {
            file_dir: Path::new("/somewhere"),
            file_stem: OsStr::new("file"),
            file_ext: OsStr::new("ext"),
            duration: Some(Duration::from_secs(10)),
            last_position: Some(Duration::from_secs(5)),
            album: Some(Either::Left(
                AlbumInsertable {
                    title: "AlbumA",
                    artist_display: "ArtistA",
                    artists: vec![Either::Left(ArtistInsertable { artist: "ArtistA" }.into())],
                }
                .into(),
            )),
            title: Some("file test"),
            genre: None,
            artist_display: Some("ArtistA"),
            artists: vec![Either::Left(ArtistInsertable { artist: "ArtistA" }.into())],
        };
        let path = Path::new("/somewhere/file.ext");
        let _track_id = track.try_insert_or_update(&db.get_connection()).unwrap();

        let last_position = get_last_position(&db.get_connection(), path).unwrap();

        assert_eq!(last_position, Some(Duration::from_secs(5)));

        set_last_position(&db.get_connection(), path, None).unwrap();

        let last_position = get_last_position(&db.get_connection(), path).unwrap();

        assert_eq!(last_position, None);
    }

    #[test]
    fn last_position_none() {
        let db = gen_database();

        let track = TrackInsertable {
            file_dir: Path::new("/somewhere"),
            file_stem: OsStr::new("file"),
            file_ext: OsStr::new("ext"),
            duration: Some(Duration::from_secs(10)),
            last_position: None,
            album: Some(Either::Left(
                AlbumInsertable {
                    title: "AlbumA",
                    artist_display: "ArtistA",
                    artists: vec![Either::Left(ArtistInsertable { artist: "ArtistA" }.into())],
                }
                .into(),
            )),
            title: Some("file test"),
            genre: None,
            artist_display: Some("ArtistA"),
            artists: vec![Either::Left(ArtistInsertable { artist: "ArtistA" }.into())],
        };
        let path = Path::new("/somewhere/file.ext");
        let _track_id = track.try_insert_or_update(&db.get_connection()).unwrap();

        let last_position = get_last_position(&db.get_connection(), path).unwrap();

        assert_eq!(last_position, None);

        set_last_position(&db.get_connection(), path, Some(Duration::from_secs(5))).unwrap();

        let last_position = get_last_position(&db.get_connection(), path).unwrap();

        assert_eq!(last_position, Some(Duration::from_secs(5)));
    }

    #[test]
    fn last_position_not_found() {
        let db = gen_database();

        let path = Path::new("/somewhere/file.ext");

        // get
        let err = get_last_position(&db.get_connection(), path).unwrap_err();
        let err = err.downcast::<rusqlite::Error>().unwrap();

        assert_eq!(err, rusqlite::Error::QueryReturnedNoRows);

        // set
        let err = set_last_position(&db.get_connection(), path, None).unwrap_err();

        assert!(err.to_string().contains("Track not found"));
    }

    #[test]
    fn tracks_by_album() {
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
        let insertable =
            TrackInsertable::try_from_track(Path::new("/somewhere/fileA1.ext"), &metadata).unwrap();
        let _ = insertable
            .try_insert_or_update(&db.get_connection())
            .unwrap();

        let metadata = TrackMetadata {
            album: Some("AlbumA".to_string()),
            album_artist: Some("ArtistA".to_string()),
            album_artists: Some(vec!["ArtistA".to_string()]),
            artist: Some("ArtistA".to_string()),
            artists: Some(vec!["ArtistA".to_string()]),
            title: Some("FileA2".to_string()),
            duration: Some(Duration::from_secs(10)),
            ..Default::default()
        };
        let insertable =
            TrackInsertable::try_from_track(Path::new("/somewhere/fileA2.ext"), &metadata).unwrap();
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
        let insertable =
            TrackInsertable::try_from_track(Path::new("/somewhere/fileB1.ext"), &metadata).unwrap();
        let _ = insertable
            .try_insert_or_update(&db.get_connection())
            .unwrap();

        let res = get_tracks_from_album(
            &db.get_connection(),
            "AlbumA",
            "ArtistA",
            RowOrdering::IdAsc,
        )
        .unwrap();
        let res: Vec<String> = res.into_iter().map(|v| v.title.unwrap()).collect();

        assert_eq!(&res, &["FileA1", "FileA2"]);
    }

    #[test]
    fn tracks_by_artist() {
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
        let insertable =
            TrackInsertable::try_from_track(Path::new("/somewhere/fileA1.ext"), &metadata).unwrap();
        let _ = insertable
            .try_insert_or_update(&db.get_connection())
            .unwrap();

        let metadata = TrackMetadata {
            album: Some("AlbumA".to_string()),
            album_artist: Some("ArtistB".to_string()),
            album_artists: Some(vec!["ArtistB".to_string()]),
            artist: Some("ArtistB".to_string()),
            artists: Some(vec!["ArtistB".to_string()]),
            title: Some("FileA2".to_string()),
            duration: Some(Duration::from_secs(10)),
            ..Default::default()
        };
        let insertable =
            TrackInsertable::try_from_track(Path::new("/somewhere/fileA2.ext"), &metadata).unwrap();
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
        let insertable =
            TrackInsertable::try_from_track(Path::new("/somewhere/fileB1.ext"), &metadata).unwrap();
        let _ = insertable
            .try_insert_or_update(&db.get_connection())
            .unwrap();

        let res =
            get_tracks_from_artist(&db.get_connection(), "ArtistB", RowOrdering::IdAsc).unwrap();
        let res: Vec<String> = res.into_iter().map(|v| v.title.unwrap()).collect();

        assert_eq!(&res, &["FileA2", "FileB1"]);
    }

    #[test]
    fn tracks_by_genre() {
        let db = gen_database();

        let metadata = TrackMetadata {
            artist: Some("ArtistA".to_string()),
            artists: Some(vec!["ArtistA".to_string()]),
            title: Some("FileA1".to_string()),
            duration: Some(Duration::from_secs(10)),
            genre: Some("Rock".to_string()),
            ..Default::default()
        };
        let insertable =
            TrackInsertable::try_from_track(Path::new("/somewhere/fileA1.ext"), &metadata).unwrap();
        let _ = insertable
            .try_insert_or_update(&db.get_connection())
            .unwrap();

        let metadata = TrackMetadata {
            artist: Some("ArtistA".to_string()),
            artists: Some(vec!["ArtistA".to_string()]),
            title: Some("FileA2".to_string()),
            duration: Some(Duration::from_secs(10)),
            genre: Some("Pop".to_string()),
            ..Default::default()
        };
        let insertable =
            TrackInsertable::try_from_track(Path::new("/somewhere/fileA2.ext"), &metadata).unwrap();
        let _ = insertable
            .try_insert_or_update(&db.get_connection())
            .unwrap();

        let metadata = TrackMetadata {
            artist: Some("ArtistA".to_string()),
            artists: Some(vec!["ArtistA".to_string()]),
            title: Some("FileB1".to_string()),
            duration: Some(Duration::from_secs(10)),
            genre: None,
            ..Default::default()
        };
        let insertable =
            TrackInsertable::try_from_track(Path::new("/somewhere/fileB1.ext"), &metadata).unwrap();
        let _ = insertable
            .try_insert_or_update(&db.get_connection())
            .unwrap();

        let res = get_tracks_from_genre(&db.get_connection(), "Rock", RowOrdering::IdAsc).unwrap();
        let res: Vec<String> = res.into_iter().map(|v| v.title.unwrap()).collect();

        assert_eq!(&res, &["FileA1"]);
    }

    #[test]
    fn exists() {
        let db = gen_database();

        let metadata = TrackMetadata {
            title: Some("FileA1".to_string()),
            duration: Some(Duration::from_secs(10)),
            ..Default::default()
        };
        let path = Path::new("/somewhere/fileA1.ext");
        let insertable = TrackInsertable::try_from_track(path, &metadata).unwrap();
        let _ = insertable
            .try_insert_or_update(&db.get_connection())
            .unwrap();

        let res = track_exists(&db.get_connection(), path).unwrap();

        assert!(res);

        let res = track_exists(&db.get_connection(), Path::new("/somewhere/else.ext")).unwrap();

        assert!(!res);
    }

    #[test]
    fn single_track() {
        let db = gen_database();

        let metadata = TrackMetadata {
            title: Some("FileA1".to_string()),
            duration: Some(Duration::from_secs(10)),
            ..Default::default()
        };
        let path = Path::new("/somewhere/fileA1.ext");
        let insertable = TrackInsertable::try_from_track(path, &metadata).unwrap();
        let _ = insertable
            .try_insert_or_update(&db.get_connection())
            .unwrap();

        let res = get_track_from_path(&db.get_connection(), path).unwrap();

        assert_eq!(res.title, Some("FileA1".to_string()));

        let err = get_track_from_path(&db.get_connection(), Path::new("/somewhere/else.ext"))
            .unwrap_err();
        let err = err.downcast::<rusqlite::Error>().unwrap();

        assert_eq!(err, rusqlite::Error::QueryReturnedNoRows);
    }

    #[test]
    fn track_read_to_path() {
        let read = TrackRead {
            id: 0,
            file_dir: PathBuf::from("/path/to/somewhere"),
            file_stem: OsString::from("filename"),
            file_ext: OsString::from("ext"),
            duration: None,
            last_position: None,
            album: None,
            title: None,
            genre: None,
            artist_display: None,
            artists: Vec::new(),
        };

        assert_eq!(
            read.as_pathbuf(),
            PathBuf::from("/path/to/somewhere/filename.ext")
        );
    }

    #[test]
    fn distinct_directories() {
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
        let insertable =
            TrackInsertable::try_from_track(Path::new("/somewhere/dirA/fileA1.ext"), &metadata)
                .unwrap();
        let _ = insertable
            .try_insert_or_update(&db.get_connection())
            .unwrap();

        let metadata = TrackMetadata {
            album: Some("AlbumA".to_string()),
            album_artist: Some("ArtistA".to_string()),
            album_artists: Some(vec!["ArtistA".to_string()]),
            artist: Some("ArtistA".to_string()),
            artists: Some(vec!["ArtistA".to_string()]),
            title: Some("FileA2".to_string()),
            duration: Some(Duration::from_secs(10)),
            ..Default::default()
        };
        let insertable =
            TrackInsertable::try_from_track(Path::new("/somewhere/dirA/fileA2.ext"), &metadata)
                .unwrap();
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
        let insertable =
            TrackInsertable::try_from_track(Path::new("/somewhere/dirB/fileB1.ext"), &metadata)
                .unwrap();
        let _ = insertable
            .try_insert_or_update(&db.get_connection())
            .unwrap();

        let res = all_distinct_directories(&db.get_connection()).unwrap();

        assert_eq!(&res, &["/somewhere/dirA", "/somewhere/dirB"]);
    }

    #[test]
    fn tracks_by_directory() {
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
        let insertable =
            TrackInsertable::try_from_track(Path::new("/somewhere/dirA/fileA1.ext"), &metadata)
                .unwrap();
        let _ = insertable
            .try_insert_or_update(&db.get_connection())
            .unwrap();

        let metadata = TrackMetadata {
            album: Some("AlbumA".to_string()),
            album_artist: Some("ArtistA".to_string()),
            album_artists: Some(vec!["ArtistA".to_string()]),
            artist: Some("ArtistA".to_string()),
            artists: Some(vec!["ArtistA".to_string()]),
            title: Some("FileA2".to_string()),
            duration: Some(Duration::from_secs(10)),
            ..Default::default()
        };
        let insertable =
            TrackInsertable::try_from_track(Path::new("/somewhere/dirA/fileA2.ext"), &metadata)
                .unwrap();
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
        let insertable =
            TrackInsertable::try_from_track(Path::new("/somewhere/dirB/fileB1.ext"), &metadata)
                .unwrap();
        let _ = insertable
            .try_insert_or_update(&db.get_connection())
            .unwrap();

        let res = get_tracks_from_directory(
            &db.get_connection(),
            Path::new("/somewhere/dirA"),
            RowOrdering::IdAsc,
        )
        .unwrap();
        let res: Vec<String> = res.into_iter().map(|v| v.title.unwrap()).collect();

        assert_eq!(&res, &["FileA1", "FileA2"]);
    }
}
