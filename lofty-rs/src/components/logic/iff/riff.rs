use crate::components::logic::iff::IffData;
use crate::{FileProperties, LoftyError, Result};

use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Write};
use std::time::Duration;

use byteorder::{LittleEndian, ReadBytesExt};

const PCM: u16 = 0x0001;
const IEEE_FLOAT: u16 = 0x0003;
const EXTENSIBLE: u16 = 0xfffe;

fn verify_riff<T>(data: &mut T) -> Result<()>
where
	T: Read + Seek,
{
	let mut id = [0; 4];
	data.read_exact(&mut id)?;

	if &id != b"RIFF" {
		return Err(LoftyError::Riff("RIFF file doesn't contain a RIFF chunk"));
	}

	Ok(())
}

pub(crate) fn read_properties(
	fmt: &mut &[u8],
	total_samples: u32,
	stream_len: u32,
) -> Result<FileProperties> {
	let mut format_tag = fmt.read_u16::<LittleEndian>()?;
	let channels = fmt.read_u16::<LittleEndian>()? as u8;

	if channels == 0 {
		return Err(LoftyError::Riff("File contains 0 channels"));
	}

	let sample_rate = fmt.read_u32::<LittleEndian>()?;
	let bytes_per_second = fmt.read_u32::<LittleEndian>()?;

	// Skip 2 bytes
	// Block align (2)
	let _ = fmt.read_u16::<LittleEndian>()?;

	let bits_per_sample = fmt.read_u16::<LittleEndian>()?;

	if format_tag == EXTENSIBLE {
		if fmt.len() < 40 {
			return Err(LoftyError::Riff(
				"Extensible format identified, invalid \"fmt \" chunk size found (< 40)",
			));
		}

		// Skip 8 bytes
		// cbSize (Size of extra format information) (2)
		// Valid bits per sample (2)
		// Channel mask (4)
		let _ = fmt.read_u64::<LittleEndian>()?;

		format_tag = fmt.read_u16::<LittleEndian>()?;
	}

	let non_pcm = format_tag != PCM && format_tag != IEEE_FLOAT;

	if non_pcm && total_samples == 0 {
		return Err(LoftyError::Riff(
			"Non-PCM format identified, no \"fact\" chunk found",
		));
	}

	let sample_frames = if non_pcm {
		total_samples
	} else if bits_per_sample > 0 {
		stream_len / u32::from(u16::from(channels) * ((bits_per_sample + 7) / 8))
	} else {
		0
	};

	let (duration, bitrate) = if sample_rate > 0 && sample_frames > 0 {
		let length = (u64::from(sample_frames) * 1000) / u64::from(sample_rate);

		(
			Duration::from_millis(length),
			(u64::from(stream_len * 8) / length) as u32,
		)
	} else if bytes_per_second > 0 {
		let length = (u64::from(stream_len) * 1000) / u64::from(bytes_per_second);

		(Duration::from_millis(length), (bytes_per_second * 8) / 1000)
	} else {
		(Duration::ZERO, 0)
	};

	Ok(FileProperties::new(
		duration,
		Some(bitrate),
		Some(sample_rate),
		Some(channels),
	))
}

pub(crate) fn read_from<T>(data: &mut T) -> Result<IffData>
where
	T: Read + Seek,
{
	verify_riff(data)?;

	data.seek(SeekFrom::Current(8))?;

	let mut stream_len = 0_u32;
	let mut total_samples = 0_u32;
	let mut fmt = Vec::new();

	let mut metadata = HashMap::<String, String>::new();
	let mut id3 = Vec::new();

	let mut fourcc = [0; 4];

	while let (Ok(()), Ok(size)) = (
		data.read_exact(&mut fourcc),
		data.read_u32::<LittleEndian>(),
	) {
		match &fourcc {
			b"fmt " => {
				if fmt.is_empty() {
					let mut value = vec![0; size as usize];
					data.read_exact(&mut value)?;

					fmt = value;
					continue;
				}

				data.seek(SeekFrom::Current(i64::from(size)))?;
			},
			b"fact" => {
				if total_samples == 0 {
					total_samples = data.read_u32::<LittleEndian>()?;
					continue;
				}

				data.seek(SeekFrom::Current(4))?;
			},
			b"data" => {
				if stream_len == 0 {
					stream_len += size
				}

				data.seek(SeekFrom::Current(i64::from(size)))?;
			},
			b"LIST" => {
				let mut list_type = [0; 4];
				data.read_exact(&mut list_type)?;

				if &list_type == b"INFO" {
					let end = data.seek(SeekFrom::Current(0))? + u64::from(size - 4);

					while data.seek(SeekFrom::Current(0))? != end {
						let mut fourcc = vec![0; 4];
						data.read_exact(&mut fourcc)?;

						let key = String::from_utf8(fourcc)?;
						let size = data.read_u32::<LittleEndian>()?;

						let mut buf = vec![0; size as usize];
						data.read_exact(&mut buf)?;

						let val = String::from_utf8(buf)?;
						metadata.insert(key.to_string(), val.trim_matches('\0').to_string());

						if data.read_u8()? != 0 {
							data.seek(SeekFrom::Current(-1))?;
						}
					}
				}
			},
			b"ID3 " | b"id3 " => {
				let mut value = vec![0; size as usize];
				data.read_exact(&mut value)?;

				id3 = value
			},
			_ => {
				data.seek(SeekFrom::Current(i64::from(size)))?;
			},
		}
	}

	if fmt.len() < 16 {
		return Err(LoftyError::Riff(
			"File does not contain a valid \"fmt \" chunk",
		));
	}

	if stream_len == 0 {
		return Err(LoftyError::Riff("File does not contain a \"data\" chunk"));
	}

	let properties = read_properties(&mut &*fmt, total_samples, stream_len)?;

	let metadata = IffData {
		properties,
		metadata,
		id3: (!id3.is_empty()).then(|| id3),
	};

	Ok(metadata)
}

fn find_info_list<T>(data: &mut T) -> Result<()>
where
	T: Read + Seek,
{
	loop {
		let mut chunk_name = [0; 4];
		data.read_exact(&mut chunk_name)?;

		if &chunk_name == b"LIST" {
			data.seek(SeekFrom::Current(4))?;

			let mut list_type = [0; 4];
			data.read_exact(&mut list_type)?;

			if &list_type == b"INFO" {
				data.seek(SeekFrom::Current(-8))?;
				return Ok(());
			}

			data.seek(SeekFrom::Current(-8))?;
		}

		let size = data.read_u32::<LittleEndian>()?;
		data.seek(SeekFrom::Current(i64::from(size)))?;
	}
}

cfg_if::cfg_if! {
	if #[cfg(feature = "format-riff")] {
		pub(crate) fn write_to(data: &mut File, metadata: HashMap<String, String>) -> Result<()> {
			let mut packet = Vec::new();

			packet.extend(b"LIST".iter());
			packet.extend(b"INFO".iter());

			for (k, v) in metadata {
				let mut val = v.as_bytes().to_vec();

				if val.len() % 2 != 0 {
					val.push(0)
				}

				let size = val.len() as u32;

				packet.extend(k.as_bytes().iter());
				packet.extend(size.to_le_bytes().iter());
				packet.extend(val.iter());
			}

			let packet_size = packet.len() - 4;

			if packet_size > u32::MAX as usize {
				return Err(LoftyError::TooMuchData);
			}

			let size = (packet_size as u32).to_le_bytes();

			#[allow(clippy::needless_range_loop)]
			for i in 0..4 {
				packet.insert(i + 4, size[i]);
			}

			verify_riff(data)?;

			data.seek(SeekFrom::Current(8))?;

			find_info_list(data)?;

			let info_list_size = data.read_u32::<LittleEndian>()? as usize;
			data.seek(SeekFrom::Current(-8))?;

			let info_list_start = data.seek(SeekFrom::Current(0))? as usize;
			let info_list_end = info_list_start + 8 + info_list_size;

			data.seek(SeekFrom::Start(0))?;
			let mut file_bytes = Vec::new();
			data.read_to_end(&mut file_bytes)?;

			let _ = file_bytes.splice(info_list_start..info_list_end, packet);

			let total_size = (file_bytes.len() - 8) as u32;
			let _ = file_bytes.splice(4..8, total_size.to_le_bytes().to_vec());

			data.seek(SeekFrom::Start(0))?;
			data.set_len(0)?;
			data.write_all(&*file_bytes)?;

			Ok(())
		}
	}
}
