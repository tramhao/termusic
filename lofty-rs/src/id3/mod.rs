//! ID3 specific items
//!
//! ID3 does things differently than other tags, making working with them a little more effort than other formats.
//! Check the other modules for important notes and/or warnings.

#[cfg(feature = "id3v1")]
pub mod v1;
pub mod v2;

use crate::error::{LoftyError, Result};
use v2::{read_id3v2_header, Id3v2Header};

use std::io::{Read, Seek, SeekFrom};
use std::ops::Neg;

pub(crate) fn find_lyrics3v2<R>(data: &mut R) -> Result<(bool, u32)>
where
	R: Read + Seek,
{
	let mut exists = false;
	let mut size = 0_u32;

	data.seek(SeekFrom::Current(-15))?;

	let mut lyrics3v2 = [0; 15];
	data.read_exact(&mut lyrics3v2)?;

	if &lyrics3v2[7..] == b"LYRICS200" {
		exists = true;

		let lyrics_size = String::from_utf8(lyrics3v2[..7].to_vec())?;
		let lyrics_size = lyrics_size
			.parse::<u32>()
			.map_err(|_| LoftyError::Ape("Lyrics3v2 tag has an invalid size string"))?;

		size += lyrics_size;

		data.seek(SeekFrom::Current(i64::from(lyrics_size + 15).neg()))?;
	}

	Ok((exists, size))
}

#[cfg(feature = "id3v1")]
pub(crate) fn find_id3v1<R>(data: &mut R, read: bool) -> Result<(bool, Option<v1::tag::Id3v1Tag>)>
where
	R: Read + Seek,
{
	let mut id3v1 = None;
	let mut exists = false;

	data.seek(SeekFrom::End(-128))?;

	let mut id3v1_header = [0; 3];
	data.read_exact(&mut id3v1_header)?;

	data.seek(SeekFrom::Current(-3))?;

	if &id3v1_header == b"TAG" {
		exists = true;

		if read {
			let mut id3v1_tag = [0; 128];
			data.read_exact(&mut id3v1_tag)?;

			data.seek(SeekFrom::End(-128))?;

			id3v1 = Some(v1::read::parse_id3v1(id3v1_tag))
		}
	} else {
		// No ID3v1 tag found
		data.seek(SeekFrom::End(0))?;
	}

	Ok((exists, id3v1))
}

#[cfg(not(feature = "id3v1"))]
pub(in crate::tag_utils) fn find_id3v1<R>(data: &mut R, _read: bool) -> Result<(bool, Option<()>)>
where
	R: Read + Seek,
{
	let mut exists = false;

	data.seek(SeekFrom::End(-128))?;

	let mut id3v1_header = [0; 3];
	data.read_exact(&mut id3v1_header)?;

	data.seek(SeekFrom::Current(-3))?;

	if &id3v1_header == b"TAG" {
		exists = true;
	} else {
		// No ID3v1 tag found
		data.seek(SeekFrom::End(0))?;
	}

	Ok((exists, None))
}

#[cfg(feature = "id3v2")]
pub(crate) fn find_id3v2<R>(
	data: &mut R,
	read: bool,
) -> Result<(Option<Id3v2Header>, Option<Vec<u8>>)>
where
	R: Read + Seek,
{
	let mut header = None;
	let mut id3v2 = None;

	if let Ok(id3v2_header) = read_id3v2_header(data) {
		if read {
			let mut tag = vec![0; id3v2_header.size as usize];
			data.read_exact(&mut tag)?;

			id3v2 = Some(tag)
		} else {
			data.seek(SeekFrom::Current(i64::from(id3v2_header.size)))?;
		}

		if id3v2_header.flags.footer {
			data.seek(SeekFrom::Current(10))?;
		}

		header = Some(id3v2_header);
	} else {
		data.seek(SeekFrom::Current(-10))?;
	}

	Ok((header, id3v2))
}

#[cfg(not(feature = "id3v2"))]
pub(crate) fn find_id3v2<R>(data: &mut R, _read: bool) -> Result<(Option<Id3v2Header>, Option<()>)>
where
	R: Read + Seek,
{
	if let Ok(id3v2_header) = read_id3v2_header(data) {
		data.seek(SeekFrom::Current(id3v2_header.size as i64))?;

		if id3v2_header.flags.footer {
			data.seek(SeekFrom::Current(10))?;
		}

		Ok((Some(id3v2_header), Some(())))
	} else {
		data.seek(SeekFrom::Current(-10))?;
		Ok((None, None))
	}
}
