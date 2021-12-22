use super::constants::APE_PREAMBLE;
#[cfg(feature = "ape")]
use super::tag::{ape_tag::ApeTag, read::read_ape_tag};
use super::{ApeFile, ApeProperties};
use crate::ape::tag::read_ape_header;
use crate::error::{LoftyError, Result};
#[cfg(feature = "id3v1")]
use crate::id3::v1::tag::Id3v1Tag;
#[cfg(feature = "id3v2")]
use crate::id3::v2::{read::parse_id3v2, tag::Id3v2Tag};
use crate::id3::{find_id3v1, find_id3v2, find_lyrics3v2};

use std::io::{Read, Seek, SeekFrom};

pub(crate) fn read_from<R>(data: &mut R, read_properties: bool) -> Result<ApeFile>
where
	R: Read + Seek,
{
	let start = data.seek(SeekFrom::Current(0))?;
	let end = data.seek(SeekFrom::End(0))?;

	data.seek(SeekFrom::Start(start))?;

	let mut stream_len = end - start;

	#[cfg(feature = "id3v2")]
	let mut id3v2_tag: Option<Id3v2Tag> = None;
	#[cfg(feature = "id3v1")]
	let mut id3v1_tag: Option<Id3v1Tag> = None;
	#[cfg(feature = "ape")]
	let mut ape_tag: Option<ApeTag> = None;

	// ID3v2 tags are unsupported in APE files, but still possible
	#[allow(unused_variables)]
	if let (Some(header), Some(content)) = find_id3v2(data, true)? {
		stream_len -= u64::from(header.size);

		// Exclude the footer
		if header.flags.footer {
			stream_len -= 10;
		}

		#[cfg(feature = "id3v2")]
		{
			let reader = &mut &*content;

			let id3v2 = parse_id3v2(reader, header)?;
			id3v2_tag = Some(id3v2)
		}
	}

	let mut found_mac = false;
	let mut mac_start = 0;

	let mut header = [0; 4];
	data.read_exact(&mut header)?;

	while !found_mac {
		match &header {
			b"MAC " => {
				mac_start = data.seek(SeekFrom::Current(0))?;

				found_mac = true;
			}
			// An APE tag at the beginning of the file goes against the spec, but is still possible.
			// This only allows for v2 tags though, since it relies on the header.
			b"APET" => {
				// Get the remaining part of the ape tag
				let mut remaining = [0; 4];
				data.read_exact(&mut remaining).map_err(|_| {
					LoftyError::Ape(
						"Found partial APE tag, but there isn't enough data left in the reader",
					)
				})?;

				if &remaining[..4] != b"AGEX" {
					return Err(LoftyError::Ape("Found incomplete APE tag"));
				}

				let ape_header = read_ape_header(data, false)?;
				stream_len -= u64::from(ape_header.size);

				#[cfg(feature = "ape")]
				{
					let ape = read_ape_tag(data, ape_header)?;
					ape_tag = Some(ape)
				}

				#[cfg(not(feature = "ape"))]
				data.seek(SeekFrom::Current(ape_header.size as i64))?;
			}
			_ => {
				return Err(LoftyError::Ape(
					"Invalid data found while reading header, expected any of [\"MAC \", \
					 \"APETAGEX\", \"ID3\"]",
				))
			}
		}
	}

	// First see if there's a ID3v1 tag
	//
	// Starts with ['T', 'A', 'G']
	// Exactly 128 bytes long (including the identifier)
	#[allow(unused_variables)]
	let (found_id3v1, id3v1) = find_id3v1(data, true)?;

	if found_id3v1 {
		stream_len -= 128;
		#[cfg(feature = "id3v1")]
		{
			id3v1_tag = id3v1;
		}
	}

	// Next, check for a Lyrics3v2 tag, and skip over it, as it's no use to us
	let (found_lyrics3v1, lyrics3v2_size) = find_lyrics3v2(data)?;

	if found_lyrics3v1 {
		stream_len -= u64::from(lyrics3v2_size)
	}

	// Next, search for an APE tag footer
	//
	// Starts with ['A', 'P', 'E', 'T', 'A', 'G', 'E', 'X']
	// Exactly 32 bytes long
	// Strongly recommended to be at the end of the file
	data.seek(SeekFrom::Current(-32))?;

	let mut ape_preamble = [0; 8];
	data.read_exact(&mut ape_preamble)?;

	if &ape_preamble == APE_PREAMBLE {
		let ape_header = read_ape_header(data, true)?;
		stream_len -= u64::from(ape_header.size);

		#[cfg(feature = "ape")]
		{
			let ape = read_ape_tag(data, ape_header)?;
			ape_tag = Some(ape)
		}

		#[cfg(not(feature = "ape"))]
		data.seek(SeekFrom::Current(ape_header.size as i64))?;
	}

	let file_length = data.seek(SeekFrom::Current(0))?;

	// Go back to the MAC header to read properties
	data.seek(SeekFrom::Start(mac_start))?;

	Ok(ApeFile {
		#[cfg(feature = "id3v1")]
		id3v1_tag,
		#[cfg(feature = "id3v2")]
		id3v2_tag,
		#[cfg(feature = "ape")]
		ape_tag,
		properties: if read_properties {
			super::properties::read_properties(data, stream_len, file_length)?
		} else {
			ApeProperties::default()
		},
	})
}
