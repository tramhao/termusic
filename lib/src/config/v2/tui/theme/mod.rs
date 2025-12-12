#![allow(clippy::module_name_repetitions)]

use std::{fs::File, io::BufReader, num::ParseIntError, path::Path};

use serde::{Deserialize, Serialize};
use tuirealm::props::Color;

use crate::config::yaml_theme::{
    YAMLTheme, YAMLThemeBright, YAMLThemeCursor, YAMLThemeNormal, YAMLThemePrimary,
};

use styles::ColorTermusic;

pub mod styles;

// TODO: combine Theme & Color?

#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq)]
#[serde(default)] // allow missing fields and fill them with the `..Self::default()` in this struct
pub struct ThemeWrap {
    pub style: styles::Styles,
    // On full-on default, also set the names to "Termusic Default"
    // this function is only used if this property does not exist at all
    #[serde(default = "ThemeColors::full_native")]
    pub theme: ThemeColors,
}

impl ThemeWrap {
    /// Resolve the given [`ColorTermusic`] to a display-able [`(ratatui)Color`](tuirealm::props::Color).
    ///
    /// This first resolves the corresponding [`ThemeColor`] from the given [`ColorTermusic`],
    /// then resolved that [`ThemeColor`] to a display-able [`(ratatui)Color`](tuirealm::props::Color).
    #[must_use]
    pub fn get_color_from_theme(&self, color: ColorTermusic) -> Color {
        // first step to get the theme path of what color to use
        let val = match color {
            ColorTermusic::Reset => return Color::Reset,
            ColorTermusic::Foreground => &self.theme.primary.foreground,
            ColorTermusic::Background => &self.theme.primary.background,
            ColorTermusic::Black => &self.theme.normal.black,
            ColorTermusic::Red => &self.theme.normal.red,
            ColorTermusic::Green => &self.theme.normal.green,
            ColorTermusic::Yellow => &self.theme.normal.yellow,
            ColorTermusic::Blue => &self.theme.normal.blue,
            ColorTermusic::Magenta => &self.theme.normal.magenta,
            ColorTermusic::Cyan => &self.theme.normal.cyan,
            ColorTermusic::White => &self.theme.normal.white,
            ColorTermusic::LightBlack => &self.theme.bright.black,
            ColorTermusic::LightRed => &self.theme.bright.red,
            ColorTermusic::LightGreen => &self.theme.bright.green,
            ColorTermusic::LightYellow => &self.theme.bright.yellow,
            ColorTermusic::LightBlue => &self.theme.bright.blue,
            ColorTermusic::LightMagenta => &self.theme.bright.magenta,
            ColorTermusic::LightCyan => &self.theme.bright.cyan,
            ColorTermusic::LightWhite => &self.theme.bright.white,
        };

        // finally resolve if that theme color is native or a rgb(hex) value
        val.resolve_color(color)
    }

    #[inline]
    #[must_use]
    pub fn library_foreground(&self) -> Color {
        self.get_color_from_theme(self.style.library.foreground_color)
    }

    #[inline]
    #[must_use]
    pub fn library_background(&self) -> Color {
        self.get_color_from_theme(self.style.library.background_color)
    }

    #[inline]
    #[must_use]
    pub fn library_highlight(&self) -> Color {
        self.get_color_from_theme(self.style.library.highlight_color)
    }

    #[inline]
    #[must_use]
    pub fn library_border(&self) -> Color {
        self.get_color_from_theme(self.style.library.border_color)
    }

    #[inline]
    #[must_use]
    pub fn playlist_foreground(&self) -> Color {
        self.get_color_from_theme(self.style.playlist.foreground_color)
    }

    #[inline]
    #[must_use]
    pub fn playlist_background(&self) -> Color {
        self.get_color_from_theme(self.style.playlist.background_color)
    }

    #[inline]
    #[must_use]
    pub fn playlist_highlight(&self) -> Color {
        self.get_color_from_theme(self.style.playlist.highlight_color)
    }

    #[inline]
    #[must_use]
    pub fn playlist_border(&self) -> Color {
        self.get_color_from_theme(self.style.playlist.border_color)
    }

    #[inline]
    #[must_use]
    pub fn progress_foreground(&self) -> Color {
        self.get_color_from_theme(self.style.progress.foreground_color)
    }

    #[inline]
    #[must_use]
    pub fn progress_background(&self) -> Color {
        self.get_color_from_theme(self.style.progress.background_color)
    }

    #[inline]
    #[must_use]
    pub fn progress_border(&self) -> Color {
        self.get_color_from_theme(self.style.progress.border_color)
    }

    #[inline]
    #[must_use]
    pub fn lyric_foreground(&self) -> Color {
        self.get_color_from_theme(self.style.lyric.foreground_color)
    }

    #[inline]
    #[must_use]
    pub fn lyric_background(&self) -> Color {
        self.get_color_from_theme(self.style.lyric.background_color)
    }

    #[inline]
    #[must_use]
    pub fn lyric_border(&self) -> Color {
        self.get_color_from_theme(self.style.lyric.border_color)
    }

    #[inline]
    #[must_use]
    pub fn important_popup_foreground(&self) -> Color {
        self.get_color_from_theme(self.style.important_popup.foreground_color)
    }

    #[inline]
    #[must_use]
    pub fn important_popup_background(&self) -> Color {
        self.get_color_from_theme(self.style.important_popup.background_color)
    }

    #[inline]
    #[must_use]
    pub fn important_popup_border(&self) -> Color {
        self.get_color_from_theme(self.style.important_popup.border_color)
    }

    #[inline]
    #[must_use]
    pub fn fallback_foreground(&self) -> Color {
        self.get_color_from_theme(self.style.fallback.foreground_color)
    }

    #[inline]
    #[must_use]
    pub fn fallback_background(&self) -> Color {
        self.get_color_from_theme(self.style.fallback.background_color)
    }

    #[inline]
    #[must_use]
    pub fn fallback_highlight(&self) -> Color {
        self.get_color_from_theme(self.style.fallback.highlight_color)
    }

    #[inline]
    #[must_use]
    pub fn fallback_border(&self) -> Color {
        self.get_color_from_theme(self.style.fallback.border_color)
    }
}

/// Error for when [`ThemeColor`] parsing fails
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum ThemeColorParseError {
    #[error("Failed to parse hex color: {0}")]
    HexParseError(#[from] ThemeColorHexParseError),
    #[error("Failed to parse color, expected prefix \"#\" or \"0x\" and length 6 or \"native\"")]
    UnknownValue(String),
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(try_from = "String")]
#[serde(into = "String")]
pub enum ThemeColor {
    /// Native theme, let the terminal decide the colors
    Native,
    /// Set explicit RGB colors
    Hex(ThemeColorHex),
}

impl ThemeColor {
    /// Create a new hex instance with the given values.
    ///
    /// Those values are explicit as this means it can be known to work (unlike a string).
    #[must_use]
    pub const fn new_hex(r: u8, g: u8, b: u8) -> Self {
        Self::Hex(ThemeColorHex::new(r, g, b))
    }

    /// Create a new instance for native colors.
    #[must_use]
    pub const fn new_native() -> Self {
        Self::Native
    }

    /// Try to parse a hex or "native" string.
    fn from_string(val: &str) -> Result<Self, ThemeColorParseError> {
        if val == "native" {
            return Ok(Self::Native);
        }

        let res = match ThemeColorHex::try_from(val) {
            Ok(v) => v,
            Err(err) => {
                return Err(match err {
                    // map unknown prefix error to more descriptive error for native / hex
                    ThemeColorHexParseError::UnknownPrefix(_) => {
                        ThemeColorParseError::UnknownValue(val.to_string())
                    }
                    v => ThemeColorParseError::HexParseError(v),
                });
            }
        };

        Ok(Self::Hex(res))
    }

    /// Output the current value as its string representation.
    #[expect(clippy::inherent_to_string)] // not wanting to implement "Display"
    fn to_string(self) -> String {
        match self {
            ThemeColor::Native => "native".to_string(),
            ThemeColor::Hex(theme_color_hex) => theme_color_hex.to_hex(),
        }
    }

    /// Resolve the current instance to either native coloring (requires `style`) or a rgb color.
    #[must_use]
    pub fn resolve_color(&self, style: ColorTermusic) -> Color {
        let hex = match self {
            ThemeColor::Native => return style.into(),
            ThemeColor::Hex(theme_color_hex) => theme_color_hex,
        };

        (*hex).into()
    }
}

impl TryFrom<String> for ThemeColor {
    type Error = ThemeColorParseError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::from_string(&value)
    }
}

impl TryFrom<&str> for ThemeColor {
    type Error = ThemeColorParseError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::from_string(value)
    }
}

impl From<ThemeColor> for String {
    fn from(val: ThemeColor) -> Self {
        ThemeColor::to_string(val)
    }
}

/// Error for when [`ThemeColor`] parsing fails
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum ThemeColorHexParseError {
    #[error("Failed to parse color because of {0}")]
    ParseIntError(#[from] ParseIntError),
    #[error(
        "Failed to parse color because of incorrect length {0}, expected prefix \"#\" or \"0x\" and length 6"
    )]
    IncorrectLength(usize),
    #[error("Failed to parse color becazse of unknown prefix \"{0}\", expected \"#\" or \"0x\"")]
    UnknownPrefix(String),
}

/// The rgb colors
#[derive(Debug, Copy, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(try_from = "String")]
#[serde(into = "String")]
pub struct ThemeColorHex {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl ThemeColorHex {
    /// Create a new instance with those values
    #[must_use]
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    /// Convert from a prefix + 6 length string
    pub fn from_hex(val: &str) -> Result<Self, ThemeColorHexParseError> {
        let Some(without_prefix) = val.strip_prefix('#').or(val.strip_prefix("0x")) else {
            return Err(ThemeColorHexParseError::UnknownPrefix(val.to_string()));
        };

        // not in a format we support
        if without_prefix.len() != 6 {
            return Err(ThemeColorHexParseError::IncorrectLength(
                without_prefix.len(),
            ));
        }

        let r = u8::from_str_radix(&without_prefix[0..=1], 16)
            .map_err(ThemeColorHexParseError::ParseIntError)?;
        let g = u8::from_str_radix(&without_prefix[2..=3], 16)
            .map_err(ThemeColorHexParseError::ParseIntError)?;
        let b = u8::from_str_radix(&without_prefix[4..=5], 16)
            .map_err(ThemeColorHexParseError::ParseIntError)?;

        Ok(Self { r, g, b })
    }

    /// Convert to hex prefix + 6 length string
    #[inline]
    #[must_use]
    pub fn to_hex(&self) -> String {
        format!("#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
    }
}

impl TryFrom<String> for ThemeColorHex {
    type Error = ThemeColorHexParseError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::from_hex(&value)
    }
}

impl TryFrom<&str> for ThemeColorHex {
    type Error = ThemeColorHexParseError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::from_hex(value)
    }
}

impl From<ThemeColorHex> for String {
    fn from(val: ThemeColorHex) -> Self {
        ThemeColorHex::to_hex(&val)
    }
}

impl From<ThemeColorHex> for Color {
    fn from(val: ThemeColorHex) -> Self {
        Color::Rgb(val.r, val.g, val.b)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(default)] // allow missing fields and fill them with the `..Self::default()` in this struct
pub struct ThemeColors {
    /// The Filename of the current theme, if a file is used.
    /// This value is skipped if empty.
    ///
    /// This is used for example to pre-select in the config editor if available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_name: Option<String>,
    pub name: String,
    pub author: String,
    pub primary: ThemePrimary,
    pub cursor: ThemeCursor,
    pub normal: ThemeNormal,
    pub bright: ThemeBright,
}

impl Default for ThemeColors {
    fn default() -> Self {
        Self::full_native()
    }
}

impl ThemeColors {
    /// Get the full default theme, including names.
    ///
    /// This function is different from [`Self::default`] as the trait impl is also used for filling empty places
    #[must_use]
    pub fn full_default() -> Self {
        Self {
            file_name: None,
            name: "Termusic Default".to_string(),
            author: "Termusic Developers".to_string(),
            primary: ThemePrimary::default(),
            cursor: ThemeCursor::default(),
            normal: ThemeNormal::default(),
            bright: ThemeBright::default(),
        }
    }

    /// Get a full native theme.
    #[must_use]
    pub fn full_native() -> Self {
        Self {
            file_name: None,
            name: "Native".to_string(),
            author: "Termusic Developers".to_string(),
            primary: ThemePrimary::native(),
            cursor: ThemeCursor::native(),
            normal: ThemeNormal::native(),
            bright: ThemeBright::native(),
        }
    }
}

/// Error for when [`ThemeColors`] parsing fails
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum ThemeColorsParseError {
    #[error("Failed to parse Theme: {0}")]
    ThemeColor(#[from] ThemeColorParseError),
}

impl TryFrom<YAMLTheme> for ThemeColors {
    type Error = ThemeColorsParseError;

    fn try_from(value: YAMLTheme) -> Result<Self, Self::Error> {
        let colors = value.colors;
        Ok(Self {
            file_name: None,
            name: colors.name.unwrap_or_else(default_name),
            author: colors.author.unwrap_or_else(default_author),
            primary: colors.primary.try_into()?,
            cursor: colors.cursor.try_into()?,
            normal: colors.normal.try_into()?,
            bright: colors.bright.try_into()?,
        })
    }
}

impl ThemeColors {
    /// Load a YAML Theme and then convert it to a [`ThemeColors`] instance
    pub fn from_yaml_file(path: &Path) -> anyhow::Result<Self> {
        let parsed: YAMLTheme = serde_yaml::from_reader(BufReader::new(File::open(path)?))?;

        let mut theme = Self::try_from(parsed)?;

        let file_name = path.file_stem();
        theme.file_name = file_name.map(|v| v.to_string_lossy().to_string());

        Ok(theme)
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
            background: ThemeColor::new_hex(0x10, 0x14, 0x21),
            foreground: ThemeColor::new_hex(0xff, 0xfb, 0xf6),
        }
    }
}

impl ThemePrimary {
    fn native() -> Self {
        Self {
            background: ThemeColor::new_native(),
            foreground: ThemeColor::new_native(),
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
            text: ThemeColor::new_hex(0x1e, 0x1e, 0x1e),
            cursor: default_fff(),
        }
    }
}

impl ThemeCursor {
    fn native() -> Self {
        Self {
            text: ThemeColor::new_native(),
            cursor: ThemeColor::new_native(),
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
            black: ThemeColor::new_hex(0x2e, 0x2e, 0x2e),
            red: ThemeColor::new_hex(0xeb, 0x41, 0x29),
            green: ThemeColor::new_hex(0xab, 0xe0, 0x47),
            yellow: ThemeColor::new_hex(0xf6, 0xc7, 0x44),
            blue: ThemeColor::new_hex(0x47, 0xa0, 0xf3),
            magenta: ThemeColor::new_hex(0x7b, 0x5c, 0xb0),
            cyan: ThemeColor::new_hex(0x64, 0xdb, 0xed),
            white: ThemeColor::new_hex(0xe5, 0xe9, 0xf0),
        }
    }
}

impl ThemeNormal {
    fn native() -> Self {
        Self {
            black: ThemeColor::new_native(),
            red: ThemeColor::new_native(),
            green: ThemeColor::new_native(),
            yellow: ThemeColor::new_native(),
            blue: ThemeColor::new_native(),
            magenta: ThemeColor::new_native(),
            cyan: ThemeColor::new_native(),
            white: ThemeColor::new_native(),
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
            black: ThemeColor::new_hex(0x56, 0x56, 0x56),
            red: ThemeColor::new_hex(0xec, 0x53, 0x57),
            green: ThemeColor::new_hex(0xc0, 0xe1, 0x7d),
            yellow: ThemeColor::new_hex(0xf9, 0xda, 0x6a),
            blue: ThemeColor::new_hex(0x49, 0xa4, 0xf8),
            magenta: ThemeColor::new_hex(0xa4, 0x7d, 0xe9),
            cyan: ThemeColor::new_hex(0x99, 0xfa, 0xf2),
            white: default_fff(),
        }
    }
}

impl ThemeBright {
    fn native() -> Self {
        Self {
            black: ThemeColor::new_native(),
            red: ThemeColor::new_native(),
            green: ThemeColor::new_native(),
            yellow: ThemeColor::new_native(),
            blue: ThemeColor::new_native(),
            magenta: ThemeColor::new_native(),
            cyan: ThemeColor::new_native(),
            white: ThemeColor::new_native(),
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
fn default_fff() -> ThemeColor {
    ThemeColor::new_hex(0xFF, 0xFF, 0xFF)
}

mod v1_interop {
    use super::{
        ThemeBright, ThemeColor, ThemeColorHex, ThemeColors, ThemeCursor, ThemeNormal,
        ThemePrimary, ThemeWrap,
    };
    use crate::config::v1;

    impl From<v1::AlacrittyColor> for ThemeColorHex {
        fn from(value: v1::AlacrittyColor) -> Self {
            Self {
                r: value.r,
                g: value.g,
                b: value.b,
            }
        }
    }

    impl From<v1::AlacrittyColor> for ThemeColor {
        fn from(value: v1::AlacrittyColor) -> Self {
            Self::Hex(value.into())
        }
    }

    impl From<&v1::Alacritty> for ThemeColors {
        fn from(value: &v1::Alacritty) -> Self {
            Self {
                file_name: None,
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
                    file_name: None,
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

#[cfg(test)]
mod tests {
    use super::ThemeColors;

    mod theme_color {
        use super::super::ThemeColor;

        #[test]
        fn should_parse_hex() {
            assert_eq!(
                ThemeColor::new_hex(1, 2, 3),
                ThemeColor::try_from("#010203").unwrap()
            );
            assert_eq!(
                ThemeColor::new_hex(1, 2, 3),
                ThemeColor::try_from("0x010203").unwrap()
            );
        }

        #[test]
        fn should_parse_native() {
            assert_eq!(
                ThemeColor::new_native(),
                ThemeColor::try_from("native").unwrap()
            );
        }

        #[test]
        fn should_serialize() {
            assert_eq!(ThemeColor::new_hex(1, 2, 3).to_string(), "#010203");
            assert_eq!(ThemeColor::new_native().to_string(), "native");
        }
    }

    mod theme_color_hex {
        use super::super::ThemeColorHex;

        #[test]
        fn should_parse_hashtag() {
            assert_eq!(
                ThemeColorHex::new(1, 2, 3),
                ThemeColorHex::from_hex("#010203").unwrap()
            );
            assert_eq!(
                ThemeColorHex::new(255, 255, 255),
                ThemeColorHex::from_hex("#ffffff").unwrap()
            );
            assert_eq!(
                ThemeColorHex::new(0, 0, 0),
                ThemeColorHex::from_hex("#000000").unwrap()
            );
        }

        #[test]
        fn should_parse_0x() {
            assert_eq!(
                ThemeColorHex::new(1, 2, 3),
                ThemeColorHex::from_hex("0x010203").unwrap()
            );
            assert_eq!(
                ThemeColorHex::new(255, 255, 255),
                ThemeColorHex::from_hex("0xffffff").unwrap()
            );
            assert_eq!(
                ThemeColorHex::new(0, 0, 0),
                ThemeColorHex::from_hex("0x000000").unwrap()
            );
        }
    }

    #[test]
    fn should_default() {
        // Test that there are no panics in the defaults, this should be able to be omitted once it is const
        let _ = ThemeColors::default();
    }
}
