use crate::{FileProperties, LoftyError, Result};

use std::convert::TryInto;
use std::io::{Read, Seek, SeekFrom};
use std::time::Duration;

use byteorder::{LittleEndian, ReadBytesExt};

pub fn properties_gt_3980<R>(data: &mut R, stream_len: u64) -> Result<FileProperties>
where
	R: Read + Seek,
{
	// First read the file descriptor
	let mut descriptor = [0; 46];
	data.read_exact(&mut descriptor)
		.map_err(|_| LoftyError::Ape("Not enough data left in reader to finish file descriptor"))?;

	// The only piece of information we need from the file descriptor
	let descriptor_len = u32::from_le_bytes(
		descriptor[2..6]
			.try_into()
			.map_err(|_| LoftyError::Ape("Unreachable error"))?,
	);

	// The descriptor should be 52 bytes long (including ['M', 'A', 'C', ' ']
	// Anything extra is unknown, and just gets skipped
	if descriptor_len > 52 {
		data.seek(SeekFrom::Current(i64::from(descriptor_len - 52)))?;
	}

	// Move on to the header
	let mut header = [0; 24];
	data.read_exact(&mut header)
		.map_err(|_| LoftyError::Ape("Not enough data left in reader to finish MAC header"))?;

	// Skip the first 4 bytes of the header
	// Compression type (2)
	// Format flags (2)
	let header_read = &mut &header[4..];

	let blocks_per_frame = header_read.read_u32::<LittleEndian>()?;
	let final_frame_blocks = header_read.read_u32::<LittleEndian>()?;
	let total_frames = header_read.read_u32::<LittleEndian>()?;

	if total_frames == 0 {
		return Err(LoftyError::Ape("File contains no frames"));
	}

	// Unused
	let _bits_per_sample = header_read.read_u16::<LittleEndian>()?;

	let channels = header_read.read_u16::<LittleEndian>()?;

	if !(1..=32).contains(&channels) {
		return Err(LoftyError::Ape(
			"File has an invalid channel count (must be between 1 and 32 inclusive)",
		));
	}

	let sample_rate = header_read.read_u32::<LittleEndian>()?;

	let (duration, bitrate) = get_duration_bitrate(
		total_frames,
		final_frame_blocks,
		blocks_per_frame,
		sample_rate,
		stream_len,
	);

	Ok(FileProperties::new(
		duration,
		Some(bitrate),
		Some(sample_rate),
		Some(channels as u8),
	))
}

pub fn properties_lt_3980<R>(data: &mut R, version: u16, stream_len: u64) -> Result<FileProperties>
where
	R: Read + Seek,
{
	// Versions < 3980 don't have a descriptor
	let mut header = [0; 26];
	data.read_exact(&mut header)
		.map_err(|_| LoftyError::Ape("Not enough data left in reader to finish MAC header"))?;

	// We don't need all the header data, so just make 2 slices
	let header_first = &mut &header[..8];

	// Skipping 8 bytes
	// WAV header length (4)
	// WAV tail length (4)
	let header_second = &mut &header[18..];

	let compression_level = header_first.read_u16::<LittleEndian>()?;

	// Unused
	let _format_flags = header_first.read_u16::<LittleEndian>()?;

	let blocks_per_frame = match version {
		_ if version >= 3950 => 73728 * 4,
		_ if version >= 3900 || (version >= 3800 && compression_level >= 4000) => 73728,
		_ => 9216,
	};

	let channels = header_first.read_u16::<LittleEndian>()?;

	if !(1..=32).contains(&channels) {
		return Err(LoftyError::Ape(
			"File has an invalid channel count (must be between 1 and 32 inclusive)",
		));
	}

	let sample_rate = header_first.read_u32::<LittleEndian>()?;

	// Move on the second part of header
	let total_frames = header_second.read_u32::<LittleEndian>()?;

	if total_frames == 0 {
		return Err(LoftyError::Ape("File contains no frames"));
	}

	let final_frame_blocks = data.read_u32::<LittleEndian>()?;

	let (duration, bitrate) = get_duration_bitrate(
		total_frames,
		final_frame_blocks,
		blocks_per_frame,
		sample_rate,
		stream_len,
	);
	Ok(FileProperties::new(
		duration,
		Some(bitrate),
		Some(sample_rate),
		Some(channels as u8),
	))
}

fn get_duration_bitrate(
	total_frames: u32,
	final_frame_blocks: u32,
	blocks_per_frame: u32,
	sample_rate: u32,
	stream_len: u64,
) -> (Duration, u32) {
	let mut total_samples = u64::from(final_frame_blocks);

	if total_samples > 1 {
		total_samples += u64::from(blocks_per_frame) * u64::from(total_frames - 1)
	}

	if sample_rate > 0 {
		let length = (total_samples * 1000) / u64::from(sample_rate);
		let bitrate = ((stream_len * 8) / length) as u32;

		(Duration::from_millis(length), bitrate)
	} else {
		(Duration::ZERO, 0)
	}
}
