use crate::error::Result;
#[cfg(feature = "id3v2")]
use crate::id3::v2::read::parse_id3v2;
#[cfg(feature = "id3v2")]
use crate::id3::v2::tag::Id3v2Tag;

use std::io::{Read, Seek, SeekFrom};
use std::marker::PhantomData;

use crate::id3::v2::read_id3v2_header;
use byteorder::{ByteOrder, ReadBytesExt};

pub(crate) struct Chunks<B>
where
	B: ByteOrder,
{
	pub fourcc: [u8; 4],
	pub size: u32,
	_phantom: PhantomData<B>,
}

impl<B: ByteOrder> Chunks<B> {
	pub fn new() -> Self {
		Self {
			fourcc: [0; 4],
			size: 0,
			_phantom: PhantomData,
		}
	}

	pub fn next<R>(&mut self, data: &mut R) -> Result<()>
	where
		R: Read,
	{
		data.read_exact(&mut self.fourcc)?;
		self.size = data.read_u32::<B>()?;

		Ok(())
	}

	pub fn content<R>(&mut self, data: &mut R) -> Result<Vec<u8>>
	where
		R: Read,
	{
		let mut content = vec![0; self.size as usize];
		data.read_exact(&mut content)?;

		Ok(content)
	}

	#[cfg(feature = "id3v2")]
	#[allow(clippy::similar_names)]
	pub fn id3_chunk<R>(&mut self, data: &mut R) -> Result<Id3v2Tag>
	where
		R: Read + Seek,
	{
		let mut value = vec![0; self.size as usize];
		data.read_exact(&mut value)?;

		let reader = &mut &*value;

		let header = read_id3v2_header(reader)?;
		let id3v2 = parse_id3v2(reader, header)?;

		// Skip over the footer
		if id3v2.flags().footer {
			data.seek(SeekFrom::Current(10))?;
		}

		Ok(id3v2)
	}

	#[cfg(not(feature = "id3v2"))]
	#[allow(clippy::similar_names)]
	pub fn id3_chunk<R>(&mut self, data: &mut R) -> Result<()>
	where
		R: Read + Seek,
	{
		let mut value = vec![0; self.size as usize];
		data.read_exact(&mut value)?;

		let reader = &mut &*value;

		let header = read_id3v2_header(reader)?;

		// Skip over the footer
		if header.flags.footer {
			data.seek(SeekFrom::Current(10))?;
		}

		Ok(())
	}

	pub fn correct_position<R>(&mut self, data: &mut R) -> Result<()>
	where
		R: Read + Seek,
	{
		// Chunks are expected to start on even boundaries, and are padded
		// with a 0 if necessary. This is NOT the null terminator of the value,
		// and it is NOT included in the chunk's size
		if self.size % 2 != 0 {
			data.seek(SeekFrom::Current(1))?;
		}

		Ok(())
	}
}
