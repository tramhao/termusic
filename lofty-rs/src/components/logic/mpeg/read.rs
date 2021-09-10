use super::header::{Header, XingHeader};
use crate::components::logic::id3::decode_u32;
use crate::components::logic::mpeg::MpegData;
use crate::{FileProperties, LoftyError, Result};

use std::convert::TryInto;
use std::io::{Read, Seek, SeekFrom};
use std::time::Duration;

use byteorder::{BigEndian, ByteOrder, ReadBytesExt};

fn read_properties(
	first_frame: (Header, u64),
	last_frame: (Header, u64),
	xing_header: Option<XingHeader>,
) -> FileProperties {
	let (duration, bitrate) = {
		if let Some(xing_header) = xing_header {
			if first_frame.0.samples_per_frame > 0 && first_frame.0.sample_rate > 0 {
				let frame_time =
					u32::from(first_frame.0.samples_per_frame) * 1000 / first_frame.0.sample_rate;
				let length = u64::from(frame_time) * u64::from(xing_header.frames);

				(
					Duration::from_millis(length),
					((u64::from(xing_header.size) * 8) / length) as u32,
				)
			} else {
				(Duration::ZERO, first_frame.0.bitrate)
			}
		} else if first_frame.0.bitrate > 0 {
			let bitrate = first_frame.0.bitrate;

			let stream_length = last_frame.1 - first_frame.1 + u64::from(first_frame.0.len);

			let length = if stream_length > 0 {
				Duration::from_millis((stream_length * 8) / u64::from(bitrate))
			} else {
				Duration::ZERO
			};

			(length, bitrate)
		} else {
			(Duration::ZERO, 0)
		}
	};

	FileProperties::new(
		duration,
		Some(bitrate),
		Some(first_frame.0.sample_rate),
		Some(first_frame.0.channels as u8),
	)
}

#[allow(clippy::similar_names)]
pub(crate) fn read_from<R>(data: &mut R) -> Result<MpegData>
where
	R: Read + Seek,
{
	let mut id3 = Vec::new();
	let mut ape = Vec::new();

	let mut first_mpeg_frame = (None, 0);
	let mut last_mpeg_frame = (None, 0);

	// Skip any invalid padding
	while data.read_u8()? == 0 {}

	data.seek(SeekFrom::Current(-1))?;

	let mut header = [0; 4];

	while let Ok(()) = data.read_exact(&mut header) {
		match &header {
			_ if u32::from_be_bytes(header) >> 21 == 0x7FF => {
				let start = data.seek(SeekFrom::Current(0))? - 4;
				let header = Header::read(u32::from_be_bytes(header))?;
				data.seek(SeekFrom::Current(i64::from(header.len - 4)))?;

				if first_mpeg_frame.0.is_none() {
					first_mpeg_frame = (Some(header), start);
				}

				last_mpeg_frame = (Some(header), start);
			},
			_ if &header[..3] == b"ID3" || &header[..3] == b"id3" => {
				let mut remaining_header = [0; 6];
				data.read_exact(&mut remaining_header)?;

				let size = (decode_u32(BigEndian::read_u32(&remaining_header[2..])) + 10) as usize;
				data.seek(SeekFrom::Current(-10))?;

				let mut id3v2 = vec![0; size];
				data.read_exact(&mut id3v2)?;

				id3 = id3v2;
				continue;
			},
			_ if &header[..3] == b"TAG" => {
				data.seek(SeekFrom::Current(-10))?;

				let mut id3v1 = vec![0; 128];
				data.read_exact(&mut id3v1)?;

				id3 = id3v1;
				continue;
			},
			b"APET" => {
				let mut header_remaining = [0; 4];
				data.read_exact(&mut header_remaining)?;

				if &header_remaining == b"AGEX" {
					// Skip version bytes (4)
					let mut info = [0; 8];
					data.read_exact(&mut info)?;

					let size = u32::from_le_bytes(
						info[4..]
							.try_into()
							.map_err(|_| LoftyError::Mpeg("Unreachable error"))?,
					);

					data.seek(SeekFrom::Current(-16))?;

					let mut apev2 = vec![0; (32 + size) as usize];
					data.read_exact(&mut apev2)?;

					ape = apev2;
					continue;
				}
			},
			_ => return Err(LoftyError::Mpeg("File contains an invalid frame")),
		}
	}

	if first_mpeg_frame.0.is_none() {
		return Err(LoftyError::Mpeg("Unable to find an MPEG frame"));
	}

	let first_mpeg_frame = (first_mpeg_frame.0.unwrap(), first_mpeg_frame.1);
	let last_mpeg_frame = (last_mpeg_frame.0.unwrap(), last_mpeg_frame.1);

	let xing_header_location = first_mpeg_frame.1 + u64::from(first_mpeg_frame.0.data_start);

	data.seek(SeekFrom::Start(xing_header_location))?;

	let mut xing_reader = [0; 32];
	data.read_exact(&mut xing_reader)?;

	let xing_header = XingHeader::read(&mut &xing_reader[..]).ok();

	let properties = read_properties(first_mpeg_frame, last_mpeg_frame, xing_header);

	Ok(MpegData {
		id3: (!id3.is_empty()).then(|| id3),
		ape: (!ape.is_empty()).then(|| ape),
		properties,
	})
}
