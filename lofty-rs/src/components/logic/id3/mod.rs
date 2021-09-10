use crate::{LoftyError, Result};

use std::io::{Read, Seek, SeekFrom};
use std::ops::Neg;

use byteorder::{BigEndian, ByteOrder};

// https://github.com/polyfloyd/rust-id3/blob/e142ec656bf70a8153f6e5b34a37f26df144c3c1/src/stream/unsynch.rs#L18-L20
pub(crate) fn decode_u32(n: u32) -> u32 {
	n & 0xFF | (n & 0xFF00) >> 1 | (n & 0xFF_0000) >> 2 | (n & 0xFF00_0000) >> 3
}

pub(crate) fn find_id3v2<R>(data: &mut R, read: bool) -> Result<Option<Vec<u8>>>
where
	R: Read + Seek,
{
	let mut id3v2 = None;

	let mut id3_header = [0; 10];
	data.read_exact(&mut id3_header)?;

	data.seek(SeekFrom::Current(-10))?;

	if &id3_header[..4] == b"ID3 " {
		let size = decode_u32(BigEndian::read_u32(&id3_header[6..]));

		if read {
			let mut tag = vec![0; size as usize];
			data.read_exact(&mut tag)?;

			id3v2 = Some(tag)
		} else {
			data.seek(SeekFrom::Current(i64::from(size)))?;
		}
	}

	Ok(id3v2)
}

pub(crate) fn find_id3v1<R>(data: &mut R, read: bool) -> Result<(bool, Option<[u8; 128]>)>
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

			id3v1 = Some(id3v1_tag)
		}
	} else {
		// No ID3v1 tag found
		data.seek(SeekFrom::End(0))?;
	}

	Ok((exists, id3v1))
}

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
