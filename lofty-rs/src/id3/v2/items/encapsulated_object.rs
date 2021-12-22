use crate::error::{LoftyError, Result};
use crate::id3::v2::util::text_utils::{decode_text, encode_text, TextEncoding};

use std::io::{Cursor, Read};

#[derive(PartialEq, Clone, Debug, Eq, Hash)]
/// Information about a [`GeneralEncapsulatedObject`]
pub struct GEOBInformation {
	/// The text encoding of `file_name` and `description`
	pub encoding: TextEncoding,
	/// The file's mimetype
	pub mime_type: Option<String>,
	/// The file's name
	pub file_name: Option<String>,
	/// A unique content descriptor
	pub descriptor: Option<String>,
}

/// Allows for encapsulation of any file type inside an ID3v2 tag
pub struct GeneralEncapsulatedObject {
	/// Information about the data
	pub information: GEOBInformation,
	/// The file's content
	pub data: Vec<u8>,
}

impl GeneralEncapsulatedObject {
	/// Read a [`GeneralEncapsulatedObject`] from a slice
	///
	/// NOTE: This expects the frame header to have already been skipped
	///
	/// # Errors
	///
	/// This function will return an error if at any point it's unable to parse the data
	pub fn parse(data: &[u8]) -> Result<Self> {
		if data.len() < 4 {
			return Err(LoftyError::Id3v2("GEOB frame has invalid size (< 4)"));
		}

		let encoding = TextEncoding::from_u8(data[0])
			.ok_or(LoftyError::TextDecode("Found invalid encoding"))?;

		let mut cursor = Cursor::new(&data[1..]);

		let mime_type = decode_text(&mut cursor, TextEncoding::Latin1, true)?;
		let file_name = decode_text(&mut cursor, encoding, true)?;
		let descriptor = decode_text(&mut cursor, encoding, true)?;

		let mut data = Vec::new();
		cursor.read_to_end(&mut data)?;

		Ok(Self {
			information: GEOBInformation {
				encoding,
				mime_type,
				file_name,
				descriptor,
			},
			data,
		})
	}

	/// Convert a [`GeneralEncapsulatedObject`] into an ID3v2 GEOB frame byte Vec
	///
	/// NOTE: This does not include a frame header
	pub fn as_bytes(&self) -> Vec<u8> {
		let mut bytes = Vec::new();

		let encoding = self.information.encoding;

		bytes.extend([self.information.encoding as u8].iter());

		if let Some(ref mime_type) = self.information.mime_type {
			bytes.extend(mime_type.as_bytes())
		} else {
			bytes.extend([0].iter());
		}

		if let Some(ref file_name) = self.information.file_name {
			bytes.extend(&*encode_text(file_name, encoding, true))
		} else {
			bytes.extend([0].iter());
		}

		if let Some(ref descriptor) = self.information.descriptor {
			bytes.extend(&*encode_text(descriptor, encoding, true))
		} else {
			bytes.extend([0].iter());
		}

		bytes.extend(&self.data);

		bytes
	}
}
