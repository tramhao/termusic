use crate::components::logic::iff::IffData;
use crate::{FileProperties, LoftyError, Result};

use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Write};
use std::time::Duration;

use byteorder::{BigEndian, LittleEndian, ReadBytesExt};

fn verify_aiff<R>(data: &mut R) -> Result<()>
where
	R: Read + Seek,
{
	let mut id = [0; 12];
	data.read_exact(&mut id)?;

	if !(&id[..4] == b"FORM" && (&id[8..] == b"AIFF" || &id[..8] == b"AIFC")) {
		return Err(LoftyError::UnknownFormat);
	}

	Ok(())
}

pub(crate) fn read_properties(comm: &mut &[u8], stream_len: u32) -> Result<FileProperties> {
	let channels = comm.read_u16::<BigEndian>()? as u8;

	if channels == 0 {
		return Err(LoftyError::Aiff("File contains 0 channels"));
	}

	let sample_frames = comm.read_u32::<BigEndian>()?;
	let _sample_size = comm.read_u16::<BigEndian>()?;

	let mut sample_rate_bytes = [0; 10];
	comm.read_exact(&mut sample_rate_bytes)?;

	let sign = u64::from(sample_rate_bytes[0] & 0x80);

	sample_rate_bytes[0] &= 0x7f;

	let mut exponent = u16::from(sample_rate_bytes[0]) << 8 | u16::from(sample_rate_bytes[1]);
	exponent = exponent - 16383 + 1023;

	let fraction = &mut sample_rate_bytes[2..];
	fraction[0] &= 0x7f;

	let fraction: Vec<u64> = fraction.iter_mut().map(|v| u64::from(*v)).collect();

	let fraction = fraction[0] << 56
		| fraction[1] << 48
		| fraction[2] << 40
		| fraction[3] << 32
		| fraction[4] << 24
		| fraction[5] << 16
		| fraction[6] << 8
		| fraction[7];

	let f64_bytes = sign << 56 | u64::from(exponent) << 52 | fraction >> 11;
	let float = f64::from_be_bytes(f64_bytes.to_be_bytes());

	let sample_rate = float.round() as u32;

	let (duration, bitrate) = if sample_rate > 0 && sample_frames > 0 {
		let length = (u64::from(sample_frames) * 1000) / u64::from(sample_rate);

		(
			Duration::from_millis(length),
			(u64::from(stream_len * 8) / length) as u32,
		)
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

pub(crate) fn read_from<R>(data: &mut R) -> Result<IffData>
where
	R: Read + Seek,
{
	verify_aiff(data)?;

	let mut comm = None;
	let mut stream_len = 0;

	let mut metadata = HashMap::<String, String>::new();
	let mut id3 = Vec::new();

	let mut fourcc = [0; 4];

	while let (Ok(()), Ok(size)) = (data.read_exact(&mut fourcc), data.read_u32::<BigEndian>()) {
		match &fourcc {
			b"NAME" | b"AUTH" | b"(c) " => {
				let mut value = vec![0; size as usize];
				data.read_exact(&mut value)?;

				metadata.insert(
					String::from_utf8(fourcc.to_vec())?,
					String::from_utf8(value)?,
				);
			},
			b"ID3 " | b"id3 " => {
				let mut value = vec![0; size as usize];
				data.read_exact(&mut value)?;

				id3 = value
			},
			b"COMM" => {
				if comm.is_none() {
					if size < 18 {
						return Err(LoftyError::Aiff(
							"File has an invalid \"COMM\" chunk size (< 18)",
						));
					}

					let mut comm_data = vec![0; size as usize];
					data.read_exact(&mut comm_data)?;

					comm = Some(comm_data);
				}
			},
			b"SSND" => {
				stream_len = size;
				data.seek(SeekFrom::Current(i64::from(size)))?;
			},
			_ => {
				data.seek(SeekFrom::Current(i64::from(size)))?;
			},
		}
	}

	if comm.is_none() {
		return Err(LoftyError::Aiff("File does not contain a \"COMM\" chunk"));
	}

	if stream_len == 0 {
		return Err(LoftyError::Aiff("File does not contain a \"SSND\" chunk"));
	}

	let properties = read_properties(&mut &*comm.unwrap(), stream_len)?;

	let metadata = IffData {
		properties,
		metadata,
		id3: (!id3.is_empty()).then(|| id3),
	};

	Ok(metadata)
}

cfg_if::cfg_if! {
	if #[cfg(feature = "format-aiff")] {
		pub(crate) fn write_to(data: &mut File, metadata: &HashMap<String, String>) -> Result<()> {
			verify_aiff(data)?;

			let mut text_chunks = Vec::new();

			for (k, v) in metadata {
				let len = (v.len() as u32).to_be_bytes();

				text_chunks.extend(k.as_bytes().iter());
				text_chunks.extend(len.iter());
				text_chunks.extend(v.as_bytes().iter());
			}

			let mut chunks_remove = Vec::new();

			while let (Ok(fourcc), Ok(size)) = (
				data.read_u32::<LittleEndian>(),
				data.read_u32::<BigEndian>(),
			) {
				let fourcc_b = &fourcc.to_le_bytes();
				let pos = (data.seek(SeekFrom::Current(0))? - 8) as usize;

				if fourcc_b == b"NAME" || fourcc_b == b"AUTH" || fourcc_b == b"(c) " {
					chunks_remove.push((pos, (pos + 8 + size as usize)))
				}

				data.seek(SeekFrom::Current(i64::from(size)))?;
			}

			data.seek(SeekFrom::Start(0))?;

			let mut file_bytes = Vec::new();
			data.read_to_end(&mut file_bytes)?;

			if chunks_remove.is_empty() {
				data.seek(SeekFrom::Start(16))?;

				let mut size = [0; 4];
				data.read_exact(&mut size)?;

				let comm_end = (20 + u32::from_le_bytes(size)) as usize;
				file_bytes.splice(comm_end..comm_end, text_chunks);
			} else {
				chunks_remove.sort_unstable();
				chunks_remove.reverse();

				let first = chunks_remove.pop().unwrap();

				for (s, e) in &chunks_remove {
					file_bytes.drain(*s as usize..*e as usize);
				}

				file_bytes.splice(first.0 as usize..first.1 as usize, text_chunks);
			}

			let total_size = ((file_bytes.len() - 8) as u32).to_be_bytes();
			file_bytes.splice(4..8, total_size.to_vec());

			data.seek(SeekFrom::Start(0))?;
			data.set_len(0)?;
			data.write_all(&*file_bytes)?;

			Ok(())
		}
	}
}
