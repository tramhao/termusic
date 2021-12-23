use super::{page_from_packet, verify_signature};
use crate::error::{LoftyError, Result};
use crate::ogg::constants::OPUSTAGS;
use crate::ogg::constants::VORBIS_COMMENT_HEAD;
use crate::ogg::tag::VorbisCommentsRef;
use crate::types::picture::PictureInformation;
use crate::types::tag::{Tag, TagType};

use std::convert::TryFrom;
use std::fs::File;
use std::io::{Cursor, Read, Seek, SeekFrom, Write};

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use ogg_pager::Page;

pub(in crate) fn write_to(data: &mut File, tag: &Tag, sig: &[u8]) -> Result<()> {
	match tag.tag_type() {
		#[cfg(feature = "vorbis_comments")]
		TagType::VorbisComments => write(data, &mut Into::<VorbisCommentsRef>::into(tag), sig),
		_ => Err(LoftyError::UnsupportedTag),
	}
}

#[cfg(feature = "vorbis_comments")]
pub(crate) fn create_comments(
	packet: &mut impl Write,
	count: &mut u32,
	items: &mut dyn Iterator<Item = (&str, &String)>,
) -> Result<()> {
	for (k, v) in items {
		let comment = format!("{}={}", k, v);

		let comment_b = comment.as_bytes();
		let bytes_len = comment_b.len();

		if u32::try_from(bytes_len as u64).is_ok() {
			*count += 1;

			packet.write_all(&(bytes_len as u32).to_le_bytes())?;
			packet.write_all(comment_b)?;
		}
	}

	Ok(())
}

#[cfg(feature = "vorbis_comments")]
fn create_pages(tag: &mut VorbisCommentsRef, writer: &mut Cursor<Vec<u8>>) -> Result<Vec<Page>> {
	const PICTURE_KEY: &str = "METADATA_BLOCK_PICTURE=";

	let item_count_pos = writer.seek(SeekFrom::Current(0))?;

	writer.write_u32::<LittleEndian>(0)?;

	let mut count = 0;
	create_comments(writer, &mut count, &mut tag.items)?;

	for (pic, _) in &mut tag.pictures {
		let picture = pic.as_flac_bytes(PictureInformation::from_picture(pic)?, true);

		let bytes_len = picture.len() + PICTURE_KEY.len();

		if u32::try_from(bytes_len as u64).is_ok() {
			count += 1;

			writer.write_u32::<LittleEndian>(bytes_len as u32)?;
			writer.write_all(PICTURE_KEY.as_bytes())?;
			writer.write_all(&*picture)?;
		}
	}

	let packet_end = writer.seek(SeekFrom::Current(0))?;

	writer.seek(SeekFrom::Start(item_count_pos))?;
	writer.write_u32::<LittleEndian>(count)?;
	writer.seek(SeekFrom::Start(packet_end))?;

	page_from_packet(writer.get_mut())
}

#[cfg(feature = "vorbis_comments")]
pub(super) fn write(data: &mut File, tag: &mut VorbisCommentsRef, sig: &[u8]) -> Result<()> {
	let first_page = Page::read(data, false)?;

	let ser = first_page.serial;

	let mut writer = Vec::new();
	writer.write_all(&*first_page.as_bytes())?;

	let first_md_page = Page::read(data, false)?;
	verify_signature(&first_md_page, sig)?;

	// Retain the file's vendor string
	let md_reader = &mut &first_md_page.content[sig.len()..];

	let vendor_len = md_reader.read_u32::<LittleEndian>()?;
	let mut vendor = vec![0; vendor_len as usize];
	md_reader.read_exact(&mut vendor)?;

	let mut packet = Cursor::new(Vec::new());

	packet.write_all(sig)?;
	packet.write_u32::<LittleEndian>(vendor_len)?;
	packet.write_all(&vendor)?;

	let mut pages = create_pages(tag, &mut packet)?;

	match sig {
		VORBIS_COMMENT_HEAD => {
			super::vorbis::write::write_to(
				data,
				&mut writer,
				first_md_page.content,
				ser,
				&mut pages,
			)?;
		},
		OPUSTAGS => {
			super::opus::write::write_to(data, &mut writer, ser, &mut pages)?;
		},
		_ => unreachable!(),
	}

	data.seek(SeekFrom::Start(0))?;
	data.set_len(first_page.end as u64)?;
	data.write_all(&*writer)?;

	Ok(())
}
