use std::io::{Cursor, Read, Seek, SeekFrom, Write};

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

use crate::error::{LoftyError, Result};
use crate::id3::v2::util::text_utils;
use crate::id3::v2::util::text_utils::{
	decode_text, encode_text, read_to_terminator, utf16_decode, TextEncoding,
};

#[derive(Copy, Clone, PartialEq, Debug, Eq, Hash)]
/// The unit used for [`SynchronizedText`] timestamps
pub enum TimestampFormat {
	/// The unit is MPEG frames
	MPEG = 1,
	/// The unit is milliseconds
	MS = 2,
}

impl TimestampFormat {
	/// Get a TimestampFormat from a u8, must be 1-2 inclusive
	pub fn from_u8(byte: u8) -> Option<Self> {
		match byte {
			1 => Some(Self::MPEG),
			2 => Some(Self::MS),
			_ => None,
		}
	}
}

#[derive(Copy, Clone, PartialEq, Debug, Eq, Hash)]
#[allow(missing_docs)]
/// The type of text stored in a [`SynchronizedText`]
pub enum SyncTextContentType {
	Other = 0,
	Lyrics = 1,
	TextTranscription = 2,
	PartName = 3,
	Events = 4,
	Chord = 5,
	Trivia = 6,
	WebpageURL = 7,
	ImageURL = 8,
}

impl SyncTextContentType {
	/// Get a SyncTextContentType from a u8, must be 0-8 inclusive
	pub fn from_u8(byte: u8) -> Option<Self> {
		match byte {
			0 => Some(Self::Other),
			1 => Some(Self::Lyrics),
			2 => Some(Self::TextTranscription),
			3 => Some(Self::PartName),
			4 => Some(Self::Events),
			5 => Some(Self::Chord),
			6 => Some(Self::Trivia),
			7 => Some(Self::WebpageURL),
			8 => Some(Self::ImageURL),
			_ => None,
		}
	}
}

#[derive(PartialEq, Clone, Debug, Eq, Hash)]
/// Information about a [`SynchronizedText`]
pub struct SyncTextInformation {
	/// The text encoding (description/text)
	pub encoding: TextEncoding,
	/// ISO-639-2 language code (3 bytes)
	pub language: String,
	/// The format of the timestamps
	pub timestamp_format: TimestampFormat,
	/// The type of content stored
	pub content_type: SyncTextContentType,
	/// Unique content description
	pub description: Option<String>,
}

/// Represents an ID3v2 synchronized text frame
pub struct SynchronizedText {
	/// Information about the synchronized text
	pub information: SyncTextInformation,
	/// Collection of timestamps and text
	pub content: Vec<(u32, String)>,
}

impl SynchronizedText {
	/// Read a [`SynchronizedText`] from a slice
	///
	/// NOTE: This expects the frame header to have already been skipped
	///
	/// # Errors
	///
	/// This function will return [`BadSyncText`][LoftyError::BadSyncText] if at any point it's unable to parse the data
	pub fn parse(data: &[u8]) -> Result<Self> {
		if data.len() < 7 {
			return Err(LoftyError::Id3v2("SYLT frame has invalid size (< 7)"));
		}

		let encoding = TextEncoding::from_u8(data[0])
			.ok_or(LoftyError::TextDecode("Found invalid encoding"))?;
		let lang = std::str::from_utf8(&data[1..4]).map_err(|_| LoftyError::BadSyncText)?;
		let timestamp_format = TimestampFormat::from_u8(data[4]).ok_or(LoftyError::BadSyncText)?;
		let content_type = SyncTextContentType::from_u8(data[5]).ok_or(LoftyError::BadSyncText)?;

		let mut cursor = Cursor::new(&data[6..]);
		let description = text_utils::decode_text(&mut cursor, encoding, true)
			.map_err(|_| LoftyError::BadSyncText)?;

		let mut endianness: fn([u8; 2]) -> u16 = u16::from_le_bytes;

		// It's possible for the description to be the only string with a BOM
		// To be safe, we change the encoding to the concrete variant determined from the description
		if encoding == TextEncoding::UTF16 {
			endianness = match [cursor.get_ref()[6], cursor.get_ref()[7]] {
				[0xFF, 0xFE] => u16::from_le_bytes,
				[0xFE, 0xFF] => u16::from_be_bytes,
				// Since the description was already read, we can assume the BOM was valid
				_ => unreachable!(),
			};
		}

		let mut pos = 0;
		let total = cursor.seek(SeekFrom::Current(0))? as u32 + (cursor.get_ref().len() - 6) as u32;

		let mut content = Vec::new();

		while pos < total {
			let text = (|| -> Result<String> {
				if encoding == TextEncoding::UTF16 {
					let mut bom = [0; 2];
					cursor
						.read_exact(&mut bom)
						.map_err(|_| LoftyError::BadSyncText)?;

					// Encountered text that doesn't include a BOM
					if bom != [0xFF, 0xFE] || bom != [0xFE, 0xFF] {
						if let Some(raw_text) = read_to_terminator(&mut cursor, TextEncoding::UTF16)
						{
							return utf16_decode(&*raw_text, endianness)
								.map_err(|_| LoftyError::BadSyncText);
						}

						return Ok(String::new());
					}
				}

				Ok(decode_text(&mut cursor, encoding, true)
					.map_err(|_| LoftyError::BadSyncText)?
					.unwrap_or_else(String::new))
			})()?;

			pos += text.len() as u32;

			if (pos + 4) > total {
				return Err(LoftyError::BadSyncText);
			}

			let time = cursor.read_u32::<BigEndian>()?;
			pos += 4;

			content.push((time, text));
		}

		Ok(Self {
			information: SyncTextInformation {
				encoding,
				language: lang.to_string(),
				timestamp_format,
				content_type,
				description,
			},
			content,
		})
	}

	/// Convert a [`SynchronizedText`] to an ID3v2 SYLT frame byte Vec
	///
	/// NOTE: This does not include the frame header
	///
	/// # Errors
	///
	/// * `language`'s length != 3
	/// * `content`'s length > [`u32::MAX`]
	pub fn as_bytes(&self) -> Result<Vec<u8>> {
		let information = &self.information;

		let mut data = vec![information.encoding as u8];

		if information.language.len() == 3 {
			data.write_all(information.language.as_bytes())?;
			data.write_u8(information.timestamp_format as u8)?;
			data.write_u8(information.content_type as u8)?;

			if let Some(description) = &information.description {
				data.write_all(&*encode_text(description, information.encoding, true))?;
			} else {
				data.write_u8(0)?;
			}

			for (time, ref text) in &self.content {
				data.write_all(&*encode_text(text, information.encoding, true))?;
				data.write_u32::<BigEndian>(*time)?;
			}

			if data.len() as u64 > u64::from(u32::MAX) {
				return Err(LoftyError::TooMuchData);
			}

			return Ok(data);
		}

		Err(LoftyError::BadSyncText)
	}
}
