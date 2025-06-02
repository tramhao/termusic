use std::{ffi::OsString, path::PathBuf, time::Duration};

use anyhow::Result;
use rusqlite::{Connection, named_params};

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

/// The lowest information required for a [`TrackRead`] to identify a Album.
#[derive(Debug, Clone, PartialEq)]
pub struct AlbumRead {
    id: Integer,

    title: String,
}

/// The lowest information required for a [`TrackRead`] to identify a Artist.
#[derive(Debug, Clone, PartialEq)]
pub struct ArtistRead {
    id: Integer,

    name: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrackRead {
    id: Integer,

    // Track identifier
    file_dir: PathBuf,
    file_stem: OsString,
    file_ext: OsString,

    // Direct data on `tracks`
    duration: Option<Duration>,
    last_position: Option<Duration>,
    /// Either a reference to a insertable to look-up or a direct integer to use as reference into `albums`.
    album: Option<AlbumRead>,

    // Data on `tracks_metadata`
    title: Option<String>,
    genre: Option<String>,
    artist_display: Option<String>,

    // mapped metadata
    artists: Vec<ArtistRead>,
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
    let stmt = format!("SELECT tracks.id AS track_id, tracks.file_dir, tracks.file_stem, tracks.file_ext, tracks.duration, tracks.last_position, tracks_metadata.title AS track_title, tracks_metadata.artist_display, tracks_metadata.genre, albums.id AS album_id, albums.title AS album_title
    FROM tracks
    LEFT JOIN tracks_metadata ON tracks.id = tracks_metadata.track
    LEFT JOIN albums ON tracks.album = albums.id
    ORDER BY {};", order.as_sql());
    let mut stmt = conn.prepare(&stmt)?;

    let result: Vec<TrackRead> = stmt
        .query_map(named_params! {}, |row| {
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

            Ok(TrackRead {
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
            })
        })?
        .collect::<Result<Vec<_>, rusqlite::Error>>()?;

    Ok(result)
}

/// Get all the artists for a given track.
///
/// # Panics
///
/// If the database schema does not match what is expected.
pub fn get_all_artists_for_track(conn: &Connection, track_id: Integer) -> Result<Vec<ArtistRead>> {
    let stmt = "SELECT artists.id AS artist_id, artists.artist FROM artists
    INNER JOIN tracks_artists ON tracks_artists.track=:track_id
    WHERE artists.id=tracks_artists.artist;";
    let mut stmt = conn.prepare(stmt)?;

    let result: Vec<ArtistRead> = stmt
        .query_map(named_params! {":track_id": track_id}, |row| {
            let artist_id = row.get("artist_id").unwrap();
            let artist_name = row.get("artist").unwrap();

            Ok(ArtistRead {
                id: artist_id,
                name: artist_name,
            })
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

    use crate::new_database::{
        album_insert::AlbumInsertable,
        artist_insert::ArtistInsertable,
        test_utils::gen_database,
        track_insert::TrackInsertable,
        track_ops::{AlbumRead, ArtistRead, RowOrdering, TrackRead, get_all_tracks},
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
}
