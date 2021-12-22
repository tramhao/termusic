use super::header::{parse_header, parse_v2_header};
use super::Frame;
use crate::error::{LoftyError, Result};
use crate::id3::v2::frame::content::parse_content;
use crate::id3::v2::FrameValue;
use crate::id3::v2::Id3v2Version;

use std::io::Read;

use byteorder::{BigEndian, ReadBytesExt};

impl Frame {
	pub(crate) fn read<R>(reader: &mut R, version: Id3v2Version) -> Result<Option<Self>>
	where
		R: Read,
	{
		// The header will be upgraded to ID3v2.4 past this point, so they can all be treated the same
		let (id, size, mut flags) = match match version {
			Id3v2Version::V2 => parse_v2_header(reader)?,
			Id3v2Version::V3 => parse_header(reader, false)?,
			Id3v2Version::V4 => parse_header(reader, true)?,
		} {
			None => return Ok(None),
			Some(frame_header) => frame_header,
		};

		let mut content = vec![0; size as usize];
		reader.read_exact(&mut content)?;

		if flags.unsynchronisation {
			content = crate::id3::v2::util::unsynch_content(content.as_slice())?;
		}

		if flags.compression {
			let mut decompressed = Vec::new();
			flate2::Decompress::new(true)
				.decompress_vec(&content, &mut decompressed, flate2::FlushDecompress::None)
				.map_err(|_| {
					LoftyError::Id3v2("Encountered a compressed frame, failed to decompress")
				})?;

			content = decompressed
		}

		let mut content_reader = &*content;

		// Get the encryption method symbol
		if flags.encryption.0 {
			flags.encryption.1 = content_reader.read_u8()?;
		}

		// Get the group identifier
		if flags.grouping_identity.0 {
			flags.grouping_identity.1 = content_reader.read_u8()?;
		}

		// Get the real data length
		if flags.data_length_indicator.0 {
			flags.data_length_indicator.1 = content_reader.read_u32::<BigEndian>()?;
		}

		let value = if flags.encryption.0 {
			if !flags.data_length_indicator.0 {
				return Err(LoftyError::Id3v2(
					"Encountered an encrypted frame without a data length indicator",
				));
			}

			FrameValue::Binary(content)
		} else {
			parse_content(&mut content_reader, id.as_str(), version)?
		};

		Ok(Some(Self { id, value, flags }))
	}
}
