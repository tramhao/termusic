use std::{borrow::Cow, ffi::OsStr, path::Path, time::Duration};

use anyhow::{Context, Result, bail};
use either::Either;
use rusqlite::{Connection, named_params};

use super::{Integer, album_db::AlbumInsertable, artist_db::ArtistInsertable};
use crate::track::TrackMetadata;

#[derive(Debug, Clone)]
pub struct TrackInsertable<'a> {
    // Track identifier
    file_dir: &'a Path,
    file_stem: &'a OsStr,
    file_ext: &'a OsStr,

    // Direct data on `tracks`
    duration: Option<Duration>,
    last_position: Option<Duration>,
    /// Either a reference to a insertable to look-up or a direct integer to use as reference into `albums`.
    album: Option<Either<Cow<'a, AlbumInsertable<'a>>, Integer>>,

    // Data on `tracks_metadata`
    title: Option<&'a str>,
    genre: Option<&'a str>,
    artist_display: Option<&'a str>,

    // mapped metadata
    artists: Vec<Either<Cow<'a, ArtistInsertable<'a>>, Integer>>,
}

// TODO: proper errors?
impl<'a> TrackInsertable<'a> {
    /// Try to create a insertable from the given options.
    ///
    /// # Errors
    ///
    /// - if the given `path` is not absolute
    /// - if the given `path` does not have components: parent, stem, ext
    ///
    /// Any other potential errors (like empty artist string) will be silently ignored.
    pub fn try_from_track(path: &'a Path, metadata: &'a TrackMetadata) -> Result<Self> {
        if !path.is_absolute() {
            bail!("Given path is not absolute!");
        }

        let Some(file_dir) = path.parent() else {
            bail!("Given path does not have a parent!");
        };

        let Some(file_stem) = path.file_stem() else {
            bail!("Given path does not have a stem!");
        };

        let Some(file_ext) = path.extension() else {
            bail!("Given path does not have a extension!");
        };

        let title = metadata
            .title
            .as_ref()
            .filter(|v| !v.is_empty())
            .map(String::as_str);
        let genre = metadata
            .genre
            .as_ref()
            .filter(|v| !v.is_empty())
            .map(String::as_str);
        let artist_display = metadata
            .artist
            .as_ref()
            .filter(|v| !v.is_empty())
            .map(String::as_str);
        let album_title = metadata
            .album
            .as_ref()
            .filter(|v| !v.is_empty())
            .map(String::as_str);
        let album_artist_display = metadata
            .album_artist
            .as_ref()
            .filter(|v| !v.is_empty())
            .map(String::as_str);
        let album_artists = metadata.album_artists.as_ref();

        let album = if let (Some(album_title), Some(album_artist_display)) =
            (album_title, album_artist_display)
        {
            let album_artists = album_artists
                .map(|v| {
                    v.iter()
                        .filter(|v| !v.is_empty())
                        .map(|v| Either::Left(Cow::Owned(ArtistInsertable { artist: v.as_str() })))
                        .collect()
                })
                .unwrap_or_default();
            Some(Either::Left(Cow::Owned(AlbumInsertable {
                title: album_title,
                artist_display: album_artist_display,
                artists: album_artists,
            })))
        } else {
            None
        };

        let artists = metadata
            .artists
            .as_ref()
            .map(|v| {
                v.iter()
                    .filter(|v| !v.is_empty())
                    .map(|v| Either::Left(Cow::Owned(ArtistInsertable { artist: v.as_str() })))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        Ok(Self {
            file_dir,
            file_stem,
            file_ext,

            duration: metadata.duration,
            last_position: None,
            album,

            title,
            genre,
            artist_display,

            artists,
        })
    }

    /// Try to insert or update the current track's data.
    ///
    /// This will also insert all metadata, album and artists.
    pub fn try_insert_or_update(&self, conn: &Connection) -> Result<Integer> {
        let album = if let Some(album) = &self.album {
            let ret = match album {
                Either::Left(insertable) => {
                    insertable.try_insert_or_update(conn).context("album")?
                }
                Either::Right(v) => *v,
            };

            Some(ret)
        } else {
            None
        };

        let insert_track = InsertTrack {
            file_dir: &self.file_dir.to_string_lossy(),
            file_stem: &self.file_stem.to_string_lossy(),
            file_ext: &self.file_ext.to_string_lossy(),
            duration: self.duration,
            last_position: self.last_position,
            album,
        };

        let id = insert_track.upsert(conn).context("tracks")?;

        let insert_metadata = InsertTrackMetadata {
            track: id,
            title: self.title,
            genre: self.genre,
            artist_display: self.artist_display,
        };

        let _ = insert_metadata.upsert(conn).context("tracks_metadata")?;

        for artist in &self.artists {
            let artist = match artist {
                Either::Left(insertable) => {
                    insertable.try_insert_or_update(conn).context("artists")?
                }
                Either::Right(v) => *v,
            };

            let insert_mapping = InsertTrackArtistMapping { track: id, artist };

            insert_mapping.upsert(conn).context("tracks_artist")?;
        }

        Ok(id)
    }
}

/// Stores references for insertion into `tracks` directly
#[derive(Debug, PartialEq)]
struct InsertTrack<'a> {
    // Track identifier
    file_dir: &'a str,
    file_stem: &'a str,
    file_ext: &'a str,

    // Direct data on `tracks`
    duration: Option<Duration>,
    last_position: Option<Duration>,
    /// Either NULL or a id to the actual album
    album: Option<Integer>,
}

impl InsertTrack<'_> {
    /// Insert or update the current data with the file paths as identifiers.
    fn upsert(&self, conn: &Connection) -> Result<Integer> {
        let mut stmt = conn.prepare_cached("
            INSERT INTO tracks (file_dir, file_stem, file_ext, duration, last_position, added_at, album)
            VALUES (:file_dir, :file_stem, :file_ext, :duration, :last_position, :added_at, :album)
            ON CONFLICT(file_dir, file_stem, file_ext) DO UPDATE SET 
                duration=excluded.duration, album=excluded.album
            RETURNING id;
        ")?;

        let now = chrono::Utc::now().to_rfc3339();
        let duration = self.duration.map(|v| v.as_secs());
        let last_position = self.last_position.map(|v| v.as_secs());

        let id = stmt.query_row(
            named_params! {
                ":file_dir": self.file_dir,
                ":file_stem": self.file_stem,
                ":file_ext": self.file_ext,
                ":duration": duration,
                ":last_position": last_position,
                ":added_at": &now,
                ":album": self.album
            },
            |row| row.get(0),
        )?;

        Ok(id)
    }

    // TODO: the following functions should be on a different struct

    /// Count all rows currently in the `tracks` database
    fn count_all(conn: &Connection) -> Result<Integer> {
        let count = conn.query_row("SELECT COUNT(id) FROM tracks;", [], |v| v.get(0))?;

        Ok(count)
    }
}

/// Stores references for insertion into `tracks_metadata` directly
#[derive(Debug, PartialEq)]
struct InsertTrackMetadata<'a> {
    // Track identifier
    track: Integer,

    // Direct data on `tracks_metadata`
    title: Option<&'a str>,
    genre: Option<&'a str>,
    artist_display: Option<&'a str>,
}

impl InsertTrackMetadata<'_> {
    /// Insert or update the current data with the file as identifier.
    fn upsert(&self, conn: &Connection) -> Result<Integer> {
        let mut stmt = conn.prepare_cached(
            "
            INSERT INTO tracks_metadata (track, title, genre, artist_display)
            VALUES (:track, :title, :genre, :artist_display)
            ON CONFLICT(track) DO UPDATE SET 
                title=excluded.title, genre=excluded.genre, artist_display=excluded.artist_display
            RETURNING track;
        ",
        )?;

        let id = stmt.query_row(
            named_params! {
                ":track": self.track,
                ":title": self.title,
                ":genre": self.genre,
                ":artist_display": self.artist_display,
            },
            |row| row.get(0),
        )?;

        Ok(id)
    }

    // TODO: the following functions should be on a different struct

    /// Count all rows currently in the `tracks_metadata` database
    fn count_all(conn: &Connection) -> Result<Integer> {
        let count = conn.query_row("SELECT COUNT(track) FROM tracks_metadata;", [], |v| {
            v.get(0)
        })?;

        Ok(count)
    }
}

/// Stores references for insertion into `tracks_artists` directly
#[derive(Debug, PartialEq)]
struct InsertTrackArtistMapping {
    track: Integer,
    artist: Integer,
}

impl InsertTrackArtistMapping {
    /// Insert the current data, not caring about the id that was inserted
    fn upsert(&self, conn: &Connection) -> Result<()> {
        let mut stmt = conn.prepare_cached(
            "
            INSERT INTO tracks_artists (track, artist)
            VALUES (:track, :artist)
            ON CONFLICT(track, artist) DO NOTHING;
        ",
        )?;

        stmt.execute(named_params! {
            ":track": self.track,
            ":artist": self.artist,
        })?;

        Ok(())
    }

    // TODO: the following functions should be on a different struct

    /// Count all rows currently in the `tracks_artists` database
    fn count_all(conn: &Connection) -> Result<Integer> {
        let count = conn.query_row("SELECT COUNT(track) FROM tracks_artists;", [], |v| v.get(0))?;

        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use crate::new_database::{
        artist_db::ArtistInsertable,
        test_utils::gen_database,
        track_db::{InsertTrackArtistMapping, InsertTrackMetadata},
    };

    use super::InsertTrack;

    /// Simple test that [`InsertTrack::upsert`] works correctly
    /// both with insertion and updating.
    #[test]
    fn should_insert_track_simple() {
        let db = gen_database();

        let data = InsertTrack {
            file_dir: "/somewhere",
            file_stem: "some file",
            file_ext: "mp3",
            duration: Some(Duration::from_secs(10)),
            last_position: None,
            album: None,
        };

        let db = db.conn.lock();

        let id = data.upsert(&db).unwrap();

        assert_eq!(id, 1);

        let new_id = data.upsert(&db).unwrap();

        // check that insertion and upsertion(update) return the same id
        assert_eq!(new_id, id);

        let count = InsertTrack::count_all(&db).unwrap();

        assert_eq!(count, 1);
    }

    /// Simple test that [`InsertTrackMetadata::upsert`] works correctly
    /// both with insertion and updating.
    #[test]
    fn should_insert_metadata_simple() {
        let db = gen_database();

        let data = InsertTrack {
            file_dir: "/somewhere",
            file_stem: "some file",
            file_ext: "mp3",
            duration: Some(Duration::from_secs(10)),
            last_position: None,
            album: None,
        };

        let db = db.conn.lock();
        let id = data.upsert(&db).unwrap();
        assert_eq!(id, 1);

        let metadata = InsertTrackMetadata {
            track: id,
            title: Some("test"),
            genre: Some("rock"),
            artist_display: Some("ArtistA"),
        };

        let id = metadata.upsert(&db).unwrap();

        assert_eq!(id, 1);

        let new_id = metadata.upsert(&db).unwrap();

        assert_eq!(new_id, id);

        let count = InsertTrackMetadata::count_all(&db).unwrap();

        assert_eq!(count, 1);
    }

    /// Simple test that [`InsertTrackArtistMapping::upsert`] works correctly
    /// both with insertion and updating.
    #[test]
    fn should_insert_artist_mapping_simple() {
        let db = gen_database();

        let data = InsertTrack {
            file_dir: "/somewhere",
            file_stem: "some file",
            file_ext: "mp3",
            duration: Some(Duration::from_secs(10)),
            last_position: None,
            album: None,
        };

        let db = db.conn.lock();
        let track_id = data.upsert(&db).unwrap();
        assert_eq!(track_id, 1);

        let artist = ArtistInsertable { artist: "ArtistA" };

        let artist_id = artist.try_insert_or_update(&db).unwrap();
        assert_eq!(artist_id, 1);

        let mapping = InsertTrackArtistMapping {
            track: track_id,
            artist: artist_id,
        };

        mapping.upsert(&db).unwrap();

        mapping.upsert(&db).unwrap();

        let count = InsertTrackArtistMapping::count_all(&db).unwrap();

        assert_eq!(count, 1);
    }
}
