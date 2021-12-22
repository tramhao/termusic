use crate::error::Result;

use std::io::{Read, Seek, SeekFrom};

use byteorder::{BigEndian, ReadBytesExt};

pub(super) struct Block {
	#[allow(unused)]
	pub(super) byte: u8,
	pub(super) ty: u8,
	pub(super) last: bool,
	pub(super) content: Vec<u8>,
	pub(super) start: u64,
	pub(super) end: u64,
}

impl Block {
	pub(in crate::ogg) fn read<R>(data: &mut R) -> Result<Self>
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
