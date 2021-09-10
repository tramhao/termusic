use super::tag::read_ape_tag;
use super::ItemType;
use crate::components::logic::ape::constants::APE_PREAMBLE;
use crate::components::logic::id3::{find_id3v1, find_id3v2, find_lyrics3v2};
use crate::{LoftyError, Result};

use std::collections::HashMap;
use std::fs::File;
use std::io::{Cursor, Read, Seek, SeekFrom, Write};

use byteorder::{LittleEndian, WriteBytesExt};
use unicase::UniCase;

pub(crate) fn write_to(
	data: &mut File,
	metadata: &HashMap<UniCase<String>, ItemType>,
) -> Result<()> {
	// We don't actually need the ID3v2 tag, but reading it will seek to the end of it if it exists
	find_id3v2(data, false)?;

	let mut ape_preamble = [0; 8];
	data.read_exact(&mut ape_preamble)?;

	// We have to check the APE tag for any read only items first
	let mut read_only_metadata = HashMap::<UniCase<String>, ItemType>::new();

	// An APE tag in the beginning of a file is against the spec
	// If one is found, it'll be removed and rewritten at the bottom, where it should be
	let mut header_ape_tag = (false, (0, 0));

	if &ape_preamble == APE_PREAMBLE {
		let start = data.seek(SeekFrom::Current(-8))?;

		data.seek(SeekFrom::Current(8))?;
		let (mut existing_metadata, size) = read_ape_tag(data, false)?;

		// Only keep metadata around that's marked read only
		retain_read_only(&mut existing_metadata);

		read_only_metadata = existing_metadata;

		header_ape_tag = (true, (start, start + u64::from(size)))
	} else {
		data.seek(SeekFrom::Current(-8))?;
	}

	// Skip over ID3v1 and Lyrics3v2 tags
	find_id3v1(data, false)?;
	find_lyrics3v2(data)?;

	// In case there's no ape tag already, this is the spot it belongs
	let ape_position = data.seek(SeekFrom::Current(0))?;

	// Now search for an APE tag at the end
	data.seek(SeekFrom::Current(-32))?;

	data.read_exact(&mut ape_preamble)?;

	let mut ape_tag_location = None;

	// Also check this tag for any read only items
	if &ape_preamble == APE_PREAMBLE {
		let start = data.seek(SeekFrom::Current(0))? as usize + 24;

		let (mut existing_metadata, size) = read_ape_tag(data, true)?;

		retain_read_only(&mut existing_metadata);

		read_only_metadata = existing_metadata;

		// Since the "start" was really at the end of the tag, this sanity check seems necessary
		if let Some(start) = start.checked_sub(size as usize) {
			ape_tag_location = Some(start..start + size as usize);
		} else {
			return Err(LoftyError::Ape("File has a tag with an invalid size"));
		}
	}

	// Preserve any metadata marked as read only
	// If there is any read only metadata, we will have to clone the HashMap
	let tag = if read_only_metadata.is_empty() {
		create_ape_tag(metadata)?
	} else {
		let mut metadata = metadata.clone();

		for (k, v) in read_only_metadata {
			metadata.insert(k, v);
		}

		create_ape_tag(&metadata)?
	};

	data.seek(SeekFrom::Start(0))?;

	let mut file_bytes = Vec::new();
	data.read_to_end(&mut file_bytes)?;

	// Write the tag in the appropriate place
	if let Some(range) = ape_tag_location {
		file_bytes.splice(range, tag);
	} else {
		file_bytes.splice(ape_position as usize..ape_position as usize, tag);
	}

	// Now, if there was a tag at the beginning, remove it
	if header_ape_tag.0 {
		file_bytes.drain(header_ape_tag.1 .0 as usize..header_ape_tag.1 .1 as usize);
	}

	data.seek(SeekFrom::Start(0))?;
	data.set_len(0)?;
	data.write_all(&*file_bytes)?;

	Ok(())
}

fn create_ape_tag(metadata: &HashMap<UniCase<String>, ItemType>) -> Result<Vec<u8>> {
	// Unnecessary to write anything if there's no metadata
	if metadata.is_empty() {
		Ok(Vec::<u8>::new())
	} else {
		let mut tag = Cursor::new(Vec::<u8>::new());

		let item_count = metadata.len() as u32;

		for (k, v) in metadata {
			let (size, flags, value) = match v {
				ItemType::Binary(value, ro) => {
					let mut flags = 1_u32 << 1;

					if *ro {
						flags |= 1_u32
					}

					(value.len() as u32, flags, value.as_slice())
				},
				ItemType::String(value, ro) => {
					let value = value.as_bytes();

					let mut flags = 0_u32;

					if *ro {
						flags |= 1_u32
					}

					(value.len() as u32, flags, value)
				},
				ItemType::Locator(value, ro) => {
					let mut flags = 2_u32 << 1;

					if *ro {
						flags |= 1_u32
					}

					(value.len() as u32, flags, value.as_bytes())
				},
			};

			tag.write_u32::<LittleEndian>(size)?;
			tag.write_u32::<LittleEndian>(flags)?;
			tag.write_all(k.as_bytes())?;
			tag.write_u8(0)?;
			tag.write_all(value)?;
		}

		let size = tag.get_ref().len();

		if size as u64 + 32 > u64::from(u32::MAX) {
			return Err(LoftyError::TooMuchData);
		}

		let mut footer = [0_u8; 32];
		let mut footer = Cursor::new(&mut footer[..]);

		footer.write_all(APE_PREAMBLE)?;
		// This is the APE tag version
		// Even if we read a v1 tag, we end up adding a header anyway
		footer.write_u32::<LittleEndian>(2000)?;
		// The total size includes the 32 bytes of the footer
		footer.write_u32::<LittleEndian>((size + 32) as u32)?;
		footer.write_u32::<LittleEndian>(item_count)?;
		// Bit 29 unset: this is the footer
		// Bit 30 set: tag contains a footer
		// Bit 31 set: tag contains a header
		footer.write_u32::<LittleEndian>((1_u32 << 30) | (1_u32 << 31))?;
		// The header/footer must end in 8 bytes of zeros
		footer.write_u64::<LittleEndian>(0)?;

		tag.write_all(footer.get_ref())?;

		let mut tag = tag.into_inner();

		// The header is exactly the same as the footer, except for the flags
		// Just reuse the footer and overwrite the flags
		footer.seek(SeekFrom::Current(-12))?;
		// Bit 29 set: this is the header
		// Bit 30 set: tag contains a footer
		// Bit 31 set: tag contains a header
		footer.write_u32::<LittleEndian>((1_u32 << 29) | (1_u32 << 30) | (1_u32 << 31))?;

		let header = footer.into_inner();

		tag.splice(0..0, header.to_vec());

		Ok(tag)
	}
}

fn retain_read_only(existing_metadata: &mut HashMap<UniCase<String>, ItemType>) {
	existing_metadata.retain(|_, ty| *{
		match ty {
			ItemType::String(_, ro) | ItemType::Binary(_, ro) | ItemType::Locator(_, ro) => ro,
		}
	});
}
