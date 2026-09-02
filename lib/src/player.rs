#![allow(clippy::module_name_repetitions)]
use anyhow::{Context, anyhow, bail};

// using lower mod to restrict clippy
#[allow(clippy::pedantic)]
pub mod protobuf {
    pub mod server {
        tonic::include_proto!("termusic.server");
    }
    pub mod player {
        tonic::include_proto!("termusic.player");
    }
    pub mod queue {
        tonic::include_proto!("termusic.queue");
    }
    pub mod stream {
        tonic::include_proto!("termusic.stream");
    }
    pub mod common {
        tonic::include_proto!("termusic.common");
    }
}

use crate::config::v2::server::LoopMode;

impl protobuf::queue::SortDirection {
    /// Swap between `Asc` and `Desc`.
    #[must_use]
    pub fn invert(self) -> Self {
        match self {
            Self::Asc => Self::Desc,
            Self::Desc => Self::Asc,
        }
    }
}

// implement transform function for easy use
impl From<protobuf::common::Duration> for std::time::Duration {
    fn from(value: protobuf::common::Duration) -> Self {
        std::time::Duration::new(value.secs, value.nanos)
    }
}

impl From<std::time::Duration> for protobuf::common::Duration {
    fn from(value: std::time::Duration) -> Self {
        Self {
            secs: value.as_secs(),
            nanos: value.subsec_nanos(),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ChangeLoopMode {
    Cycle,
    Mode(LoopMode),
}

impl From<ChangeLoopMode> for protobuf::queue::ChangeLoopModeRequest {
    fn from(value: ChangeLoopMode) -> Self {
        use protobuf::queue::{ChangeLoopModeRequest, change_loop_mode_request};
        match value {
            ChangeLoopMode::Cycle => ChangeLoopModeRequest {
                r#type: Some(change_loop_mode_request::Type::Cycle(
                    protobuf::common::Empty {},
                )),
            },
            ChangeLoopMode::Mode(loop_mode) => ChangeLoopModeRequest {
                r#type: Some(change_loop_mode_request::Type::Mode(loop_mode.into())),
            },
        }
    }
}

impl TryFrom<protobuf::queue::ChangeLoopModeRequest> for ChangeLoopMode {
    type Error = anyhow::Error;

    fn try_from(value: protobuf::queue::ChangeLoopModeRequest) -> Result<Self, Self::Error> {
        use protobuf::queue::change_loop_mode_request;
        let value = unwrap_msg(value.r#type, "ChangeLoopModeRequest.type")?;
        Ok(match value {
            change_loop_mode_request::Type::Cycle(_) => Self::Cycle,
            change_loop_mode_request::Type::Mode(val) => Self::Mode(val.try_into()?),
        })
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ChangeRunningState {
    Toggle = 0,
    Pause = 1,
    Resume = 2,
}

impl From<ChangeRunningState> for protobuf::player::ChangeRunningStateRequest {
    fn from(value: ChangeRunningState) -> Self {
        protobuf::player::ChangeRunningStateRequest {
            state: value as u32,
        }
    }
}

impl TryFrom<protobuf::player::ChangeRunningStateRequest> for ChangeRunningState {
    type Error = anyhow::Error;

    fn try_from(value: protobuf::player::ChangeRunningStateRequest) -> Result<Self, Self::Error> {
        Ok(match value.state {
            0 => Self::Toggle,
            1 => Self::Pause,
            2 => Self::Resume,
            v => bail!("Invalid value {v} for ChangeRunningState"),
        })
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ChangeVolume {
    Steps(i16),
    Unit(i16),
}

impl From<ChangeVolume> for protobuf::player::ChangeVolumeRequest {
    fn from(value: ChangeVolume) -> Self {
        use protobuf::player::{ChangeVolumeRequest, change_volume_request};
        match value {
            ChangeVolume::Steps(v) => ChangeVolumeRequest {
                r#type: Some(change_volume_request::Type::Step(i32::from(v))),
            },
            ChangeVolume::Unit(v) => ChangeVolumeRequest {
                r#type: Some(change_volume_request::Type::Volume(i32::from(v))),
            },
        }
    }
}

impl TryFrom<protobuf::player::ChangeVolumeRequest> for ChangeVolume {
    type Error = anyhow::Error;

    fn try_from(value: protobuf::player::ChangeVolumeRequest) -> Result<Self, Self::Error> {
        use protobuf::player::change_volume_request;
        let value = unwrap_msg(value.r#type, "ChangeVolumeRequest.type")?;
        Ok(match value {
            change_volume_request::Type::Step(v) => ChangeVolume::Steps(clamp_i16(v)),
            change_volume_request::Type::Volume(v) => ChangeVolume::Unit(clamp_i16(v)),
        })
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ChangeSpeed {
    Steps(i32),
    Unit(i32),
    Reset,
}

impl From<ChangeSpeed> for protobuf::player::ChangeSpeedRequest {
    fn from(value: ChangeSpeed) -> Self {
        use protobuf::player::{ChangeSpeedRequest, change_speed_request};
        match value {
            ChangeSpeed::Steps(v) => ChangeSpeedRequest {
                r#type: Some(change_speed_request::Type::Step(v)),
            },
            ChangeSpeed::Unit(v) => ChangeSpeedRequest {
                r#type: Some(change_speed_request::Type::Speed(v)),
            },
            ChangeSpeed::Reset => ChangeSpeedRequest {
                r#type: Some(change_speed_request::Type::Reset(
                    protobuf::common::Empty {},
                )),
            },
        }
    }
}

impl TryFrom<protobuf::player::ChangeSpeedRequest> for ChangeSpeed {
    type Error = anyhow::Error;

    fn try_from(value: protobuf::player::ChangeSpeedRequest) -> Result<Self, Self::Error> {
        use protobuf::player::change_speed_request;
        let value = unwrap_msg(value.r#type, "ChangeSpeedRequest.type")?;
        Ok(match value {
            change_speed_request::Type::Step(v) => ChangeSpeed::Steps(v),
            change_speed_request::Type::Speed(v) => ChangeSpeed::Unit(v),
            change_speed_request::Type::Reset(_) => ChangeSpeed::Reset,
        })
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SeekReq {
    Steps(i32),
    // Seconds
    Unit(i64),
    RestartTrack,
}

impl From<SeekReq> for protobuf::player::SeekRequest {
    fn from(value: SeekReq) -> Self {
        use protobuf::player::{SeekRequest, seek_request};
        match value {
            SeekReq::Steps(v) => SeekRequest {
                r#type: Some(seek_request::Type::Step(v)),
            },
            SeekReq::Unit(v) => SeekRequest {
                r#type: Some(seek_request::Type::Time(v)),
            },
            SeekReq::RestartTrack => SeekRequest {
                r#type: Some(seek_request::Type::RestartTrack(protobuf::common::Empty {})),
            },
        }
    }
}

impl TryFrom<protobuf::player::SeekRequest> for SeekReq {
    type Error = anyhow::Error;

    fn try_from(value: protobuf::player::SeekRequest) -> Result<Self, Self::Error> {
        use protobuf::player::seek_request;
        let value = unwrap_msg(value.r#type, "SeekRequest.type")?;
        Ok(match value {
            seek_request::Type::Step(v) => SeekReq::Steps(v),
            seek_request::Type::Time(v) => SeekReq::Unit(v),
            seek_request::Type::RestartTrack(_) => SeekReq::RestartTrack,
        })
    }
}

/// The primitive in which time (current position / total duration) will be stored as
pub type PlayerTimeUnit = std::time::Duration;

#[derive(Clone, Copy, Default, PartialEq, Eq, Debug)]
pub enum RunningStatus {
    #[default]
    Stopped,
    Running,
    Paused,
}

impl RunningStatus {
    #[must_use]
    pub fn as_u32(&self) -> u32 {
        match self {
            RunningStatus::Stopped => 0,
            RunningStatus::Running => 1,
            RunningStatus::Paused => 2,
        }
    }

    #[must_use]
    pub fn from_u32(status: u32) -> Self {
        match status {
            1 => RunningStatus::Running,
            2 => RunningStatus::Paused,
            _ => RunningStatus::Stopped,
        }
    }
}

impl std::fmt::Display for RunningStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Running => write!(f, "Running"),
            Self::Stopped => write!(f, "Stopped"),
            Self::Paused => write!(f, "Paused"),
        }
    }
}

/// Struct to keep both values with a name, as tuples cannot have named fields
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlayerProgress {
    pub position: Option<PlayerTimeUnit>,
    /// Total duration of the currently playing track, if there is a known total duration
    pub total_duration: Option<PlayerTimeUnit>,
}

impl From<protobuf::player::PlayerTime> for PlayerProgress {
    fn from(value: protobuf::player::PlayerTime) -> Self {
        Self {
            position: value.position.map(Into::into),
            total_duration: value.total_duration.map(Into::into),
        }
    }
}

impl From<PlayerProgress> for protobuf::player::PlayerTime {
    fn from(value: PlayerProgress) -> Self {
        Self {
            position: value.position.map(Into::into),
            total_duration: value.total_duration.map(Into::into),
        }
    }
}

impl TryFrom<protobuf::stream::UpdateProgress> for PlayerProgress {
    type Error = anyhow::Error;

    fn try_from(value: protobuf::stream::UpdateProgress) -> Result<Self, Self::Error> {
        let Some(val) = value.progress else {
            bail!("Expected \"UpdateProgress\" to contain \"Some(progress)\"");
        };

        Ok(Self::from(val))
    }
}

impl From<PlayerProgress> for protobuf::stream::UpdateProgress {
    fn from(value: PlayerProgress) -> Self {
        Self {
            progress: Some(value.into()),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrackChangedInfo {
    /// Current track index in the playlist
    pub current_track_index: u64,
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
    GaplessChanged { gapless: bool },
    PlaylistChanged(UpdatePlaylistEvents),
    Progress(PlayerProgress),
}

// might not be fully true, but necessary for Msg
impl Eq for UpdateEvents {}

type StreamTypes = protobuf::stream::stream_updates::Type;

// mainly for server to grpc
impl From<UpdateEvents> for protobuf::stream::StreamUpdates {
    fn from(value: UpdateEvents) -> Self {
        use protobuf::player::{GaplessState, PlayState, SpeedReply, VolumeReply};
        use protobuf::stream::{
            UpdateGaplessChanged, UpdateMissedEvents, UpdatePlayStateChanged, UpdateSpeedChanged,
            UpdateTrackChanged, UpdateVolumeChanged,
        };
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
                    msg: Some(PlayState { status: playing }),
                })
            }
            UpdateEvents::TrackChanged(info) => StreamTypes::TrackChanged(UpdateTrackChanged {
                current_track_index: info.current_track_index,
                optional_title: info
                    .title
                    .map(protobuf::stream::update_track_changed::OptionalTitle::Title),
                progress: info.progress.map(Into::into),
            }),
            UpdateEvents::GaplessChanged { gapless } => {
                StreamTypes::GaplessChanged(UpdateGaplessChanged {
                    msg: Some(GaplessState { gapless }),
                })
            }
            UpdateEvents::PlaylistChanged(ev) => StreamTypes::PlaylistChanged(ev.into()),
            UpdateEvents::Progress(ev) => StreamTypes::ProgressChanged(ev.into()),
        };

        Self { r#type: Some(val) }
    }
}

// mainly for grpc to client(tui)
impl TryFrom<protobuf::stream::StreamUpdates> for UpdateEvents {
    type Error = anyhow::Error;

    fn try_from(value: protobuf::stream::StreamUpdates) -> Result<Self, Self::Error> {
        let value = unwrap_msg(value.r#type, "StreamUpdates.type")?;

        let res = match value {
            StreamTypes::VolumeChanged(ev) => Self::VolumeChanged {
                volume: clamp_u16(
                    unwrap_msg(ev.msg, "StreamUpdates.types.volume_changed.msg")?.volume,
                ),
            },
            StreamTypes::SpeedChanged(ev) => Self::SpeedChanged {
                speed: unwrap_msg(ev.msg, "StreamUpdates.types.speed_changed.msg")?.speed,
            },
            StreamTypes::PlayStateChanged(ev) => Self::PlayStateChanged {
                playing: unwrap_msg(ev.msg, "StreamUpdates.types.play_state_changed.msg")?.status,
            },
            StreamTypes::MissedEvents(ev) => Self::MissedEvents { amount: ev.amount },
            StreamTypes::TrackChanged(ev) => Self::TrackChanged(TrackChangedInfo {
                current_track_index: ev.current_track_index,
                title: ev.optional_title.map(|v| {
                    let protobuf::stream::update_track_changed::OptionalTitle::Title(v) = v;
                    v
                }),
                progress: ev.progress.map(Into::into),
            }),
            StreamTypes::GaplessChanged(ev) => Self::GaplessChanged {
                gapless: unwrap_msg(ev.msg, "StreamUpdates.types.gapless_changed.msg")?.gapless,
            },
            StreamTypes::PlaylistChanged(ev) => Self::PlaylistChanged(
                ev.try_into()
                    .context("In \"StreamUpdates.types.playlist_changed\"")?,
            ),
            StreamTypes::ProgressChanged(ev) => Self::Progress(
                ev.try_into()
                    .context("In \"StreamUpdates.types.progress_changed\"")?,
            ),
        };

        Ok(res)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlaylistAddTrackInfo {
    /// The Index at which a track was added at.
    /// If this is not at the end, all tracks at this index and beyond should be shifted.
    pub at_index: u64,
    pub tracks: Vec<playlist_helpers::PlaylistTrackSource>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlaylistRemoveTrackInfo {
    /// The Index at which a track was removed at.
    pub at_index: u64,
    /// The Id of the removed track.
    pub trackid: playlist_helpers::PlaylistTrackSource,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlaylistLoopModeInfo {
    /// The actual mode, mapped to [`LoopMode`]
    pub mode: u32,
}

impl From<LoopMode> for PlaylistLoopModeInfo {
    fn from(value: LoopMode) -> Self {
        Self {
            mode: u32::from(value.discriminant()),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlaylistSwapInfo {
    pub index_a: u64,
    pub index_b: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlaylistShuffledInfo {
    pub tracks: protobuf::queue::PlaylistState,
}

/// Separate nested enum to handle all playlist related events
#[derive(Debug, Clone, PartialEq)]
pub enum UpdatePlaylistEvents {
    PlaylistAddTrack(PlaylistAddTrackInfo),
    PlaylistRemoveTrack(PlaylistRemoveTrackInfo),
    PlaylistCleared,
    PlaylistLoopMode(PlaylistLoopModeInfo),
    PlaylistSwapTracks(PlaylistSwapInfo),
    PlaylistShuffled(PlaylistShuffledInfo),
}

type PPlaylistTypes = protobuf::stream::update_playlist::Type;

// mainly for server to grpc
impl From<UpdatePlaylistEvents> for protobuf::stream::UpdatePlaylist {
    fn from(value: UpdatePlaylistEvents) -> Self {
        use protobuf::stream::{
            PlaylistAddTracks, PlaylistLoopMode, PlaylistRemoveTrack, PlaylistShuffled,
            PlaylistSwapTracks,
        };
        let val = match value {
            UpdatePlaylistEvents::PlaylistAddTrack(vals) => {
                PPlaylistTypes::AddTrack(PlaylistAddTracks {
                    at_index: vals.at_index,
                    tracks: vals.tracks.into_iter().map(Into::into).collect(),
                })
            }
            UpdatePlaylistEvents::PlaylistRemoveTrack(vals) => {
                PPlaylistTypes::RemoveTrack(PlaylistRemoveTrack {
                    at_index: vals.at_index,
                    id: Some(vals.trackid.into()),
                })
            }
            UpdatePlaylistEvents::PlaylistCleared => {
                PPlaylistTypes::Cleared(protobuf::common::Empty {})
            }
            UpdatePlaylistEvents::PlaylistLoopMode(vals) => {
                PPlaylistTypes::LoopMode(PlaylistLoopMode {
                    mode: protobuf::queue::PlaylistLoopMode { mode: vals.mode }.into(),
                })
            }
            UpdatePlaylistEvents::PlaylistSwapTracks(vals) => {
                PPlaylistTypes::SwapTracks(PlaylistSwapTracks {
                    index_a: vals.index_a,
                    index_b: vals.index_b,
                })
            }
            UpdatePlaylistEvents::PlaylistShuffled(vals) => {
                PPlaylistTypes::Shuffled(PlaylistShuffled {
                    shuffled: Some(vals.tracks),
                })
            }
        };

        Self { r#type: Some(val) }
    }
}

// mainly for grpc to client(tui)
impl TryFrom<protobuf::stream::UpdatePlaylist> for UpdatePlaylistEvents {
    type Error = anyhow::Error;

    fn try_from(value: protobuf::stream::UpdatePlaylist) -> Result<Self, Self::Error> {
        let value = unwrap_msg(value.r#type, "UpdatePlaylist.type")?;

        let res = match value {
            PPlaylistTypes::AddTrack(ev) => Self::PlaylistAddTrack(PlaylistAddTrackInfo {
                at_index: ev.at_index,
                tracks: ev
                    .tracks
                    .into_iter()
                    .map(TryInto::try_into)
                    .collect::<Result<Vec<_>, _>>()?,
            }),
            PPlaylistTypes::RemoveTrack(ev) => Self::PlaylistRemoveTrack(PlaylistRemoveTrackInfo {
                at_index: ev.at_index,
                trackid: unwrap_msg(
                    unwrap_msg(ev.id, "UpdatePlaylist.type.remove_track.id")?.source,
                    "UpdatePlaylist.type.remove_track.id.source",
                )?
                .try_into()?,
            }),
            PPlaylistTypes::Cleared(_) => Self::PlaylistCleared,
            PPlaylistTypes::LoopMode(ev) => Self::PlaylistLoopMode(PlaylistLoopModeInfo {
                mode: unwrap_msg(ev.mode, "UpdatePlaylist.type.loop_mode.mode")?.mode,
            }),
            PPlaylistTypes::SwapTracks(ev) => Self::PlaylistSwapTracks(PlaylistSwapInfo {
                index_a: ev.index_a,
                index_b: ev.index_b,
            }),
            PPlaylistTypes::Shuffled(ev) => {
                let shuffled = unwrap_msg(ev.shuffled, "UpdatePlaylist.type.shuffled.shuffled")?;
                Self::PlaylistShuffled(PlaylistShuffledInfo { tracks: shuffled })
            }
        };

        Ok(res)
    }
}

/// Easily unwrap a given grpc option and convert it to a result, with a location on None
fn unwrap_msg<T>(opt: Option<T>, place: &str) -> Result<T, anyhow::Error> {
    match opt {
        Some(val) => Ok(val),
        None => Err(anyhow!("Got \"None\" in grpc \"{place}\"!")),
    }
}

/// Clamp a given `u32` to be `u16`.
///
/// This is mainly used for volume clamping as we only use u16 for that, but protobuf minimal number is u32.
#[allow(clippy::cast_possible_truncation)]
#[must_use]
pub fn clamp_u16(val: u32) -> u16 {
    val.min(u32::from(u16::MAX)) as u16
}

/// Clamp a given `i32` to be `i16`.
///
/// This is mainly used for volume clamping as we only use i16 for that, but protobuf minimal number is i32.
#[allow(clippy::cast_possible_truncation)]
#[must_use]
pub fn clamp_i16(val: i32) -> i16 {
    val.min(i32::from(i16::MAX)) as i16
}

pub mod playlist_helpers {
    use anyhow::{Context, anyhow};

    use crate::config::v2::server::LoopMode;

    use super::{protobuf, unwrap_msg};

    /// A Id / Source for a given Track
    #[derive(Debug, Clone, PartialEq)]
    pub enum PlaylistTrackSource {
        Path(String),
        Url(String),
        PodcastUrl(String),
    }

    impl From<PlaylistTrackSource> for protobuf::common::track_id::Source {
        fn from(value: PlaylistTrackSource) -> Self {
            match value {
                PlaylistTrackSource::Path(v) => Self::Path(v),
                PlaylistTrackSource::Url(v) => Self::Url(v),
                PlaylistTrackSource::PodcastUrl(v) => Self::PodcastUrl(v),
            }
        }
    }

    impl From<PlaylistTrackSource> for protobuf::common::TrackId {
        fn from(value: PlaylistTrackSource) -> Self {
            Self {
                source: Some(value.into()),
            }
        }
    }

    impl TryFrom<protobuf::common::track_id::Source> for PlaylistTrackSource {
        type Error = anyhow::Error;

        fn try_from(value: protobuf::common::track_id::Source) -> Result<Self, Self::Error> {
            Ok(match value {
                protobuf::common::track_id::Source::Path(v) => Self::Path(v),
                protobuf::common::track_id::Source::Url(v) => Self::Url(v),
                protobuf::common::track_id::Source::PodcastUrl(v) => Self::PodcastUrl(v),
            })
        }
    }

    impl TryFrom<protobuf::common::TrackId> for PlaylistTrackSource {
        type Error = anyhow::Error;

        fn try_from(value: protobuf::common::TrackId) -> Result<Self, Self::Error> {
            unwrap_msg(value.source, "TrackId.source").and_then(Self::try_from)
        }
    }

    impl From<LoopMode> for protobuf::queue::PlaylistLoopMode {
        fn from(value: LoopMode) -> Self {
            protobuf::queue::PlaylistLoopMode {
                mode: u32::from(value.discriminant()),
            }
        }
    }

    impl TryFrom<protobuf::queue::PlaylistLoopMode> for LoopMode {
        type Error = anyhow::Error;

        fn try_from(value: protobuf::queue::PlaylistLoopMode) -> Result<Self, Self::Error> {
            let mode = u8::try_from(value.mode).context("Unsupported LoopMode")?;
            LoopMode::tryfrom_discriminant(mode)
                .ok_or(anyhow!("Failed to convert PlaylistLoopMode to LoopMode"))
        }
    }

    /// Data for requesting some tracks to be added in the server
    #[derive(Debug, Clone, PartialEq)]
    pub struct PlaylistAddTrack {
        pub at_index: u64,
        pub tracks: Vec<PlaylistTrackSource>,
    }

    impl PlaylistAddTrack {
        #[must_use]
        pub fn new_single(at_index: u64, track: PlaylistTrackSource) -> Self {
            Self {
                at_index,
                tracks: vec![track],
            }
        }

        #[must_use]
        pub fn new_vec(at_index: u64, tracks: Vec<PlaylistTrackSource>) -> Self {
            Self { at_index, tracks }
        }
    }

    impl From<PlaylistAddTrack> for protobuf::queue::PlaylistTracksToAddRequest {
        fn from(value: PlaylistAddTrack) -> Self {
            Self {
                at_index: value.at_index,
                tracks: value.tracks.into_iter().map(Into::into).collect(),
            }
        }
    }

    impl TryFrom<protobuf::queue::PlaylistTracksToAddRequest> for PlaylistAddTrack {
        type Error = anyhow::Error;

        fn try_from(
            value: protobuf::queue::PlaylistTracksToAddRequest,
        ) -> Result<Self, Self::Error> {
            let tracks = value
                .tracks
                .into_iter()
                .map(|v| {
                    PlaylistTrackSource::try_from(v).context("PlaylistTracksToAddRequest.tracks")
                })
                .collect::<Result<Vec<_>, anyhow::Error>>()?;

            Ok(Self {
                at_index: value.at_index,
                tracks,
            })
        }
    }

    /// Data for requesting some tracks to be removed in the server
    #[derive(Debug, Clone, PartialEq)]
    pub struct PlaylistRemoveTrackIndexed {
        pub at_index: u64,
        pub tracks: Vec<PlaylistTrackSource>,
    }

    impl PlaylistRemoveTrackIndexed {
        #[must_use]
        pub fn new_single(at_index: u64, track: PlaylistTrackSource) -> Self {
            Self {
                at_index,
                tracks: vec![track],
            }
        }

        #[must_use]
        pub fn new_vec(at_index: u64, tracks: Vec<PlaylistTrackSource>) -> Self {
            Self { at_index, tracks }
        }
    }

    impl From<PlaylistRemoveTrackIndexed> for protobuf::queue::PlaylistTracksToRemoveIndexed {
        fn from(value: PlaylistRemoveTrackIndexed) -> Self {
            Self {
                at_index: value.at_index,
                tracks: value.tracks.into_iter().map(Into::into).collect(),
            }
        }
    }

    impl TryFrom<protobuf::queue::PlaylistTracksToRemoveIndexed> for PlaylistRemoveTrackIndexed {
        type Error = anyhow::Error;

        fn try_from(
            value: protobuf::queue::PlaylistTracksToRemoveIndexed,
        ) -> Result<Self, Self::Error> {
            let tracks = value
                .tracks
                .into_iter()
                .map(|v| {
                    PlaylistTrackSource::try_from(v).context("PlaylistTracksToRemoveIndexed.tracks")
                })
                .collect::<Result<Vec<_>, anyhow::Error>>()?;

            Ok(Self {
                at_index: value.at_index,
                tracks,
            })
        }
    }

    /// Data for requesting some tracks to be removed in the server
    #[derive(Debug, Clone, PartialEq)]
    pub enum PlaylistRemoveTrackType {
        Indexed(PlaylistRemoveTrackIndexed),
        Clear,
    }

    type PToRemoveTypes = protobuf::queue::playlist_tracks_to_remove_request::Type;

    impl From<PlaylistRemoveTrackType> for protobuf::queue::PlaylistTracksToRemoveRequest {
        fn from(value: PlaylistRemoveTrackType) -> Self {
            Self {
                r#type: Some(match value {
                    PlaylistRemoveTrackType::Indexed(v) => PToRemoveTypes::Indexed(v.into()),
                    PlaylistRemoveTrackType::Clear => {
                        PToRemoveTypes::Clear(protobuf::common::Empty {})
                    }
                }),
            }
        }
    }

    impl TryFrom<protobuf::queue::PlaylistTracksToRemoveRequest> for PlaylistRemoveTrackType {
        type Error = anyhow::Error;

        fn try_from(
            value: protobuf::queue::PlaylistTracksToRemoveRequest,
        ) -> Result<Self, Self::Error> {
            let value = unwrap_msg(value.r#type, "PlaylistTracksToRemoveRequest.type")?;

            Ok(match value {
                PToRemoveTypes::Indexed(v) => Self::Indexed(v.try_into()?),
                PToRemoveTypes::Clear(_) => Self::Clear,
            })
        }
    }

    /// Data for requesting some tracks to be swapped in the server
    #[derive(Debug, Clone, PartialEq)]
    pub struct PlaylistSwapTrack {
        pub index_a: u64,
        pub index_b: u64,
    }

    impl From<PlaylistSwapTrack> for protobuf::queue::PlaylistSwapTracksRequest {
        fn from(value: PlaylistSwapTrack) -> Self {
            Self {
                index_a: value.index_a,
                index_b: value.index_b,
            }
        }
    }

    impl TryFrom<protobuf::queue::PlaylistSwapTracksRequest> for PlaylistSwapTrack {
        type Error = anyhow::Error;

        fn try_from(
            value: protobuf::queue::PlaylistSwapTracksRequest,
        ) -> Result<Self, Self::Error> {
            Ok(Self {
                index_a: value.index_a,
                index_b: value.index_b,
            })
        }
    }

    /// Data for requesting to skip / play a specific track
    #[derive(Debug, Clone, PartialEq)]
    pub struct PlaylistPlaySpecific {
        pub track_index: u64,
        pub id: PlaylistTrackSource,
    }

    impl From<PlaylistPlaySpecific> for protobuf::queue::PlaylistPlaySpecificRequest {
        fn from(value: PlaylistPlaySpecific) -> Self {
            Self {
                track_index: value.track_index,
                id: Some(value.id.into()),
            }
        }
    }

    impl TryFrom<protobuf::queue::PlaylistPlaySpecificRequest> for PlaylistPlaySpecific {
        type Error = anyhow::Error;

        fn try_from(
            value: protobuf::queue::PlaylistPlaySpecificRequest,
        ) -> Result<Self, Self::Error> {
            Ok(Self {
                track_index: value.track_index,
                id: unwrap_msg(value.id, "PlaylistPlaySpecificRequest.id").and_then(|v| {
                    PlaylistTrackSource::try_from(v).context("PlaylistPlaySpecificRequest.id")
                })?,
            })
        }
    }
}
