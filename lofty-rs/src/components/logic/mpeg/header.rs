use super::constants::{
	BITRATES, PADDING_SIZES, SAMPLES_PER_FRAME, SAMPLE_RATES, SIDE_INFORMATION_SIZES,
};
use crate::{LoftyError, Result};

use std::io::Read;

use byteorder::{BigEndian, ReadBytesExt};

pub(crate) fn verify_frame_sync(byte_1: u8, byte_2: u8) -> bool {
	byte_1 == 0xFF && byte_2 != 0xFF && (byte_2 & 0xE0) == 0xE0
}

#[derive(PartialEq, Copy, Clone)]
enum MpegVersion {
	V1,
	V2,
	V2_5,
}

#[derive(Copy, Clone)]
enum Layer {
	Layer1 = 1,
	Layer2 = 2,
	Layer3 = 3,
}

#[derive(Copy, Clone, PartialEq)]
enum Mode {
	Stereo = 0,
	JointStereo = 1,
	DualChannel = 2,
	SingleChannel = 3,
}

#[derive(Copy, Clone)]
pub(crate) struct Header {
	pub sample_rate: u32,
	pub channels: u8,
	pub len: u32,
	pub data_start: u32,
	pub samples_per_frame: u16,
	pub bitrate: u32,
}

impl Header {
	pub fn read(header: u32) -> Result<Self> {
		let version = match (header >> 19) & 0b11 {
			0 => MpegVersion::V2_5,
			2 => MpegVersion::V2,
			3 => MpegVersion::V1,
			_ => return Err(LoftyError::Mpeg("Frame header has an invalid version")),
		};

		let layer = match (header >> 17) & 0b11 {
			1 => Layer::Layer3,
			2 => Layer::Layer2,
			3 => Layer::Layer1,
			_ => return Err(LoftyError::Mpeg("Frame header uses a reserved layer")),
		};

		let bitrate = (header >> 12) & 0b1111;
		let sample_rate = (header >> 10) & 0b11;

		if sample_rate == 0 {
			return Err(LoftyError::Mpeg("Frame header has a sample rate of 0"));
		}

		let mode = match (header >> 6) & 0b11 {
			0 => Mode::Stereo,
			1 => Mode::JointStereo,
			2 => Mode::DualChannel,
			3 => Mode::SingleChannel,
			_ => return Err(LoftyError::Mpeg("Unreachable error")),
		};

		let layer_index = (layer as usize).saturating_sub(1);
		let version_index = if version == MpegVersion::V1 { 0 } else { 1 };
		let has_padding = ((header >> 9) & 1) != 0;

		let mut data_start = SIDE_INFORMATION_SIZES[version_index][mode as usize] + 4;

		let bitrate = BITRATES[version_index][layer_index][bitrate as usize];
		let sample_rate = SAMPLE_RATES[version as usize][sample_rate as usize];
		let samples_per_frame = SAMPLES_PER_FRAME[layer_index][version_index];

		let mut len = (u32::from(samples_per_frame) * (bitrate * 125)) / sample_rate;

		if has_padding {
			let padding = u32::from(PADDING_SIZES[layer_index]);
			len += padding;
			data_start += padding
		}

		let channels = if mode == Mode::SingleChannel { 1 } else { 2 };

		Ok(Self {
			sample_rate,
			channels,
			len,
			data_start,
			samples_per_frame,
			bitrate,
		})
	}
}

pub(crate) struct XingHeader {
	pub frames: u32,
	pub size: u32,
}

impl XingHeader {
	#[allow(clippy::similar_names)]
	pub fn read(reader: &mut &[u8]) -> Result<Self> {
		let reader_len = reader.len();

		let mut header = [0; 4];
		reader.read_exact(&mut header)?;

		match &header {
			b"Xing" | b"Info" => {
				if reader_len < 16 {
					return Err(LoftyError::Mpeg("Xing header has an invalid size (< 16)"));
				}

				let mut flags = [0; 4];
				reader.read_exact(&mut flags)?;

				if flags[3] & 0x03 != 0x03 {
					return Err(LoftyError::Mpeg(
						"Xing header doesn't have required flags set (0x0001 and 0x0002)",
					));
				}

				let frames = reader.read_u32::<BigEndian>()?;
				let size = reader.read_u32::<BigEndian>()?;

				Ok(Self { frames, size })
			},
			b"VBRI" => {
				if reader_len < 32 {
					return Err(LoftyError::Mpeg("VBRI header has an invalid size (< 32)"));
				}

				// Skip 6 bytes
				// Version ID (2)
				// Delay float (2)
				// Quality indicator (2)
				let _info = reader.read_uint::<BigEndian>(6)?;

				let size = reader.read_u32::<BigEndian>()?;
				let frames = reader.read_u32::<BigEndian>()?;

				Ok(Self { frames, size })
			},
			_ => Err(LoftyError::Mpeg("No Xing, LAME, or VBRI header located")),
		}
	}
}
