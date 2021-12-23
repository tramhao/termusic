use super::header::{verify_frame_sync, Header, XingHeader};
use super::{Mp3File, Mp3Properties};
#[cfg(feature = "ape")]
use crate::ape::tag::ape_tag::ApeTag;
use crate::ape::tag::read_ape_header;
use crate::error::{LoftyError, Result};
#[cfg(feature = "id3v1")]
use crate::id3::v1::tag::Id3v1Tag;
#[cfg(feature = "id3v2")]
use crate::id3::v2::read::parse_id3v2;
use crate::id3::v2::read_id3v2_header;
#[cfg(feature = "id3v2")]
use crate::id3::v2::Id3v2Tag;

use std::io::{Read, Seek, SeekFrom};

use byteorder::ReadBytesExt;

#[allow(clippy::similar_names)]
pub(crate) fn read_from<R>(data: &mut R, read_properties: bool) -> Result<Mp3File>
where
	R: Read + Seek,
{
	#[cfg(feature = "id3v2")]
	let mut id3v2_tag: Option<Id3v2Tag> = None;
	#[cfg(feature = "id3v1")]
	let mut id3v1_tag: Option<Id3v1Tag> = None;
	#[cfg(feature = "ape")]
	let mut ape_tag: Option<ApeTag> = None;

	let mut first_mpeg_frame = (None, 0);
	let mut last_mpeg_frame = (None, 0);

	// Skip any invalid padding
	while data.read_u8()? == 0 {}

	data.seek(SeekFrom::Current(-1))?;

	let mut header = [0; 4];

	while let Ok(()) = data.read_exact(&mut header) {
		match header {
			_ if verify_frame_sync([header[0], header[1]]) => {
				let start = data.seek(SeekFrom::Current(0))? - 4;
				let header = Header::read(u32::from_be_bytes(header))?;

				if read_properties {
					if first_mpeg_frame.0.is_none() {
						first_mpeg_frame = (Some(header), start);
					}

					last_mpeg_frame = (Some(header), start);
				}

				data.seek(SeekFrom::Current(i64::from(header.len - 4)))?;
			},
			// [I, D, 3, ver_major, ver_minor, flags, size (4 bytes)]
			[b'I', b'D', b'3', ..] => {
				let mut remaining_header = [0; 6];
				data.read_exact(&mut remaining_header)?;

				let header = read_id3v2_header(
					&mut &*[header.as_slice(), remaining_header.as_slice()].concat(),
				)?;
				let skip_footer = header.flags.footer;

				#[cfg(feature = "id3v2")]
				{
					let id3v2 = parse_id3v2(data, header)?;
					id3v2_tag = Some(id3v2);
				}

				// Skip over the footer
				if skip_footer {
					data.seek(SeekFrom::Current(10))?;
				}

				continue;
			},
			[b'T', b'A', b'G', ..] => {
				data.seek(SeekFrom::Current(-4))?;

				let mut id3v1_read = [0; 128];
				data.read_exact(&mut id3v1_read)?;

				#[cfg(feature = "id3v1")]
				{
					id3v1_tag = Some(crate::id3::v1::read::parse_id3v1(id3v1_read));
				}

				continue;
			},
			[b'A', b'P', b'E', b'T'] => {
				let mut header_remaining = [0; 4];
				data.read_exact(&mut header_remaining)?;

				if &header_remaining == b"AGEX" {
					let ape_header = read_ape_header(data, false)?;

					#[cfg(not(feature = "ape"))]
					{
						let size = ape_header.size;
						data.seek(SeekFrom::Current(size as i64))?;
					}

					#[cfg(feature = "ape")]
					{
						ape_tag = Some(crate::ape::tag::read::read_ape_tag(data, ape_header)?);
					}

					continue;
				}
			},
			_ => return Err(LoftyError::Mp3("File contains an invalid frame")),
		}
	}

	let properties = if read_properties {
		if first_mpeg_frame.0.is_none() {
			return Err(LoftyError::Mp3("Unable to find an MPEG frame"));
		}

		let file_length = data.seek(SeekFrom::Current(0))?;

		let first_mpeg_frame = (first_mpeg_frame.0.unwrap(), first_mpeg_frame.1);
		let last_mpeg_frame = (last_mpeg_frame.0.unwrap(), last_mpeg_frame.1);

		let xing_header_location = first_mpeg_frame.1 + u64::from(first_mpeg_frame.0.data_start);

		data.seek(SeekFrom::Start(xing_header_location))?;

		let mut xing_reader = [0; 32];
		data.read_exact(&mut xing_reader)?;

		let xing_header = XingHeader::read(&mut &xing_reader[..]).ok();

		super::properties::read_properties(
			first_mpeg_frame,
			last_mpeg_frame,
			xing_header,
			file_length,
		)
	} else {
		Mp3Properties::default()
	};

	Ok(Mp3File {
		#[cfg(feature = "id3v2")]
		id3v2_tag,
		#[cfg(feature = "id3v1")]
		id3v1_tag,
		#[cfg(feature = "ape")]
		ape_tag,
		properties,
	})
}
