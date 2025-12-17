mod key;
mod theme;

use anyhow::{Context, Result};
pub use key::{BindingForEvent, Keys};
use serde::{Deserialize, Serialize};
use std::{net::IpAddr, path::Path};
use std::{path::PathBuf, sync::LazyLock};
pub use theme::{Alacritty, AlacrittyColor, ColorTermusic, StyleColorSymbol};

/// The filename of the config
pub const FILE_NAME: &str = "config.toml";

static MUSIC_DIR: LazyLock<Vec<PathBuf>> = LazyLock::new(|| {
    let path =
        dirs::audio_dir().unwrap_or_else(|| PathBuf::from(shellexpand::path::tilde("~/Music")));
    let path2 = path.join("mp3");
    Vec::from([path, path2])
});

static PODCAST_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
    dirs::audio_dir()
        .unwrap_or_else(|| PathBuf::from(shellexpand::path::tilde("~/Music")))
        .join("podcast")
});

#[derive(Clone, Copy, Default, Deserialize, Serialize, Debug)]
pub enum Loop {
    Single,
    #[default]
    Playlist,
    Random,
}

#[derive(Clone, Deserialize, Serialize, Debug)]
#[serde(default)] // allow missing fields and fill them with the `..Self::default()` in this struct
pub struct Xywh {
    pub x_between_1_100: u32,
    pub y_between_1_100: u32,
    pub width_between_1_100: u32,
    #[serde(skip)]
    pub x: u32,
    #[serde(skip)]
    pub y: u32,
    #[serde(skip)]
    pub width: u32,
    #[serde(skip)]
    pub height: u32,
    pub align: Alignment,
}

#[derive(Clone, Copy, Deserialize, Serialize, Debug)]
pub enum Alignment {
    BottomRight,
    BottomLeft,
    TopRight,
    TopLeft,
}

impl Default for Xywh {
    fn default() -> Self {
        let width = 20_u32;
        let height = 20_u32;
        let (term_width, term_height) = (50, 70);
        let x = term_width - 1;
        let y = term_height - 9;

        Self {
            x_between_1_100: 100,
            y_between_1_100: 77,
            width_between_1_100: width,
            x,
            y,
            width,
            height,
            align: Alignment::BottomRight,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub enum SeekStep {
    Short,
    Long,
    Auto,
}

impl std::fmt::Display for SeekStep {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let seek_step = match self {
            Self::Short => "short(5 seconds)",
            Self::Long => "long(30 seconds)",
            Self::Auto => "auto(depend on audio length)",
        };
        write!(f, "{seek_step}")
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub enum LastPosition {
    Yes,
    No,
    Auto,
}

impl std::fmt::Display for LastPosition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let save_last_position = match self {
            Self::Yes => "yes",
            Self::No => "no",
            Self::Auto => "auto",
        };
        write!(f, "{save_last_position}")
    }
}

#[derive(Clone, Deserialize, Serialize, Debug)]
#[allow(clippy::struct_excessive_bools)]
#[serde(default)] // allow missing fields and fill them with the `..Self::default()` in this struct
pub struct Settings {
    pub music_dir: Vec<PathBuf>,
    pub player_port: u16,
    pub player_interface: IpAddr,
    pub player_loop_mode: Loop,
    pub player_volume: u16,
    pub player_speed: i32,
    pub player_gapless: bool,
    pub podcast_simultanious_download: usize,
    pub podcast_max_retries: usize,
    pub podcast_dir: PathBuf,
    pub player_seek_step: SeekStep,
    pub player_remember_last_played_position: LastPosition,
    pub enable_exit_confirmation: bool,
    pub playlist_display_symbol: bool,
    pub playlist_select_random_track_quantity: u32,
    pub playlist_select_random_album_quantity: u32,
    pub theme_selected: String,
    pub kill_daemon_when_quit: bool,
    pub player_use_mpris: bool,
    pub player_use_discord: bool,
    pub album_photo_xywh: Xywh,
    pub style_color_symbol: StyleColorSymbol,
    pub keys: Keys,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            music_dir: MUSIC_DIR.clone(),
            player_loop_mode: Loop::Random,
            player_volume: 70,
            player_speed: 10,
            player_gapless: true,
            player_remember_last_played_position: LastPosition::Auto,
            enable_exit_confirmation: true,
            playlist_display_symbol: true,
            keys: Keys::default(),
            theme_selected: "default".to_string(),
            style_color_symbol: StyleColorSymbol::default(),
            album_photo_xywh: Xywh::default(),
            playlist_select_random_track_quantity: 20,
            playlist_select_random_album_quantity: 5,
            podcast_simultanious_download: 3,
            podcast_dir: PODCAST_DIR.clone(),
            podcast_max_retries: 3,
            player_seek_step: SeekStep::Auto,
            kill_daemon_when_quit: true,
            player_use_mpris: true,
            player_use_discord: true,
            player_port: 50101,
            player_interface: "::1".parse().unwrap(),
        }
    }
}

impl Settings {
    /// Load the V1 Config from the given path, not creating it if it does not exist.
    ///
    /// If the path does not exist, returns [`Self::default`].
    pub fn load(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }

        let file_data = std::fs::read_to_string(path).context("Reading v1 Config from file")?;

        toml::from_str(&file_data).context("Parsing v1 Config")
    }
}
