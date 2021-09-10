//! [![GitHub Workflow Status](https://img.shields.io/github/workflow/status/Serial-ATA/lofty-rs/CI?style=for-the-badge&logo=github)](https://github.com/Serial-ATA/lofty-rs/actions/workflows/ci.yml)
//! [![Downloads](https://img.shields.io/crates/d/lofty?style=for-the-badge&logo=rust)](https://crates.io/crates/lofty)
//! [![Version](https://img.shields.io/crates/v/lofty?style=for-the-badge&logo=rust)](https://crates.io/crates/lofty)
//!
//! This is a fork of [Audiotags](https://github.com/TianyiShi2001/audiotags), adding support for more file types.
//!
//! Parse, convert, and write metadata to audio formats.
//!
//! # Supported Formats
//!
//! | File Format | Extensions                                | Read | Write | Metadata Format(s)                                 |
//! |-------------|-------------------------------------------|------|-------|----------------------------------------------------|
//! | Ape         | `ape`                                     |**X** |**X**  |`APEv2`, `APEv1`, `ID3v2` (Not officially), `ID3v1` |
//! | AIFF        | `aiff`, `aif`                             |**X** |**X**  |`ID3v2`, `Text Chunks`                              |
//! | FLAC        | `flac`                                    |**X** |**X**  |`Vorbis Comments`                                   |
//! | MP3         | `mp3`                                     |**X** |**X**  |`ID3v2`, `ID3v1`, `APEv2`, `APEv1`                  |
//! | MP4         | `mp4`, `m4a`, `m4b`, `m4p`, `m4v`, `isom` |**X** |**X**  |`Atoms`                                             |
//! | Opus        | `opus`                                    |**X** |**X**  |`Vorbis Comments`                                   |
//! | Ogg Vorbis  | `ogg`                                     |**X** |**X**  |`Vorbis Comments`                                   |
//! | WAV         | `wav`, `wave`                             |**X** |**X**  |`ID3v2`, `RIFF INFO`                                |
//!
//! # Examples
//!
//! ## Guessing from extension
//! ```
//! use lofty::{Tag, TagType};
//!
//! let mut tag = Tag::new().read_from_path("tests/assets/a.mp3").unwrap();
//! tag.set_title("Foo");
//!
//! assert_eq!(tag.title(), Some("Foo"));
//! ```
//!
//! ## Guessing from file signature
//! ```
//! use lofty::Tag;
//!
//! let mut tag_sig = Tag::new().read_from_path_signature("tests/assets/a.wav").unwrap();
//! tag_sig.set_artist("Foo artist");
//!
//! assert_eq!(tag_sig.artist(), Some("Foo artist"));
//! ```
//!
//! ## Specifying a TagType
//! ```
//! use lofty::{Tag, TagType};
//!
//! let mut tag = Tag::new().with_tag_type(TagType::Mp4).read_from_path("tests/assets/a.m4a").unwrap();
//! tag.set_album_title("Foo album title");
//!
//! assert_eq!(tag.album_title(), Some("Foo album title"));
//! ```
//!
//! ## Converting between TagTypes
//! ```
//! use lofty::{Tag, TagType};
//!
//! let mut tag = Tag::new().read_from_path("tests/assets/a.mp3").unwrap();
//! tag.set_title("Foo");
//!
//! // You can convert the tag type and save it to another file.
//! tag.to_dyn_tag(TagType::Mp4).write_to_path("tests/assets/a.m4a");
//! assert_eq!(tag.title(), Some("Foo"));
//! ```
//!
//! ## Converting from [`AnyTag`]
//! ```
//! use lofty::{AnyTag, OggTag, AudioTagEdit};
//!
//! let mut anytag = AnyTag::new();
//!
//! anytag.title = Some("Foo title");
//! anytag.artist = Some("Foo artist");
//!
//! let oggtag: OggTag = anytag.into();
//!
//! assert_eq!(oggtag.title(), Some("Foo title"));
//! assert_eq!(oggtag.artist(), Some("Foo artist"));
//! ```
//!
//! # Concrete types
//! * AiffTag (AIFF Text Chunks)
//! * ApeTag
//! * Id3v2Tag
//! * Mp4Tag
//! * OggTag
//! * RiffTag (RIFF LIST INFO)
//!
//! # Features
//!
//! ## Applies to all
//! * `all_tags` - Enables all formats
//!
//! ## Individual formats
//! These features are available if you have a specific usecase, or just don't want certain formats.
//!
//! All format features a prefixed with `format-`
//! * `format-ape`
//! * `format-flac`
//! * `format-id3`
//! * `format-mp4`
//! * `format-opus`
//! * `format-vorbis`
//! * `format-riff`
//!
//! ## Umbrella features
//! These cover all formats under a container format.
//!
//! * `format-ogg` (`format-opus`, `format-vorbis`, `format-flac`)

#![deny(clippy::pedantic, clippy::all, missing_docs)]
#![allow(
	clippy::too_many_lines,
	clippy::cast_precision_loss,
	clippy::cast_sign_loss,
	clippy::cast_possible_wrap,
	clippy::cast_possible_truncation,
	clippy::module_name_repetitions,
	clippy::must_use_candidate,
	clippy::doc_markdown,
	clippy::let_underscore_drop,
	clippy::match_wildcard_for_single_variants,
	clippy::semicolon_if_nothing_returned,
	clippy::used_underscore_binding
)]

mod types;
pub use crate::types::{
	album::Album,
	anytag::AnyTag,
	picture::{MimeType, Picture, PictureType},
	properties::FileProperties,
};

mod tag;
#[cfg(feature = "format-id3")]
pub use crate::tag::Id3Format;
#[cfg(any(
	feature = "format-opus",
	feature = "format-vorbis",
	feature = "format-flac"
))]
pub use crate::tag::OggFormat;
pub use crate::tag::{Tag, TagType};

mod error;
pub use crate::error::{LoftyError, Result};

mod components;
pub use crate::components::tags::*;

mod traits;
pub use crate::traits::{AudioTag, AudioTagEdit, AudioTagWrite, ToAny, ToAnyTag};
