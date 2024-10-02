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
pub enum UpdateEvents {
    VolumeChanged { volume: u16 },
    SpeedChanged { speed: i32 },
    PlayStateChanged { playing: u32 },
    // TrackChanged,
}

type StreamTypes = protobuf::stream_updates::Type;

// mainly for server to grpc
impl From<UpdateEvents> for protobuf::StreamUpdates {
    fn from(value: UpdateEvents) -> Self {
        let val = match value {
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
            } // UpdateEvents::TrackChanged => StreamTypes::TrackChanged(UpdateTrackChanged {}),
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
