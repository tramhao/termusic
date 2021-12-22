use crate::error::{LoftyError, Result};
use crate::types::properties::FileProperties;

use std::time::Duration;

use byteorder::{LittleEndian, ReadBytesExt};

const PCM: u16 = 0x0001;
const IEEE_FLOAT: u16 = 0x0003;
const EXTENSIBLE: u16 = 0xfffe;

#[allow(missing_docs, non_camel_case_types)]
#[derive(Debug, Copy, Clone, PartialEq)]
/// A WAV file's format
pub enum WavFormat {
	PCM,
	IEEE_FLOAT,
	Other(u16),
}

impl Default for WavFormat {
	fn default() -> Self {
		Self::Other(0)
	}
}

#[derive(Debug, Copy, Clone, PartialEq, Default)]
/// A WAV file's audio properties
pub struct WavProperties {
	format: WavFormat,
	duration: Duration,
	overall_bitrate: u32,
	audio_bitrate: u32,
	sample_rate: u32,
	channels: u8,
}

impl From<WavProperties> for FileProperties {
	fn from(input: WavProperties) -> Self {
		Self {
			duration: input.duration,
			overall_bitrate: Some(input.overall_bitrate),
			audio_bitrate: Some(input.audio_bitrate),
			sample_rate: Some(input.sample_rate),
			channels: Some(input.channels),
		}
	}
}

impl WavProperties {
	/// Create a new [`WavProperties`]
	pub const fn new(
		format: WavFormat,
		duration: Duration,
		overall_bitrate: u32,
		audio_bitrate: u32,
		sample_rate: u32,
		channels: u8,
	) -> Self {
		Self {
			format,
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

	/// WAV format
	pub fn format(&self) -> &WavFormat {
		&self.format
	}
}

pub(super) fn read_properties(
	fmt: &mut &[u8],
	mut total_samples: u32,
	stream_len: u32,
	file_length: u64,
) -> Result<WavProperties> {
	let mut format_tag = fmt.read_u16::<LittleEndian>()?;
	let channels = fmt.read_u16::<LittleEndian>()? as u8;

	if channels == 0 {
		return Err(LoftyError::Wav("File contains 0 channels"));
	}

	let sample_rate = fmt.read_u32::<LittleEndian>()?;
	let bytes_per_second = fmt.read_u32::<LittleEndian>()?;

	// Skip 2 bytes
	// Block align (2)
	fmt.read_u16::<LittleEndian>()?;

	let bits_per_sample = fmt.read_u16::<LittleEndian>()?;

	if format_tag == EXTENSIBLE {
		if fmt.len() < 40 {
			return Err(LoftyError::Wav(
				"Extensible format identified, invalid \"fmt \" chunk size found (< 40)",
			));
		}

		// Skip 8 bytes
		// cbSize (Size of extra format information) (2)
		// Valid bits per sample (2)
		// Channel mask (4)
		fmt.read_u64::<LittleEndian>()?;

		format_tag = fmt.read_u16::<LittleEndian>()?;
	}

	let non_pcm = format_tag != PCM && format_tag != IEEE_FLOAT;

	if non_pcm && total_samples == 0 {
		return Err(LoftyError::Wav(
			"Non-PCM format identified, no \"fact\" chunk found",
		));
	}

	if bits_per_sample > 0 {
		total_samples = stream_len / u32::from(u16::from(channels) * ((bits_per_sample + 7) / 8))
	} else if !non_pcm {
		total_samples = 0
	}

	let (duration, overall_bitrate, audio_bitrate) = if sample_rate > 0 && total_samples > 0 {
		let length = (u64::from(total_samples) * 1000) / u64::from(sample_rate);

		let overall_bitrate = ((file_length * 8) / length) as u32;
		let audio_bitrate = (u64::from(stream_len * 8) / length) as u32;

		(
			Duration::from_millis(length),
			overall_bitrate,
			audio_bitrate,
		)
	} else if bytes_per_second > 0 {
		let length = (u64::from(stream_len) * 1000) / u64::from(bytes_per_second);

		let overall_bitrate = ((file_length * 8) / length) as u32;
		let audio_bitrate = (bytes_per_second * 8) / 1000;

		(
			Duration::from_millis(length),
			overall_bitrate,
			audio_bitrate,
		)
	} else {
		(Duration::ZERO, 0, 0)
	};

	Ok(WavProperties {
		format: match format_tag {
			PCM => WavFormat::PCM,
			IEEE_FLOAT => WavFormat::IEEE_FLOAT,
			other => WavFormat::Other(other),
		},
		duration,
		overall_bitrate,
		audio_bitrate,
		sample_rate,
		channels,
	})
}
