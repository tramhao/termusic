use std::{fmt, str::FromStr};

use serde::{de, Deserialize, Deserializer, Serialize};
use thiserror::Error;

use crate::{player::Loop, track::Track};

#[derive(Error, Debug)]
pub enum WebApiError {
    #[error("webapi error internal error occurs {0}")]
    Internal(String),
}

impl std::fmt::Debug for Track {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Track (title: {}, file: {}, ...)",
            self.title().unwrap_or(""),
            self.file().unwrap_or("")
        )
    }
}

#[derive(Clone)]
pub enum PlayerInfo {
    CurrentStatus(bool),
    CurrentTrack(Track),
    NextTrack(Track),
    NotFoundTrack,
    CurrentGapless(bool),
    CurrentLoopMode(Loop),
    CurrentPlaylistAddFront(bool),
}

#[derive(Debug, Deserialize)]
pub enum Op {
    PlayerStart,
    PlayerStop,
    PlayerTogglePause,
    PlayerToggleGapless,
    PlayerNext,
    PlayerPrev,
    PlayerVolumeUp,
    PlayerVolumeDown,
    ChangeLoopModeQueue,
    ChangeLoopModePlaylist,
    ChangeLoopModeSingle,
    EnablePlaylistAddFront,
    EnablePlaylistAddTail,
    PlayerSpeedUp,
    PlayerSpeedDown,
    PlayerEnableGapless,
    PlayerDisableGapless,
    // NOTE: for retriveing player status or data
    GetPlayerStatus,
    GetCurrentTrack,
    GetPlayerGapless,
    GetNextTrack,
    // GetPrevTrack,
    GetCurrentLoopMode,
    GetPlaylistAddFront,
}

#[derive(Debug, Deserialize)]
pub struct PlayerActionPayload {
    pub op: Op,
}

#[derive(Debug, Deserialize)]
pub struct AddTrackPayload {
    pub path: String,
    // play added track if it's true
    pub play_now: bool,
}

#[derive(Debug, Deserialize)]
pub struct GetStatusPayload {
    pub op: Option<Op>,
    pub key: Option<String>,
}

// QueryString
#[derive(Deserialize)]
pub struct QueryParamUploadTrack {
    // the file_name must validate name in OS
    // Note:
    //  1. file_name can't be none
    //  2. file will be saved to temp folder with the random name (md5) if save_to_muse_folder is not true
    #[serde(default, deserialize_with = "empty_string_as_none")]
    pub file_name: Option<String>,
    // will save to music folder, will return failure if music folder path can't be retrieved by rust
    pub save_to_music_folder: Option<bool>,
    // play the music now if play_now=true
    pub play_now: Option<bool>,
}

/// Serde deserialization decorator to map empty Strings to None,
fn empty_string_as_none<'de, D, T>(de: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: FromStr,
    T::Err: fmt::Display,
{
    let opt = Option::<String>::deserialize(de)?;
    match opt.as_deref() {
        None | Some("") => Ok(None),
        Some(s) => FromStr::from_str(s).map_err(de::Error::custom).map(Some),
    }
}

#[derive(Serialize)]
#[serde(untagged)]
pub enum WebApiResponse {
    // pub result: bool,
    // pub message: String,
    // pub player_status: Option<bool>,
    GeneralResult {
        result: bool,
        message: String,
        key: Option<String>,
    },
    GetPlayerStatus {
        result: bool,
        message: String,
        is_playing: bool,
    },
    GetTrack {
        result: bool,
        message: String,

        artist: Option<String>,
        album: Option<String>,
        title: Option<String>,
        duration: u64,
        file_path: Option<String>,
    },
    GetPlayerGapless {
        result: bool,
        message: String,
        is_gapless: bool,
    },
    GetLoopMode {
        result: bool,
        message: String,
        loop_mode: Loop,
    },
    GetAddFront {
        result: bool,
        message: String,
        add_front: bool,
    },
    UploadTrack {
        result: bool,
        message: String,
        file_name: Option<String>,
        play_now: Option<bool>,
    },
}
