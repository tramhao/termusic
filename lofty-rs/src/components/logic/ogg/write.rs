use super::{opus, page_from_packet, verify_signature, vorbis};
#[cfg(feature = "format-opus")]
use crate::components::logic::ogg::constants::OPUSTAGS;
#[cfg(feature = "format-vorbis")]
use crate::components::logic::ogg::constants::VORBIS_COMMENT_HEAD;
use crate::{Picture, Result};

use std::borrow::Cow;
use std::collections::HashMap;
use std::fs::File;
use std::io::{Seek, SeekFrom, Write};

use ogg_pager::Page;
use unicase::UniCase;

pub(crate) fn create_comments(packet: &mut Vec<u8>, comments: &HashMap<UniCase<String>, String>) {
	for (a, b) in comments {
		let comment = format!("{}={}", a, b);
		let comment_b = comment.as_bytes();
		packet.extend((comment_b.len() as u32).to_le_bytes().iter());
		packet.extend(comment_b.iter());
	}
}

pub(crate) fn create_pages(
	file: &mut File,
	sig: &[u8],
	vendor: &str,
	comments: &HashMap<UniCase<String>, String>,
	pictures: &Option<Cow<'static, [Picture]>>,
) -> Result<()> {
	let mut packet = Vec::new();

	packet.extend(sig.iter());
	packet.extend((vendor.len() as u32).to_le_bytes().iter());
	packet.extend(vendor.as_bytes().iter());

	let comments_len = pictures.as_ref().map_or_else(
		|| comments.len() as u32,
		|pictures| (comments.len() + pictures.len()) as u32,
	);

	packet.extend(comments_len.to_le_bytes().iter());
	create_comments(&mut packet, comments);

	if let Some(pics) = pictures {
		for pic in pics.iter() {
			let comment = format!(
				"METADATA_BLOCK_PICTURE={}",
				base64::encode(pic.as_apic_bytes())
			);
			let comment_b = comment.as_bytes();
			packet.extend((comment_b.len() as u32).to_le_bytes().iter());
			packet.extend(comment_b.iter());
		}
	}

	let mut pages = page_from_packet(&mut *packet)?;
	write_to(file, &mut pages, sig)?;

	Ok(())
}

fn write_to(mut data: &mut File, pages: &mut [Page], sig: &[u8]) -> Result<()> {
	let first_page = Page::read(&mut data, false)?;

	let ser = first_page.serial;

	let mut writer = Vec::new();
	writer.write_all(&*first_page.as_bytes())?;

	let first_md_page = Page::read(&mut data, false)?;
	verify_signature(&first_md_page, sig)?;

	#[cfg(feature = "format-vorbis")]
	if sig == VORBIS_COMMENT_HEAD {
		vorbis::write_to(data, &mut writer, first_md_page.content, ser, pages)?;
	}

	#[cfg(feature = "format-opus")]
	if sig == OPUSTAGS {
		opus::write_to(data, &mut writer, ser, pages)?;
	}

	data.seek(SeekFrom::Start(0))?;
	data.set_len(first_page.end as u64)?;
	data.write_all(&*writer)?;

	Ok(())
}
