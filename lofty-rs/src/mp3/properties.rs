use super::header::{ChannelMode, Header, Layer, MpegVersion, XingHeader};
use crate::types::properties::FileProperties;

use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Default)]
/// An MP3 file's audio properties
pub struct Mp3Properties {
	version: MpegVersion,
	layer: Layer,
	channel_mode: ChannelMode,
	duration: Duration,
	overall_bitrate: u32,
	audio_bitrate: u32,
	sample_rate: u32,
	channels: u8,
}

impl From<Mp3Properties> for FileProperties {
	fn from(input: Mp3Properties) -> Self {
		Self {
			duration: input.duration,
			overall_bitrate: Some(input.overall_bitrate),
			audio_bitrate: Some(input.audio_bitrate),
			sample_rate: Some(input.sample_rate),
			channels: Some(input.channels),
		}
	}
}

impl Mp3Properties {
	/// Creates a new [`Mp3Properties`]
	pub const fn new(
		version: MpegVersion,
		layer: Layer,
		channel_mode: ChannelMode,
		duration: Duration,
		overall_bitrate: u32,
		audio_bitrate: u32,
		sample_rate: u32,
		channels: u8,
	) -> Self {
		Self {
			version,
			layer,
			channel_mode,
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

	/// MPEG version
	pub fn version(&self) -> &MpegVersion {
		&self.version
	}

	/// MPEG layer
	pub fn layer(&self) -> &Layer {
		&self.layer
	}

	/// MPEG channel mode
	pub fn channel_mode(&self) -> &ChannelMode {
		&self.channel_mode
	}
}

pub(super) fn read_properties(
	first_frame: (Header, u64),
	last_frame: (Header, u64),
	xing_header: Option<XingHeader>,
	file_length: u64,
) -> Mp3Properties {
	let (duration, overall_bitrate, audio_bitrate) = {
		match xing_header {
			Some(xing_header) if first_frame.0.sample_rate > 0 => {
				let frame_time =
					u32::from(first_frame.0.samples) * 1000 / first_frame.0.sample_rate;
				let length = u64::from(frame_time) * u64::from(xing_header.frames);

				let overall_bitrate = ((file_length * 8) / length) as u32;
				let audio_bitrate = ((u64::from(xing_header.size) * 8) / length) as u32;

				(
					Duration::from_millis(length),
					overall_bitrate,
					audio_bitrate,
				)
			},
			_ if first_frame.0.bitrate > 0 => {
				let audio_bitrate = first_frame.0.bitrate;

				let stream_length = last_frame.1 - first_frame.1 + u64::from(first_frame.0.len);
				let length = (stream_length * 8) / u64::from(audio_bitrate);

				let overall_bitrate = ((file_length * 8) / length) as u32;

				let duration = Duration::from_millis(length);

				(duration, overall_bitrate, audio_bitrate)
			},
			_ => (Duration::ZERO, 0, 0),
		}
	};

	Mp3Properties {
		version: first_frame.0.version,
		layer: first_frame.0.layer,
		channel_mode: first_frame.0.channel_mode,
		duration,
		overall_bitrate,
		audio_bitrate,
		sample_rate: first_frame.0.sample_rate,
		channels: first_frame.0.channels as u8,
	}
}
