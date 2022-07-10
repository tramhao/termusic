use crate::ui::model::Model;
use crate::ui::IdColorEditor;
use crate::utils::{get_pin_yin, parse_hex_color};
use crate::{config::get_app_config_path, ui::Id};
use anyhow::Result;
use include_dir::{include_dir, Dir, DirEntry};
use serde::{Deserialize, Serialize};
use std::fs::read_to_string;
use std::path::PathBuf;
use tuirealm::props::{Color, PropPayload, PropValue, TableBuilder, TextSpan};
use tuirealm::{AttrValue, Attribute};
use yaml_rust::YamlLoader;

static THEME_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/themes");

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub enum ColorTermusic {
    Reset,
    Foreground,
    Background,
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
    LightBlack,
    LightRed,
    LightGreen,
    LightYellow,
    LightBlue,
    LightMagenta,
    LightCyan,
    LightWhite,
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
    pub fn color(&self, alacritty_theme: &Alacritty) -> Option<Color> {
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
    pub const fn as_usize(&self) -> usize {
        match self {
            Self::Reset => 0,
            Self::Foreground => 1,
            Self::Background => 2,
            Self::Black => 3,
            Self::Red => 4,
            Self::Green => 5,
            Self::Yellow => 6,
            Self::Blue => 7,
            Self::Magenta => 8,
            Self::Cyan => 9,
            Self::White => 10,
            Self::LightBlack => 11,
            Self::LightRed => 12,
            Self::LightGreen => 13,
            Self::LightYellow => 14,
            Self::LightBlue => 15,
            Self::LightMagenta => 16,
            Self::LightCyan => 17,
            Self::LightWhite => 18,
        }
    }
}

#[derive(Clone, Deserialize, Serialize, PartialEq)]
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

#[derive(Clone, Deserialize, Serialize, PartialEq)]
pub struct Alacritty {
    path: String,
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
            path: "".to_string(),
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
impl Model {
    pub fn theme_select_save() -> Result<()> {
        let mut path = get_app_config_path()?;
        path.push("themes");
        if !path.exists() {
            std::fs::create_dir_all(&path)?;
        }

        let base_path = &path;
        for entry in THEME_DIR.entries() {
            let path = base_path.join(entry.path());

            match entry {
                DirEntry::Dir(d) => {
                    std::fs::create_dir_all(&path)?;
                    d.extract(base_path)?;
                }
                DirEntry::File(f) => {
                    if !path.exists() {
                        std::fs::write(path, f.contents())?;
                    }
                }
            }
        }

        Ok(())
    }
    pub fn theme_select_load_themes(&mut self) -> Result<()> {
        let mut path = get_app_config_path()?;
        path.push("themes");
        if let Ok(paths) = std::fs::read_dir(path) {
            let mut paths: Vec<_> = paths.filter_map(std::result::Result::ok).collect();

            paths.sort_by_cached_key(|k| get_pin_yin(&k.file_name().to_string_lossy().to_string()));
            for p in paths {
                self.ce_themes.push(p.path().to_string_lossy().to_string());
            }
        }

        Ok(())
    }

    pub fn theme_select_sync(&mut self) {
        let mut table: TableBuilder = TableBuilder::default();

        for (idx, record) in self.ce_themes.iter().enumerate() {
            if idx > 0 {
                table.add_row();
            }

            let path = PathBuf::from(record);
            let name = path.file_stem();

            if let Some(n) = name {
                table
                    .add_col(TextSpan::new(idx.to_string()))
                    .add_col(TextSpan::new(n.to_string_lossy()));
            }
        }
        if self.ce_themes.is_empty() {
            table.add_col(TextSpan::from("0"));
            table.add_col(TextSpan::from("empty theme list"));
        }

        let table = table.build();
        self.app
            .attr(
                &Id::ColorEditor(IdColorEditor::ThemeSelect),
                Attribute::Content,
                AttrValue::Table(table),
            )
            .ok();
        // select theme currently used
        let mut index = 0;
        for (idx, v) in self.ce_themes.iter().enumerate() {
            if *v == self.ce_style_color_symbol.alacritty_theme.path {
                index = idx;
                break;
            }
        }
        assert!(self
            .app
            .attr(
                &Id::ColorEditor(IdColorEditor::ThemeSelect),
                Attribute::Value,
                AttrValue::Payload(PropPayload::One(PropValue::Usize(index))),
            )
            .is_ok());
    }
}

pub fn load_alacritty(path_str: &str) -> Result<Alacritty> {
    let path = PathBuf::from(path_str);
    let path = path.to_string_lossy().to_string();
    let string = read_to_string(&path)?;
    let docs = YamlLoader::load_from_str(&string)?;
    let doc = &docs[0];
    let doc = &doc["colors"];
    Ok(Alacritty {
        path,
        name: doc["name"].as_str().unwrap_or("empty name").to_string(),
        author: doc["author"].as_str().unwrap_or("empty author").to_string(),
        background: doc["primary"]["background"]
            .as_str()
            .unwrap_or("#000000")
            .to_string(),
        foreground: doc["primary"]["foreground"]
            .as_str()
            .unwrap_or("#FFFFFF")
            .to_string(),
        cursor: doc["cursor"]["cursor"]
            .as_str()
            .unwrap_or("#FFFFFF")
            .to_string(),
        text: doc["cursor"]["text"]
            .as_str()
            .unwrap_or("#FFFFFF")
            .to_string(),
        black: doc["normal"]["black"]
            .as_str()
            .unwrap_or("#000000")
            .to_string(),
        red: doc["normal"]["red"]
            .as_str()
            .unwrap_or("#ff0000")
            .to_string(),
        green: doc["normal"]["green"]
            .as_str()
            .unwrap_or("#00ff00")
            .to_string(),
        yellow: doc["normal"]["yellow"]
            .as_str()
            .unwrap_or("#ffff00")
            .to_string(),
        blue: doc["normal"]["blue"]
            .as_str()
            .unwrap_or("#0000ff")
            .to_string(),
        magenta: doc["normal"]["magenta"]
            .as_str()
            .unwrap_or("#ff00ff")
            .to_string(),
        cyan: doc["normal"]["cyan"]
            .as_str()
            .unwrap_or("#00ffff")
            .to_string(),
        white: doc["normal"]["white"]
            .as_str()
            .unwrap_or("#FFFFFF")
            .to_string(),
        light_black: doc["bright"]["black"]
            .as_str()
            .unwrap_or("#777777")
            .to_string(),
        light_red: doc["bright"]["red"]
            .as_str()
            .unwrap_or("#00000")
            .to_string(),
        light_green: doc["bright"]["green"]
            .as_str()
            .unwrap_or("#00000")
            .to_string(),
        light_yellow: doc["bright"]["yellow"]
            .as_str()
            .unwrap_or("#00000")
            .to_string(),
        light_blue: doc["bright"]["blue"]
            .as_str()
            .unwrap_or("#00000")
            .to_string(),
        light_magenta: doc["bright"]["magenta"]
            .as_str()
            .unwrap_or("#00000")
            .to_string(),
        light_cyan: doc["bright"]["cyan"]
            .as_str()
            .unwrap_or("#00000")
            .to_string(),
        light_white: doc["bright"]["white"]
            .as_str()
            .unwrap_or("#00000")
            .to_string(),
    })
}
