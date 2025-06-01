use std::borrow::Cow;

use either::Either;

use super::{Integer, artist_db::ArtistInsertable};

#[derive(Debug, Clone)]
pub struct AlbumInsertable<'a> {
    pub title: &'a str,
    pub artist_display: Option<&'a str>,

    // mapped metadata
    /// Either a reference to a insertable to look-up or a direct integer to use as reference into `artists`.
    pub artists: Vec<Either<Cow<'a, ArtistInsertable<'a>>, Integer>>,
}
