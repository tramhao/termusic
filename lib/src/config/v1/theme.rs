use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::num::ParseIntError;
use tuirealm::props::Color;

#[derive(Copy, Clone, Deserialize, Serialize, PartialEq, Eq, Debug)]
pub enum ColorTermusic {
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

impl From<ColorTermusic> for &'static str {
    fn from(cc: ColorTermusic) -> Self {
        match cc {
            ColorTermusic::Reset => "default",
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

impl From<ColorTermusic> for String {
    fn from(cc: ColorTermusic) -> Self {
        <ColorTermusic as Into<&'static str>>::into(cc).to_owned()
    }
}

#[derive(Clone, Deserialize, Serialize, PartialEq, Eq, Debug)]
pub struct StyleColorSymbol {
    pub library_foreground: ColorTermusic,
    pub library_background: ColorTermusic,
    pub library_border: ColorTermusic,
    pub library_highlight: ColorTermusic,
    pub library_highlight_symbol: String,

    pub playlist_foreground: ColorTermusic,
    pub playlist_background: ColorTermusic,
    pub playlist_border: ColorTermusic,
    pub playlist_highlight: ColorTermusic,
    pub playlist_highlight_symbol: String,

    pub progress_foreground: ColorTermusic,
    pub progress_background: ColorTermusic,
    pub progress_border: ColorTermusic,

    pub lyric_foreground: ColorTermusic,
    pub lyric_background: ColorTermusic,
    pub lyric_border: ColorTermusic,

    pub important_popup_foreground: ColorTermusic,
    pub important_popup_background: ColorTermusic,
    pub important_popup_border: ColorTermusic,

    pub fallback_foreground: ColorTermusic,
    pub fallback_background: ColorTermusic,
    pub fallback_border: ColorTermusic,
    pub fallback_highlight: ColorTermusic,

    pub alacritty_theme: Alacritty,
    pub currently_playing_track_symbol: String,
}

impl Default for StyleColorSymbol {
    fn default() -> Self {
        Self {
            library_foreground: ColorTermusic::Foreground,
            library_background: ColorTermusic::Reset,
            library_border: ColorTermusic::Blue,
            library_highlight: ColorTermusic::LightYellow,
            library_highlight_symbol: "\u{1f984}".to_string(),

            playlist_foreground: ColorTermusic::Foreground,
            playlist_background: ColorTermusic::Reset,
            playlist_border: ColorTermusic::Blue,
            playlist_highlight: ColorTermusic::LightYellow,
            playlist_highlight_symbol: "\u{1f680}".to_string(),

            progress_foreground: ColorTermusic::LightBlack,
            progress_background: ColorTermusic::Reset,
            progress_border: ColorTermusic::Blue,

            lyric_foreground: ColorTermusic::Foreground,
            lyric_background: ColorTermusic::Reset,
            lyric_border: ColorTermusic::Blue,

            important_popup_foreground: ColorTermusic::Yellow,
            important_popup_background: ColorTermusic::Reset,
            important_popup_border: ColorTermusic::Yellow,

            fallback_foreground: ColorTermusic::Foreground,
            fallback_background: ColorTermusic::Reset,
            fallback_border: ColorTermusic::Blue,
            fallback_highlight: ColorTermusic::LightYellow,

            alacritty_theme: Alacritty::default(),
            currently_playing_track_symbol: "►".to_string(),
        }
    }
}

/// Error for when [`ThemeColor`] parsing fails
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum ColorParseError {
    #[error("Failed to parse color because of {0}")]
    ParseIntError(#[from] ParseIntError),
    #[error("Failed to parse color. Incorrect length {0}, expected 1(prefix) + 6")]
    IncorrectLength(usize),
}

impl ColorParseError {
    #[cfg(test)]
    fn is_parseint_error(&self) -> bool {
        matches!(self, Self::ParseIntError(_))
    }
}

/// The rgb colors
#[derive(Debug, Copy, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(try_from = "String")]
#[serde(into = "String")]
pub struct AlacrittyColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl AlacrittyColor {
    /// Convert from a prefix + 6 length string
    ///
    /// Example: input `#010101`
    pub fn from_hex(input: &str) -> Result<Self, ColorParseError> {
        let without_prefix = input.trim_start_matches('#');

        // not in a format we support
        if without_prefix.len() != 6 {
            return Err(ColorParseError::IncorrectLength(without_prefix.len()));
        }

        let r = u8::from_str_radix(&without_prefix[0..=1], 16)?;
        let g = u8::from_str_radix(&without_prefix[2..=3], 16)?;
        let b = u8::from_str_radix(&without_prefix[4..=5], 16)?;

        Ok(Self { r, g, b })
    }

    /// Convert to hex prefix + 6 length string
    #[inline]
    pub fn to_hex(self) -> String {
        format!("#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
    }
}

impl TryFrom<String> for AlacrittyColor {
    type Error = ColorParseError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::from_hex(&value)
    }
}

impl TryFrom<&str> for AlacrittyColor {
    type Error = ColorParseError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::from_hex(value)
    }
}

impl From<AlacrittyColor> for String {
    fn from(val: AlacrittyColor) -> Self {
        AlacrittyColor::to_hex(val)
    }
}

impl From<AlacrittyColor> for Color {
    fn from(val: AlacrittyColor) -> Self {
        Color::Rgb(val.r, val.g, val.b)
    }
}

#[inline]
fn default_name() -> String {
    "default".to_string()
}

#[inline]
fn default_author() -> String {
    "Larry Hao".to_string()
}

#[derive(Clone, Deserialize, Serialize, PartialEq, Eq, Debug)]
pub struct Alacritty {
    pub path: String,
    pub name: String,
    pub author: String,
    pub background: AlacrittyColor,
    pub foreground: AlacrittyColor,
    pub cursor: AlacrittyColor,
    pub text: AlacrittyColor,
    pub black: AlacrittyColor,
    pub red: AlacrittyColor,
    pub green: AlacrittyColor,
    pub yellow: AlacrittyColor,
    pub blue: AlacrittyColor,
    pub magenta: AlacrittyColor,
    pub cyan: AlacrittyColor,
    pub white: AlacrittyColor,
    pub light_black: AlacrittyColor,
    pub light_red: AlacrittyColor,
    pub light_green: AlacrittyColor,
    pub light_yellow: AlacrittyColor,
    pub light_blue: AlacrittyColor,
    pub light_magenta: AlacrittyColor,
    pub light_cyan: AlacrittyColor,
    pub light_white: AlacrittyColor,
}

impl Default for Alacritty {
    fn default() -> Self {
        Self {
            path: String::new(),
            name: default_name(),
            author: default_author(),
            background: AlacrittyColor::from_hex("#101421").unwrap(),
            foreground: AlacrittyColor::from_hex("#fffbf6").unwrap(),
            cursor: AlacrittyColor::from_hex("#ffffff").unwrap(),
            text: AlacrittyColor::from_hex("#1e1e1e").unwrap(),
            black: AlacrittyColor::from_hex("#2e2e2e").unwrap(),
            red: AlacrittyColor::from_hex("#eb4129").unwrap(),
            green: AlacrittyColor::from_hex("#abe047").unwrap(),
            yellow: AlacrittyColor::from_hex("#f6c744").unwrap(),
            blue: AlacrittyColor::from_hex("#47a0f3").unwrap(),
            magenta: AlacrittyColor::from_hex("#7b5cb0").unwrap(),
            cyan: AlacrittyColor::from_hex("#64dbed").unwrap(),
            white: AlacrittyColor::from_hex("#e5e9f0").unwrap(),
            light_black: AlacrittyColor::from_hex("#565656").unwrap(),
            light_red: AlacrittyColor::from_hex("#ec5357").unwrap(),
            light_green: AlacrittyColor::from_hex("#c0e17d").unwrap(),
            light_yellow: AlacrittyColor::from_hex("#f9da6a").unwrap(),
            light_blue: AlacrittyColor::from_hex("#49a4f8").unwrap(),
            light_magenta: AlacrittyColor::from_hex("#a47de9").unwrap(),
            light_cyan: AlacrittyColor::from_hex("#99faf2").unwrap(),
            light_white: AlacrittyColor::from_hex("#ffffff").unwrap(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_parse_default_colors() {
        let def = Alacritty::default();
        assert_eq!(Color::from(def.background), Color::Rgb(16, 20, 33));
    }

    #[test]
    fn should_not_parse_incorrect_input() {
        assert_eq!(
            AlacrittyColor::from_hex(""),
            Err(ColorParseError::IncorrectLength(0))
        );
        assert_eq!(
            AlacrittyColor::from_hex("#111"),
            Err(ColorParseError::IncorrectLength(3))
        );
        assert_eq!(
            AlacrittyColor::from_hex("#01010101"),
            Err(ColorParseError::IncorrectLength(8))
        );
        assert!(
            AlacrittyColor::from_hex("#ZZZZZZ")
                .unwrap_err()
                .is_parseint_error()
        );
    }
}
