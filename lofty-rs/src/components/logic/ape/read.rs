use super::constants::APE_PREAMBLE;
use super::properties::{properties_gt_3980, properties_lt_3980};
use super::tag::read_ape_tag;
use super::ApeData;
use crate::components::logic::id3::{find_id3v1, find_id3v2, find_lyrics3v2};
use crate::{FileProperties, LoftyError, Result};

use std::io::{Read, Seek, SeekFrom};

use byteorder::{LittleEndian, ReadBytesExt};

fn read_properties<R>(data: &mut R, stream_len: u64) -> Result<FileProperties>
where
	R: Read + Seek,
{
	let version = data
		.read_u16::<LittleEndian>()
		.map_err(|_| LoftyError::Ape("Unable to read version"))?;

	// Property reading differs between versions
	if version >= 3980 {
		properties_gt_3980(data, stream_len)
	} else {
		properties_lt_3980(data, version, stream_len)
	}
}

pub(crate) fn read_from<R>(data: &mut R) -> Result<ApeData>
where
	R: Read + Seek,
{
	let start = data.seek(SeekFrom::Current(0))?;
	let end = data.seek(SeekFrom::End(0))?;

	data.seek(SeekFrom::Start(start))?;

	let mut stream_len = end - start;

	let mut ape_data = ApeData {
		id3v1: None,
		id3v2: None,
		ape: None,
		properties: FileProperties::default(),
	};

	let mut found_mac = false;
	let mut mac_start = 0;

	// ID3v2 tags are unsupported in APE files, but still possible
	if let Some(id3v2) = find_id3v2(data, true)? {
		stream_len -= id3v2.len() as u64;
		ape_data.id3v2 = Some(id3v2)
	}

	let mut header = [0; 4];
	data.read_exact(&mut header)?;

	while !found_mac {
		match &header {
			b"MAC " => {
				mac_start = data.seek(SeekFrom::Current(0))?;

				found_mac = true;
			},
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

				let (ape_tag, size) = read_ape_tag(data, false)?;

				stream_len -= u64::from(size);
				ape_data.ape = Some(ape_tag)
			},
			_ => {
				return Err(LoftyError::Ape(
					"Invalid data found while reading header, expected any of [\"MAC \", \"ID3 \
					 \", \"APETAGEX\"]",
				))
			},
		}
	}

	// First see if there's a ID3v1 tag
	//
	// Starts with ['T', 'A', 'G']
	// Exactly 128 bytes long (including the identifier)
	let (found_id3v1, id3v1) = find_id3v1(data, true)?;

	if found_id3v1 {
		stream_len -= 128;
		ape_data.id3v1 = id3v1;
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
		let (ape_tag, size) = read_ape_tag(data, true)?;

		stream_len -= u64::from(size);
		ape_data.ape = Some(ape_tag)
	}

	// Go back to the MAC header to read properties
	data.seek(SeekFrom::Start(mac_start))?;

	ape_data.properties = read_properties(data, stream_len)?;

	Ok(ape_data)
}
