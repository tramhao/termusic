//! ID3v1 items
//!
//! # ID3v1 notes
//!
//! See also: [Id3v1Tag]
//!
//! ## Genres
//!
//! ID3v1 stores the genre in a single byte ranging from 0 to 192 (inclusive).
//! All possible genres have been stored in the [`GENRES`] constant.
//!
//! ## Track Numbers
//!
//! ID3v1 stores the track number in a non-zero byte.
//! A track number of 0 will be treated as an empty field.
//! Additionally, there is no track total field.
pub(crate) mod constants;
pub(crate) mod read;
pub(crate) mod tag;
pub(crate) mod write;

pub use crate::id3::v1::constants::GENRES;
pub use crate::id3::v1::tag::Id3v1Tag;
