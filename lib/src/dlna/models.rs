use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub  struct DlnaDevice {
    pub name: String,
    pub uri: String,
    pub udn: String,
    pub device_type: String,
    pub location: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub  struct MediaItem {
    pub id: String,
    pub title: String,
    pub url: String,
    pub duration: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub  struct MediaContainer {
    pub id: String,
    pub name: String,
}