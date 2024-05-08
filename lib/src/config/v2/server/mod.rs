use std::{
    net::IpAddr,
    num::{NonZeroI16, NonZeroU32, NonZeroU8},
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::track::MediaType;

/// Extra things necessary for a config file, like wrappers for versioning
pub mod config_extra;

pub type MusicDirsOwned = Vec<PathBuf>;

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default)] // allow missing fields and fill them with the `..Self::default()` in this struct
#[allow(clippy::module_name_repetitions)]
pub struct ServerSettings {
    pub com: ComSettings,
    pub player: PlayerSettings,
    pub podcast: PodcastSettings,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
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
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum ScanDepth {
    /// Only go X deep
    // realistically, we dont have any more than u32::MAX depth
    Limited(u32),
    /// Allow going fully down without limit
    Unlimited,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum SeekStep {
    /// Only have one seek-step value
    Both(NonZeroU32),
    /// Have different values depending on track type
    Depends {
        /// Music
        short_tracks: NonZeroU32,
        /// Podcasts
        long_tracks: NonZeroU32,
    },
}

impl SeekStep {
    #[allow(clippy::missing_panics_doc)] // const unwrap
    pub fn default_both() -> Self {
        Self::Both(NonZeroU32::new(5).unwrap())
    }

    #[allow(clippy::missing_panics_doc)] // const unwrap
    pub fn default_depends() -> Self {
        Self::Depends {
            short_tracks: NonZeroU32::new(5).unwrap(),
            long_tracks: NonZeroU32::new(30).unwrap(),
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
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum PositionYesNo {
    /// Simple wrapper to workaround the `"serde(rename_all)" and "serde(untagged)"` problem
    Simple(PositionYesNoLower),
    /// Remember Position after custom time (in seconds)
    YesTime(u64),
}

impl PositionYesNo {
    /// Get the time before saving the track position, if enabled
    pub fn get_time(&self, media_type: MediaType) -> Option<u64> {
        match self {
            PositionYesNo::Simple(v) => match v {
                PositionYesNoLower::Yes => match media_type {
                    MediaType::Music => Some(DEFAULT_YES_TIME_BEFORE_SAVE_MUSIC),
                    MediaType::Podcast => Some(DEFAULT_YES_TIME_BEFORE_SAVE_PODCAST),
                    MediaType::LiveRadio => None,
                },
                PositionYesNoLower::No => None,
            },
            PositionYesNo::YesTime(v) => Some(*v),
        }
    }

    /// Get if the current value means "it is enabled"
    pub fn is_enabled(&self) -> bool {
        match self {
            PositionYesNo::Simple(v) => *v == PositionYesNoLower::Yes,
            PositionYesNo::YesTime(_) => true,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
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
    pub fn get_time(&self, media_type: MediaType) -> Option<u64> {
        match self {
            RememberLastPosition::All(v) => v.get_time(media_type),
            RememberLastPosition::Depends { music, podcast } => match media_type {
                MediaType::Music => music.get_time(media_type),
                MediaType::Podcast => podcast.get_time(media_type),
                MediaType::LiveRadio => None,
            },
        }
    }

    /// Get if remembering for the given [`MediaType`] is enabled or not
    ///
    /// use case is in the restore of the last position
    #[allow(clippy::needless_pass_by_value)] // "MediaType" is a 1-byte copy
    pub fn is_enabled_for(&self, media_type: MediaType) -> bool {
        match self {
            RememberLastPosition::All(v) => v.is_enabled(),
            RememberLastPosition::Depends { music, podcast } => match media_type {
                MediaType::Music => music.is_enabled(),
                MediaType::Podcast => podcast.is_enabled(),
                // liveradio cannot store a position
                MediaType::LiveRadio => false,
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

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)] // allow missing fields and fill them with the `..Self::default()` in this struct
pub struct PlayerSettings {
    /// Music Directories
    pub music_dirs: MusicDirsOwned,
    /// Max depth for music library scanning
    ///
    /// This for example affects how deep the auto-tag extraction will go
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
    pub speed: NonZeroI16,
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
            library_scan_depth: ScanDepth::Limited(10),
            remember_position: RememberLastPosition::default(),

            loop_mode: LoopMode::default(),
            // rather use a lower value than a high so that ears dont get blown off
            volume: 30,
            speed: NonZeroI16::new(10).unwrap(),
            gapless: true,
            seek_step: SeekStep::default(),

            use_mediacontrols: true,
            set_discord_status: true,

            random_track_quantity: NonZeroU32::new(20).unwrap(),
            random_album_min_quantity: NonZeroU32::new(5).unwrap(),
        }
    }
}

/// Playlist loop modes
#[derive(Debug, Clone, Copy, Deserialize, Serialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum LoopMode {
    /// Loop one track
    Single,
    /// Loop the entire Playlist (after last index comes the first)
    #[default]
    Playlist,
    /// Select a random track on each next track
    Random,
}

/// Settings for the gRPC server (and potentially future ways to communicate)
#[derive(Debug, Clone, Deserialize, Serialize)]
// for now, require that both port and ip are specified at once
// #[serde(default)] // allow missing fields and fill them with the `..Self::default()` in this struct
pub struct ComSettings {
    /// gRPC server Port
    pub port: u16,
    /// gRPC server interface / address
    pub address: IpAddr,
}

impl Default for ComSettings {
    fn default() -> Self {
        Self {
            port: 50101,
            address: "::".parse().unwrap(),
        }
    }
}

pub fn test_save() {
    let path = Path::new("/tmp/termusic_config_server_save.toml");

    let data = ServerSettings::default();

    config_extra::ServerConfigVersionedDefaulted::save_file(path, &data).unwrap();
}

pub fn test_load() {
    let path = Path::new("/tmp/termusic_config_server_load.toml");

    let data = config_extra::ServerConfigVersionedDefaulted::from_file(path);

    error!("TEST {:#?}", data);
}
