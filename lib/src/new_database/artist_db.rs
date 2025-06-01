#[derive(Debug, Clone, PartialEq)]
pub struct ArtistInsertable<'a> {
    /// The name of the artist
    pub artist: &'a str,
}
