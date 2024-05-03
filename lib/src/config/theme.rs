use crate::utils::parse_hex_color;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use tuirealm::props::Color;

use super::yaml_theme::YAMLTheme;

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
    pub fn color(self, alacritty_theme: &Alacritty) -> Option<Color> {
        match self {
            Self::Foreground => parse_hex_color(&alacritty_theme.foreground),
            Self::Background => parse_hex_color(&alacritty_theme.background),
            Self::Black => parse_hex_color(&alacritty_theme.black),
            Self::Red => parse_hex_color(&alacritty_theme.red),
            Self::Green => parse_hex_color(&alacritty_theme.green),
            Self::Yellow => parse_hex_color(&alacritty_theme.yellow),
            Self::Blue => parse_hex_color(&alacritty_theme.blue),
            Self::Magenta => parse_hex_color(&alacritty_theme.magenta),
            Self::Cyan => parse_hex_color(&alacritty_theme.cyan),
            Self::White => parse_hex_color(&alacritty_theme.white),
            Self::LightBlack => parse_hex_color(&alacritty_theme.light_black),
            Self::LightRed => parse_hex_color(&alacritty_theme.light_red),
            Self::LightGreen => parse_hex_color(&alacritty_theme.light_green),
            Self::LightYellow => parse_hex_color(&alacritty_theme.light_yellow),
            Self::LightBlue => parse_hex_color(&alacritty_theme.light_blue),
            Self::LightMagenta => parse_hex_color(&alacritty_theme.light_magenta),
            Self::LightCyan => parse_hex_color(&alacritty_theme.light_cyan),
            Self::LightWhite => parse_hex_color(&alacritty_theme.light_white),
            Self::Reset => Some(Color::Reset),
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
            alacritty_theme: Alacritty::default(),
            currently_playing_track_symbol: "►".to_string(),
        }
    }
}

impl StyleColorSymbol {
    pub fn library_foreground(&self) -> Option<Color> {
        self.library_foreground.color(&self.alacritty_theme)
    }

    pub fn library_background(&self) -> Option<Color> {
        self.library_background.color(&self.alacritty_theme)
    }
    pub fn library_highlight(&self) -> Option<Color> {
        self.library_highlight.color(&self.alacritty_theme)
    }
    pub fn library_border(&self) -> Option<Color> {
        self.library_border.color(&self.alacritty_theme)
    }
    pub fn playlist_foreground(&self) -> Option<Color> {
        self.playlist_foreground.color(&self.alacritty_theme)
    }
    pub fn playlist_background(&self) -> Option<Color> {
        self.playlist_background.color(&self.alacritty_theme)
    }
    pub fn playlist_highlight(&self) -> Option<Color> {
        self.playlist_highlight.color(&self.alacritty_theme)
    }
    pub fn playlist_border(&self) -> Option<Color> {
        self.playlist_border.color(&self.alacritty_theme)
    }
    pub fn progress_foreground(&self) -> Option<Color> {
        self.progress_foreground.color(&self.alacritty_theme)
    }
    pub fn progress_background(&self) -> Option<Color> {
        self.progress_background.color(&self.alacritty_theme)
    }
    pub fn progress_border(&self) -> Option<Color> {
        self.progress_border.color(&self.alacritty_theme)
    }
    pub fn lyric_foreground(&self) -> Option<Color> {
        self.lyric_foreground.color(&self.alacritty_theme)
    }
    pub fn lyric_background(&self) -> Option<Color> {
        self.lyric_background.color(&self.alacritty_theme)
    }
    pub fn lyric_border(&self) -> Option<Color> {
        self.lyric_border.color(&self.alacritty_theme)
    }
}

#[derive(Clone, Deserialize, Serialize, PartialEq, Eq, Debug)]
pub struct Alacritty {
    pub path: String,
    name: String,
    author: String,
    background: String,
    foreground: String,
    cursor: String,
    text: String,
    black: String,
    red: String,
    green: String,
    yellow: String,
    blue: String,
    magenta: String,
    cyan: String,
    white: String,
    light_black: String,
    light_red: String,
    light_green: String,
    light_yellow: String,
    light_blue: String,
    light_magenta: String,
    light_cyan: String,
    light_white: String,
}

impl Default for Alacritty {
    fn default() -> Self {
        Self {
            path: String::new(),
            name: "default".to_string(),
            author: "Larry Hao".to_string(),
            background: "#101421".to_string(),
            foreground: "#fffbf6".to_string(),
            cursor: "#FFFFFF".to_string(),
            text: "#1E1E1E".to_string(),
            black: "#2e2e2e".to_string(),
            red: "#eb4129".to_string(),
            green: "#abe047".to_string(),
            yellow: "#f6c744".to_string(),
            blue: "#47a0f3".to_string(),
            magenta: "#7b5cb0".to_string(),
            cyan: "#64dbed".to_string(),
            white: "#e5e9f0".to_string(),
            light_black: "#565656".to_string(),
            light_red: "#ec5357".to_string(),
            light_green: "#c0e17d".to_string(),
            light_yellow: "#f9da6a".to_string(),
            light_blue: "#49a4f8".to_string(),
            light_magenta: "#a47de9".to_string(),
            light_cyan: "#99faf2".to_string(),
            light_white: "#ffffff".to_string(),
        }
    }
}

impl Alacritty {
    /// Convert a [`YAMLTheme`] to this type
    ///
    /// Cannot be a [`From`] implementation because of the additional set `path` parameter
    pub fn from_yaml_theme(value: YAMLTheme, path: String) -> Self {
        let colors = value.colors;
        Alacritty {
            path,
            name: colors.name,
            author: colors.author,
            background: colors.primary.background,
            foreground: colors.primary.foreground,
            cursor: colors.cursor.cursor,
            text: colors.cursor.text,
            black: colors.normal.black,
            red: colors.normal.red,
            green: colors.normal.green,
            yellow: colors.normal.yellow,
            blue: colors.normal.blue,
            magenta: colors.normal.magenta,
            cyan: colors.normal.cyan,
            white: colors.normal.white,
            light_black: colors.bright.black,
            light_red: colors.bright.red,
            light_green: colors.bright.green,
            light_yellow: colors.bright.yellow,
            light_blue: colors.bright.blue,
            light_magenta: colors.bright.magenta,
            light_cyan: colors.bright.cyan,
            light_white: colors.bright.white,
        }
    }

    /// Load a YAML Theme and then convert it to a [`Alacritty`] instance
    pub fn from_yaml_file(path: &Path) -> Result<Self> {
        let parsed: YAMLTheme = serde_yaml::from_reader(BufReader::new(File::open(path)?))?;
        let path_str = path.to_string_lossy().to_string();

        Ok(Self::from_yaml_theme(parsed, path_str))
    }
}
