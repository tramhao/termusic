use std::mem::ManuallyDrop;
use serde::{Deserialize, Serialize};
use crate::track::Track;

#[derive(Debug, Clone)]
pub struct DlnaDevice {
    pub id: u32,
    pub name: String,
    pub uri: String,
    pub udn: String,
    pub device_type: String,
    pub location: String,
}

//#[derive(Clone)] //, Serialize, Deserialize)]
#[repr(C)]
pub union MediaChild {
    pub container: ManuallyDrop<Vec<MediaContainer>>,
    pub children: ManuallyDrop<Vec<MediaItem>>,
}

#[derive(Debug, Clone)] //, Serialize, Deserialize)]
pub struct MediaItem {
    pub id: String,
    pub track: String,
    pub title: String,
    pub url: String,
    pub duration: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub genre: Option<String>,
}

#[derive(Clone)] //, Serialize, Deserialize)]
pub struct MediaContainer {
    pub id: String,
    pub name: String,
    pub childs: Vec<MediaContainer>,
    pub items: Vec<MediaItem>,
}