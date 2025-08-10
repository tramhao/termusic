use bytesize::ByteSize;
use serde::{Deserialize, Serialize};

/// Settings specific to a backend
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
#[serde(default)] // allow missing fields and fill them with the `..Self::default()` in this struct
pub struct BackendSettings {
    pub rusty: RustyBackendSettings,
    pub mpv: MpvBackendSettings,
    #[serde(skip)] // skip as long as there are no values
    pub gst: GstBackendSettings,
}

/// Default Buffer capacity in bytes
///
/// 1024 * 1024 * 4 = 4 MiB
///
/// [`BufReader`]'s default size is 8 Kib
pub const FILEBUF_SIZE_DEFAULT: u64 = 1024 * 1024 * 4;

/// The minimal and default size the decode-ringbuffer should have.
///
/// Currently the size is based on 192kHz * 1 channel * 1 seconds, or 2 seconds of 48kHz stereo(2 channel) audio.
// NOTE: this may desync with the actual `MIN_RING_SIZE` if the type or message size should change, and that should be consulted instead
pub const DECODEDBUF_SIZE_DEFAULT: u64 = 192_000 * size_of::<f32>() as u64;

/// Settings specific to the `rusty` backend
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(default)] // allow missing fields and fill them with the `..Self::default()` in this struct
pub struct RustyBackendSettings {
    /// Enable or disable `soundtouch`; only has a effect if `rusty-soundtouch` is compiled-in
    pub soundtouch: bool,
    /// Set the buffer size for the raw file.
    /// This value will be clamped to the actual file's size.
    /// Note this only applies to local files like music or downloaded podcasts. Does not apply to streamed podcasts or radio.
    ///
    /// If the given value is less than the default, the default will be used instead.
    pub file_buffer_size: ByteSize,
    /// Set the decoded ring buffer size.
    /// This controls how many decoded audio bytes are stored.
    /// Unlike `file_buffer_size`, this buffer will always be this size, regardless if there is less data.
    /// Note this only applies to local files like music or downloaded podcasts. Does not apply to streamed podcasts or radio.
    ///
    /// If the given value is less than the default, the default will be used instead.
    pub decoded_buffer_size: ByteSize,
    /// Set the preferred output sample rate.
    ///
    /// Default `48_000`
    /// Recommeded Values: `44_100`, `48_000`, `96_000` `192_000`.
    pub output_sample_rate: u32,
}

impl Default for RustyBackendSettings {
    fn default() -> Self {
        Self {
            soundtouch: true,
            file_buffer_size: ByteSize::b(FILEBUF_SIZE_DEFAULT),
            decoded_buffer_size: ByteSize::b(DECODEDBUF_SIZE_DEFAULT),
            output_sample_rate: 48_000,
        }
    }
}

/// Settings specific to the `mpv` backend
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(default)] // allow missing fields and fill them with the `..Self::default()` in this struct
pub struct MpvBackendSettings {
    /// Select the audio device mpv should be using, analog to mpv's `--audio-device=` option.
    ///
    /// See all available options for mpv by running `mpv --audio-device=help`
    ///
    /// Default: `auto`
    pub audio_device: String,
}

impl Default for MpvBackendSettings {
    fn default() -> Self {
        Self {
            audio_device: "auto".to_string(),
        }
    }
}

/// Settings specific to the `gst` backend
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
#[serde(default)] // allow missing fields and fill them with the `..Self::default()` in this struct
pub struct GstBackendSettings {
    // None for now
}
