#![allow(clippy::module_name_repetitions)]

use std::{fmt::Display, num::ParseIntError};

use serde::{Deserialize, Serialize};
use tuirealm::props::Color;

use crate::config::{
    v1::AlacrittyColor,
    yaml_theme::{YAMLTheme, YAMLThemeBright, YAMLThemeCursor, YAMLThemeNormal, YAMLThemePrimary},
};

use styles::ColorTermusic;

pub mod styles;

// TODO: combine Theme & Color?

#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq)]
#[serde(default)] // allow missing fields and fill them with the `..Self::default()` in this struct
pub struct ThemeWrap {
    pub style: styles::Styles,
    pub theme: ThemeColors,
}

impl ThemeWrap {
    fn get_color_from_theme(&self, color: ColorTermusic) -> Color {
        match color {
            ColorTermusic::Reset => Color::Reset,
            ColorTermusic::Foreground => self.theme.primary.foreground.into(),
            ColorTermusic::Background => self.theme.primary.background.into(),
            ColorTermusic::Black => self.theme.normal.black.into(),
            ColorTermusic::Red => self.theme.normal.red.into(),
            ColorTermusic::Green => self.theme.normal.green.into(),
            ColorTermusic::Yellow => self.theme.normal.yellow.into(),
            ColorTermusic::Blue => self.theme.normal.blue.into(),
            ColorTermusic::Magenta => self.theme.normal.magenta.into(),
            ColorTermusic::Cyan => self.theme.normal.cyan.into(),
            ColorTermusic::White => self.theme.normal.white.into(),
            ColorTermusic::LightBlack => self.theme.bright.black.into(),
            ColorTermusic::LightRed => self.theme.bright.red.into(),
            ColorTermusic::LightGreen => self.theme.bright.green.into(),
            ColorTermusic::LightYellow => self.theme.bright.yellow.into(),
            ColorTermusic::LightBlue => self.theme.bright.blue.into(),
            ColorTermusic::LightMagenta => self.theme.bright.magenta.into(),
            ColorTermusic::LightCyan => self.theme.bright.cyan.into(),
            ColorTermusic::LightWhite => self.theme.bright.white.into(),
        }
    }

    #[inline]
    pub fn library_foreground(&self) -> Color {
        self.get_color_from_theme(self.style.library.foreground)
    }

    #[inline]
    pub fn library_background(&self) -> Color {
        self.get_color_from_theme(self.style.library.background)
    }

    #[inline]
    pub fn library_highlight(&self) -> Color {
        self.get_color_from_theme(self.style.library.highlight)
    }

    #[inline]
    pub fn library_border(&self) -> Color {
        self.get_color_from_theme(self.style.library.border)
    }

    #[inline]
    pub fn playlist_foreground(&self) -> Color {
        self.get_color_from_theme(self.style.playlist.foreground)
    }

    #[inline]
    pub fn playlist_background(&self) -> Color {
        self.get_color_from_theme(self.style.playlist.background)
    }

    #[inline]
    pub fn playlist_highlight(&self) -> Color {
        self.get_color_from_theme(self.style.playlist.highlight)
    }

    #[inline]
    pub fn playlist_border(&self) -> Color {
        self.get_color_from_theme(self.style.playlist.border)
    }

    #[inline]
    pub fn progress_foreground(&self) -> Color {
        self.get_color_from_theme(self.style.progress.foreground)
    }

    #[inline]
    pub fn progress_background(&self) -> Color {
        self.get_color_from_theme(self.style.progress.background)
    }

    #[inline]
    pub fn progress_border(&self) -> Color {
        self.get_color_from_theme(self.style.progress.border)
    }

    #[inline]
    pub fn lyric_foreground(&self) -> Color {
        self.get_color_from_theme(self.style.lyric.foreground)
    }

    #[inline]
    pub fn lyric_background(&self) -> Color {
        self.get_color_from_theme(self.style.lyric.background)
    }

    #[inline]
    pub fn lyric_border(&self) -> Color {
        self.get_color_from_theme(self.style.lyric.border)
    }

    #[inline]
    pub fn important_popup_foreground(&self) -> Color {
        self.get_color_from_theme(self.style.important_popup.foreground)
    }

    #[inline]
    pub fn important_popup_background(&self) -> Color {
        self.get_color_from_theme(self.style.important_popup.background)
    }

    #[inline]
    pub fn important_popup_border(&self) -> Color {
        self.get_color_from_theme(self.style.important_popup.border)
    }

    #[inline]
    pub fn fallback_foreground(&self) -> Color {
        self.get_color_from_theme(self.style.fallback.foreground)
    }

    #[inline]
    pub fn fallback_background(&self) -> Color {
        self.get_color_from_theme(self.style.fallback.background)
    }

    #[inline]
    pub fn fallback_highlight(&self) -> Color {
        self.get_color_from_theme(self.style.fallback.highlight)
    }

    #[inline]
    pub fn fallback_border(&self) -> Color {
        self.get_color_from_theme(self.style.fallback.border)
    }
}

// TODO: consider upgrading this with "thiserror"
/// Error for when [`ThemeColor`] parsing fails
#[derive(Debug, Clone, PartialEq)]
pub enum ThemeColorParseError {
    ParseIntError(ParseIntError),
    IncorrectLength(usize),
}

impl Display for ThemeColorParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let alternate = f.alternate();
        write!(
            f,
            "Failed to parse color because of {}",
            match self {
                Self::ParseIntError(v) =>
                    if alternate {
                        format!("{v:#}")
                    } else {
                        format!("{v}")
                    },
                Self::IncorrectLength(length) => format!("Incorrect length {length}, expected 6"),
            }
        )
    }
}

/// The rgb colors
#[derive(Debug, Copy, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(try_from = "String")]
#[serde(into = "String")]
pub struct ThemeColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl ThemeColor {
    /// Convert from a prefix + 6 length string
    pub fn from_hex(val: &str) -> Result<Self, ThemeColorParseError> {
        let without_prefix = val.trim_start_matches('#');

        // not in a format we support
        if without_prefix.len() != 6 {
            return Err(ThemeColorParseError::IncorrectLength(without_prefix.len()));
        }

        let r = u8::from_str_radix(&without_prefix[0..=1], 16)
            .map_err(ThemeColorParseError::ParseIntError)?;
        let g = u8::from_str_radix(&without_prefix[2..=3], 16)
            .map_err(ThemeColorParseError::ParseIntError)?;
        let b = u8::from_str_radix(&without_prefix[4..=5], 16)
            .map_err(ThemeColorParseError::ParseIntError)?;

        Ok(Self { r, g, b })
    }

    /// Convert to hex prefix + 6 length string
    #[inline]
    pub fn to_hex(&self) -> String {
        format!("#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
    }
}

impl TryFrom<String> for ThemeColor {
    type Error = ThemeColorParseError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::from_hex(&value)
    }
}

impl TryFrom<&str> for ThemeColor {
    type Error = ThemeColorParseError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::from_hex(value)
    }
}

impl From<AlacrittyColor> for ThemeColor {
    fn from(value: AlacrittyColor) -> Self {
        Self {
            r: value.r,
            g: value.g,
            b: value.b,
        }
    }
}

impl From<ThemeColor> for String {
    fn from(val: ThemeColor) -> Self {
        ThemeColor::to_hex(&val)
    }
}

impl From<ThemeColor> for Color {
    fn from(val: ThemeColor) -> Self {
        Color::Rgb(val.r, val.g, val.b)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(default)] // allow missing fields and fill them with the `..Self::default()` in this struct
pub struct ThemeColors {
    pub name: String,
    pub author: String,
    pub primary: ThemePrimary,
    pub cursor: ThemeCursor,
    pub normal: ThemeNormal,
    pub bright: ThemeBright,
}

impl Default for ThemeColors {
    fn default() -> Self {
        Self {
            name: default_name(),
            author: default_author(),
            primary: ThemePrimary::default(),
            cursor: ThemeCursor::default(),
            normal: ThemeNormal::default(),
            bright: ThemeBright::default(),
        }
    }
}

// TODO: consider upgrading this with "thiserror"
/// Error for when [`ThemeColors`] parsing fails
#[derive(Debug, Clone, PartialEq)]
pub enum ThemeColorsParseError {
    ThemeColor(ThemeColorParseError),
}

impl Display for ThemeColorsParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let alternate = f.alternate();
        write!(
            f,
            "Failed to parse Theme because of {}",
            match self {
                Self::ThemeColor(v) =>
                    if alternate {
                        format!("{v:#}")
                    } else {
                        format!("{v}")
                    },
            }
        )
    }
}

impl From<ThemeColorParseError> for ThemeColorsParseError {
    fn from(value: ThemeColorParseError) -> Self {
        Self::ThemeColor(value)
    }
}

impl TryFrom<YAMLTheme> for ThemeColors {
    type Error = ThemeColorsParseError;

    fn try_from(value: YAMLTheme) -> Result<Self, Self::Error> {
        let colors = value.colors;
        Ok(Self {
            name: colors.name,
            author: colors.author,
            primary: colors.primary.try_into()?,
            cursor: colors.cursor.try_into()?,
            normal: colors.normal.try_into()?,
            bright: colors.bright.try_into()?,
        })
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct ThemePrimary {
    pub background: ThemeColor,
    pub foreground: ThemeColor,
}

impl Default for ThemePrimary {
    fn default() -> Self {
        Self {
            background: default_000(),
            foreground: default_fff(),
        }
    }
}

impl TryFrom<YAMLThemePrimary> for ThemePrimary {
    type Error = ThemeColorsParseError;

    fn try_from(value: YAMLThemePrimary) -> Result<Self, Self::Error> {
        Ok(Self {
            background: value.background.try_into()?,
            foreground: value.foreground.try_into()?,
        })
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(default)] // allow missing fields and fill them with the `..Self::default()` in this struct
pub struct ThemeCursor {
    pub text: ThemeColor,
    pub cursor: ThemeColor,
}

impl Default for ThemeCursor {
    fn default() -> Self {
        Self {
            text: default_fff(),
            cursor: default_fff(),
        }
    }
}

impl TryFrom<YAMLThemeCursor> for ThemeCursor {
    type Error = ThemeColorsParseError;

    fn try_from(value: YAMLThemeCursor) -> Result<Self, Self::Error> {
        Ok(Self {
            text: value.text.try_into()?,
            cursor: value.cursor.try_into()?,
        })
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(default)] // allow missing fields and fill them with the `..Self::default()` in this struct
pub struct ThemeNormal {
    pub black: ThemeColor,
    pub red: ThemeColor,
    pub green: ThemeColor,
    pub yellow: ThemeColor,
    pub blue: ThemeColor,
    pub magenta: ThemeColor,
    pub cyan: ThemeColor,
    pub white: ThemeColor,
}

impl Default for ThemeNormal {
    fn default() -> Self {
        Self {
            black: default_000(),
            red: ThemeColor::from_hex("#ff0000").unwrap(),
            green: ThemeColor::from_hex("#00ff00").unwrap(),
            yellow: ThemeColor::from_hex("#ffff00").unwrap(),
            blue: ThemeColor::from_hex("#0000ff").unwrap(),
            magenta: ThemeColor::from_hex("#ff00ff").unwrap(),
            cyan: ThemeColor::from_hex("#00ffff").unwrap(),
            white: default_fff(),
        }
    }
}

impl TryFrom<YAMLThemeNormal> for ThemeNormal {
    type Error = ThemeColorsParseError;

    fn try_from(value: YAMLThemeNormal) -> Result<Self, Self::Error> {
        Ok(Self {
            black: value.black.try_into()?,
            red: value.red.try_into()?,
            green: value.green.try_into()?,
            yellow: value.yellow.try_into()?,
            blue: value.blue.try_into()?,
            magenta: value.magenta.try_into()?,
            cyan: value.cyan.try_into()?,
            white: value.white.try_into()?,
        })
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(default)] // allow missing fields and fill them with the `..Self::default()` in this struct
pub struct ThemeBright {
    pub black: ThemeColor,
    pub red: ThemeColor,
    pub green: ThemeColor,
    pub yellow: ThemeColor,
    pub blue: ThemeColor,
    pub magenta: ThemeColor,
    pub cyan: ThemeColor,
    pub white: ThemeColor,
}

impl Default for ThemeBright {
    fn default() -> Self {
        Self {
            black: ThemeColor::from_hex("#777777").unwrap(),
            red: default_000(),
            green: default_000(),
            yellow: default_000(),
            blue: default_000(),
            magenta: default_000(),
            cyan: default_000(),
            white: default_000(),
        }
    }
}

impl TryFrom<YAMLThemeBright> for ThemeBright {
    type Error = ThemeColorsParseError;

    fn try_from(value: YAMLThemeBright) -> Result<Self, Self::Error> {
        Ok(Self {
            black: value.black.try_into()?,
            red: value.red.try_into()?,
            green: value.green.try_into()?,
            yellow: value.yellow.try_into()?,
            blue: value.blue.try_into()?,
            magenta: value.magenta.try_into()?,
            cyan: value.cyan.try_into()?,
            white: value.white.try_into()?,
        })
    }
}

#[inline]
fn default_name() -> String {
    "empty name".to_string()
}

#[inline]
fn default_author() -> String {
    "empty author".to_string()
}

#[inline]
fn default_000() -> ThemeColor {
    ThemeColor::from_hex("#000000").unwrap()
}

#[inline]
fn default_fff() -> ThemeColor {
    ThemeColor::from_hex("#FFFFFF").unwrap()
}

mod v1_interop {
    use super::{ThemeBright, ThemeColors, ThemeCursor, ThemeNormal, ThemePrimary, ThemeWrap};
    use crate::config::v1;

    impl From<&v1::Alacritty> for ThemeColors {
        fn from(value: &v1::Alacritty) -> Self {
            Self {
                name: value.name.clone(),
                author: value.author.clone(),
                primary: ThemePrimary {
                    background: value.background.into(),
                    foreground: value.foreground.into(),
                },
                cursor: ThemeCursor {
                    text: value.text.into(),
                    cursor: value.cursor.into(),
                },
                normal: ThemeNormal {
                    black: value.black.into(),
                    red: value.red.into(),
                    green: value.green.into(),
                    yellow: value.yellow.into(),
                    blue: value.blue.into(),
                    magenta: value.magenta.into(),
                    cyan: value.cyan.into(),
                    white: value.white.into(),
                },
                bright: ThemeBright {
                    black: value.light_black.into(),
                    red: value.light_red.into(),
                    green: value.light_green.into(),
                    yellow: value.light_yellow.into(),
                    blue: value.light_blue.into(),
                    magenta: value.light_magenta.into(),
                    cyan: value.light_cyan.into(),
                    white: value.light_white.into(),
                },
            }
        }
    }

    impl From<&v1::Settings> for ThemeWrap {
        fn from(value: &v1::Settings) -> Self {
            Self {
                theme: (&value.style_color_symbol.alacritty_theme).into(),
                style: value.into(),
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn should_convert_default_without_error() {
            let converted: ThemeColors = (&v1::StyleColorSymbol::default().alacritty_theme).into();

            assert_eq!(
                converted,
                ThemeColors {
                    name: "default".into(),
                    author: "Larry Hao".into(),
                    primary: ThemePrimary {
                        background: "#101421".try_into().unwrap(),
                        foreground: "#fffbf6".try_into().unwrap()
                    },
                    cursor: ThemeCursor {
                        text: "#1E1E1E".try_into().unwrap(),
                        cursor: "#FFFFFF".try_into().unwrap()
                    },
                    normal: ThemeNormal {
                        black: "#2e2e2e".try_into().unwrap(),
                        red: "#eb4129".try_into().unwrap(),
                        green: "#abe047".try_into().unwrap(),
                        yellow: "#f6c744".try_into().unwrap(),
                        blue: "#47a0f3".try_into().unwrap(),
                        magenta: "#7b5cb0".try_into().unwrap(),
                        cyan: "#64dbed".try_into().unwrap(),
                        white: "#e5e9f0".try_into().unwrap()
                    },
                    bright: ThemeBright {
                        black: "#565656".try_into().unwrap(),
                        red: "#ec5357".try_into().unwrap(),
                        green: "#c0e17d".try_into().unwrap(),
                        yellow: "#f9da6a".try_into().unwrap(),
                        blue: "#49a4f8".try_into().unwrap(),
                        magenta: "#a47de9".try_into().unwrap(),
                        cyan: "#99faf2".try_into().unwrap(),
                        white: "#ffffff".try_into().unwrap()
                    }
                }
            );
        }
    }
}
