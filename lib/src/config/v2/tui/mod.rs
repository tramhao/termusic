use std::path::Path;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use super::server::ComSettings;

pub mod config_extra;
pub mod keys;
pub mod theme;

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Default)]
#[serde(default)] // allow missing fields and fill them with the `..Self::default()` in this struct
#[allow(clippy::module_name_repetitions)]
pub struct TuiSettings {
    pub com: MaybeComSettings,
    /// Field that holds the resolved `com` data, in case `same` was used
    #[serde(skip)]
    pub com_resolved: Option<ComSettings>,
    pub behavior: BehaviorSettings,
    pub coverart: CoverArtPosition,
    pub symbols: Symbols,
    #[serde(flatten)]
    pub theme: theme::ThemeColorWrap,
    pub keys: keys::Keys,
}

impl TuiSettings {
    /// Resolve the [`ComSettings`] or directly get them.
    ///
    /// If result is [`Ok`], then `com_resolved` is set and [`Self::get_com`] will always return [`Some`]
    pub fn resolve_com(&mut self, tui_path: &Path) -> Result<()> {
        if self.com_resolved.is_some() {
            return Ok(());
        }

        match self.com {
            MaybeComSettings::ComSettings(ref v) => {
                // this could likely be avoided, but for simplicity this is set
                self.com_resolved = Some(v.clone());
                return Ok(());
            }
            MaybeComSettings::Same => (),
        }

        let server_path = tui_path
            .parent()
            .context("tui_path should have a parent directory")?
            .join(super::server::config_extra::FILE_NAME);

        let server_settings =
            super::server::config_extra::ServerConfigVersionedDefaulted::from_file(server_path)
                .context("parsing server config")?;
        self.com_resolved = Some(server_settings.into_settings().com);

        Ok(())
    }

    /// Get the resolved com-settings, if resolved
    pub fn get_com(&self) -> &Option<ComSettings> {
        &self.com_resolved
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
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

#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum MaybeComSettings {
    ComSettings(ComSettings),
    // Same as server, local, read adjacent server config for configuration
    #[default]
    Same,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(default)] // allow missing fields and fill them with the `..Self::default()` in this struct
#[derive(Default)]
pub struct CoverArtPosition {
    /// Alignment of the Cover-Art in the tui
    // TODO: clarify whether it is about the whole terminal size or just a specific component
    pub align: Alignment,
    /// Scale of the image
    pub size_scale: i8,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq, Eq)]
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

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct Symbols {
    /// Music Library selected node highlight symbol
    pub library_highlight: String,
    /// Playlist selected track highlight symbol
    pub playlist_highlight: String,
    /// Playlist current playing track symbol
    pub playlist_current_track: String,

    /// If enabled use a symbol for the Loop-Mode, otherwise use text
    ///
    /// Example: true -> "Mode: 🔁"; false -> "Mode: playlist"
    pub playlist_loop_mode_symbol: bool,
}

impl Default for Symbols {
    fn default() -> Self {
        Self {
            library_highlight: "🦄".into(),
            playlist_highlight: "🚀".into(),
            playlist_current_track: "►".into(),

            playlist_loop_mode_symbol: true,
        }
    }
}

mod v1_interop {
    use super::{
        Alignment, BehaviorSettings, CoverArtPosition, MaybeComSettings, Symbols, TuiSettings,
    };
    use crate::config::v1;

    impl From<v1::Settings> for Symbols {
        fn from(value: v1::Settings) -> Self {
            let playlist_loop_mode_symbol = value.playlist_display_symbol;
            let value = value.style_color_symbol;

            Self {
                library_highlight: value.library_highlight_symbol,
                playlist_highlight: value.playlist_highlight_symbol,
                playlist_current_track: value.currently_playing_track_symbol,

                playlist_loop_mode_symbol,
            }
        }
    }

    impl From<v1::Alignment> for Alignment {
        fn from(value: v1::Alignment) -> Self {
            match value {
                v1::Alignment::BottomRight => Self::BottomRight,
                v1::Alignment::BottomLeft => Self::BottomLeft,
                v1::Alignment::TopRight => Self::TopRight,
                v1::Alignment::TopLeft => Self::TopLeft,
            }
        }
    }

    #[allow(clippy::cast_possible_truncation)] // clamped casts
    impl From<v1::Xywh> for CoverArtPosition {
        fn from(value: v1::Xywh) -> Self {
            Self {
                align: value.align.into(),
                // the value is named "width", but more use like a scale on both axis
                size_scale: value.width_between_1_100.clamp(0, i8::MAX as u32) as i8,
            }
        }
    }

    impl From<v1::Settings> for TuiSettings {
        fn from(value: v1::Settings) -> Self {
            let settings_clone = value.clone();
            Self {
                // using "same" as the previous config version was a combined config and so only really working for local interop
                com: MaybeComSettings::Same,
                com_resolved: None,
                behavior: BehaviorSettings {
                    quit_server_on_exit: value.kill_daemon_when_quit,
                    confirm_quit: value.enable_exit_confirmation,
                },
                coverart: value.album_photo_xywh.into(),
                symbols: settings_clone.into(),
                theme: value.style_color_symbol.into(),
                keys: value.keys.into(),
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn should_convert_default_without_error() {
            let converted: TuiSettings = v1::Settings::default().into();

            assert_eq!(converted.com, MaybeComSettings::Same);
            assert_eq!(
                converted.behavior,
                BehaviorSettings {
                    quit_server_on_exit: true,
                    confirm_quit: true
                }
            );

            assert_eq!(
                converted.symbols,
                Symbols {
                    library_highlight: "\u{1f984}".into(),
                    playlist_highlight: "\u{1f680}".into(),
                    playlist_current_track: "►".into(),

                    playlist_loop_mode_symbol: true
                }
            );

            assert_eq!(
                converted.coverart,
                CoverArtPosition {
                    align: Alignment::BottomRight,
                    size_scale: 20
                }
            );

            // the following below are already checked in their separate tests and do not need to be repeated
            // assert_eq!(converted.theme, ());
            // assert_eq!(converted.keys, ());
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
