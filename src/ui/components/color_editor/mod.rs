// use crate::song::Song;
mod ce_select;
use crate::config::parse_hex_color;
use crate::ui::components::music_library::get_pin_yin;
use crate::{
    config::get_app_config_path,
    // song::Song,
    ui::{Id, Model, Msg},
};
use anyhow::Result;
pub use ce_select::CESelectColor;
use serde::{Deserialize, Serialize};
use tui_realm_stdlib::Table;
use tuirealm::command::{Cmd, CmdResult, Direction, Position};
use tuirealm::props::{Alignment, BorderType, TableBuilder, TextSpan};
use tuirealm::props::{Borders, Color};
use tuirealm::{
    event::{Key, KeyEvent, KeyModifiers},
    AttrValue, Attribute, Component, Event, MockComponent, NoUserEvent, State, StateValue,
};
#[derive(Clone, Deserialize, Serialize)]
pub enum ColorConfig {
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
    pub fn color(&self, theme: &Theme) -> Option<Color> {
        match self {
            ColorConfig::Foreground => parse_hex_color(&theme.alacritty_theme.foreground),
            ColorConfig::Background => parse_hex_color(&theme.alacritty_theme.background),
            ColorConfig::Text => parse_hex_color(&theme.alacritty_theme.text),
            ColorConfig::Cursor => parse_hex_color(&theme.alacritty_theme.cursor),
            ColorConfig::Black => parse_hex_color(&theme.alacritty_theme.black),
            ColorConfig::Red => parse_hex_color(&theme.alacritty_theme.red),
            ColorConfig::Green => parse_hex_color(&theme.alacritty_theme.green),
            ColorConfig::Yellow => parse_hex_color(&theme.alacritty_theme.yellow),
            ColorConfig::Blue => parse_hex_color(&theme.alacritty_theme.blue),
            ColorConfig::Magenta => parse_hex_color(&theme.alacritty_theme.magenta),
            ColorConfig::Cyan => parse_hex_color(&theme.alacritty_theme.cyan),
            ColorConfig::White => parse_hex_color(&theme.alacritty_theme.white),
            ColorConfig::LightBlack => parse_hex_color(&theme.alacritty_theme.light_black),
            ColorConfig::LightRed => parse_hex_color(&theme.alacritty_theme.light_red),
            ColorConfig::LightGreen => parse_hex_color(&theme.alacritty_theme.light_green),
            ColorConfig::LightYellow => parse_hex_color(&theme.alacritty_theme.light_yellow),
            ColorConfig::LightBlue => parse_hex_color(&theme.alacritty_theme.light_blue),
            ColorConfig::LightMagenta => parse_hex_color(&theme.alacritty_theme.light_magenta),
            ColorConfig::LightCyan => parse_hex_color(&theme.alacritty_theme.light_cyan),
            ColorConfig::LightWhite => parse_hex_color(&theme.alacritty_theme.light_white),
        }
    }
}

#[derive(Clone, Deserialize, Serialize)]
pub struct Theme {
    name: String,
    pub library_foreground: ColorConfig,
    pub library_background: ColorConfig,
    pub library_highlight: ColorConfig,
    pub alacritty_theme: AlacrittyTheme,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            name: "default".to_string(),
            library_foreground: ColorConfig::Blue,
            library_background: ColorConfig::Background,
            library_highlight: ColorConfig::LightYellow,
            alacritty_theme: AlacrittyTheme::default(),
        }
    }
}
#[derive(Clone, Deserialize, Serialize)]
pub struct AlacrittyTheme {
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
            name: "default".to_string(),
            author: "Larry Hao".to_string(),
            background: "#2D2A2E".to_string(),
            foreground: "#FCFCFA".to_string(),
            cursor: "#FFFFFF".to_string(),
            text: "#1E1E1E".to_string(),
            black: "#403E41".to_string(),
            red: "#FF6188".to_string(),
            green: "#A9DC76".to_string(),
            yellow: "#FFD866".to_string(),
            blue: "#FC9867".to_string(),
            magenta: "#AB9DF2".to_string(),
            cyan: "#78DCE8".to_string(),
            white: "#FCFCFA".to_string(),
            light_black: "#727072".to_string(),
            light_red: "#FF6188".to_string(),
            light_green: "#A9DC76".to_string(),
            light_yellow: "#FFD866".to_string(),
            light_blue: "#FC9867".to_string(),
            light_magenta: "#AB9DF2".to_string(),
            light_cyan: "#78DCE8".to_string(),
            light_white: "#FCFCFA".to_string(),
        }
    }
}
impl Theme {
    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn library_foreground(&self) -> Option<Color> {
        self.library_foreground.color(self)
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
                        .modifiers(BorderType::Double)
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
                .column_spacing(3)
                .widths(&[10, 90])
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
            Event::Keyboard(KeyEvent { code: Key::Esc, .. }) => {
                return Some(Msg::CEThemeSelectCloseCancel);
            }
            Event::Keyboard(KeyEvent {
                code: Key::Enter, ..
            }) => {
                if let State::One(StateValue::Usize(index)) = self.state() {
                    return Some(Msg::CEThemeSelectCloseOk(index));
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
                self.themes.push(Theme {
                    name: p.file_name().to_string_lossy().to_string(),
                    library_foreground: ColorConfig::Red,
                    library_background: ColorConfig::Background,
                    library_highlight: ColorConfig::Yellow,
                    alacritty_theme: AlacrittyTheme::default(),
                });
            }
        }

        Ok(())
    }

    pub fn theme_select_sync(&mut self) {
        let mut table: TableBuilder = TableBuilder::default();

        for (idx, record) in self.themes.iter().enumerate() {
            if idx > 0 {
                table.add_row();
            }

            // let duration = record.duration_formatted().to_string();
            // let duration_string = format!("[{:^6.6}]", duration);

            // let noname_string = "No Name".to_string();
            // let name = record.name().unwrap_or(&noname_string);
            // let artist = record.artist().unwrap_or(name);
            // let title = record.title().unwrap_or("Unknown Title");

            table
                .add_col(TextSpan::new(idx.to_string()))
                .add_col(TextSpan::new(record.name()));
        }
        if self.playlist_items.is_empty() {
            table.add_col(TextSpan::from("0"));
            table.add_col(TextSpan::from("empty playlist"));
            table.add_col(TextSpan::from(""));
            table.add_col(TextSpan::from(""));
        }

        let table = table.build();
        self.app
            .attr(
                &Id::CEThemeSelect,
                Attribute::Content,
                AttrValue::Table(table),
            )
            .ok();
    }

    pub fn theme(&self) -> Theme {
        let mut theme = Theme::default();
        for i in &self.themes {
            if self.config.theme_selected == i.name() {
                theme = i.clone();
            }
        }
        theme
    }
}
