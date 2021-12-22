use crate::error::{LoftyError, Result};
use crate::types::properties::FileProperties;

use std::convert::TryInto;
use std::io::{Read, Seek, SeekFrom};
use std::time::Duration;

use byteorder::{LittleEndian, ReadBytesExt};

#[derive(Clone, Debug, PartialEq, Default)]
/// An APE file's audio properties
pub struct ApeProperties {
	version: u16,
	duration: Duration,
	overall_bitrate: u32,
	audio_bitrate: u32,
	sample_rate: u32,
	channels: u8,
}

impl From<ApeProperties> for FileProperties {
	fn from(input: ApeProperties) -> Self {
		Self {
			duration: input.duration,
			overall_bitrate: Some(input.overall_bitrate),
			audio_bitrate: Some(input.audio_bitrate),
			sample_rate: Some(input.sample_rate),
			channels: Some(input.channels),
		}
	}
}

impl ApeProperties {
	/// Creates a new [`ApeProperties`]
	pub const fn new(
		version: u16,
		duration: Duration,
		overall_bitrate: u32,
		audio_bitrate: u32,
		sample_rate: u32,
		channels: u8,
	) -> Self {
		Self {
			version,
			duration,
			overall_bitrate,
			audio_bitrate,
			sample_rate,
			channels,
		}
	}

	/// Duration
	pub fn duration(&self) -> Duration {
		self.duration
	}

	/// Overall bitrate (kbps)
	pub fn overall_bitrate(&self) -> u32 {
		self.overall_bitrate
	}

	/// Audio bitrate (kbps)
	pub fn bitrate(&self) -> u32 {
		self.audio_bitrate
	}

	/// Sample rate (Hz)
	pub fn sample_rate(&self) -> u32 {
		self.sample_rate
	}

	/// Channel count
	pub fn channels(&self) -> u8 {
		self.channels
	}

	/// APE version
	pub fn version(&self) -> u16 {
		self.version
	}
}

pub(super) fn read_properties<R>(
	data: &mut R,
	stream_len: u64,
	file_length: u64,
) -> Result<ApeProperties>
where
	R: Read + Seek,
{
	let version = data
		.read_u16::<LittleEndian>()
		.map_err(|_| LoftyError::Ape("Unable to read version"))?;

	// Property reading differs between versions
	if version >= 3980 {
		properties_gt_3980(data, version, stream_len, file_length)
	} else {
		properties_lt_3980(data, version, stream_len, file_length)
	}
}

fn properties_gt_3980<R>(
	data: &mut R,
	version: u16,
	stream_len: u64,
	file_length: u64,
) -> Result<ApeProperties>
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

	let (duration, overall_bitrate, audio_bitrate) = get_duration_bitrate(
		file_length,
		total_frames,
		final_frame_blocks,
		blocks_per_frame,
		sample_rate,
		stream_len,
	);

	Ok(ApeProperties {
		version,
		duration,
		overall_bitrate,
		audio_bitrate,
		sample_rate,
		channels: channels as u8,
	})
}

fn properties_lt_3980<R>(
	data: &mut R,
	version: u16,
	stream_len: u64,
	file_length: u64,
) -> Result<ApeProperties>
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

	let (duration, overall_bitrate, audio_bitrate) = get_duration_bitrate(
		file_length,
		total_frames,
		final_frame_blocks,
		blocks_per_frame,
		sample_rate,
		stream_len,
	);

	Ok(ApeProperties {
		version,
		duration,
		overall_bitrate,
		audio_bitrate,
		sample_rate,
		channels: channels as u8,
	})
}

fn get_duration_bitrate(
	file_length: u64,
	total_frames: u32,
	final_frame_blocks: u32,
	blocks_per_frame: u32,
	sample_rate: u32,
	stream_len: u64,
) -> (Duration, u32, u32) {
	let mut total_samples = u64::from(final_frame_blocks);

	if total_samples > 1 {
		total_samples += u64::from(blocks_per_frame) * u64::from(total_frames - 1)
	}

	if sample_rate > 0 {
		let length = (total_samples * 1000) / u64::from(sample_rate);

		let overall_bitrate = ((file_length * 8) / length) as u32;
		let audio_bitrate = ((stream_len * 8) / length) as u32;

		(
			Duration::from_millis(length),
			overall_bitrate,
			audio_bitrate,
		)
	} else {
		(Duration::ZERO, 0, 0)
	}
}
