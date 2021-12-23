use super::block::Block;
use super::FlacFile;
use crate::error::{LoftyError, Result};
#[cfg(feature = "vorbis_comments")]
use crate::ogg::{read::read_comments, tag::VorbisComments};
#[cfg(feature = "vorbis_comments")]
use crate::types::picture::Picture;
use crate::types::properties::FileProperties;

use std::io::{Read, Seek, SeekFrom};

pub(super) fn verify_flac<R>(data: &mut R) -> Result<Block>
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

pub(super) fn read_from<R>(data: &mut R, read_properties: bool) -> Result<FlacFile>
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

	#[cfg(feature = "vorbis_comments")]
	let mut tag = VorbisComments {
		vendor: String::new(),
		items: vec![],
		pictures: vec![],
	};

	while !last_block {
		let block = Block::read(data)?;
		last_block = block.last;

		match block.ty {
			#[cfg(feature = "vorbis_comments")]
			4 => read_comments(&mut &*block.content, &mut tag)?,
			#[cfg(feature = "vorbis_comments")]
			6 => tag
				.pictures
				.push(Picture::from_flac_bytes(&*block.content)?),
			_ => {},
		}
	}

	let (stream_length, file_length) = {
		let current = data.seek(SeekFrom::Current(0))?;
		let end = data.seek(SeekFrom::End(0))?;

		(end - current, end)
	};

	let properties = if read_properties {
		super::properties::read_properties(&mut &*stream_info.content, stream_length, file_length)?
	} else {
		FileProperties::default()
	};

	Ok(FlacFile {
		properties,
		#[cfg(feature = "vorbis_comments")]
		vorbis_comments: (!(tag.items.is_empty() && tag.pictures.is_empty())).then(|| tag),
	})
}
