#![allow(clippy::module_name_repetitions)]
use anyhow::anyhow;

// using lower mod to restrict clippy
#[allow(clippy::pedantic)]
mod protobuf {
    tonic::include_proto!("player");
}

pub use protobuf::*;

// implement transform function for easy use
impl From<protobuf::Duration> for std::time::Duration {
    fn from(value: protobuf::Duration) -> Self {
        std::time::Duration::new(value.secs, value.nanos)
    }
}

impl From<std::time::Duration> for protobuf::Duration {
    fn from(value: std::time::Duration) -> Self {
        Self {
            secs: value.as_secs(),
            nanos: value.subsec_nanos(),
        }
    }
}

/// The primitive in which time (current position / total duration) will be stored as
pub type PlayerTimeUnit = std::time::Duration;

/// Struct to keep both values with a name, as tuples cannot have named fields
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlayerProgress {
    pub position: Option<PlayerTimeUnit>,
    /// Total duration of the currently playing track, if there is a known total duration
    pub total_duration: Option<PlayerTimeUnit>,
}

impl From<protobuf::PlayerTime> for PlayerProgress {
    fn from(value: protobuf::PlayerTime) -> Self {
        Self {
            position: value.position.map(Into::into),
            total_duration: value.total_duration.map(Into::into),
        }
    }
}

impl From<PlayerProgress> for protobuf::PlayerTime {
    fn from(value: PlayerProgress) -> Self {
        Self {
            position: value.position.map(Into::into),
            total_duration: value.total_duration.map(Into::into),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrackChangedInfo {
    /// Current track index in the playlist
    pub current_track_index: u32,
    /// Indicate if the track changed to another track
    pub current_track_updated: bool,
    /// Title of the current track / radio
    pub title: Option<String>,
    /// Current progress of the track
    pub progress: Option<PlayerProgress>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UpdateEvents {
    MissedEvents { amount: u64 },
    VolumeChanged { volume: u16 },
    SpeedChanged { speed: i32 },
    PlayStateChanged { playing: u32 },
    TrackChanged(TrackChangedInfo),
}

type StreamTypes = protobuf::stream_updates::Type;

// mainly for server to grpc
impl From<UpdateEvents> for protobuf::StreamUpdates {
    fn from(value: UpdateEvents) -> Self {
        let val = match value {
            UpdateEvents::MissedEvents { amount } => {
                StreamTypes::MissedEvents(UpdateMissedEvents { amount })
            }
            UpdateEvents::VolumeChanged { volume } => {
                StreamTypes::VolumeChanged(UpdateVolumeChanged {
                    msg: Some(VolumeReply {
                        volume: u32::from(volume),
                    }),
                })
            }
            UpdateEvents::SpeedChanged { speed } => StreamTypes::SpeedChanged(UpdateSpeedChanged {
                msg: Some(SpeedReply { speed }),
            }),
            UpdateEvents::PlayStateChanged { playing } => {
                StreamTypes::PlayStateChanged(UpdatePlayStateChanged {
                    msg: Some(TogglePauseResponse { status: playing }),
                })
            }
            UpdateEvents::TrackChanged(info) => StreamTypes::TrackChanged(UpdateTrackChanged {
                current_track_index: info.current_track_index,
                current_track_updated: info.current_track_updated,
                optional_title: info
                    .title
                    .map(protobuf::update_track_changed::OptionalTitle::Title),
                progress: info.progress.map(Into::into),
            }),
        };

        Self { r#type: Some(val) }
    }
}

// mainly for grpc to client(tui)
impl TryFrom<protobuf::StreamUpdates> for UpdateEvents {
    type Error = anyhow::Error;

    fn try_from(value: protobuf::StreamUpdates) -> Result<Self, Self::Error> {
        let value = unwrap_msg(value.r#type, "StreamUpdates.type")?;

        let res = match value {
            stream_updates::Type::VolumeChanged(ev) => Self::VolumeChanged {
                volume: clamp_u16(
                    unwrap_msg(ev.msg, "StreamUpdates.types.volume_changed.msg")?.volume,
                ),
            },
            stream_updates::Type::SpeedChanged(ev) => Self::SpeedChanged {
                speed: unwrap_msg(ev.msg, "StreamUpdates.types.speed_changed.msg")?.speed,
            },
            stream_updates::Type::PlayStateChanged(ev) => Self::PlayStateChanged {
                playing: unwrap_msg(ev.msg, "StreamUpdates.types.play_state_changed.msg")?.status,
            },
            stream_updates::Type::MissedEvents(ev) => Self::MissedEvents { amount: ev.amount },
            stream_updates::Type::TrackChanged(ev) => Self::TrackChanged(TrackChangedInfo {
                current_track_index: ev.current_track_index,
                current_track_updated: ev.current_track_updated,
                title: ev.optional_title.map(|v| {
                    let protobuf::update_track_changed::OptionalTitle::Title(v) = v;
                    v
                }),
                progress: ev.progress.map(Into::into),
            }),
        };

        Ok(res)
    }
}

/// Easily unwrap a given grpc option and covert it to a result, with a location on None
fn unwrap_msg<T>(opt: Option<T>, place: &str) -> Result<T, anyhow::Error> {
    match opt {
        Some(val) => Ok(val),
        None => Err(anyhow!("Got \"None\" in grpc \"{place}\"!")),
    }
}

#[allow(clippy::cast_possible_truncation)]
fn clamp_u16(val: u32) -> u16 {
    val.min(u32::from(u16::MAX)) as u16
}
