#![allow(clippy::module_name_repetitions)]

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
