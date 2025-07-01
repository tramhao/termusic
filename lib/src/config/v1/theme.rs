use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::BufReader;
use std::num::ParseIntError;
use std::path::Path;
use tuirealm::props::Color;

use crate::config::yaml_theme::YAMLTheme;

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

impl ColorTermusic {
    pub fn color(self, alacritty_theme: &Alacritty) -> Color {
        match self {
            Self::Foreground => alacritty_theme.foreground.into(),
            Self::Background => alacritty_theme.background.into(),
            Self::Black => alacritty_theme.black.into(),
            Self::Red => alacritty_theme.red.into(),
            Self::Green => alacritty_theme.green.into(),
            Self::Yellow => alacritty_theme.yellow.into(),
            Self::Blue => alacritty_theme.blue.into(),
            Self::Magenta => alacritty_theme.magenta.into(),
            Self::Cyan => alacritty_theme.cyan.into(),
            Self::White => alacritty_theme.white.into(),
            Self::LightBlack => alacritty_theme.light_black.into(),
            Self::LightRed => alacritty_theme.light_red.into(),
            Self::LightGreen => alacritty_theme.light_green.into(),
            Self::LightYellow => alacritty_theme.light_yellow.into(),
            Self::LightBlue => alacritty_theme.light_blue.into(),
            Self::LightMagenta => alacritty_theme.light_magenta.into(),
            Self::LightCyan => alacritty_theme.light_cyan.into(),
            Self::LightWhite => alacritty_theme.light_white.into(),
            Self::Reset => Color::Reset,
        }
    }

    pub const fn as_usize(self) -> usize {
        self as usize
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

impl StyleColorSymbol {
    pub fn library_foreground(&self) -> Color {
        self.library_foreground.color(&self.alacritty_theme)
    }
    pub fn library_background(&self) -> Color {
        self.library_background.color(&self.alacritty_theme)
    }
    pub fn library_highlight(&self) -> Color {
        self.library_highlight.color(&self.alacritty_theme)
    }
    pub fn library_border(&self) -> Color {
        self.library_border.color(&self.alacritty_theme)
    }

    pub fn playlist_foreground(&self) -> Color {
        self.playlist_foreground.color(&self.alacritty_theme)
    }
    pub fn playlist_background(&self) -> Color {
        self.playlist_background.color(&self.alacritty_theme)
    }
    pub fn playlist_highlight(&self) -> Color {
        self.playlist_highlight.color(&self.alacritty_theme)
    }
    pub fn playlist_border(&self) -> Color {
        self.playlist_border.color(&self.alacritty_theme)
    }

    pub fn progress_foreground(&self) -> Color {
        self.progress_foreground.color(&self.alacritty_theme)
    }
    pub fn progress_background(&self) -> Color {
        self.progress_background.color(&self.alacritty_theme)
    }
    pub fn progress_border(&self) -> Color {
        self.progress_border.color(&self.alacritty_theme)
    }

    pub fn lyric_foreground(&self) -> Color {
        self.lyric_foreground.color(&self.alacritty_theme)
    }
    pub fn lyric_background(&self) -> Color {
        self.lyric_background.color(&self.alacritty_theme)
    }
    pub fn lyric_border(&self) -> Color {
        self.lyric_border.color(&self.alacritty_theme)
    }

    pub fn important_popup_foreground(&self) -> Color {
        self.important_popup_foreground.color(&self.alacritty_theme)
    }
    pub fn important_popup_background(&self) -> Color {
        self.important_popup_background.color(&self.alacritty_theme)
    }
    pub fn important_popup_border(&self) -> Color {
        self.important_popup_border.color(&self.alacritty_theme)
    }

    pub fn fallback_foreground(&self) -> Color {
        self.fallback_foreground.color(&self.alacritty_theme)
    }
    pub fn fallback_background(&self) -> Color {
        self.fallback_background.color(&self.alacritty_theme)
    }
    pub fn fallback_highlight(&self) -> Color {
        self.fallback_highlight.color(&self.alacritty_theme)
    }
    pub fn fallback_border(&self) -> Color {
        self.fallback_border.color(&self.alacritty_theme)
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

impl Alacritty {
    /// Convert a [`YAMLTheme`] to this type
    ///
    /// Cannot be a [`From`] implementation because of the additional set `path` parameter
    pub fn from_yaml_theme(value: YAMLTheme, path: String) -> Result<Self, ColorParseError> {
        let colors = value.colors;
        Ok(Alacritty {
            path,
            name: colors.name.unwrap_or_else(default_name),
            author: colors.author.unwrap_or_else(default_name),
            background: colors.primary.background.try_into()?,
            foreground: colors.primary.foreground.try_into()?,
            cursor: colors.cursor.cursor.try_into()?,
            text: colors.cursor.text.try_into()?,
            black: colors.normal.black.try_into()?,
            red: colors.normal.red.try_into()?,
            green: colors.normal.green.try_into()?,
            yellow: colors.normal.yellow.try_into()?,
            blue: colors.normal.blue.try_into()?,
            magenta: colors.normal.magenta.try_into()?,
            cyan: colors.normal.cyan.try_into()?,
            white: colors.normal.white.try_into()?,
            light_black: colors.bright.black.try_into()?,
            light_red: colors.bright.red.try_into()?,
            light_green: colors.bright.green.try_into()?,
            light_yellow: colors.bright.yellow.try_into()?,
            light_blue: colors.bright.blue.try_into()?,
            light_magenta: colors.bright.magenta.try_into()?,
            light_cyan: colors.bright.cyan.try_into()?,
            light_white: colors.bright.white.try_into()?,
        })
    }

    /// Load a YAML Theme and then convert it to a [`Alacritty`] instance
    pub fn from_yaml_file(path: &Path) -> Result<Self> {
        let parsed: YAMLTheme = serde_yaml::from_reader(BufReader::new(File::open(path)?))?;
        let path_str = path.to_string_lossy().to_string();

        Ok(Self::from_yaml_theme(parsed, path_str)?)
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
