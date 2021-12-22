use super::find_last_page;
use crate::error::{LoftyError, Result};
use crate::types::properties::FileProperties;

use std::io::{Read, Seek, SeekFrom};
use std::time::Duration;

use byteorder::{LittleEndian, ReadBytesExt};
use ogg_pager::Page;

#[derive(Copy, Clone, Debug, PartialEq, Default)]
/// An OGG Vorbis file's audio properties
pub struct VorbisProperties {
	duration: Duration,
	overall_bitrate: u32,
	audio_bitrate: u32,
	sample_rate: u32,
	channels: u8,
	version: u32,
	bitrate_maximum: i32,
	bitrate_nominal: i32,
	bitrate_minimum: i32,
}

impl From<VorbisProperties> for FileProperties {
	fn from(input: VorbisProperties) -> Self {
		Self {
			duration: input.duration,
			overall_bitrate: Some(input.overall_bitrate),
			audio_bitrate: Some(input.audio_bitrate),
			sample_rate: Some(input.sample_rate),
			channels: Some(input.channels),
		}
	}
}

impl VorbisProperties {
	/// Creates a new [`VorbisProperties`]
	pub const fn new(
		duration: Duration,
		overall_bitrate: u32,
		audio_bitrate: u32,
		sample_rate: u32,
		channels: u8,
		version: u32,
		bitrate_maximum: i32,
		bitrate_nominal: i32,
		bitrate_minimum: i32,
	) -> Self {
		Self {
			duration,
			overall_bitrate,
			audio_bitrate,
			sample_rate,
			channels,
			version,
			bitrate_maximum,
			bitrate_nominal,
			bitrate_minimum,
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
	pub fn audio_bitrate(&self) -> u32 {
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

	/// Vorbis version
	pub fn version(&self) -> u32 {
		self.version
	}

	/// Maximum bitrate
	pub fn bitrate_max(&self) -> i32 {
		self.bitrate_maximum
	}

	/// Nominal bitrate
	pub fn bitrate_nominal(&self) -> i32 {
		self.bitrate_nominal
	}

	/// Minimum bitrate
	pub fn bitrate_min(&self) -> i32 {
		self.bitrate_minimum
	}
}

pub(in crate::ogg) fn read_properties<R>(
	data: &mut R,
	first_page: &Page,
) -> Result<VorbisProperties>
where
	R: Read + Seek,
{
	let first_page_abgp = first_page.abgp;

	// Skip identification header
	let first_page_content = &mut &first_page.content[7..];

	let version = first_page_content.read_u32::<LittleEndian>()?;

	let channels = first_page_content.read_u8()?;
	let sample_rate = first_page_content.read_u32::<LittleEndian>()?;

	let bitrate_maximum = first_page_content.read_i32::<LittleEndian>()?;
	let bitrate_nominal = first_page_content.read_i32::<LittleEndian>()?;
	let bitrate_minimum = first_page_content.read_i32::<LittleEndian>()?;

	let last_page = find_last_page(data)?;
	let last_page_abgp = last_page.abgp;

	let file_length = data.seek(SeekFrom::End(0))?;

	last_page_abgp.checked_sub(first_page_abgp).map_or_else(
		|| Err(LoftyError::Vorbis("File contains incorrect PCM values")),
		|frame_count| {
			let length = frame_count * 1000 / u64::from(sample_rate);
			let duration = Duration::from_millis(length as u64);

			let overall_bitrate = ((file_length * 8) / length) as u32;
			let audio_bitrate = bitrate_nominal as u64 / 1000;

			Ok(VorbisProperties {
				duration,
				overall_bitrate,
				audio_bitrate: audio_bitrate as u32,
				sample_rate,
				channels,
				version,
				bitrate_maximum,
				bitrate_nominal,
				bitrate_minimum,
			})
		},
	)
}
