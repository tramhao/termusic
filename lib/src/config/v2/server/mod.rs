use std::{
    fmt::Display,
    net::{IpAddr, SocketAddr},
    num::{NonZeroU8, NonZeroU32},
    path::PathBuf,
};

use serde::{Deserialize, Serialize};

use crate::track::MediaTypesSimple;
use backends::BackendSettings;
use metadata::MetadataSettings;

pub mod backends;
/// Extra things necessary for a config file, like wrappers for versioning
pub mod config_extra;
pub mod metadata;

pub type MusicDirsOwned = Vec<PathBuf>;

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Default)]
#[serde(default)] // allow missing fields and fill them with the `..Self::default()` in this struct
#[allow(clippy::module_name_repetitions)]
pub struct ServerSettings {
    pub com: ComSettings,
    pub player: PlayerSettings,
    pub podcast: PodcastSettings,
    pub backends: BackendSettings,
    pub metadata: MetadataSettings,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(default)] // allow missing fields and fill them with the `..Self::default()` in this struct
pub struct PodcastSettings {
    /// Max Concurrent Downloads for Podcasts
    // realistically, we dont have any more than 255 running
    pub concurrent_downloads_max: NonZeroU8,
    /// Max retries for Podcast downloads
    // realistically, we dont have any more than 255 retries
    pub max_download_retries: u8,
    /// Directory for downloaded Podcasts
    pub download_dir: PathBuf,
}

/// Get the default podcast dir, which uses OS-specific paths, or home/Music/podcast
fn default_podcast_dir() -> PathBuf {
    dirs::audio_dir().map_or_else(
        || PathBuf::from(shellexpand::tilde("~/Music").as_ref()),
        |mut v| {
            v.push("podcast");
            v
        },
    )
}

impl Default for PodcastSettings {
    fn default() -> Self {
        Self {
            concurrent_downloads_max: NonZeroU8::new(3).unwrap(),
            max_download_retries: 3,
            download_dir: default_podcast_dir(),
        }
    }
}

// note that regardless of options, loops should never happen and also should never go outside of the root music_dir
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum ScanDepth {
    /// Only go X deep
    // realistically, we dont have any more than u32::MAX depth
    Limited(u32),
    /// Allow going fully down without limit
    Unlimited,
}

/// What determines a long track length in seconds, 10 minutes
const LONG_TRACK_TIME: u64 = 600; // 60 * 10

/// Seek amount maybe depending on track length
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum SeekStep {
    /// Only have one seek-step value
    Both(NonZeroU32),
    /// Have different values depending on track type
    Depends {
        /// tracks < 10 minutes (like Music)
        short_tracks: NonZeroU32,
        /// tracks =>10 minutes (like Podcasts)
        long_tracks: NonZeroU32,
    },
}

impl SeekStep {
    #[allow(clippy::missing_panics_doc)] // const unwrap
    #[must_use]
    pub fn default_both() -> Self {
        Self::Both(NonZeroU32::new(5).unwrap())
    }

    #[allow(clippy::missing_panics_doc)] // const unwrap
    #[must_use]
    pub fn default_depends() -> Self {
        Self::Depends {
            short_tracks: NonZeroU32::new(5).unwrap(),
            long_tracks: NonZeroU32::new(30).unwrap(),
        }
    }

    /// Get Seek Step, depending on track-length
    ///
    /// directly returns a i64, though the value is never negative returned from here
    #[must_use]
    pub fn get_step(&self, track_len: u64) -> i64 {
        match self {
            SeekStep::Both(v) => v.get().into(),
            SeekStep::Depends {
                short_tracks,
                long_tracks,
            } => {
                if track_len >= LONG_TRACK_TIME {
                    long_tracks.get().into()
                } else {
                    short_tracks.get().into()
                }
            }
        }
    }
}

impl Default for SeekStep {
    fn default() -> Self {
        Self::default_depends()
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum PositionYesNoLower {
    /// Remember position, automatically decide after how much time
    Yes,
    /// Dont remember position
    No,
}

/// Default for [`PositionYesNoLower::Yes`] for [`MediaType::Music`]
const DEFAULT_YES_TIME_BEFORE_SAVE_MUSIC: u64 = 3;

/// Default for [`PositionYesNoLower::Yes`] for [`MediaType::Podcast`]
const DEFAULT_YES_TIME_BEFORE_SAVE_PODCAST: u64 = 10;

// this exists because "serde(rename_all)" and "serde(untagged)" dont work well together
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum PositionYesNo {
    /// Simple wrapper to workaround the `"serde(rename_all)" and "serde(untagged)"` problem
    Simple(PositionYesNoLower),
    /// Remember Position after custom time (in seconds)
    YesTime(u64),
}

impl PositionYesNo {
    /// Get the time before saving the track position, if enabled
    #[must_use]
    pub fn get_time(&self, media_type: MediaTypesSimple) -> Option<u64> {
        match self {
            PositionYesNo::Simple(v) => match v {
                PositionYesNoLower::Yes => match media_type {
                    MediaTypesSimple::Music => Some(DEFAULT_YES_TIME_BEFORE_SAVE_MUSIC),
                    MediaTypesSimple::Podcast => Some(DEFAULT_YES_TIME_BEFORE_SAVE_PODCAST),
                    MediaTypesSimple::LiveRadio => None,
                },
                PositionYesNoLower::No => None,
            },
            PositionYesNo::YesTime(v) => Some(*v),
        }
    }

    /// Get if the current value means "it is enabled"
    #[must_use]
    pub fn is_enabled(&self) -> bool {
        match self {
            PositionYesNo::Simple(v) => *v == PositionYesNoLower::Yes,
            PositionYesNo::YesTime(_) => true,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum RememberLastPosition {
    /// Apply a single value to all media types
    All(PositionYesNo),
    /// Set specific values for each media type
    Depends {
        music: PositionYesNo,
        podcast: PositionYesNo,
    },
}

impl RememberLastPosition {
    /// Get the time before saving the track position, if enabled
    #[must_use]
    pub fn get_time(&self, media_type: MediaTypesSimple) -> Option<u64> {
        match self {
            RememberLastPosition::All(v) => v.get_time(media_type),
            RememberLastPosition::Depends { music, podcast } => match media_type {
                MediaTypesSimple::Music => music.get_time(media_type),
                MediaTypesSimple::Podcast => podcast.get_time(media_type),
                MediaTypesSimple::LiveRadio => None,
            },
        }
    }

    /// Get if remembering for the given [`MediaTypesSimple`] is enabled or not
    ///
    /// use case is in the restore of the last position
    #[allow(clippy::needless_pass_by_value)] // "MediaTypesSimple" is a 1-byte copy
    #[must_use]
    pub fn is_enabled_for(&self, media_type: MediaTypesSimple) -> bool {
        match self {
            RememberLastPosition::All(v) => v.is_enabled(),
            RememberLastPosition::Depends { music, podcast } => match media_type {
                MediaTypesSimple::Music => music.is_enabled(),
                MediaTypesSimple::Podcast => podcast.is_enabled(),
                // liveradio cannot store a position
                MediaTypesSimple::LiveRadio => false,
            },
        }
    }
}

impl Default for RememberLastPosition {
    fn default() -> Self {
        Self::Depends {
            music: PositionYesNo::Simple(PositionYesNoLower::No),
            podcast: PositionYesNo::Simple(PositionYesNoLower::Yes),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(default)] // allow missing fields and fill them with the `..Self::default()` in this struct
pub struct PlayerSettings {
    /// Music Directories
    pub music_dirs: MusicDirsOwned,
    /// Legacy value, this still exists so that existing (older)configs parse without error.
    /// But the actual value will be unused and discared.
    /// The following is the old description:
    ///
    /// Max depth the TUI will scan for the music library tree
    #[serde(skip_serializing)]
    pub library_scan_depth: ScanDepth,
    /// Set if the position should be remembered for tracks
    pub remember_position: RememberLastPosition,

    /// Playlist loop mode
    pub loop_mode: LoopMode,
    /// Volume, how loud something is
    pub volume: u16,
    /// Speed, both positive (forward) or negative (backwards)
    ///
    /// speed / 10 = actual speed (float but not floats)
    // the number should never be 0, because that would effectively be paused forever
    pub speed: i32,
    /// Enable gapless decoding & prefetching the next track
    pub gapless: bool,
    /// How much to seek on a seek event
    pub seek_step: SeekStep,

    /// Controls if support via Media-Controls (like mpris on linux) is enabled
    pub use_mediacontrols: bool,
    /// Controls if discord status setting is enabled
    pub set_discord_status: bool,

    /// Amount of tracks to add on "random track add"
    pub random_track_quantity: NonZeroU32,
    /// Minimal amount of tracks a album needs to have before being chosen for "random album add"
    pub random_album_min_quantity: NonZeroU32,

    /// The backend to use
    pub backend: Backend,
}

/// Get the default Music dir, which uses OS-specific paths, or home/Music
fn default_music_dirs() -> MusicDirsOwned {
    Vec::from([
        dirs::audio_dir().unwrap_or_else(|| PathBuf::from(shellexpand::tilde("~/Music").as_ref()))
    ])
}

impl Default for PlayerSettings {
    fn default() -> Self {
        Self {
            music_dirs: default_music_dirs(),
            library_scan_depth: ScanDepth::Limited(0),
            remember_position: RememberLastPosition::default(),

            loop_mode: LoopMode::default(),
            // rather use a lower value than a high so that ears dont get blown off
            volume: 30,
            speed: 10,
            gapless: true,
            seek_step: SeekStep::default(),

            use_mediacontrols: true,
            set_discord_status: true,

            random_track_quantity: NonZeroU32::new(20).unwrap(),
            random_album_min_quantity: NonZeroU32::new(5).unwrap(),

            backend: Backend::default(),
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, Default, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Backend {
    #[serde(rename = "gst")]
    #[serde(alias = "gstreamer")]
    Gstreamer,
    Mpv,
    #[default]
    Rusty,
}

impl Backend {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Backend::Gstreamer => "gst",
            Backend::Mpv => "mpv",
            Backend::Rusty => "rusty",
        }
    }
}

impl Display for Backend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Playlist loop modes
#[derive(Debug, Clone, Copy, Deserialize, Serialize, Default, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
#[repr(u8)]
pub enum LoopMode {
    /// Loop one track
    Single = 0,
    /// Loop the entire Playlist (after last index comes the first)
    #[default]
    Playlist = 1,
    /// Select a random track on each next track
    Random = 2,
}

impl LoopMode {
    #[must_use]
    pub fn display(self, display_symbol: bool) -> &'static str {
        if display_symbol {
            match self {
                Self::Single => "🔂",
                Self::Playlist => "🔁",
                Self::Random => "🔀",
            }
        } else {
            match self {
                Self::Single => "single",
                Self::Playlist => "playlist",
                Self::Random => "random",
            }
        }
    }

    /// Convert the current enum variant into its number representation
    #[must_use]
    pub fn discriminant(&self) -> u8 {
        (*self) as u8
    }

    /// Try to convert the input number representation to a variant
    #[must_use]
    pub fn tryfrom_discriminant(num: u8) -> Option<Self> {
        Some(match num {
            0 => Self::Single,
            1 => Self::Playlist,
            2 => Self::Random,
            _ => return None,
        })
    }
}

/// Error for when [`ComProtocol`] parsing fails
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum ComProtocolParseError {
    #[error("Expected \"uds\" or \"http\", found \"{0}\"")]
    UnknownValue(String),
}

/// The Protocol to use for the server-client communication
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(try_from = "String")]
#[serde(into = "String")]
pub enum ComProtocol {
    HTTP,
    /// Unix socket
    UDS,
}

impl TryFrom<String> for ComProtocol {
    type Error = ComProtocolParseError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_from(value.as_str())
    }
}

impl TryFrom<&str> for ComProtocol {
    type Error = ComProtocolParseError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let lowercase = value.to_ascii_lowercase();
        Ok(match lowercase.as_str() {
            "http" => Self::HTTP,
            "uds" => Self::UDS,
            _ => return Err(ComProtocolParseError::UnknownValue(lowercase)),
        })
    }
}

impl From<ComProtocol> for String {
    fn from(val: ComProtocol) -> Self {
        match val {
            ComProtocol::HTTP => "http",
            ComProtocol::UDS => "uds",
        }
        .to_string()
    }
}

/// Settings for the gRPC server (and potentially future ways to communicate)
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
// for now, require that both port and ip are specified at once
#[serde(default)] // allow missing fields and fill them with the `..Self::default()` in this struct
pub struct ComSettings {
    // General Settings
    pub protocol: ComProtocol,

    // UDS settings
    pub socket_path: PathBuf,

    // Below are HTTP settings
    /// gRPC server Port
    pub port: u16,
    /// gRPC server interface / address
    pub address: IpAddr,
}

/// Helper function to get the default UDS socker path.
#[must_use]
pub fn default_uds_socket_path() -> PathBuf {
    // TODO: maybe default to include user id like "termusic-1000.socket"?
    std::env::temp_dir().join("termusic.socket")
}

impl Default for ComSettings {
    fn default() -> Self {
        Self {
            #[cfg(unix)]
            protocol: ComProtocol::UDS,
            #[cfg(not(unix))]
            protocol: ComProtocol::HTTP,

            socket_path: default_uds_socket_path(),

            port: 50101,
            address: "::1".parse().unwrap(),
        }
    }
}

impl From<&ComSettings> for SocketAddr {
    fn from(value: &ComSettings) -> Self {
        Self::new(value.address, value.port)
    }
}

mod v1_interop {
    use std::num::TryFromIntError;

    use super::{
        Backend, ComSettings, LoopMode, NonZeroU8, NonZeroU32, PlayerSettings, PodcastSettings,
        PositionYesNo, PositionYesNoLower, RememberLastPosition, ScanDepth, SeekStep,
        ServerSettings, backends::BackendSettings,
    };
    use crate::config::{v1, v2::server::metadata::MetadataSettings};

    impl From<v1::Loop> for LoopMode {
        fn from(value: v1::Loop) -> Self {
            match value {
                v1::Loop::Single => Self::Single,
                v1::Loop::Playlist => Self::Playlist,
                v1::Loop::Random => Self::Random,
            }
        }
    }

    impl From<v1::SeekStep> for SeekStep {
        fn from(value: v1::SeekStep) -> Self {
            match value {
                v1::SeekStep::Short => Self::Both(NonZeroU32::new(5).unwrap()),
                v1::SeekStep::Long => Self::Both(NonZeroU32::new(30).unwrap()),
                v1::SeekStep::Auto => Self::Depends {
                    short_tracks: NonZeroU32::new(5).unwrap(),
                    long_tracks: NonZeroU32::new(30).unwrap(),
                },
            }
        }
    }

    impl From<v1::LastPosition> for RememberLastPosition {
        fn from(value: v1::LastPosition) -> Self {
            match value {
                v1::LastPosition::Yes => Self::All(PositionYesNo::Simple(PositionYesNoLower::Yes)),
                v1::LastPosition::No => Self::All(PositionYesNo::Simple(PositionYesNoLower::No)),
                // "Yes" is already automatic based on MediaType, using this here so that it will get serialized differently than the normal "All-Yes"
                v1::LastPosition::Auto => Self::Depends {
                    music: PositionYesNo::Simple(PositionYesNoLower::No),
                    podcast: PositionYesNo::Simple(PositionYesNoLower::Yes),
                },
            }
        }
    }

    /// Error for when [`ServerSettings`] convertion fails
    #[derive(Debug, Clone, PartialEq, thiserror::Error)]
    pub enum ServerSettingsConvertError {
        /// Recieved a zero value expecting a non-zero value
        #[error(
            "Zero value where expecting a non-zero value. old-config key: '{old_key}', new-config key: '{new_key}', error: {source}"
        )]
        ZeroValue {
            old_key: &'static str,
            new_key: &'static str,
            #[source]
            source: TryFromIntError,
        },
    }

    impl TryFrom<v1::Settings> for ServerSettings {
        type Error = ServerSettingsConvertError;

        #[allow(clippy::cast_possible_truncation)] // checked casts
        fn try_from(value: v1::Settings) -> Result<Self, Self::Error> {
            let com_settings = ComSettings {
                // when coming from v1, continue using http until manually changed
                protocol: super::ComProtocol::HTTP,
                port: value.player_port,
                address: value.player_interface,
                ..Default::default()
            };

            let podcast_settings = PodcastSettings {
                concurrent_downloads_max: NonZeroU8::try_from(
                    value
                        .podcast_simultanious_download
                        .clamp(0, u8::MAX as usize) as u8,
                )
                .map_err(|err| ServerSettingsConvertError::ZeroValue {
                    old_key: "podcast_simultanious_download",
                    new_key: "podcast.concurrent_downloads_max",
                    source: err,
                })?,
                max_download_retries: value.podcast_max_retries.clamp(0, u8::MAX as usize) as u8,
                download_dir: value.podcast_dir,
            };

            let player_settings = PlayerSettings {
                music_dirs: value.music_dir,
                // not converting old scan_depth as that is not stored in the config, but set via CLI, using default instead
                // library_scan_depth: ScanDepth::Limited(value.max_depth_cli),
                library_scan_depth: ScanDepth::Limited(10),
                remember_position: value.player_remember_last_played_position.into(),
                loop_mode: value.player_loop_mode.into(),
                volume: value.player_volume,
                speed: value.player_speed,
                gapless: value.player_gapless,
                seek_step: value.player_seek_step.into(),

                use_mediacontrols: value.player_use_mpris,
                set_discord_status: value.player_use_discord,

                random_track_quantity: NonZeroU32::try_from(
                    value.playlist_select_random_track_quantity,
                )
                .map_err(|err| ServerSettingsConvertError::ZeroValue {
                    old_key: "playlist_select_random_track_quantity",
                    new_key: "player.random_track_quantity",
                    source: err,
                })?,
                random_album_min_quantity: NonZeroU32::try_from(
                    value.playlist_select_random_album_quantity,
                )
                .map_err(|err| ServerSettingsConvertError::ZeroValue {
                    old_key: "playlist_select_random_album_quantity",
                    new_key: "player.random_album_min_quantity",
                    source: err,
                })?,

                backend: Backend::default(),
            };

            Ok(Self {
                com: com_settings,
                player: player_settings,
                podcast: podcast_settings,
                backends: BackendSettings::default(),
                metadata: MetadataSettings::default(),
            })
        }
    }

    #[cfg(test)]
    mod tests {
        use pretty_assertions::assert_eq;
        use std::path::PathBuf;

        use crate::config::v2::server::ComProtocol;

        use super::*;

        #[test]
        fn should_convert_default_without_error() {
            let converted: ServerSettings = v1::Settings::default().try_into().unwrap();
            assert!(converted.podcast.download_dir.components().count() > 0);
            let podcast_settings = {
                let mut set = converted.podcast;
                // ignore this while comparing
                set.download_dir = PathBuf::new();
                set
            };

            assert_eq!(
                podcast_settings,
                PodcastSettings {
                    concurrent_downloads_max: NonZeroU8::new(3).unwrap(),
                    max_download_retries: 3,
                    download_dir: PathBuf::new()
                }
            );

            assert_eq!(
                converted.com,
                ComSettings {
                    protocol: ComProtocol::HTTP,
                    port: 50101,
                    address: "::1".parse().unwrap(),
                    ..Default::default()
                }
            );

            assert!(!converted.player.music_dirs.is_empty());

            let player_settings = {
                let mut set = converted.player;
                // ignore this while comparing
                set.music_dirs.clear();
                set
            };

            assert_eq!(
                player_settings,
                PlayerSettings {
                    music_dirs: Vec::new(),
                    library_scan_depth: ScanDepth::Limited(10),
                    remember_position: RememberLastPosition::Depends {
                        music: PositionYesNo::Simple(PositionYesNoLower::No),
                        podcast: PositionYesNo::Simple(PositionYesNoLower::Yes),
                    },
                    loop_mode: LoopMode::Random,
                    volume: 70,
                    speed: 10,
                    gapless: true,
                    seek_step: SeekStep::Depends {
                        short_tracks: NonZeroU32::new(5).unwrap(),
                        long_tracks: NonZeroU32::new(30).unwrap(),
                    },
                    use_mediacontrols: true,
                    set_discord_status: true,
                    random_track_quantity: NonZeroU32::new(20).unwrap(),
                    random_album_min_quantity: NonZeroU32::new(5).unwrap(),
                    backend: Backend::default(),
                }
            );
        }
    }
}
