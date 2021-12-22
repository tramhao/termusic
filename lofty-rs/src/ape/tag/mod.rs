#[cfg(feature = "ape")]
pub(crate) mod ape_tag;
#[cfg(feature = "ape")]
pub(crate) mod item;
#[cfg(feature = "ape")]
pub(crate) mod read;
#[cfg(feature = "ape")]
mod write;

use crate::error::{LoftyError, Result};

use std::io::{Read, Seek, SeekFrom};
use std::ops::Neg;

use byteorder::{LittleEndian, ReadBytesExt};

#[derive(Copy, Clone)]
pub(crate) struct ApeHeader {
	pub(crate) size: u32,
	#[cfg(feature = "ape")]
	pub(crate) item_count: u32,
}

pub(crate) fn read_ape_header<R>(data: &mut R, footer: bool) -> Result<ApeHeader>
where
	R: Read + Seek,
{
	let version = data.read_u32::<LittleEndian>()?;

	let mut size = data.read_u32::<LittleEndian>()?;

	if size < 32 {
		// If the size is < 32, something went wrong during encoding
		// The size includes the footer and all items
		return Err(LoftyError::Ape("Tag has an invalid size (< 32)"));
	}

	#[cfg(feature = "ape")]
	let item_count = data.read_u32::<LittleEndian>()?;

	#[cfg(not(feature = "ape"))]
	data.seek(SeekFrom::Current(4))?;

	if footer {
		// No point in reading the rest of the footer, just seek back to the end of the header
		data.seek(SeekFrom::Current(i64::from(size - 12).neg()))?;
	} else {
		// There are 12 bytes remaining in the header
		// Flags (4)
		// Reserved (8)
		data.seek(SeekFrom::Current(12))?;
	}

	// Version 1 doesn't include a header
	if version == 2000 {
		size += 32
	}

	Ok(ApeHeader {
		size,
		#[cfg(feature = "ape")]
		item_count,
	})
}
