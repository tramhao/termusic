use std::time::Duration;

/// Various *immutable* audio properties
///
/// NOTE: All fields are invalidated after any type of conversion
pub struct FileProperties {
	duration: Duration,
	bitrate: Option<u32>,
	sample_rate: Option<u32>,
	channels: Option<u8>,
}

impl Default for FileProperties {
	fn default() -> Self {
		Self {
			duration: Duration::ZERO,
			bitrate: None,
			sample_rate: None,
			channels: None,
		}
	}
}

impl FileProperties {
	/// Create a new FileProperties
	pub const fn new(
		duration: Duration,
		bitrate: Option<u32>,
		sample_rate: Option<u32>,
		channels: Option<u8>,
	) -> Self {
		Self {
			duration,
			bitrate,
			sample_rate,
			channels,
		}
	}

	/// Duration
	pub fn duration(&self) -> Duration {
		self.duration
	}

	/// Bitrate (kbps)
	pub fn bitrate(&self) -> Option<u32> {
		self.bitrate
	}

	/// Sample rate (Hz)
	pub fn sample_rate(&self) -> Option<u32> {
		self.sample_rate
	}

	/// Channel count
	pub fn channels(&self) -> Option<u8> {
		self.channels
	}
}
