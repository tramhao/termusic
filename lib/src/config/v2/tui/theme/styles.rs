use serde::{Deserialize, Serialize};
use tuirealm::props::Color;

/// All values correspond to the Theme's selected color for that
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
    LightBlack = 11,
    LightRed = 12,
    LightGreen = 13,
    LightYellow = 14,
    LightBlue = 15,
    LightMagenta = 16,
    LightCyan = 17,
    LightWhite = 18,
}

impl AsRef<str> for ColorTermusic {
    fn as_ref(&self) -> &str {
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

impl ColorTermusic {
    #[must_use]
    pub const fn as_usize(self) -> usize {
        self as usize
    }
}

impl From<ColorTermusic> for Color {
    fn from(value: ColorTermusic) -> Self {
        match value {
            ColorTermusic::Reset | ColorTermusic::Foreground | ColorTermusic::Background => {
                Color::Reset
            }
            ColorTermusic::Black => Color::Black,
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
            ColorTermusic::LightWhite => Color::White,
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
            background_color: ColorTermusic::Reset,
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

    /// If enabled use a symbol for the Loop-Mode, otherwise use text
    ///
    /// Example: true -> "Mode: 🔁"; false -> "Mode: playlist"
    pub use_loop_mode_symbol: bool,
}

impl Default for StylePlaylist {
    fn default() -> Self {
        Self {
            foreground_color: ColorTermusic::Foreground,
            background_color: ColorTermusic::Reset,
            border_color: ColorTermusic::Blue,
            highlight_color: ColorTermusic::LightYellow,

            highlight_symbol: "🚀".into(),
            current_track_symbol: "►".into(),

            use_loop_mode_symbol: true,
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
            background_color: ColorTermusic::Reset,
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
            foreground_color: ColorTermusic::LightBlack,
            background_color: ColorTermusic::Reset,
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
            background_color: ColorTermusic::Reset,
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
            background_color: ColorTermusic::Reset,
            border_color: ColorTermusic::Blue,
            highlight_color: ColorTermusic::LightYellow,
        }
    }
}

mod v1_interop {
    use super::{
        ColorTermusic, StyleFallback, StyleImportantPopup, StyleLibrary, StyleLyric, StylePlaylist,
        StyleProgress, Styles,
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
            let use_loop_mode_symbol = value.playlist_display_symbol;
            let value = &value.style_color_symbol;
            Self {
                foreground_color: value.playlist_foreground.into(),
                background_color: value.playlist_background.into(),
                border_color: value.playlist_border.into(),
                highlight_color: value.playlist_highlight.into(),
                highlight_symbol: value.playlist_highlight_symbol.clone(),
                current_track_symbol: value.currently_playing_track_symbol.clone(),
                use_loop_mode_symbol,
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
                use_loop_mode_symbol: true,
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
