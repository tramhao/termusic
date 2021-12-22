//! [![GitHub Workflow Status](https://img.shields.io/github/workflow/status/Serial-ATA/lofty-rs/CI?style=for-the-badge&logo=github)](https://github.com/Serial-ATA/lofty-rs/actions/workflows/ci.yml)
//! [![Downloads](https://img.shields.io/crates/d/lofty?style=for-the-badge&logo=rust)](https://crates.io/crates/lofty)
//! [![Version](https://img.shields.io/crates/v/lofty?style=for-the-badge&logo=rust)](https://crates.io/crates/lofty)
//!
//! Parse, convert, and write metadata to audio formats.
//!
//! # Supported Formats
//!
//! | File Format | Extensions                                      | Read | Write | Metadata Format(s)                            |
//! |-------------|-------------------------------------------------|------|-------|-----------------------------------------------|
//! | APE         | `ape`                                           |**X** |**X**  |`APEv2`, `APEv1`, `ID3v2` (Read only), `ID3v1` |
//! | AIFF        | `aiff`, `aif`                                   |**X** |**X**  |`ID3v2`, `Text Chunks`                         |
//! | FLAC        | `flac`                                          |**X** |**X**  |`Vorbis Comments`                              |
//! | MP3         | `mp3`                                           |**X** |**X**  |`ID3v2`, `ID3v1`, `APEv2`, `APEv1`             |
//! | MP4         | `mp4`, `m4a`, `m4b`, `m4p`, `m4r`, `m4v`, `3gp` |**X** |**X**  |`iTunes-style ilst`                            |
//! | Opus        | `opus`                                          |**X** |**X**  |`Vorbis Comments`                              |
//! | Ogg Vorbis  | `ogg`                                           |**X** |**X**  |`Vorbis Comments`                              |
//! | WAV         | `wav`, `wave`                                   |**X** |**X**  |`ID3v2`, `RIFF INFO`                           |
//!
//! # Examples
//!
//! ## Reading a generic file
//!
//! It isn't always convenient to [use concrete file types](#using-concrete-file-types), which is where [`TaggedFile`]
//! comes in.
//!
//! ### Using a path
//!
//! ```rust
//! # use lofty::LoftyError;
//! # fn main() -> Result<(), LoftyError> {
//! use lofty::{read_from_path, Probe};
//!
//! // First, create a probe.
//! // This will guess the format from the extension
//! // ("mp3" in this case), but we can guess from the content if we want to.
//! let tagged_file = read_from_path("tests/files/assets/a.mp3", false)?;
//!
//! // Let's guess the format from the content just in case.
//! // This is not necessary in this case!
//! let tagged_file2 = Probe::open("tests/files/assets/a.mp3")?.guess_file_type()?.read(false)?;
//! # Ok(())
//! # }
//! ```
//!
//! ### Using an existing reader
//!
//! ```rust
//! # use lofty::LoftyError;
//! # fn main() -> Result<(), LoftyError> {
//! use std::fs::File;
//! use lofty::read_from;
//!
//! // Let's read from an open file
//! let mut file = File::open("tests/files/assets/a.mp3")?;
//!
//! // Here, we have to guess the file type prior to reading
//! let tagged_file = read_from(&mut file, false)?;
//! # Ok(())
//! # }
//! ```
//!
//! ### Accessing tags
//!
//! ```rust
//! # use lofty::LoftyError;
//! # fn main() -> Result<(), LoftyError> {
//! use lofty::read_from_path;
//!
//! let tagged_file = read_from_path("tests/files/assets/a.mp3", false)?;
//!
//! // Get the primary tag (ID3v2 in this case)
//! let id3v2 = tagged_file.primary_tag().unwrap();
//!
//! // If the primary tag doesn't exist, or the tag types
//! // don't matter, the first tag can be retrieved
//! let unknown_first_tag = tagged_file.first_tag().unwrap();
//! # Ok(())
//! # }
//! ```
//!
//! ## Using concrete file types
//!
//! ```rust
//! # use lofty::LoftyError;
//! # fn main() -> Result<(), LoftyError> {
//! use lofty::mp3::Mp3File;
//! use lofty::AudioFile;
//! use lofty::TagType;
//! use std::fs::File;
//!
//! let mut file_content = File::open("tests/files/assets/a.mp3")?;
//!
//! // We are expecting an MP3 file
//! let mpeg_file = Mp3File::read_from(&mut file_content, true)?;
//!
//! assert_eq!(mpeg_file.properties().channels(), 2);
//!
//! // Here we have a file with multiple tags
//! assert!(mpeg_file.contains_tag_type(&TagType::Id3v2));
//! assert!(mpeg_file.contains_tag_type(&TagType::Ape));
//! # Ok(())
//! # }
//! ```
//!
//! # Features
//!
//! ## Individual metadata formats
//! These features are available if you have a specific use case, or just don't want certain formats.
//!
//! * `aiff_text_chunks`
//! * `ape`
//! * `id3v1`
//! * `id3v2`
//! * `mp4_ilst`
//! * `riff_info_list`
//! * `vorbis_comments`
//!
//! ## Utilities
//! * `id3v2_restrictions` - Parses ID3v2 extended headers and exposes flags for fine grained control
//!
//! # Important format-specific notes
//!
//! All formats have their own quirks that may produce unexpected results between conversions.
//! Be sure to read the module documentation of each format to see important notes and warnings.
#![deny(
	clippy::pedantic,
	clippy::all,
	missing_docs,
	rustdoc::broken_intra_doc_links
)]
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
	clippy::used_underscore_binding,
	clippy::new_without_default,
	clippy::unused_self,
	clippy::from_over_into,
	clippy::upper_case_acronyms,
	clippy::too_many_arguments,
	clippy::single_match_else
)]

pub mod ape;
mod error;
pub mod id3;
pub mod iff;
pub mod mp3;
pub mod mp4;
pub mod ogg;
mod probe;
pub(crate) mod tag_utils;
mod types;

pub use crate::error::{LoftyError, Result};

pub use crate::probe::Probe;

pub use crate::types::{
	file::{FileType, TaggedFile},
	item::{ItemKey, ItemValue, TagItem},
	properties::FileProperties,
	tag::{Accessor, Tag, TagType},
};

pub use crate::types::file::AudioFile;

pub use crate::types::picture::{MimeType, Picture, PictureType};

#[cfg(feature = "vorbis_comments")]
pub use crate::types::picture::PictureInformation;

pub use probe::{read_from, read_from_path};
