//! ID3v2 items and utilities
//!
//! ## Important notes
//!
//! See:
//!
//! * [Id3v2Tag]
//! * [Frame]
mod flags;
#[cfg(feature = "id3v2")]
mod frame;
#[cfg(feature = "id3v2")]
mod items;
#[cfg(feature = "id3v2")]
pub(crate) mod read;
#[cfg(feature = "id3v2_restrictions")]
mod restrictions;
#[cfg(feature = "id3v2")]
pub(crate) mod tag;
pub(crate) mod util;
#[cfg(feature = "id3v2")]
pub(crate) mod write;

#[cfg(feature = "id3v2")]
pub use flags::Id3v2TagFlags;
#[cfg(feature = "id3v2")]
pub use frame::{EncodedTextFrame, Frame, FrameFlags, FrameID, FrameValue, LanguageFrame};
#[cfg(feature = "id3v2")]
pub use items::{
	encapsulated_object::{GEOBInformation, GeneralEncapsulatedObject},
	sync_text::{SyncTextContentType, SyncTextInformation, SynchronizedText, TimestampFormat},
};
#[cfg(feature = "id3v2_restrictions")]
pub use restrictions::{
	ImageSizeRestrictions, TagRestrictions, TagSizeRestrictions, TextSizeRestrictions,
};
#[cfg(feature = "id3v2")]
pub use tag::Id3v2Tag;
pub use util::text_utils::TextEncoding;
#[cfg(feature = "id3v2")]
pub use util::upgrade::{upgrade_v2, upgrade_v3};

#[cfg(not(feature = "id3v2"))]
use flags::Id3v2TagFlags;

use crate::error::{LoftyError, Result};

use std::io::Read;

use byteorder::{BigEndian, ByteOrder, ReadBytesExt};

#[derive(PartialEq, Debug, Clone, Copy)]
/// The ID3v2 version
pub enum Id3v2Version {
	/// ID3v2.2
	V2,
	/// ID3v2.3
	V3,
	/// ID3v2.4
	V4,
}

// https://github.com/polyfloyd/rust-id3/blob/e142ec656bf70a8153f6e5b34a37f26df144c3c1/src/stream/unsynch.rs#L18-L20
pub(crate) fn unsynch_u32(n: u32) -> u32 {
	n & 0xFF | (n & 0xFF00) >> 1 | (n & 0xFF_0000) >> 2 | (n & 0xFF00_0000) >> 3
}

#[cfg(feature = "id3v2")]
// https://github.com/polyfloyd/rust-id3/blob/e142ec656bf70a8153f6e5b34a37f26df144c3c1/src/stream/unsynch.rs#L9-L15
pub(crate) fn synch_u32(n: u32) -> Result<u32> {
	if n > 0x1000_0000 {
		return Err(LoftyError::TooMuchData);
	}

	let mut x: u32 = n & 0x7F | (n & 0xFFFF_FF80) << 1;
	x = x & 0x7FFF | (x & 0xFFFF_8000) << 1;
	x = x & 0x7F_FFFF | (x & 0xFF80_0000) << 1;
	Ok(x)
}

#[derive(Copy, Clone)]
pub(crate) struct Id3v2Header {
	#[cfg(feature = "id3v2")]
	pub version: Id3v2Version,
	pub flags: Id3v2TagFlags,
	pub size: u32,
}

pub(crate) fn read_id3v2_header<R>(bytes: &mut R) -> Result<Id3v2Header>
where
	R: Read,
{
	let mut header = [0; 10];
	bytes.read_exact(&mut header)?;

	if &header[..3] != b"ID3" {
		return Err(LoftyError::FakeTag);
	}

	// Version is stored as [major, minor], but here we don't care about minor revisions unless there's an error.
	let version = match header[3] {
		2 => Id3v2Version::V2,
		3 => Id3v2Version::V3,
		4 => Id3v2Version::V4,
		major => return Err(LoftyError::BadId3v2Version(major, header[4])),
	};

	let flags = header[5];

	// Compression was a flag only used in ID3v2.2 (bit 2).
	// At the time the ID3v2.2 specification was written, a compression scheme wasn't decided.
	// The spec recommends just ignoring the tag in this case.
	if version == Id3v2Version::V2 && flags & 0x40 == 0x40 {
		return Err(LoftyError::Id3v2("Encountered a compressed ID3v2.2 tag"));
	}

	let mut flags_parsed = Id3v2TagFlags {
		unsynchronisation: flags & 0x80 == 0x80,
		experimental: (version == Id3v2Version::V4 || version == Id3v2Version::V3)
			&& flags & 0x20 == 0x20,
		footer: (version == Id3v2Version::V4 || version == Id3v2Version::V3)
			&& flags & 0x10 == 0x10,
		crc: false, // Retrieved later if applicable
		#[cfg(feature = "id3v2_restrictions")]
		restrictions: (false, TagRestrictions::default()), // Retrieved later if applicable
	};

	let extended_header =
		(version == Id3v2Version::V4 || version == Id3v2Version::V3) && flags & 0x40 == 0x40;

	if extended_header {
		let extended_size = unsynch_u32(bytes.read_u32::<BigEndian>()?);

		if extended_size < 6 {
			return Err(LoftyError::Id3v2(
				"Found an extended header with an invalid size (< 6)",
			));
		}

		// Useless byte since there's only 1 byte for flags
		let _num_flag_bytes = bytes.read_u8()?;

		let extended_flags = bytes.read_u8()?;

		// The only flags we care about here are the CRC and restrictions

		if extended_flags & 0x20 == 0x20 {
			flags_parsed.crc = true;

			// We don't care about the existing CRC (5) or its length byte (1)
			let mut crc = [0; 6];
			bytes.read_exact(&mut crc)?;
		}

		#[cfg(feature = "id3v2_restrictions")]
		if extended_flags & 0x10 == 0x10 {
			flags_parsed.restrictions.0 = true;

			// We don't care about the length byte, it is always 1
			let _data_length = bytes.read_u8()?;

			flags_parsed.restrictions.1 = TagRestrictions::from_byte(bytes.read_u8()?);
		}
	}

	let size = unsynch_u32(BigEndian::read_u32(&header[6..]));

	Ok(Id3v2Header {
		#[cfg(feature = "id3v2")]
		version,
		flags: flags_parsed,
		size,
	})
}
