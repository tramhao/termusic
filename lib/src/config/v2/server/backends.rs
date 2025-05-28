use bytesize::ByteSize;
use serde::{Deserialize, Serialize};

/// Settings specific to a backend
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
#[serde(default)] // allow missing fields and fill them with the `..Self::default()` in this struct
pub struct BackendSettings {
    pub rusty: RustyBackendSettings,
    #[serde(skip)] // skip as long as there are no values
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

/// Settings specific to the `rusty` backend
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(default)] // allow missing fields and fill them with the `..Self::default()` in this struct
pub struct RustyBackendSettings {
    /// Enable or disable `soundtouch`; only has a effect if `rusty-soundtouch` is compiled-in
    pub soundtouch: bool,
    /// Set the buffer size for the raw file.
    /// Note this only applies to local files like music or downloaded podcasts. Does not apply to streamed podcasts or radio.
    ///
    /// If the given value is less than the default, the default will be used instead.
    pub file_buffer_size: ByteSize,
}

impl Default for RustyBackendSettings {
    fn default() -> Self {
        Self {
            soundtouch: true,
            file_buffer_size: ByteSize::b(FILEBUF_SIZE_DEFAULT),
        }
    }
}

/// Settings specific to the `mpv` backend
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
#[serde(default)] // allow missing fields and fill them with the `..Self::default()` in this struct
pub struct MpvBackendSettings {
    // None for now
}

/// Settings specific to the `gst` backend
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
#[serde(default)] // allow missing fields and fill them with the `..Self::default()` in this struct
pub struct GstBackendSettings {
    // None for now
}
