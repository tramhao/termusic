use super::read::{read_comments, OGGTags};
use super::write::create_comments;
use crate::{FileProperties, LoftyError, OggFormat, Picture, Result};

use std::borrow::Cow;
use std::collections::HashMap;
use std::fs::File;
use std::io::{Cursor, Read, Seek, SeekFrom, Write};
use std::time::Duration;

use byteorder::{BigEndian, LittleEndian, ReadBytesExt, WriteBytesExt};
use unicase::UniCase;

struct Block {
	byte: u8,
	ty: u8,
	last: bool,
	content: Vec<u8>,
	start: u64,
	end: u64,
}

impl Block {
	fn read<R>(data: &mut R) -> Result<Self>
	where
		R: Read + Seek,
	{
		let start = data.seek(SeekFrom::Current(0))?;

		let byte = data.read_u8()?;
		let last = (byte & 0x80) != 0;
		let ty = byte & 0x7f;

		let size = data.read_uint::<BigEndian>(3)? as u32;

		let mut content = vec![0; size as usize];
		data.read_exact(&mut content)?;

		let end = data.seek(SeekFrom::Current(0))?;

		Ok(Self {
			byte,
			ty,
			last,
			content,
			start,
			end,
		})
	}
}

fn verify_flac<R>(data: &mut R) -> Result<Block>
where
	R: Read + Seek,
{
	let mut marker = [0; 4];
	data.read_exact(&mut marker)?;

	if &marker != b"fLaC" {
		return Err(LoftyError::Flac("File missing \"fLaC\" stream marker"));
	}

	let block = Block::read(data)?;

	if block.ty != 0 {
		return Err(LoftyError::Flac("File missing mandatory STREAMINFO block"));
	}

	Ok(block)
}

fn read_properties<R>(stream_info: &mut R, stream_length: u64) -> Result<FileProperties>
where
	R: Read,
{
	// Skip 4 bytes
	// Minimum block size (2)
	// Maximum block size (2)
	stream_info.read_u32::<BigEndian>()?;

	// Skip 6 bytes
	// Minimum frame size (3)
	// Maximum frame size (3)
	stream_info.read_uint::<BigEndian>(6)?;

	// Read 4 bytes
	// Sample rate (20 bits)
	// Number of channels (3 bits)
	// Bits per sample (5 bits)
	// Total samples (first 4 bits)
	let info = stream_info.read_u32::<BigEndian>()?;

	let sample_rate = info >> 12;
	let channels = ((info >> 9) & 7) + 1;

	// Read the remaining 32 bits of the total samples
	let total_samples = stream_info.read_u32::<BigEndian>()? | (info << 28);

	let (duration, bitrate) = if sample_rate > 0 && total_samples > 0 {
		let length = (u64::from(total_samples) * 1000) / u64::from(sample_rate);

		(
			Duration::from_millis(length),
			((stream_length * 8) / length) as u32,
		)
	} else {
		(Duration::ZERO, 0)
	};

	Ok(FileProperties::new(
		duration,
		Some(bitrate),
		Some(sample_rate as u32),
		Some(channels as u8),
	))
}

pub(crate) fn read_from<R>(data: &mut R) -> Result<OGGTags>
where
	R: Read + Seek,
{
	let stream_info = verify_flac(data)?;
	let stream_info_len = (stream_info.end - stream_info.start) as u32;

	if stream_info_len < 18 {
		return Err(LoftyError::Flac(
			"File has an invalid STREAMINFO block size (< 18)",
		));
	}

	let mut last_block = stream_info.last;

	let mut vendor = String::new();
	let mut comments = HashMap::<UniCase<String>, String>::new();
	let mut pictures = Vec::<Picture>::new();

	while !last_block {
		let block = Block::read(data)?;
		last_block = block.last;

		match block.ty {
			4 => vendor = read_comments(&mut &*block.content, &mut comments, &mut pictures)?,
			6 => pictures.push(Picture::from_apic_bytes(&*block.content)?),
			_ => {},
		}
	}

	let stream_length = {
		let current = data.seek(SeekFrom::Current(0))?;
		let end = data.seek(SeekFrom::End(0))?;
		end - current
	};

	let properties = read_properties(&mut &*stream_info.content, stream_length)?;

	Ok((vendor, pictures, comments, properties, OggFormat::Flac))
}

pub(crate) fn write_to(
	data: &mut File,
	vendor: &str,
	comments: &HashMap<UniCase<String>, String>,
	pictures: &Option<Cow<'static, [Picture]>>,
) -> Result<()> {
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
			_ => {},
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

	let mut comment_blocks = Vec::new();
	create_comment_block(&mut comment_blocks, vendor, comments)?;
	create_picture_blocks(&mut comment_blocks, pictures)?;

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
	writer: &mut Vec<u8>,
	vendor: &str,
	comments: &HashMap<UniCase<String>, String>,
) -> Result<()> {
	if !comments.is_empty() {
		let mut byte = 0_u8;
		byte |= 4 & 0x7f;

		writer.write_u8(byte)?;
		writer.write_u32::<LittleEndian>(vendor.len() as u32)?;
		writer.write_all(vendor.as_bytes())?;
		writer.write_u32::<LittleEndian>(comments.len() as u32)?;

		create_comments(writer, comments);

		let len = (writer.len() - 1) as u32;

		if len > 65535 {
			return Err(LoftyError::TooMuchData);
		}

		writer.splice(1..1, len.to_be_bytes()[1..].to_vec());
	}

	Ok(())
}

fn create_picture_blocks(
	writer: &mut Vec<u8>,
	pictures: &Option<Cow<'static, [Picture]>>,
) -> Result<()> {
	let mut byte = 0_u8;
	byte |= 6 & 0x7f;

	if let Some(pictures) = pictures {
		for pic in pictures.iter() {
			writer.write_u8(byte)?;

			let pic_bytes = pic.as_apic_bytes();
			let pic_len = pic_bytes.len() as u32;

			if pic_len > 65535 {
				return Err(LoftyError::TooMuchData);
			}

			writer.write_all(&pic_len.to_be_bytes()[1..])?;
			writer.write_all(&*pic_bytes)?;
		}
	}

	Ok(())
}
