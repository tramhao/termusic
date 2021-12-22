use crate::error::Result;
use crate::iff::chunk::Chunks;

use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Write};

use byteorder::{ByteOrder, WriteBytesExt};

pub(in crate::id3::v2) fn write_to_chunk_file<B>(data: &mut File, tag: &[u8]) -> Result<()>
where
	B: ByteOrder,
{
	data.seek(SeekFrom::Current(12))?;

	let mut id3v2_chunk = (None, None);

	let mut chunks = Chunks::<B>::new();

	while chunks.next(data).is_ok() {
		if &chunks.fourcc == b"ID3 " || &chunks.fourcc == b"id3 " {
			id3v2_chunk = (
				Some(data.seek(SeekFrom::Current(0))? - 8),
				Some(chunks.size),
			);
			break;
		}

		data.seek(SeekFrom::Current(i64::from(chunks.size)))?;

		chunks.correct_position(data)?;
	}

	if let (Some(chunk_start), Some(mut chunk_size)) = id3v2_chunk {
		data.seek(SeekFrom::Start(0))?;

		// We need to remove the padding byte if it exists
		if chunk_size % 2 != 0 {
			chunk_size += 1;
		}

		let mut file_bytes = Vec::new();
		data.read_to_end(&mut file_bytes)?;

		file_bytes.splice(
			chunk_start as usize..(chunk_start + u64::from(chunk_size) + 8) as usize,
			[],
		);

		data.seek(SeekFrom::Start(0))?;
		data.set_len(0)?;
		data.write_all(&*file_bytes)?;
	}

	if !tag.is_empty() {
		data.seek(SeekFrom::End(0))?;
		data.write_all(&[b'I', b'D', b'3', b' '])?;
		data.write_u32::<B>(tag.len() as u32)?;
		data.write_all(tag)?;

		// It is required an odd length chunk be padded with a 0
		// The 0 isn't included in the chunk size, however
		if tag.len() % 2 != 0 {
			data.write_u8(0)?;
		}

		let total_size = data.seek(SeekFrom::Current(0))? - 8;

		data.seek(SeekFrom::Start(4))?;

		data.write_u32::<B>(total_size as u32)?;
	}

	Ok(())
}
