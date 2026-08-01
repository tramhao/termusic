use serde::{Deserialize, Serialize};
use tuirealm::props::Color;

use crate::config::v2::server::LoopMode;

/// Nerd Font icons for loop mode display (from the Nerd Font PUA range).
mod nf_loop_icons {
    pub const TRACK: &str = "\u{f0456}";
    pub const PLAYLIST: &str = "\u{f0458}";
    pub const RANDOM: &str = "\u{f049f}";
    pub const PLAYLIST_ONCE: &str = "\u{f049e}";
}

/// Display mode for loop/shuffle icons in the playlist header.
///
/// Serialized as a string for built-in modes: `"text"`, `"base_symbols"`, `"nerd_font"`
/// or as an object for custom: `{ "track": "...", "playlist": "...", "random": "...", "playlist_once": "..." }`
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum LoopModeDisplay {
    Base(LoopModeDisplayBase),
    Custom(CustomLoopSymbols),
}

impl LoopModeDisplay {
    /// Get the text to display for the given loopmode.
    #[must_use]
    pub fn display(&self, mode: LoopMode) -> &str {
        match self {
            Self::Base(base) => base.display(mode),
            Self::Custom(symbols) => match mode {
                LoopMode::Track => &symbols.track,
                LoopMode::Playlist => &symbols.playlist,
                LoopMode::Random => &symbols.random,
                LoopMode::PlaylistOnce => &symbols.playlist_once,
            },
        }
    }
}

impl Default for LoopModeDisplay {
    fn default() -> Self {
        Self::Base(LoopModeDisplayBase::default())
    }
}

/// User-defined symbols for each loop mode.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(default)]
pub struct CustomLoopSymbols {
    pub track: String,
    pub playlist: String,
    pub random: String,
    pub playlist_once: String,
}

impl Default for CustomLoopSymbols {
    fn default() -> Self {
        Self::from(LoopModeDisplayBase::BaseSymbols)
    }
}

/// Convert a existing [`LoopModeDisplayBase`] to a Custom one to customize individual fields.
impl From<LoopModeDisplayBase> for CustomLoopSymbols {
    fn from(value: LoopModeDisplayBase) -> Self {
        Self {
            track: value.display(LoopMode::Track).to_string(),
            playlist: value.display(LoopMode::Playlist).to_string(),
            random: value.display(LoopMode::Random).to_string(),
            playlist_once: value.display(LoopMode::PlaylistOnce).to_string(),
        }
    }
}

/// Built-in loop mode display modes.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum LoopModeDisplayBase {
    Text,
    #[default]
    BaseSymbols,
    NerdFont,
}

impl LoopModeDisplayBase {
    /// Get the text to display for the given loopmode.
    #[must_use]
    pub fn display(&self, mode: LoopMode) -> &'static str {
        match self {
            LoopModeDisplayBase::Text => mode.display_text(),
            LoopModeDisplayBase::BaseSymbols => match mode {
                LoopMode::Track => "🔂",
                LoopMode::Playlist => "🔁",
                LoopMode::Random => "🔀",
                LoopMode::PlaylistOnce => "⮕",
            },
            LoopModeDisplayBase::NerdFont => match mode {
                LoopMode::Track => nf_loop_icons::TRACK,
                LoopMode::Playlist => nf_loop_icons::PLAYLIST,
                LoopMode::Random => nf_loop_icons::RANDOM,
                LoopMode::PlaylistOnce => nf_loop_icons::PLAYLIST_ONCE,
            },
        }
    }
}

#[derive(Copy, Clone, Deserialize, Serialize, PartialEq, Eq, Debug)]
pub enum ColorTermusic {
    /// Reset to Terminal default (resulting color will depend on what context it is set)
    Reset = 0,
    Foreground = 1,
    Background = 2,
    Black = 3,
    Red = 4,
    Green = 5,
    Yellow = 6,
    Blue = 7,
    Magenta = 8,
    Cyan = 9,
    White = 10,
    /// Also known as "Grey"
    LightBlack = 11,
    LightRed = 12,
    LightGreen = 13,
    LightYellow = 14,
    LightBlue = 15,
    LightMagenta = 16,
    LightCyan = 17,
    LightWhite = 18,
}

impl ColorTermusic {
    /// Same as [`AsRef`], but allowable in `const` and giving static references
    #[must_use]
    pub const fn as_ref_const(self) -> &'static str {
        match self {
            ColorTermusic::Reset => "reset",
            ColorTermusic::Foreground => "foreground",
            ColorTermusic::Background => "background",
            ColorTermusic::Black => "black",
            ColorTermusic::Red => "red",
            ColorTermusic::Green => "green",
            ColorTermusic::Yellow => "yellow",
            ColorTermusic::Blue => "blue",
            ColorTermusic::Magenta => "magenta",
            ColorTermusic::Cyan => "cyan",
            ColorTermusic::White => "white",
            ColorTermusic::LightBlack => "bright_black",
            ColorTermusic::LightRed => "bright_red",
            ColorTermusic::LightGreen => "bright_green",
            ColorTermusic::LightYellow => "bright_yellow",
            ColorTermusic::LightBlue => "bright_blue",
            ColorTermusic::LightMagenta => "bright_magenta",
            ColorTermusic::LightCyan => "bright_cyan",
            ColorTermusic::LightWhite => "bright_white",
        }
    }
}

impl AsRef<str> for ColorTermusic {
    fn as_ref(&self) -> &str {
        self.as_ref_const()
    }
}

impl ColorTermusic {
    #[must_use]
    pub const fn as_usize(self) -> usize {
        self as usize
    }
}

/// Mainly necessary for Native Theme
impl From<ColorTermusic> for Color {
    fn from(value: ColorTermusic) -> Self {
        match value {
            ColorTermusic::Reset => Color::Reset,
            ColorTermusic::Background | ColorTermusic::Black => Color::Black,
            ColorTermusic::Red => Color::Red,
            ColorTermusic::Green => Color::Green,
            ColorTermusic::Yellow => Color::Yellow,
            ColorTermusic::Blue => Color::Blue,
            ColorTermusic::Magenta => Color::Magenta,
            ColorTermusic::Cyan => Color::Cyan,
            ColorTermusic::White => Color::Gray,
            ColorTermusic::LightBlack => Color::DarkGray,
            ColorTermusic::LightRed => Color::LightRed,
            ColorTermusic::LightGreen => Color::LightGreen,
            ColorTermusic::LightYellow => Color::LightYellow,
            ColorTermusic::LightBlue => Color::LightBlue,
            ColorTermusic::LightMagenta => Color::LightMagenta,
            ColorTermusic::LightCyan => Color::LightCyan,
            ColorTermusic::Foreground | ColorTermusic::LightWhite => Color::White,
        }
    }
}

/// Style for the Library view
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
#[serde(default)] // allow missing fields and fill them with the `..Self::default()` in this struct
pub struct Styles {
    pub library: StyleLibrary,
    pub playlist: StylePlaylist,
    pub lyric: StyleLyric,
    pub progress: StyleProgress,
    pub important_popup: StyleImportantPopup,
    pub fallback: StyleFallback,
}

/// Style for the Library view
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(default)] // allow missing fields and fill them with the `..Self::default()` in this struct
pub struct StyleLibrary {
    /// Music Library foreground color (text)
    pub foreground_color: ColorTermusic,
    /// Music Library background color (background)
    pub background_color: ColorTermusic,
    /// Music Library border color (when focused)
    pub border_color: ColorTermusic,
    /// Music Library selected node highlight color
    pub highlight_color: ColorTermusic,

    /// Music Library selected node highlight symbol
    pub highlight_symbol: String,
}

impl Default for StyleLibrary {
    fn default() -> Self {
        Self {
            foreground_color: ColorTermusic::Foreground,
            background_color: ColorTermusic::Background,
            border_color: ColorTermusic::Blue,
            highlight_color: ColorTermusic::LightYellow,

            highlight_symbol: "🦄".into(),
        }
    }
}

/// Style for the Playlist Widget
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(default)] // allow missing fields and fill them with the `..Self::default()` in this struct
pub struct StylePlaylist {
    /// Playlist foreground color (text)
    pub foreground_color: ColorTermusic,
    /// Playlist background color (text)
    pub background_color: ColorTermusic,
    /// Playlist border color (when focused)
    pub border_color: ColorTermusic,
    /// Playlist selected node highlight color
    pub highlight_color: ColorTermusic,

    /// Playlist selected track highlight symbol
    pub highlight_symbol: String,
    /// Playlist current playing track symbol
    pub current_track_symbol: String,

    /// Display mode for loop/shuffle icons in the playlist header.
    pub loop_mode_display: LoopModeDisplay,

    /// Deprecated: reads from `use_loop_mode_symbol` in config for backward compat.
    #[serde(skip_serializing, rename = "use_loop_mode_symbol")]
    pub use_loop_mode_symbol_deprecated: Option<bool>,
}

impl StylePlaylist {
    // TODO: consider adding a migration trait similar to what "Keys" has as "CheckConflict"
    #[must_use]
    pub fn effective_loop_mode_display(&self) -> LoopModeDisplay {
        match self.use_loop_mode_symbol_deprecated {
            Some(true) => LoopModeDisplay::Base(LoopModeDisplayBase::BaseSymbols),
            Some(false) => LoopModeDisplay::Base(LoopModeDisplayBase::Text),
            None => self.loop_mode_display.clone(),
        }
    }
}

impl Default for StylePlaylist {
    fn default() -> Self {
        Self {
            foreground_color: ColorTermusic::Foreground,
            background_color: ColorTermusic::Background,
            border_color: ColorTermusic::Blue,
            highlight_color: ColorTermusic::LightYellow,

            highlight_symbol: "🚀".into(),
            current_track_symbol: "►".into(),

            loop_mode_display: LoopModeDisplay::default(),
            use_loop_mode_symbol_deprecated: None,
        }
    }
}

/// Style for the Lyric text view widget (also the radio text)
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(default)] // allow missing fields and fill them with the `..Self::default()` in this struct
pub struct StyleLyric {
    /// Lyrics foreground color (text)
    pub foreground_color: ColorTermusic,
    /// Lyrics background color (background)
    pub background_color: ColorTermusic,
    /// Lyrics border color (when focused)
    pub border_color: ColorTermusic,
}

impl Default for StyleLyric {
    fn default() -> Self {
        Self {
            foreground_color: ColorTermusic::Foreground,
            background_color: ColorTermusic::Background,
            border_color: ColorTermusic::Blue,
        }
    }
}

/// Style for the Player Progress widget
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(default)] // allow missing fields and fill them with the `..Self::default()` in this struct
pub struct StyleProgress {
    /// Track Progressbar foreground color (text)
    pub foreground_color: ColorTermusic,
    /// Track Progressbar background color (background)
    pub background_color: ColorTermusic,
    /// Track Progressbar border (always)
    pub border_color: ColorTermusic,
}

impl Default for StyleProgress {
    fn default() -> Self {
        Self {
            foreground_color: ColorTermusic::Foreground,
            background_color: ColorTermusic::Background,
            border_color: ColorTermusic::Blue,
        }
    }
}

/// Style for Important Popups (quit, save config, delete, NOT Error)
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(default)] // allow missing fields and fill them with the `..Self::default()` in this struct
pub struct StyleImportantPopup {
    /// Important Popup (like Error or Delete) foreground color (text)
    pub foreground_color: ColorTermusic,
    /// Important Popup (like Error or Delete) background color (background)
    pub background_color: ColorTermusic,
    /// Important Popup (like Error or Delete) border color (always)
    pub border_color: ColorTermusic,
}

impl Default for StyleImportantPopup {
    fn default() -> Self {
        Self {
            foreground_color: ColorTermusic::Yellow,
            background_color: ColorTermusic::Background,
            border_color: ColorTermusic::Yellow,
        }
    }
}

/// Generic is when there is no specific config entry for it, like the `AskQuit` popup
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(default)] // allow missing fields and fill them with the `..Self::default()` in this struct
pub struct StyleFallback {
    /// Generic foreground color (text)
    pub foreground_color: ColorTermusic,
    /// Generic background color (background)
    pub background_color: ColorTermusic,
    /// Generic border color (always)
    pub border_color: ColorTermusic,
    /// Generic Highlight color
    pub highlight_color: ColorTermusic,
}

impl Default for StyleFallback {
    fn default() -> Self {
        Self {
            foreground_color: ColorTermusic::Foreground,
            background_color: ColorTermusic::Background,
            border_color: ColorTermusic::Blue,
            highlight_color: ColorTermusic::LightYellow,
        }
    }
}

#[cfg(feature = "config-v1-compat")]
mod v1_interop {
    use super::{
        ColorTermusic, LoopModeDisplay, LoopModeDisplayBase, StyleFallback, StyleImportantPopup,
        StyleLibrary, StyleLyric, StylePlaylist, StyleProgress, Styles,
    };
    use crate::config::v1;

    impl From<v1::ColorTermusic> for ColorTermusic {
        fn from(value: v1::ColorTermusic) -> Self {
            match value {
                v1::ColorTermusic::Reset => Self::Reset,
                v1::ColorTermusic::Foreground => Self::Foreground,
                v1::ColorTermusic::Background => Self::Background,
                v1::ColorTermusic::Black => Self::Black,
                v1::ColorTermusic::Red => Self::Red,
                v1::ColorTermusic::Green => Self::Green,
                v1::ColorTermusic::Yellow => Self::Yellow,
                v1::ColorTermusic::Blue => Self::Blue,
                v1::ColorTermusic::Magenta => Self::Magenta,
                v1::ColorTermusic::Cyan => Self::Cyan,
                v1::ColorTermusic::White => Self::White,
                v1::ColorTermusic::LightBlack => Self::LightBlack,
                v1::ColorTermusic::LightRed => Self::LightRed,
                v1::ColorTermusic::LightGreen => Self::LightGreen,
                v1::ColorTermusic::LightYellow => Self::LightYellow,
                v1::ColorTermusic::LightBlue => Self::LightBlue,
                v1::ColorTermusic::LightMagenta => Self::LightMagenta,
                v1::ColorTermusic::LightCyan => Self::LightCyan,
                v1::ColorTermusic::LightWhite => Self::LightWhite,
            }
        }
    }

    impl From<&v1::StyleColorSymbol> for StyleLibrary {
        fn from(value: &v1::StyleColorSymbol) -> Self {
            Self {
                foreground_color: value.library_foreground.into(),
                background_color: value.library_background.into(),
                border_color: value.library_border.into(),
                highlight_color: value.library_highlight.into(),

                highlight_symbol: value.library_highlight_symbol.clone(),
            }
        }
    }

    impl From<&v1::Settings> for StylePlaylist {
        fn from(value: &v1::Settings) -> Self {
            let loop_mode_display = if value.playlist_display_symbol {
                LoopModeDisplay::Base(LoopModeDisplayBase::BaseSymbols)
            } else {
                LoopModeDisplay::Base(LoopModeDisplayBase::Text)
            };
            let value = &value.style_color_symbol;
            Self {
                foreground_color: value.playlist_foreground.into(),
                background_color: value.playlist_background.into(),
                border_color: value.playlist_border.into(),
                highlight_color: value.playlist_highlight.into(),
                highlight_symbol: value.playlist_highlight_symbol.clone(),
                current_track_symbol: value.currently_playing_track_symbol.clone(),
                loop_mode_display,
                use_loop_mode_symbol_deprecated: None,
            }
        }
    }

    impl From<&v1::StyleColorSymbol> for StyleLyric {
        fn from(value: &v1::StyleColorSymbol) -> Self {
            Self {
                foreground_color: value.lyric_foreground.into(),
                background_color: value.lyric_background.into(),
                border_color: value.lyric_border.into(),
            }
        }
    }

    impl From<&v1::StyleColorSymbol> for StyleProgress {
        fn from(value: &v1::StyleColorSymbol) -> Self {
            Self {
                foreground_color: value.progress_foreground.into(),
                background_color: value.progress_background.into(),
                border_color: value.progress_border.into(),
            }
        }
    }

    impl From<&v1::StyleColorSymbol> for StyleImportantPopup {
        fn from(value: &v1::StyleColorSymbol) -> Self {
            Self {
                foreground_color: value.important_popup_foreground.into(),
                background_color: value.important_popup_background.into(),
                border_color: value.important_popup_border.into(),
            }
        }
    }

    impl From<&v1::StyleColorSymbol> for StyleFallback {
        fn from(value: &v1::StyleColorSymbol) -> Self {
            Self {
                foreground_color: value.fallback_foreground.into(),
                background_color: value.fallback_background.into(),
                border_color: value.fallback_border.into(),
                highlight_color: value.fallback_highlight.into(),
            }
        }
    }

    impl From<&v1::Settings> for Styles {
        fn from(value: &v1::Settings) -> Self {
            let playlist = value.into();
            let value = &value.style_color_symbol;
            Self {
                library: value.into(),
                playlist,
                lyric: value.into(),
                progress: value.into(),
                important_popup: value.into(),
                fallback: value.into(),
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn should_convert_default_without_error() {
            let converted: Styles = (&v1::Settings::default()).into();

            let expected_library = StyleLibrary {
                foreground_color: ColorTermusic::Foreground,
                background_color: ColorTermusic::Reset,
                border_color: ColorTermusic::Blue,
                highlight_color: ColorTermusic::LightYellow,

                highlight_symbol: "🦄".into(),
            };
            assert_eq!(converted.library, expected_library);

            let expected_playlist = StylePlaylist {
                foreground_color: ColorTermusic::Foreground,
                background_color: ColorTermusic::Reset,
                border_color: ColorTermusic::Blue,
                highlight_color: ColorTermusic::LightYellow,

                highlight_symbol: "🚀".into(),
                current_track_symbol: "►".into(),
                loop_mode_display: LoopModeDisplay::Base(LoopModeDisplayBase::BaseSymbols),
                use_loop_mode_symbol_deprecated: None,
            };
            assert_eq!(converted.playlist, expected_playlist);

            let expected_lyric = StyleLyric {
                foreground_color: ColorTermusic::Foreground,
                background_color: ColorTermusic::Reset,
                border_color: ColorTermusic::Blue,
            };
            assert_eq!(converted.lyric, expected_lyric);

            let expected_progress = StyleProgress {
                foreground_color: ColorTermusic::LightBlack,
                background_color: ColorTermusic::Reset,
                border_color: ColorTermusic::Blue,
            };
            assert_eq!(converted.progress, expected_progress);

            let expected_important_popup = StyleImportantPopup {
                foreground_color: ColorTermusic::Yellow,
                background_color: ColorTermusic::Reset,
                border_color: ColorTermusic::Yellow,
            };
            assert_eq!(converted.important_popup, expected_important_popup);

            let expected_fallback = StyleFallback {
                foreground_color: ColorTermusic::Foreground,
                background_color: ColorTermusic::Reset,
                border_color: ColorTermusic::Blue,
                highlight_color: ColorTermusic::LightYellow,
            };
            assert_eq!(converted.fallback, expected_fallback);

            assert_eq!(
                converted,
                Styles {
                    library: expected_library,
                    playlist: expected_playlist,
                    lyric: expected_lyric,
                    progress: expected_progress,
                    important_popup: expected_important_popup,
                    fallback: expected_fallback
                }
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_serde_custom_loop_symbols_with_partial_config() {
        let input = r#"{"track":"A"}"#;
        let symbols: CustomLoopSymbols = serde_json::from_str(input).unwrap();
        assert_eq!(symbols.track, "A");
        assert_eq!(symbols.playlist, CustomLoopSymbols::default().playlist);
        assert_eq!(symbols.random, CustomLoopSymbols::default().random);
        assert_eq!(
            symbols.playlist_once,
            CustomLoopSymbols::default().playlist_once
        );
    }

    #[test]
    fn should_convert_base_display_to_custom_symbols() {
        let nerd_font: CustomLoopSymbols = LoopModeDisplayBase::NerdFont.into();
        assert_eq!(nerd_font.track, nf_loop_icons::TRACK);
        assert_eq!(nerd_font.playlist, nf_loop_icons::PLAYLIST);
        assert_eq!(nerd_font.random, nf_loop_icons::RANDOM);
        assert_eq!(nerd_font.playlist_once, nf_loop_icons::PLAYLIST_ONCE);
    }
}
