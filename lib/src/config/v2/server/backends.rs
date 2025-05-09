use serde::{Deserialize, Serialize};

/// Settings specific to a backend
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
#[serde(default)] // allow missing fields and fill them with the `..Self::default()` in this struct
pub struct BackendSettings {
    #[serde(skip)] // skip as long as there are no values
    pub rusty: RustyBackendSettings,
    #[serde(skip)] // skip as long as there are no values
    pub mpv: MpvBackendSettings,
    #[serde(skip)] // skip as long as there are no values
    pub gst: GstBackendSettings,
}

/// Settings specific to the `rusty` backend
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
#[serde(default)] // allow missing fields and fill them with the `..Self::default()` in this struct
pub struct RustyBackendSettings {
    // None for now
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
