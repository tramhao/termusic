use super::{Atom, AtomData, AtomIdent, Ilst};
use crate::error::{LoftyError, Result};
use crate::id3::v2::util::text_utils::utf16_decode;
use crate::mp4::atom_info::AtomInfo;
use crate::mp4::read::skip_unneeded;
use crate::types::picture::{MimeType, Picture, PictureType};

use std::borrow::Cow;
use std::io::{Cursor, Read, Seek, SeekFrom};

use byteorder::ReadBytesExt;

pub(crate) fn parse_ilst<R>(reader: &mut R, len: u64) -> Result<Ilst>
where
	R: Read,
{
	let mut contents = vec![0; len as usize];
	reader.read_exact(&mut contents)?;

	let mut cursor = Cursor::new(contents);

	let mut tag = Ilst::default();

	while let Ok(atom) = AtomInfo::read(&mut cursor) {
		let ident = match &atom.ident {
			AtomIdent::Fourcc(ref fourcc) => match fourcc {
				b"free" | b"skip" => {
					skip_unneeded(&mut cursor, atom.extended, atom.len)?;
					continue;
				},
				b"covr" => {
					let value = parse_data(&mut cursor)?;

					let (mime_type, data) = match value {
						AtomData::Unknown { code, data } => match code {
							// Type 0 is implicit
							0 => (MimeType::None, data),
							// GIF is deprecated
							12 => (MimeType::Gif, data),
							13 => (MimeType::Jpeg, data),
							14 => (MimeType::Png, data),
							27 => (MimeType::Bmp, data),
							_ => {
								return Err(LoftyError::BadAtom(
									"\"covr\" atom has an unknown type",
								))
							},
						},
						_ => return Err(LoftyError::BadAtom("\"covr\" atom has an unknown type")),
					};

					tag.atoms.push(Atom {
						ident: atom.ident,
						data: AtomData::Picture(Picture {
							pic_type: PictureType::Other,
							mime_type,
							description: None,
							data: Cow::from(data),
						}),
					});

					continue;
				},
				_ => atom.ident,
			},
			_ => atom.ident,
		};

		let data = parse_data(&mut cursor)?;

		tag.atoms.push(Atom { ident, data })
	}

	Ok(tag)
}

fn parse_data<R>(data: &mut R) -> Result<AtomData>
where
	R: Read + Seek,
{
	let atom = AtomInfo::read(data)?;

	match atom.ident {
		AtomIdent::Fourcc(ref name) if name == b"data" => {},
		_ => return Err(LoftyError::BadAtom("Expected atom \"data\" to follow name")),
	}

	// We don't care about the version
	let _version = data.read_u8()?;

	let mut flags = [0; 3];
	data.read_exact(&mut flags)?;

	let flags = u32::from_be_bytes([0, flags[0], flags[1], flags[2]]);

	// We don't care about the locale
	data.seek(SeekFrom::Current(4))?;

	let mut content = vec![0; (atom.len - 16) as usize];
	data.read_exact(&mut content)?;

	// https://developer.apple.com/library/archive/documentation/QuickTime/QTFF/Metadata/Metadata.html#//apple_ref/doc/uid/TP40000939-CH1-SW35
	let value = match flags {
		1 => AtomData::UTF8(String::from_utf8(content)?),
		2 => AtomData::UTF16(utf16_decode(&*content, u16::from_be_bytes)?),
		21 => AtomData::SignedInteger(parse_int(&content)?),
		22 => AtomData::UnsignedInteger(parse_uint(&content)?),
		code => AtomData::Unknown {
			code,
			data: content,
		},
	};

	Ok(value)
}

fn parse_uint(bytes: &[u8]) -> Result<u32> {
	Ok(match bytes.len() {
		1 => u32::from(bytes[0]),
		2 => u32::from(u16::from_be_bytes([bytes[0], bytes[1]])),
		3 => u32::from_be_bytes([0, bytes[0], bytes[1], bytes[2]]),
		4 => u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
		_ => {
			return Err(LoftyError::BadAtom(
				"Unexpected atom size for type \"BE unsigned integer\"",
			))
		},
	})
}

fn parse_int(bytes: &[u8]) -> Result<i32> {
	Ok(match bytes.len() {
		1 => i32::from(bytes[0]),
		2 => i32::from(i16::from_be_bytes([bytes[0], bytes[1]])),
		3 => i32::from_be_bytes([0, bytes[0], bytes[1], bytes[2]]) as i32,
		4 => i32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as i32,
		_ => {
			return Err(LoftyError::BadAtom(
				"Unexpected atom size for type \"BE signed integer\"",
			))
		},
	})
}
