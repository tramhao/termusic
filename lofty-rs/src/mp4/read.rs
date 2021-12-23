use super::atom_info::{AtomIdent, AtomInfo};
use super::moov::Moov;
use super::properties::Mp4Properties;
use super::Mp4File;
use crate::error::{LoftyError, Result};

use std::io::{Read, Seek, SeekFrom};

pub(in crate::mp4) fn verify_mp4<R>(data: &mut R) -> Result<String>
where
	R: Read + Seek,
{
	let atom = AtomInfo::read(data)?;

	if atom.ident != AtomIdent::Fourcc(*b"ftyp") {
		return Err(LoftyError::UnknownFormat);
	}

	let mut major_brand = vec![0; 4];
	data.read_exact(&mut major_brand)?;

	data.seek(SeekFrom::Current((atom.len - 12) as i64))?;

	String::from_utf8(major_brand)
		.map_err(|_| LoftyError::BadAtom("Unable to parse \"ftyp\"'s major brand"))
}

#[allow(clippy::similar_names)]
pub(crate) fn read_from<R>(data: &mut R, read_properties: bool) -> Result<Mp4File>
where
	R: Read + Seek,
{
	let ftyp = verify_mp4(data)?;

	Moov::find(data)?;
	let moov = Moov::parse(data, read_properties)?;

	let file_length = data.seek(SeekFrom::End(0))?;

	Ok(Mp4File {
		ftyp,
		#[cfg(feature = "mp4_ilst")]
		ilst: moov.meta,
		properties: if read_properties {
			super::properties::read_properties(data, &moov.traks, file_length)?
		} else {
			Mp4Properties::default()
		},
	})
}

pub(crate) fn skip_unneeded<R>(data: &mut R, ext: bool, len: u64) -> Result<()>
where
	R: Read + Seek,
{
	if ext {
		let pos = data.seek(SeekFrom::Current(0))?;

		if let (pos, false) = pos.overflowing_add(len - 8) {
			data.seek(SeekFrom::Start(pos))?;
		} else {
			return Err(LoftyError::TooMuchData);
		}
	} else {
		data.seek(SeekFrom::Current(i64::from(len as u32) - 8))?;
	}

	Ok(())
}

pub(crate) fn nested_atom<R>(data: &mut R, len: u64, expected: &[u8]) -> Result<Option<AtomInfo>>
where
	R: Read + Seek,
{
	let mut read = 8;
	let mut ret = None;

	while read < len {
		let atom = AtomInfo::read(data)?;

		match atom.ident {
			AtomIdent::Fourcc(ref fourcc) if fourcc == expected => {
				ret = Some(atom);
				break;
			},
			_ => {
				skip_unneeded(data, atom.extended, atom.len)?;
				read += atom.len
			},
		}
	}

	Ok(ret)
}
