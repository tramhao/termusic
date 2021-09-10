use super::find_last_page;
use crate::components::logic::ogg::constants::VORBIS_SETUP_HEAD;
use crate::{FileProperties, LoftyError, Result};

use std::fs::File;
use std::io::{Cursor, Read, Seek, SeekFrom, Write};
use std::time::Duration;

use byteorder::{LittleEndian, ReadBytesExt};
use ogg_pager::Page;

pub(in crate::components) fn read_properties<R>(
	data: &mut R,
	first_page: &Page,
) -> Result<FileProperties>
where
	R: Read + Seek,
{
	let first_page_abgp = first_page.abgp;

	// Skip identification header and version
	let first_page_content = &mut &first_page.content[11..];

	let channels = first_page_content.read_u8()?;
	let sample_rate = first_page_content.read_u32::<LittleEndian>()?;

	let _bitrate_max = first_page_content.read_u32::<LittleEndian>()?;
	let bitrate_nominal = first_page_content.read_u32::<LittleEndian>()?;

	let last_page = find_last_page(data)?;
	let last_page_abgp = last_page.abgp;

	last_page_abgp.checked_sub(first_page_abgp).map_or_else(
		|| Err(LoftyError::Vorbis("File contains incorrect PCM values")),
		|frame_count| {
			let length = frame_count * 1000 / u64::from(sample_rate);
			let duration = Duration::from_millis(length as u64);
			let bitrate = bitrate_nominal / 1000;

			Ok(FileProperties::new(
				duration,
				Some(bitrate),
				Some(sample_rate),
				Some(channels),
			))
		},
	)
}

pub fn write_to(
	data: &mut File,
	writer: &mut Vec<u8>,
	first_md_content: Vec<u8>,
	ser: u32,
	pages: &mut [Page],
) -> Result<()> {
	let mut remaining = Vec::new();

	let reached_md_end: bool;

	// Find the total comment count in the first page's content
	let mut c = Cursor::new(first_md_content);

	// Skip the header
	c.seek(SeekFrom::Start(7))?;

	// Skip the vendor
	let vendor_len = c.read_u32::<LittleEndian>()?;
	c.seek(SeekFrom::Current(i64::from(vendor_len)))?;

	let total_comments = c.read_u32::<LittleEndian>()?;
	let comments_pos = c.seek(SeekFrom::Current(0))?;

	c.seek(SeekFrom::End(0))?;

	loop {
		let p = Page::read(data, false)?;

		if p.header_type != 1 {
			data.seek(SeekFrom::Start(p.start as u64))?;
			data.read_to_end(&mut remaining)?;

			reached_md_end = true;
			break;
		}

		c.write_all(&p.content)?;
	}

	if !reached_md_end {
		return Err(LoftyError::Vorbis("File ends with comment header"));
	}

	c.seek(SeekFrom::Start(comments_pos))?;

	for _ in 0..total_comments {
		let len = c.read_u32::<LittleEndian>()?;
		c.seek(SeekFrom::Current(i64::from(len)))?;
	}

	if c.read_u8()? != 1 {
		return Err(LoftyError::Vorbis("File is missing a framing bit"));
	}

	// Comments should be followed by the setup header
	let mut header_ident = [0; 7];
	c.read_exact(&mut header_ident)?;

	if header_ident != VORBIS_SETUP_HEAD {
		return Err(LoftyError::Vorbis("File is missing setup header"));
	}

	c.seek(SeekFrom::Current(-7))?;

	let mut setup = Vec::new();
	c.read_to_end(&mut setup)?;

	let pages_len = pages.len() - 1;

	for (i, mut p) in pages.iter_mut().enumerate() {
		p.serial = ser;

		if i == pages_len {
			// Add back the framing bit
			p.content.push(1);

			// The segment tables of current page and the setup header have to be combined
			let mut seg_table = Vec::new();
			seg_table.extend(p.segments().iter());
			seg_table.extend(ogg_pager::segments(&*setup));

			let mut seg_table_len = seg_table.len();

			if seg_table_len > 255 {
				seg_table = seg_table.split_at(255).0.to_vec();
				seg_table_len = 255;
			}

			seg_table.insert(0, seg_table_len as u8);

			let page = p.extend(&*setup);

			let mut p_bytes = p.as_bytes();
			let seg_count = p_bytes[26] as usize;

			// Replace segment table and checksum
			p_bytes.splice(26..27 + seg_count, seg_table);
			p_bytes.splice(22..26, ogg_pager::crc32(&*p_bytes).to_le_bytes().to_vec());

			writer.write_all(&*p_bytes)?;

			if let Some(mut page) = page {
				page.serial = ser;
				page.gen_crc();

				writer.write_all(&*page.as_bytes())?;
			}

			break;
		}

		p.gen_crc();
		writer.write_all(&*p.as_bytes())?;
	}

	writer.write_all(&*remaining)?;

	Ok(())
}
