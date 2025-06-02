use anyhow::{Context, Result};
use rusqlite::{Connection, named_params};

use super::Integer;

#[derive(Debug, Clone, PartialEq)]
pub struct ArtistInsertable<'a> {
    /// The name of the artist
    pub artist: &'a str,
}

impl ArtistInsertable<'_> {
    /// Try to insert or update the current artist's data.
    pub fn try_insert_or_update(&self, conn: &Connection) -> Result<Integer> {
        let insert_artist = InsertArtist {
            artist: self.artist,
        };

        let id = insert_artist.upsert(conn).context("artists")?;

        Ok(id)
    }
}

// the following may not be necessary with how bare `ArtistInsertable` currently is, but for consistency it exists anyway.
/// Stores references for insertion into `artists` directly
#[derive(Debug, PartialEq)]
struct InsertArtist<'a> {
    /// Artist name and identifier
    artist: &'a str,
}

impl InsertArtist<'_> {
    /// Insert or update the current data with the file as identifier.
    fn upsert(&self, conn: &Connection) -> Result<Integer> {
        // using "artist=artist" as "DO NOTHING" would not be returning the id
        let mut stmt = conn.prepare_cached(
            "
            INSERT INTO artists (artist, added_at)
            VALUES (:artist, :added_at)
            ON CONFLICT(artist) DO UPDATE SET 
                artist=artist
            RETURNING id;
        ",
        )?;

        let now = chrono::Utc::now().to_rfc3339();

        let id = stmt.query_row(
            named_params! {
                ":artist": self.artist,
                ":added_at": now,
            },
            |row| row.get(0),
        )?;

        Ok(id)
    }

    // TODO: the following functions should be on a different struct

    /// Count all rows currently in the `artists` database
    fn count_all(conn: &Connection) -> Result<Integer> {
        let count = conn.query_row("SELECT COUNT(id) FROM artists;", [], |v| v.get(0))?;

        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use crate::new_database::{artist_insert::InsertArtist, test_utils::gen_database};

    /// Simple test that [`InsertArtist::upsert`] works correctly
    /// both with insertion and updating.
    #[test]
    fn should_insert_artist_simple() {
        let db = gen_database();

        let data = InsertArtist { artist: "ArtistA" };

        let db = db.conn.lock();

        let id = data.upsert(&db).unwrap();

        assert_eq!(id, 1);

        let new_id = data.upsert(&db).unwrap();

        // check that insertion and upsertion(update) return the same id
        assert_eq!(new_id, id);

        let count = InsertArtist::count_all(&db).unwrap();

        assert_eq!(count, 1);
    }
}
