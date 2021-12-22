use super::RiffInfoListRef;
use crate::error::{LoftyError, Result};
use crate::iff::chunk::Chunks;
use crate::iff::wav::read::verify_wav;

use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Write};

use byteorder::{LittleEndian, WriteBytesExt};

pub(in crate::iff::wav) fn write_riff_info(
	data: &mut File,
	tag: &mut RiffInfoListRef,
) -> Result<()> {
	verify_wav(data)?;

	let mut riff_info_bytes = Vec::new();
	create_riff_info(&mut tag.items, &mut riff_info_bytes)?;

	let (info_list, info_list_size) = find_info_list(data)?;

	if info_list {
		let info_list_start = data.seek(SeekFrom::Current(-12))? as usize;
		let info_list_end = info_list_start + 8 + info_list_size as usize;

		data.seek(SeekFrom::Start(0))?;

		let mut file_bytes = Vec::new();
		data.read_to_end(&mut file_bytes)?;

		let _ = file_bytes.splice(info_list_start..info_list_end, riff_info_bytes);

		let total_size = (file_bytes.len() - 8) as u32;
		let _ = file_bytes.splice(4..8, total_size.to_le_bytes());

		data.seek(SeekFrom::Start(0))?;
		data.set_len(0)?;
		data.write_all(&*file_bytes)?;
	} else {
		data.seek(SeekFrom::End(0))?;

		data.write_all(&riff_info_bytes)?;

		let len = (data.seek(SeekFrom::Current(0))? - 8) as u32;

		data.seek(SeekFrom::Start(4))?;
		data.write_u32::<LittleEndian>(len)?;
	}

	Ok(())
}

fn find_info_list<R>(data: &mut R) -> Result<(bool, u32)>
where
	R: Read + Seek,
{
	let mut info = (false, 0);

	let mut chunks = Chunks::<LittleEndian>::new();

	while chunks.next(data).is_ok() {
		if &chunks.fourcc == b"LIST" {
			let mut list_type = [0; 4];
			data.read_exact(&mut list_type)?;

			if &list_type == b"INFO" {
				info = (true, chunks.size);
				break;
			}

			data.seek(SeekFrom::Current(-8))?;
		}

		data.seek(SeekFrom::Current(i64::from(chunks.size)))?;

		chunks.correct_position(data)?;
	}

	Ok(info)
}

fn create_riff_info(
	items: &mut dyn Iterator<Item = (&str, &String)>,
	bytes: &mut Vec<u8>,
) -> Result<()> {
	let mut items = items.peekable();

	if items.peek().is_none() {
		return Ok(());
	}

	bytes.extend(b"LIST".iter());
	bytes.extend(b"INFO".iter());

	for (k, v) in items {
		if v.is_empty() {
			continue;
		}

		let val_b = v.as_bytes();
		// Account for null terminator
		let len = val_b.len() + 1;

		// Each value has to be null terminated and have an even length
		let terminator: &[u8] = if len % 2 == 0 { &[0] } else { &[0, 0] };

		bytes.extend(k.as_bytes().iter());
		bytes.extend((len as u32).to_le_bytes().iter());
		bytes.extend(val_b.iter());
		bytes.extend(terminator.iter());
	}

	let packet_size = bytes.len() - 4;

	if packet_size > u32::MAX as usize {
		return Err(LoftyError::TooMuchData);
	}

	let size = (packet_size as u32).to_le_bytes();

	#[allow(clippy::needless_range_loop)]
	for i in 0..4 {
		bytes.insert(i + 4, size[i]);
	}

	Ok(())
}
