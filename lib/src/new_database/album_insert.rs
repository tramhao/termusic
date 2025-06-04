use std::borrow::Cow;

use anyhow::{Context, Result};
use either::Either;
use indoc::indoc;
use rusqlite::{Connection, named_params};

use super::{Integer, artist_insert::ArtistInsertable};

#[derive(Debug, Clone)]
pub struct AlbumInsertable<'a> {
    pub title: &'a str,
    pub artist_display: &'a str,

    // mapped metadata
    /// Either a reference to a insertable to look-up or a direct integer to use as reference into `artists`.
    pub artists: Vec<Either<Cow<'a, ArtistInsertable<'a>>, Integer>>,
}

impl AlbumInsertable<'_> {
    /// Try to insert or update the current albums's data.
    pub fn try_insert_or_update(&self, conn: &Connection) -> Result<Integer> {
        let insert_album = InsertAlbum {
            title: self.title,
            artist_display: self.artist_display,
        };

        let id = insert_album.upsert(conn).context("albums")?;

        for artist in &self.artists {
            let artist = match artist {
                Either::Left(insertable) => {
                    insertable.try_insert_or_update(conn).context("artists")?
                }
                Either::Right(v) => *v,
            };

            let insert_mapping = InsertAlbumArtistMapping { album: id, artist };

            insert_mapping.upsert(conn).context("albums_artists")?;
        }

        Ok(id)
    }
}

impl<'a> From<AlbumInsertable<'a>> for Cow<'a, AlbumInsertable<'a>> {
    fn from(value: AlbumInsertable<'a>) -> Self {
        Cow::Owned(value)
    }
}

impl<'a> From<&'a AlbumInsertable<'a>> for Cow<'a, AlbumInsertable<'a>> {
    fn from(value: &'a AlbumInsertable<'a>) -> Self {
        Cow::Borrowed(value)
    }
}

/// Stores references for insertion into `albums` directly
#[derive(Debug, PartialEq)]
struct InsertAlbum<'a> {
    /// The title and identifier
    title: &'a str,

    artist_display: &'a str,
}

impl InsertAlbum<'_> {
    /// Insert or update the current data with the `title` and `artist_display` as identifier.
    fn upsert(&self, conn: &Connection) -> Result<Integer> {
        // using "title=title" as "DO NOTHING" would not be returning the id
        let mut stmt = conn.prepare_cached(indoc! {"
            INSERT INTO albums (title, artist_display)
            VALUES (:title, :artist_display)
            ON CONFLICT(title, artist_display) DO UPDATE SET
                title=title
            RETURNING id;
        "})?;

        let id = stmt.query_row(
            named_params! {
                ":title": self.title,
                ":artist_display": self.artist_display,
            },
            |row| row.get(0),
        )?;

        Ok(id)
    }
}

/// Stores references for insertion into `albums_artists` directly
#[derive(Debug, PartialEq)]
struct InsertAlbumArtistMapping {
    album: Integer,
    artist: Integer,
}

impl InsertAlbumArtistMapping {
    /// Insert the current data, not caring about the id that was inserted
    fn upsert(&self, conn: &Connection) -> Result<()> {
        let mut stmt = conn.prepare_cached(indoc! {"
            INSERT INTO albums_artists (album, artist)
            VALUES (:album, :artist)
            ON CONFLICT(album, artist) DO NOTHING;
        "})?;

        stmt.execute(named_params! {
            ":album": self.album,
            ":artist": self.artist,
        })?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::new_database::{
        album_insert::{InsertAlbum, InsertAlbumArtistMapping},
        album_ops::{count_all_albums, count_all_albums_artist_mapping},
        artist_insert::ArtistInsertable,
        test_utils::gen_database,
    };

    /// Simple test that [`InsertAlbum::upsert`] works correctly
    /// both with insertion and updating.
    #[test]
    fn should_insert_album_simple() {
        let db = gen_database();

        let data = InsertAlbum {
            title: "AlbumA",
            artist_display: "ArtistA",
        };

        let db = db.conn.lock();

        let id = data.upsert(&db).unwrap();

        assert_eq!(id, 1);

        let new_id = data.upsert(&db).unwrap();

        // check that insertion and upsertion(update) return the same id
        assert_eq!(new_id, id);

        let count = count_all_albums(&db).unwrap();

        assert_eq!(count, 1);
    }

    /// Simple test that [`InsertAlbumArtistMapping::upsert`] works correctly
    /// both with insertion and updating.
    #[test]
    fn should_insert_artist_mapping_simple() {
        let db = gen_database();

        let data = InsertAlbum {
            title: "AlbumA",
            artist_display: "ArtistA",
        };

        let db = db.conn.lock();
        let album_id = data.upsert(&db).unwrap();
        assert_eq!(album_id, 1);

        let artist = ArtistInsertable { artist: "ArtistA" };

        let artist_id = artist.try_insert_or_update(&db).unwrap();
        assert_eq!(artist_id, 1);

        let mapping = InsertAlbumArtistMapping {
            album: album_id,
            artist: artist_id,
        };

        mapping.upsert(&db).unwrap();

        mapping.upsert(&db).unwrap();

        let count = count_all_albums_artist_mapping(&db).unwrap();

        assert_eq!(count, 1);
    }
}
