use super::block::Block;
use super::read::verify_flac;
use crate::error::{LoftyError, Result};
use crate::ogg::tag::VorbisCommentsRef;
use crate::ogg::write::create_comments;
use crate::types::picture::{Picture, PictureInformation};

use std::fs::File;
use std::io::{Cursor, Read, Seek, SeekFrom, Write};

use byteorder::{LittleEndian, WriteBytesExt};

pub(in crate) fn write_to(data: &mut File, tag: &mut VorbisCommentsRef) -> Result<()> {
	let stream_info = verify_flac(data)?;
	let stream_info_end = stream_info.end as usize;

	let mut last_block = stream_info.last;

	let mut file_bytes = Vec::new();
	data.read_to_end(&mut file_bytes)?;

	let mut cursor = Cursor::new(file_bytes);

	let mut padding = false;
	let mut last_block_info = (
		stream_info.byte,
		stream_info_end - ((stream_info.end - stream_info.start) as u32 + 4) as usize,
		stream_info_end,
	);

	let mut blocks_remove = Vec::new();

	while !last_block {
		let block = Block::read(&mut cursor)?;
		let start = block.start;
		let end = block.end;

		let block_type = block.ty;
		last_block = block.last;

		if last_block {
			last_block_info = (block.byte, (end - start) as usize, end as usize)
		}

		match block_type {
			4 | 6 => blocks_remove.push((start, end)),
			1 => padding = true,
			_ => {}
		}
	}

	let mut file_bytes = cursor.into_inner();

	if !padding {
		let mut first_byte = 0_u8;
		first_byte |= last_block_info.0 & 0x7f;

		file_bytes[last_block_info.1 as usize] = first_byte;

		let mut padding_block = [0; 1028];
		let mut padding_byte = 0;
		padding_byte |= 0x80;
		padding_byte |= 1 & 0x7f;

		padding_block[0] = padding_byte;

		// [0, 4, 0] = 1024
		padding_block[2] = 4;

		file_bytes.splice(
			last_block_info.2 as usize..last_block_info.2 as usize,
			padding_block,
		);
	}

	let mut comment_blocks = Cursor::new(Vec::new());

	create_comment_block(&mut comment_blocks, tag.vendor, &mut tag.items)?;

	let mut comment_blocks = comment_blocks.into_inner();

	create_picture_blocks(&mut comment_blocks, &mut tag.pictures)?;

	if blocks_remove.is_empty() {
		file_bytes.splice(0..0, comment_blocks);
	} else {
		blocks_remove.sort_unstable();
		blocks_remove.reverse();

		let first = blocks_remove.pop().unwrap();

		for (s, e) in &blocks_remove {
			file_bytes.drain(*s as usize..*e as usize);
		}

		file_bytes.splice(first.0 as usize..first.1 as usize, comment_blocks);
	}

	data.seek(SeekFrom::Start(stream_info_end as u64))?;
	data.set_len(stream_info_end as u64)?;
	data.write_all(&*file_bytes)?;

	Ok(())
}

fn create_comment_block(
	writer: &mut Cursor<Vec<u8>>,
	vendor: &str,
	items: &mut dyn Iterator<Item = (&str, &String)>,
) -> Result<()> {
	let mut peek = items.peekable();

	if peek.peek().is_some() {
		let mut byte = 0_u8;
		byte |= 4 & 0x7f;

		writer.write_u8(byte)?;
		writer.write_u32::<LittleEndian>(vendor.len() as u32)?;
		writer.write_all(vendor.as_bytes())?;

		let item_count_pos = writer.seek(SeekFrom::Current(0))?;
		let mut count = 0;

		writer.write_u32::<LittleEndian>(count)?;

		create_comments(writer, &mut count, items)?;

		let len = (writer.get_ref().len() - 1) as u32;

		if len > 65535 {
			return Err(LoftyError::TooMuchData);
		}

		let comment_end = writer.seek(SeekFrom::Current(0))?;

		writer.seek(SeekFrom::Start(item_count_pos))?;
		writer.write_u32::<LittleEndian>(count)?;

		writer.seek(SeekFrom::Start(comment_end))?;
		writer
			.get_mut()
			.splice(1..1, len.to_be_bytes()[1..].to_vec());
	}

	Ok(())
}

fn create_picture_blocks(
	writer: &mut Vec<u8>,
	pictures: &mut dyn Iterator<Item = (&Picture, PictureInformation)>,
) -> Result<()> {
	let mut byte = 0_u8;
	byte |= 6 & 0x7f;

	for (pic, info) in pictures {
		writer.write_u8(byte)?;

		let pic_bytes = pic.as_flac_bytes(info, false);
		let pic_len = pic_bytes.len() as u32;

		if pic_len > 65535 {
			return Err(LoftyError::TooMuchData);
		}

		writer.write_all(&pic_len.to_be_bytes()[1..])?;
		writer.write_all(pic_bytes.as_slice())?;
	}

	Ok(())
}
