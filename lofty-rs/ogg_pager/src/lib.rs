mod crc;
mod error;

use std::io::{Read, Seek, SeekFrom};

use byteorder::{LittleEndian, ReadBytesExt};

pub use crc::crc32;
pub use error::{PageError, Result};

#[derive(Clone)]
pub struct Page {
	pub content: Vec<u8>,
	pub header_type: u8,
	pub abgp: u64,
	pub serial: u32,
	pub seq_num: u32,
	pub checksum: u32,
	pub start: u64,
	pub end: u64,
}

impl Page {
	/// Convert the Page to Vec<u8> for writing
	pub fn as_bytes(&self) -> Vec<u8> {
		let mut bytes = Vec::new();
		let segments = self.segments();
		let segment_count = [segments.len() as u8];

		bytes.extend(b"OggS".iter());
		bytes.extend([0_u8].iter());
		bytes.extend(self.header_type.to_le_bytes().iter());
		bytes.extend(self.abgp.to_le_bytes().iter());
		bytes.extend(self.serial.to_le_bytes().iter());
		bytes.extend(self.seq_num.to_le_bytes().iter());
		bytes.extend(self.checksum.to_le_bytes().iter());
		bytes.extend(segment_count.iter());
		bytes.extend(segments.iter());
		bytes.extend(self.content.iter());

		bytes
	}

	/// Returns the Page's segment table as Vec<u8>
	pub fn segments(&self) -> Vec<u8> {
		segments(&*self.content)
	}

	/// Attempts to get a Page from a reader
	pub fn read<V>(data: &mut V, skip_content: bool) -> Result<Self>
	where
		V: Read + Seek,
	{
		let start = data.seek(SeekFrom::Current(0))?;

		let mut sig = [0; 4];
		data.read_exact(&mut sig)?;

		if &sig != b"OggS" {
			return Err(PageError::MissingMagic);
		}

		// Version, always 0
		let version = data.read_u8()?;

		if version != 0 {
			return Err(PageError::InvalidVersion);
		}

		let header_type = data.read_u8()?;

		let abgp = data.read_u64::<LittleEndian>()?;
		let serial = data.read_u32::<LittleEndian>()?;
		let seq_num = data.read_u32::<LittleEndian>()?;
		let checksum = data.read_u32::<LittleEndian>()?;

		let segments = data.read_u8()?;

		if segments < 1 {
			return Err(PageError::BadSegmentCount);
		}

		let mut segment_table = vec![0; segments as usize];
		data.read_exact(&mut segment_table)?;

		let mut content: Vec<u8> = Vec::new();
		let content_len = segment_table.iter().map(|&b| b as i64).sum();

		if skip_content {
			data.seek(SeekFrom::Current(content_len))?;
		} else {
			content = vec![0; content_len as usize];
			data.read_exact(&mut content)?;
		}

		let end = data.seek(SeekFrom::Current(0))?;

		Ok(Page {
			content,
			header_type,
			abgp,
			serial,
			seq_num,
			checksum,
			start,
			end,
		})
	}

	/// Generates the CRC checksum of the page
	pub fn gen_crc(&mut self) {
		self.checksum = crc::crc32(&*self.as_bytes());
	}

	/// Extends the Page's content, returning another Page if too much data was provided
	pub fn extend(&mut self, content: &[u8]) -> Option<Page> {
		let self_len = self.content.len();
		let content_len = content.len();

		if self_len <= 65025 && self_len + content_len <= 65025 {
			self.content.extend(content.iter());
			self.end += content_len as u64;

			return None;
		}

		if content_len <= 65025 {
			let remaining = 65025 - self_len;

			self.content.extend(content[0..remaining].iter());
			self.end += remaining as u64;

			let mut p = Page {
				content: content[remaining..].to_vec(),
				header_type: 1,
				abgp: 0,
				serial: self.serial,
				seq_num: self.seq_num + 1,
				checksum: 0,
				start: self.end,
				end: self.start + content.len() as u64,
			};

			p.gen_crc();

			return Some(p);
		}

		None
	}
}

pub fn segments(cont: &[u8]) -> Vec<u8> {
	let len = cont.len();

	let mut last_len = (len % 255) as u8;
	if last_len == 0 {
		last_len = 255
	}

	let mut needed = len / 255;
	if needed != 255 {
		needed += 1
	}

	let mut segments = Vec::new();

	for i in 0..needed {
		if i + 1 < needed {
			segments.push(255)
		} else {
			segments.push(last_len)
		}
	}

	segments
}
