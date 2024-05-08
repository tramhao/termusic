use std::path::Path;

use serde::{Deserialize, Serialize};

use super::server::ComSettings;

pub mod config_extra;
pub mod keys;
pub mod theme;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)] // allow missing fields and fill them with the `..Self::default()` in this struct
#[allow(clippy::module_name_repetitions)]
#[derive(Default)]
pub struct TuiSettings {
    pub com: MaybeComSettings,
    pub behavior: BehaviorSettings,
    pub coverart: CoverArtPosition,
    pub symbols: Symbols,
    #[serde(flatten)]
    pub theme: theme::ThemeColorWrap,
    // TODO: what does this property do???
    // pub playlist_display_symbol: bool,
    pub keys: keys::Keys,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BehaviorSettings {
    /// Stop / Exit the Server on TUI quit
    pub quit_server_on_exit: bool,
    /// Ask before exiting the TUI (popup)
    pub confirm_quit: bool,
}

impl Default for BehaviorSettings {
    fn default() -> Self {
        Self {
            quit_server_on_exit: true,
            confirm_quit: true,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum MaybeComSettings {
    ComSettings(ComSettings),
    // Same as server, local, read adjacent server config for configuration
    #[default]
    Same,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)] // allow missing fields and fill them with the `..Self::default()` in this struct
#[derive(Default)]
pub struct CoverArtPosition {
    /// Alignment of the Cover-Art in the tui
    // TODO: clarify whether it is about the whole terminal size or just a specific component
    pub align: Alignment,
    /// Scale of the image
    pub size_scale: i8,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub enum Alignment {
    #[serde(rename = "top right")]
    TopRight,
    #[serde(rename = "top left")]
    TopLeft,
    #[serde(rename = "bottom right")]
    #[default]
    BottomRight,
    #[serde(rename = "bottom left")]
    BottomLeft,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Symbols {
    /// Music Library selected node highlight symbol
    pub library_highlight: String,
    /// Playlist selected track highlight symbol
    pub playlist_highlight: String,
    /// Playlist current playing track symbol
    pub playlist_current_track: String,
}

impl Default for Symbols {
    fn default() -> Self {
        Self {
            library_highlight: "🦄".into(),
            playlist_highlight: "🚀".into(),
            playlist_current_track: "►".into(),
        }
    }
}

pub fn test_save() {
    let path = Path::new("/tmp/termusic_config_tui_save.toml");

    let data = TuiSettings::default();

    config_extra::TuiConfigVersionedDefaulted::save_file(path, &data).unwrap();
}

pub fn test_load() {
    let path = Path::new("/tmp/termusic_config_tui_load.toml");

    let data = config_extra::TuiConfigVersionedDefaulted::from_file(path);

    error!("TEST {:#?}", data);
}
