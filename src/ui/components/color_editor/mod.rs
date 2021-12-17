// use crate::song::Song;
mod ce_input;
mod ce_select;
use crate::config::parse_hex_color;
use crate::ui::components::music_library::get_pin_yin;
use crate::ui::IdColorEditor;
use crate::{
    config::get_app_config_path,
    // song::Song,
    ui::{CEMsg, Id, Model, Msg},
};
use anyhow::Result;
pub use ce_input::{CELibraryHighlightSymbol, CEPlaylistHighlightSymbol};
pub use ce_select::{
    CELibraryBackground, CELibraryBorder, CELibraryForeground, CELibraryHighlight, CELibraryTitle,
    CELyricBackground, CELyricBorder, CELyricForeground, CELyricTitle, CEPlaylistBackground,
    CEPlaylistBorder, CEPlaylistForeground, CEPlaylistHighlight, CEPlaylistTitle,
    CEProgressBackground, CEProgressBorder, CEProgressForeground, CEProgressTitle, CESelectColor,
};
use serde::{Deserialize, Serialize};
use std::fs::read_to_string;
use std::path::PathBuf;
use tui_realm_stdlib::{Radio, Table};
use tuirealm::command::{Cmd, CmdResult, Direction, Position};
use tuirealm::props::{
    Alignment, BorderType, Borders, Color, PropPayload, PropValue, TableBuilder, TextSpan,
};
use tuirealm::{
    event::{Key, KeyEvent, KeyModifiers},
    AttrValue, Attribute, Component, Event, MockComponent, NoUserEvent, State, StateValue,
};
use yaml_rust::YamlLoader;

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub enum ColorConfig {
    Reset,
    Foreground,
    Background,
    Text,
    Cursor,
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
impl ColorConfig {
    pub fn color(&self, alacritty_theme: &AlacrittyTheme) -> Option<Color> {
        match self {
            ColorConfig::Foreground => parse_hex_color(&alacritty_theme.foreground),
            ColorConfig::Background => parse_hex_color(&alacritty_theme.background),
            ColorConfig::Text => parse_hex_color(&alacritty_theme.text),
            ColorConfig::Cursor => parse_hex_color(&alacritty_theme.cursor),
            ColorConfig::Black => parse_hex_color(&alacritty_theme.black),
            ColorConfig::Red => parse_hex_color(&alacritty_theme.red),
            ColorConfig::Green => parse_hex_color(&alacritty_theme.green),
            ColorConfig::Yellow => parse_hex_color(&alacritty_theme.yellow),
            ColorConfig::Blue => parse_hex_color(&alacritty_theme.blue),
            ColorConfig::Magenta => parse_hex_color(&alacritty_theme.magenta),
            ColorConfig::Cyan => parse_hex_color(&alacritty_theme.cyan),
            ColorConfig::White => parse_hex_color(&alacritty_theme.white),
            ColorConfig::LightBlack => parse_hex_color(&alacritty_theme.light_black),
            ColorConfig::LightRed => parse_hex_color(&alacritty_theme.light_red),
            ColorConfig::LightGreen => parse_hex_color(&alacritty_theme.light_green),
            ColorConfig::LightYellow => parse_hex_color(&alacritty_theme.light_yellow),
            ColorConfig::LightBlue => parse_hex_color(&alacritty_theme.light_blue),
            ColorConfig::LightMagenta => parse_hex_color(&alacritty_theme.light_magenta),
            ColorConfig::LightCyan => parse_hex_color(&alacritty_theme.light_cyan),
            ColorConfig::LightWhite => parse_hex_color(&alacritty_theme.light_white),
            ColorConfig::Reset => Some(Color::Reset),
        }
    }
}

#[derive(Clone, Deserialize, Serialize, PartialEq)]
pub struct StyleColorSymbol {
    pub library_foreground: ColorConfig,
    pub library_background: ColorConfig,
    pub library_border: ColorConfig,
    pub library_highlight: ColorConfig,
    pub library_highlight_symbol: String,
    pub playlist_foreground: ColorConfig,
    pub playlist_background: ColorConfig,
    pub playlist_border: ColorConfig,
    pub playlist_highlight: ColorConfig,
    pub playlist_highlight_symbol: String,
    pub progress_foreground: ColorConfig,
    pub progress_background: ColorConfig,
    pub progress_border: ColorConfig,
    pub lyric_foreground: ColorConfig,
    pub lyric_background: ColorConfig,
    pub lyric_border: ColorConfig,
    pub alacritty_theme: AlacrittyTheme,
}

impl Default for StyleColorSymbol {
    fn default() -> Self {
        Self {
            library_foreground: ColorConfig::Foreground,
            library_background: ColorConfig::Reset,
            library_border: ColorConfig::Blue,
            library_highlight: ColorConfig::LightYellow,
            library_highlight_symbol: "\u{1f984}".to_string(),
            playlist_foreground: ColorConfig::Foreground,
            playlist_background: ColorConfig::Reset,
            playlist_border: ColorConfig::Blue,
            playlist_highlight: ColorConfig::LightYellow,
            playlist_highlight_symbol: "\u{1f680}".to_string(),
            progress_foreground: ColorConfig::Foreground,
            progress_background: ColorConfig::Reset,
            progress_border: ColorConfig::Blue,
            lyric_foreground: ColorConfig::Foreground,
            lyric_background: ColorConfig::Reset,
            lyric_border: ColorConfig::Blue,
            alacritty_theme: AlacrittyTheme::default(),
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
pub struct AlacrittyTheme {
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

impl Default for AlacrittyTheme {
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
#[derive(MockComponent)]
pub struct ThemeSelectTable {
    component: Table,
}

impl Default for ThemeSelectTable {
    fn default() -> Self {
        Self {
            component: Table::default()
                .borders(
                    Borders::default()
                        .modifiers(BorderType::Rounded)
                        .color(Color::Blue),
                )
                // .foreground(Color::Yellow)
                .background(Color::Reset)
                .title("Themes Selector", Alignment::Left)
                .scroll(true)
                .highlighted_color(Color::LightBlue)
                .highlighted_str("\u{1f680}")
                // .highlighted_str("🚀")
                .rewind(true)
                .step(4)
                .row_height(1)
                .headers(&["index", "Theme Name"])
                .column_spacing(1)
                .widths(&[18, 82])
                .table(
                    TableBuilder::default()
                        .add_col(TextSpan::from("Empty"))
                        .add_col(TextSpan::from("Empty Queue"))
                        .add_col(TextSpan::from("Empty"))
                        .build(),
                ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ThemeSelectTable {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        let _cmd_result = match ev {
            Event::Keyboard(KeyEvent {
                code: Key::Down | Key::Char('j'),
                ..
            }) => self.perform(Cmd::Move(Direction::Down)),
            Event::Keyboard(KeyEvent {
                code: Key::Up | Key::Char('k'),
                ..
            }) => self.perform(Cmd::Move(Direction::Up)),
            Event::Keyboard(KeyEvent {
                code: Key::PageDown,
                ..
            }) => self.perform(Cmd::Scroll(Direction::Down)),
            Event::Keyboard(KeyEvent {
                code: Key::PageUp, ..
            }) => self.perform(Cmd::Scroll(Direction::Up)),
            Event::Keyboard(KeyEvent {
                code: Key::Home | Key::Char('g'),
                ..
            }) => self.perform(Cmd::GoTo(Position::Begin)),
            Event::Keyboard(
                KeyEvent { code: Key::End, .. }
                | KeyEvent {
                    code: Key::Char('G'),
                    modifiers: KeyModifiers::SHIFT,
                },
            ) => self.perform(Cmd::GoTo(Position::End)),
            Event::Keyboard(KeyEvent {
                code: Key::Esc | Key::Char('q'),
                ..
            }) => return Some(Msg::ColorEditor(CEMsg::ColorEditorCloseCancel)),
            Event::Keyboard(KeyEvent {
                code: Key::Char('h'),
                modifiers: KeyModifiers::CONTROL,
            }) => return Some(Msg::ColorEditor(CEMsg::HelpPopupShow)),

            Event::Keyboard(KeyEvent { code: Key::Tab, .. }) => {
                return Some(Msg::ColorEditor(CEMsg::ThemeSelectBlur));
            }
            Event::Keyboard(KeyEvent {
                code: Key::Enter, ..
            }) => {
                if let State::One(StateValue::Usize(index)) = self.state() {
                    return Some(Msg::ColorEditor(CEMsg::ThemeSelectLoad(index)));
                }
                CmdResult::None
            }

            _ => CmdResult::None,
        };
        // match cmd_result {
        // CmdResult::Submit(State::One(StateValue::Usize(_index))) => {
        //     return Some(Msg::PlaylistPlaySelected);
        // }
        //_ =>
        Some(Msg::None)
        // }
    }
}

impl ThemeSelectTable {}

impl Model {
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
        if self.playlist_items.is_empty() {
            table.add_col(TextSpan::from("0"));
            table.add_col(TextSpan::from("empty playlist"));
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

pub fn load_alacritty_theme(path_str: &str) -> Result<AlacrittyTheme> {
    let path = PathBuf::from(path_str);
    let path = path.to_string_lossy().to_string();
    let string = read_to_string(&path)?;
    let docs = YamlLoader::load_from_str(&string)?;
    let doc = &docs[0];
    let doc = &doc["colors"];
    Ok(AlacrittyTheme {
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

#[derive(MockComponent)]
pub struct CERadioOk {
    component: Radio,
}
impl Default for CERadioOk {
    fn default() -> Self {
        Self {
            component: Radio::default()
                .foreground(Color::Yellow)
                // .background(Color::Black)
                .borders(
                    Borders::default()
                        .color(Color::Yellow)
                        .modifiers(BorderType::Rounded),
                )
                // .title("Additional operation:", Alignment::Left)
                .rewind(true)
                .choices(&["Save and Close"])
                .value(0),
        }
    }
}

impl Component<Msg, NoUserEvent> for CERadioOk {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        let cmd_result = match ev {
            Event::Keyboard(KeyEvent { code: Key::Tab, .. }) => {
                return Some(Msg::ColorEditor(CEMsg::ColorEditorOkBlur))
            }
            Event::Keyboard(KeyEvent {
                code: Key::Esc | Key::Char('q'),
                ..
            }) => return Some(Msg::ColorEditor(CEMsg::ColorEditorCloseCancel)),

            Event::Keyboard(KeyEvent {
                code: Key::Char('h'),
                modifiers: KeyModifiers::CONTROL,
            }) => return Some(Msg::ColorEditor(CEMsg::HelpPopupShow)),

            Event::Keyboard(KeyEvent {
                code: Key::Left | Key::Char('h' | 'j'),
                ..
            }) => self.perform(Cmd::Move(Direction::Left)),
            Event::Keyboard(KeyEvent {
                code: Key::Right | Key::Char('l' | 'k'),
                ..
            }) => self.perform(Cmd::Move(Direction::Right)),
            Event::Keyboard(KeyEvent {
                code: Key::Enter, ..
            }) => self.perform(Cmd::Submit),
            _ => return None,
        };
        if matches!(
            cmd_result,
            CmdResult::Submit(State::One(StateValue::Usize(0)))
        ) {
            return Some(Msg::ColorEditor(CEMsg::ColorEditorCloseOk));
        }
        Some(Msg::None)
    }
}

#[derive(MockComponent)]
pub struct CEHelpPopup {
    component: Table,
}

impl Default for CEHelpPopup {
    fn default() -> Self {
        Self {
            component: Table::default()
                .borders(
                    Borders::default()
                        .modifiers(BorderType::Rounded)
                        .color(Color::Green),
                )
                // .foreground(Color::Yellow)
                // .background(Color::Black)
                .title("Help: Esc or Enter to exit.", Alignment::Center)
                .scroll(false)
                // .highlighted_color(Color::LightBlue)
                // .highlighted_str("\u{1f680}")
                // .highlighted_str("🚀")
                // .rewind(true)
                .step(4)
                .row_height(1)
                .headers(&["Key", "Function"])
                .column_spacing(3)
                .widths(&[30, 70])
                .table(
                    TableBuilder::default()
                        .add_col(TextSpan::new("<TAB>").bold().fg(Color::Cyan))
                        .add_col(TextSpan::from("Switch focus"))
                        .add_row()
                        .add_col(TextSpan::new("<ESC> or <q>").bold().fg(Color::Cyan))
                        .add_col(TextSpan::from("Exit without saving"))
                        .add_row()
                        .add_col(TextSpan::new("Theme Select").bold().fg(Color::LightYellow))
                        .add_row()
                        .add_col(TextSpan::new("<h,j,k,l,g,G>").bold().fg(Color::Cyan))
                        .add_col(TextSpan::from("Move cursor(vim style)"))
                        .add_row()
                        .add_col(TextSpan::new("<ENTER>").bold().fg(Color::Cyan))
                        .add_col(TextSpan::from("load a theme for preview"))
                        .add_row()
                        .add_col(TextSpan::new("Color Select").bold().fg(Color::LightYellow))
                        .add_row()
                        .add_col(TextSpan::new("<ENTER>").bold().fg(Color::Cyan))
                        .add_col(TextSpan::from("select a color"))
                        .add_row()
                        .add_col(TextSpan::new("<h,j>").bold().fg(Color::Cyan))
                        .add_col(TextSpan::from("Move cursor(vim style)"))
                        .add_row()
                        .add_col(
                            TextSpan::new("Highlight String")
                                .bold()
                                .fg(Color::LightYellow),
                        )
                        .add_row()
                        .add_col(TextSpan::new("").bold().fg(Color::Cyan))
                        .add_col(TextSpan::from("You can paste symbol, or input."))
                        .add_row()
                        .add_col(TextSpan::new("<ENTER>").bold().fg(Color::Cyan))
                        .add_col(TextSpan::from("preview unicode symbol."))
                        .build(),
                ),
        }
    }
}

impl Component<Msg, NoUserEvent> for CEHelpPopup {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        match ev {
            Event::Keyboard(KeyEvent {
                code: Key::Enter | Key::Esc,
                ..
            }) => Some(Msg::ColorEditor(CEMsg::HelpPopupClose)),
            _ => None,
        }
    }
}
