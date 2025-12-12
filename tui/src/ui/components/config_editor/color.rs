/**
 * MIT License
 *
 * tuifeed - Copyright (c) 2021 Christian Visintin
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy
 * of this software and associated documentation files (the "Software"), to deal
 * in the Software without restriction, including without limitation the rights
 * to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 * copies of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in all
 * copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 * AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
 * SOFTWARE.
 */
use anyhow::Result;
use termusiclib::config::SharedTuiSettings;
use termusiclib::config::v2::tui::theme::ThemeWrap;
use termusiclib::config::v2::tui::theme::styles::ColorTermusic;
use tui_realm_stdlib::{Label, Select, Table};
use tuirealm::command::{Cmd, CmdResult, Direction, Position};
use tuirealm::event::{Key, KeyEvent, KeyModifiers};
use tuirealm::props::{
    Alignment, BorderType, Borders, Color, InputType, Style, TableBuilder, TextModifiers, TextSpan,
};
use tuirealm::ratatui::style::Modifier;
use tuirealm::{AttrValue, Attribute, Component, Event, MockComponent, State, StateValue};

use crate::ui::components::vendored::tui_realm_stdlib_input::Input;
use crate::ui::ids::{Id, IdCETheme, IdConfigEditor};
use crate::ui::model::{Model, UserEvent};
use crate::ui::msg::{ConfigEditorMsg, KFMsg, Msg};

const COLOR_LIST: [ColorTermusic; 19] = [
    ColorTermusic::Reset,
    ColorTermusic::Foreground,
    ColorTermusic::Background,
    ColorTermusic::Black,
    ColorTermusic::Red,
    ColorTermusic::Green,
    ColorTermusic::Yellow,
    ColorTermusic::Blue,
    ColorTermusic::Magenta,
    ColorTermusic::Cyan,
    ColorTermusic::White,
    ColorTermusic::LightBlack,
    ColorTermusic::LightRed,
    ColorTermusic::LightGreen,
    ColorTermusic::LightYellow,
    ColorTermusic::LightBlue,
    ColorTermusic::LightMagenta,
    ColorTermusic::LightCyan,
    ColorTermusic::LightWhite,
];

#[derive(MockComponent)]
pub struct CEThemeSelectTable {
    component: Table,
    config: SharedTuiSettings,
}

impl CEThemeSelectTable {
    pub fn new(config: SharedTuiSettings) -> Self {
        let component = {
            let config = config.read();
            Table::default()
                .borders(
                    Borders::default()
                        .modifiers(BorderType::Rounded)
                        .color(config.settings.theme.fallback_border()),
                )
                .foreground(config.settings.theme.fallback_foreground())
                .background(config.settings.theme.fallback_background())
                .title(" Themes: <Enter> to preview ", Alignment::Left)
                .scroll(true)
                .highlighted_color(config.settings.theme.fallback_highlight())
                .highlighted_str(&config.settings.theme.style.library.highlight_symbol)
                .rewind(true)
                .step(4)
                .row_height(1)
                .headers(["index", "Theme Name"])
                .column_spacing(1)
                .widths(&[18, 82])
                .table(
                    TableBuilder::default()
                        .add_col(TextSpan::from("Empty"))
                        .add_col(TextSpan::from("Empty Queue"))
                        .add_col(TextSpan::from("Empty"))
                        .build(),
                )
        };

        Self { component, config }
    }
}

impl Component<Msg, UserEvent> for CEThemeSelectTable {
    fn on(&mut self, ev: Event<UserEvent>) -> Option<Msg> {
        let config = self.config.clone();
        let keys = &config.read().settings.keys;
        let cmd_result = match ev {
            // Global Hotkeys
            Event::Keyboard(keyevent) if keyevent == keys.config_keys.save.get() => {
                return Some(Msg::ConfigEditor(ConfigEditorMsg::CloseOk));
            }
            Event::Keyboard(KeyEvent { code: Key::Esc, .. }) => {
                return Some(Msg::ConfigEditor(ConfigEditorMsg::CloseCancel));
            }
            Event::Keyboard(keyevent) if keyevent == keys.quit.get() => {
                return Some(Msg::ConfigEditor(ConfigEditorMsg::CloseCancel));
            }

            // Local Hotkeys
            Event::Keyboard(KeyEvent {
                code: Key::Down, ..
            }) => self.perform(Cmd::Move(Direction::Down)),
            Event::Keyboard(keyevent) if keyevent == keys.navigation_keys.down.get() => {
                self.perform(Cmd::Move(Direction::Down))
            }

            Event::Keyboard(KeyEvent { code: Key::Up, .. }) => {
                self.perform(Cmd::Move(Direction::Up))
            }
            Event::Keyboard(keyevent) if keyevent == keys.navigation_keys.up.get() => {
                self.perform(Cmd::Move(Direction::Up))
            }
            Event::Keyboard(KeyEvent {
                code: Key::PageDown,
                ..
            }) => self.perform(Cmd::Scroll(Direction::Down)),
            Event::Keyboard(KeyEvent {
                code: Key::PageUp, ..
            }) => self.perform(Cmd::Scroll(Direction::Up)),

            Event::Keyboard(keyevent) if keyevent == keys.navigation_keys.goto_top.get() => {
                self.perform(Cmd::GoTo(Position::Begin))
            }

            Event::Keyboard(keyevent) if keyevent == keys.navigation_keys.goto_bottom.get() => {
                self.perform(Cmd::GoTo(Position::End))
            }

            Event::Keyboard(KeyEvent { code: Key::End, .. }) => {
                self.perform(Cmd::GoTo(Position::End))
            }
            Event::Keyboard(KeyEvent { code: Key::Tab, .. }) => {
                return Some(Msg::ConfigEditor(ConfigEditorMsg::Theme(KFMsg::Next)));
            }
            Event::Keyboard(KeyEvent {
                code: Key::BackTab,
                modifiers: KeyModifiers::SHIFT,
            }) => {
                return Some(Msg::ConfigEditor(ConfigEditorMsg::Theme(KFMsg::Previous)));
            }

            Event::Keyboard(KeyEvent {
                code: Key::Enter, ..
            }) => {
                if let State::One(StateValue::Usize(index)) = self.state() {
                    return Some(Msg::ConfigEditor(ConfigEditorMsg::ThemeSelectLoad(index)));
                }
                CmdResult::None
            }

            _ => CmdResult::None,
        };
        match cmd_result {
            CmdResult::None => None,
            _ => Some(Msg::ForceRedraw),
        }
    }
}

#[derive(MockComponent)]
pub struct CEColorSelect {
    component: Select,
    id: IdCETheme,
    config: SharedTuiSettings,
    on_key_shift: Msg,
    on_key_backshift: Msg,
}

impl CEColorSelect {
    /// Get a new instance for a color selector.
    ///
    /// # Panics
    ///
    /// The only IDs expected are color IDs, everything else(like `*Symbol` or `*Label`) will panic.
    pub fn new(
        name: &str,
        id: IdCETheme,
        color: Color,
        config: SharedTuiSettings,
        on_key_shift: Msg,
        on_key_backshift: Msg,
    ) -> Self {
        let init_value = Self::init_color_select(id, &config.read().settings.theme);
        let mut choices = Vec::new();
        for color in &COLOR_LIST {
            choices.push(color.as_ref());
        }
        Self {
            component: Select::default()
                .borders(
                    Borders::default()
                        .modifiers(BorderType::Rounded)
                        .color(color),
                )
                // .foreground(color)
                .background(color)
                .title(name, Alignment::Left)
                .rewind(false)
                .inactive(Style::default().add_modifier(Modifier::BOLD).bg(color))
                .highlighted_color(Color::LightGreen)
                .highlighted_str(">> ")
                .choices(choices)
                .value(init_value),
            id,
            config,
            on_key_shift,
            on_key_backshift,
        }
    }

    /// Get the current color index in the current theme for the given ID.
    ///
    /// # Panics
    ///
    /// The only IDs expected are color IDs, everything else will panic.
    const fn init_color_select(id: IdCETheme, theme: &ThemeWrap) -> usize {
        match id {
            IdCETheme::LibraryForeground => theme.style.library.foreground_color.as_usize(),
            IdCETheme::LibraryBackground => theme.style.library.background_color.as_usize(),
            IdCETheme::LibraryBorder => theme.style.library.border_color.as_usize(),
            IdCETheme::LibraryHighlight => theme.style.library.highlight_color.as_usize(),
            IdCETheme::PlaylistForeground => theme.style.playlist.foreground_color.as_usize(),
            IdCETheme::PlaylistBackground => theme.style.playlist.background_color.as_usize(),
            IdCETheme::PlaylistBorder => theme.style.playlist.border_color.as_usize(),
            IdCETheme::PlaylistHighlight => theme.style.playlist.highlight_color.as_usize(),
            IdCETheme::ProgressForeground => theme.style.progress.foreground_color.as_usize(),
            IdCETheme::ProgressBackground => theme.style.progress.background_color.as_usize(),
            IdCETheme::ProgressBorder => theme.style.progress.border_color.as_usize(),
            IdCETheme::LyricForeground => theme.style.lyric.foreground_color.as_usize(),
            IdCETheme::LyricBackground => theme.style.lyric.background_color.as_usize(),
            IdCETheme::LyricBorder => theme.style.lyric.border_color.as_usize(),
            IdCETheme::ImportantPopupForeground => {
                theme.style.important_popup.foreground_color.as_usize()
            }
            IdCETheme::ImportantPopupBackground => {
                theme.style.important_popup.background_color.as_usize()
            }
            IdCETheme::ImportantPopupBorder => theme.style.important_popup.border_color.as_usize(),
            IdCETheme::FallbackForeground => theme.style.fallback.foreground_color.as_usize(),
            IdCETheme::FallbackBackground => theme.style.fallback.background_color.as_usize(),
            IdCETheme::FallbackBorder => theme.style.fallback.border_color.as_usize(),
            IdCETheme::FallbackHighlight => theme.style.fallback.highlight_color.as_usize(),

            // explicitly handle all cases
            IdCETheme::ThemeSelectTable
            | IdCETheme::LibraryHighlightSymbol
            | IdCETheme::LibraryLabel
            | IdCETheme::LyricLabel
            | IdCETheme::PlaylistHighlightSymbol
            | IdCETheme::PlaylistLabel
            | IdCETheme::CurrentlyPlayingTrackSymbol
            | IdCETheme::ProgressLabel
            | IdCETheme::ImportantPopupLabel
            | IdCETheme::FallbackLabel => unreachable!(),
        }
    }

    fn update_color(&mut self, index: usize) -> Msg {
        if let Some(color_config) = COLOR_LIST.get(index) {
            let color = self
                .config
                .read()
                .settings
                .theme
                .get_color_from_theme(*color_config);
            // self.attr(Attribute::Foreground, AttrValue::Color(color));
            self.attr(Attribute::Background, AttrValue::Color(color));
            self.attr(
                Attribute::Borders,
                AttrValue::Borders(
                    Borders::default()
                        .modifiers(BorderType::Rounded)
                        .color(color),
                ),
            );
            self.attr(
                Attribute::FocusStyle,
                AttrValue::Style(Style::default().add_modifier(Modifier::BOLD).bg(color)),
            );
            Msg::ConfigEditor(ConfigEditorMsg::ColorChanged(
                IdConfigEditor::Theme(self.id),
                *color_config,
            ))
        } else {
            self.attr(Attribute::Background, AttrValue::Color(Color::Red));
            self.attr(
                Attribute::Borders,
                AttrValue::Borders(
                    Borders::default()
                        .modifiers(BorderType::Rounded)
                        .color(Color::Red),
                ),
            );
            self.attr(
                Attribute::FocusStyle,
                AttrValue::Style(Style::default().add_modifier(Modifier::BOLD).bg(Color::Red)),
            );

            Msg::ForceRedraw
        }
    }
}

impl Component<Msg, UserEvent> for CEColorSelect {
    fn on(&mut self, ev: Event<UserEvent>) -> Option<Msg> {
        let config = self.config.clone();
        let keys = &config.read().settings.keys;
        let cmd_result = match ev {
            // Global Hotkeys
            Event::Keyboard(keyevent) if keyevent == keys.config_keys.save.get() => {
                return Some(Msg::ConfigEditor(ConfigEditorMsg::CloseOk));
            }
            Event::Keyboard(KeyEvent { code: Key::Tab, .. }) => {
                return Some(Msg::ConfigEditor(ConfigEditorMsg::ChangeLayout));
            }

            Event::Keyboard(key) if key == keys.escape.get() => match self.state() {
                State::One(_) => return Some(Msg::ConfigEditor(ConfigEditorMsg::CloseCancel)),
                _ => self.perform(Cmd::Cancel),
            },
            Event::Keyboard(keyevent) if keyevent == keys.quit.get() => match self.state() {
                State::One(_) => return Some(Msg::ConfigEditor(ConfigEditorMsg::CloseCancel)),
                _ => self.perform(Cmd::Cancel),
            },

            Event::Keyboard(key) if key == keys.navigation_keys.up.get() => match self.state() {
                State::One(_) => return Some(self.on_key_backshift.clone()),
                _ => self.perform(Cmd::Move(Direction::Up)),
            },

            Event::Keyboard(key) if key == keys.navigation_keys.down.get() => match self.state() {
                State::One(_) => return Some(self.on_key_shift.clone()),
                _ => self.perform(Cmd::Move(Direction::Down)),
            },

            Event::Keyboard(KeyEvent { code: Key::Up, .. }) => match self.state() {
                State::One(_) => return Some(self.on_key_backshift.clone()),
                _ => self.perform(Cmd::Move(Direction::Up)),
            },

            Event::Keyboard(KeyEvent {
                code: Key::Down, ..
            }) => match self.state() {
                State::One(_) => return Some(self.on_key_shift.clone()),
                _ => self.perform(Cmd::Move(Direction::Down)),
            },

            Event::Keyboard(KeyEvent {
                code: Key::Enter, ..
            }) => {
                // "Select" returns "None" as a result for "Submit" when transitioning from "closed" to "open" state.
                // But does return something when transitioning from "open" to "closed" state.
                match self.perform(Cmd::Submit) {
                    CmdResult::None => return Some(Msg::ForceRedraw),
                    v => v,
                }
            }
            _ => CmdResult::None,
        };
        match cmd_result {
            CmdResult::Submit(State::One(StateValue::Usize(index))) => {
                Some(self.update_color(index))
            }
            CmdResult::None => None,
            _ => Some(Msg::ForceRedraw),
        }
    }
}

#[derive(MockComponent)]
pub struct CEStyleTitle {
    component: Label,
}

impl CEStyleTitle {
    pub fn new(config: &SharedTuiSettings, text: &str) -> Self {
        let config_tui = config.read_recursive();

        Self {
            component: Label::default()
                .foreground(config_tui.settings.theme.lyric_foreground())
                .background(config_tui.settings.theme.lyric_background())
                .modifiers(TextModifiers::BOLD)
                .text(text),
        }
    }
}

impl Component<Msg, UserEvent> for CEStyleTitle {
    fn on(&mut self, _ev: Event<UserEvent>) -> Option<Msg> {
        None
    }
}

#[inline]
fn library_title(config: &SharedTuiSettings) -> CEStyleTitle {
    CEStyleTitle::new(config, " Library style ")
}

#[derive(MockComponent)]
pub struct ConfigLibraryForeground {
    component: CEColorSelect,
}

impl ConfigLibraryForeground {
    pub fn new(config: SharedTuiSettings) -> Self {
        let color = config.read().settings.theme.library_foreground();
        Self {
            component: CEColorSelect::new(
                " Foreground ",
                IdCETheme::LibraryForeground,
                color,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::Theme(KFMsg::Next)),
                Msg::ConfigEditor(ConfigEditorMsg::Theme(KFMsg::Previous)),
            ),
        }
    }
}

impl Component<Msg, UserEvent> for ConfigLibraryForeground {
    fn on(&mut self, ev: Event<UserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct ConfigLibraryBackground {
    component: CEColorSelect,
}

impl ConfigLibraryBackground {
    pub fn new(config: SharedTuiSettings) -> Self {
        let color = config.read().settings.theme.library_background();
        Self {
            component: CEColorSelect::new(
                " Background ",
                IdCETheme::LibraryBackground,
                color,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::Theme(KFMsg::Next)),
                Msg::ConfigEditor(ConfigEditorMsg::Theme(KFMsg::Previous)),
            ),
        }
    }
}

impl Component<Msg, UserEvent> for ConfigLibraryBackground {
    fn on(&mut self, ev: Event<UserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct ConfigLibraryBorder {
    component: CEColorSelect,
}

impl ConfigLibraryBorder {
    pub fn new(config: SharedTuiSettings) -> Self {
        let color = config.read().settings.theme.library_border();
        Self {
            component: CEColorSelect::new(
                " Border ",
                IdCETheme::LibraryBorder,
                color,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::Theme(KFMsg::Next)),
                Msg::ConfigEditor(ConfigEditorMsg::Theme(KFMsg::Previous)),
            ),
        }
    }
}

impl Component<Msg, UserEvent> for ConfigLibraryBorder {
    fn on(&mut self, ev: Event<UserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct ConfigLibraryHighlight {
    component: CEColorSelect,
}

impl ConfigLibraryHighlight {
    pub fn new(config: SharedTuiSettings) -> Self {
        let color = config.read().settings.theme.library_highlight();
        Self {
            component: CEColorSelect::new(
                " Highlight ",
                IdCETheme::LibraryHighlight,
                color,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::Theme(KFMsg::Next)),
                Msg::ConfigEditor(ConfigEditorMsg::Theme(KFMsg::Previous)),
            ),
        }
    }
}

impl Component<Msg, UserEvent> for ConfigLibraryHighlight {
    fn on(&mut self, ev: Event<UserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[inline]
fn playlist_title(config: &SharedTuiSettings) -> CEStyleTitle {
    CEStyleTitle::new(config, " Playlist style ")
}

#[derive(MockComponent)]
pub struct ConfigPlaylistForeground {
    component: CEColorSelect,
}

impl ConfigPlaylistForeground {
    pub fn new(config: SharedTuiSettings) -> Self {
        let color = config.read().settings.theme.playlist_foreground();
        Self {
            component: CEColorSelect::new(
                " Foreground ",
                IdCETheme::PlaylistForeground,
                color,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::Theme(KFMsg::Next)),
                Msg::ConfigEditor(ConfigEditorMsg::Theme(KFMsg::Previous)),
            ),
        }
    }
}

impl Component<Msg, UserEvent> for ConfigPlaylistForeground {
    fn on(&mut self, ev: Event<UserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct ConfigPlaylistBackground {
    component: CEColorSelect,
}

impl ConfigPlaylistBackground {
    pub fn new(config: SharedTuiSettings) -> Self {
        let color = config.read().settings.theme.playlist_background();
        Self {
            component: CEColorSelect::new(
                " Background ",
                IdCETheme::PlaylistBackground,
                color,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::Theme(KFMsg::Next)),
                Msg::ConfigEditor(ConfigEditorMsg::Theme(KFMsg::Previous)),
            ),
        }
    }
}

impl Component<Msg, UserEvent> for ConfigPlaylistBackground {
    fn on(&mut self, ev: Event<UserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct ConfigPlaylistBorder {
    component: CEColorSelect,
}

impl ConfigPlaylistBorder {
    pub fn new(config: SharedTuiSettings) -> Self {
        let color = config.read().settings.theme.playlist_border();
        Self {
            component: CEColorSelect::new(
                " Border ",
                IdCETheme::PlaylistBorder,
                color,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::Theme(KFMsg::Next)),
                Msg::ConfigEditor(ConfigEditorMsg::Theme(KFMsg::Previous)),
            ),
        }
    }
}

impl Component<Msg, UserEvent> for ConfigPlaylistBorder {
    fn on(&mut self, ev: Event<UserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct ConfigPlaylistHighlight {
    component: CEColorSelect,
}

impl ConfigPlaylistHighlight {
    pub fn new(config: SharedTuiSettings) -> Self {
        let color = config.read().settings.theme.playlist_highlight();
        Self {
            component: CEColorSelect::new(
                " Highlight ",
                IdCETheme::PlaylistHighlight,
                color,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::Theme(KFMsg::Next)),
                Msg::ConfigEditor(ConfigEditorMsg::Theme(KFMsg::Previous)),
            ),
        }
    }
}

impl Component<Msg, UserEvent> for ConfigPlaylistHighlight {
    fn on(&mut self, ev: Event<UserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[inline]
fn progress_title(config: &SharedTuiSettings) -> CEStyleTitle {
    CEStyleTitle::new(config, " Progress style ")
}

#[derive(MockComponent)]
pub struct ConfigProgressForeground {
    component: CEColorSelect,
}

impl ConfigProgressForeground {
    pub fn new(config: SharedTuiSettings) -> Self {
        let color = config.read().settings.theme.progress_foreground();
        Self {
            component: CEColorSelect::new(
                " Foreground ",
                IdCETheme::ProgressForeground,
                color,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::Theme(KFMsg::Next)),
                Msg::ConfigEditor(ConfigEditorMsg::Theme(KFMsg::Previous)),
            ),
        }
    }
}

impl Component<Msg, UserEvent> for ConfigProgressForeground {
    fn on(&mut self, ev: Event<UserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct ConfigProgressBackground {
    component: CEColorSelect,
}

impl ConfigProgressBackground {
    pub fn new(config: SharedTuiSettings) -> Self {
        let color = config.read().settings.theme.progress_background();
        Self {
            component: CEColorSelect::new(
                " Background ",
                IdCETheme::ProgressBackground,
                color,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::Theme(KFMsg::Next)),
                Msg::ConfigEditor(ConfigEditorMsg::Theme(KFMsg::Previous)),
            ),
        }
    }
}

impl Component<Msg, UserEvent> for ConfigProgressBackground {
    fn on(&mut self, ev: Event<UserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct ConfigProgressBorder {
    component: CEColorSelect,
}

impl ConfigProgressBorder {
    pub fn new(config: SharedTuiSettings) -> Self {
        let color = config.read().settings.theme.progress_border();
        Self {
            component: CEColorSelect::new(
                " Border ",
                IdCETheme::ProgressBorder,
                color,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::Theme(KFMsg::Next)),
                Msg::ConfigEditor(ConfigEditorMsg::Theme(KFMsg::Previous)),
            ),
        }
    }
}

impl Component<Msg, UserEvent> for ConfigProgressBorder {
    fn on(&mut self, ev: Event<UserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[inline]
fn lyric_title(config: &SharedTuiSettings) -> CEStyleTitle {
    CEStyleTitle::new(config, " Lyric style ")
}

#[derive(MockComponent)]
pub struct ConfigLyricForeground {
    component: CEColorSelect,
}

impl ConfigLyricForeground {
    pub fn new(config: SharedTuiSettings) -> Self {
        let color = config.read().settings.theme.lyric_foreground();
        Self {
            component: CEColorSelect::new(
                " Foreground ",
                IdCETheme::LyricForeground,
                color,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::Theme(KFMsg::Next)),
                Msg::ConfigEditor(ConfigEditorMsg::Theme(KFMsg::Previous)),
            ),
        }
    }
}

impl Component<Msg, UserEvent> for ConfigLyricForeground {
    fn on(&mut self, ev: Event<UserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct ConfigLyricBackground {
    component: CEColorSelect,
}

impl ConfigLyricBackground {
    pub fn new(config: SharedTuiSettings) -> Self {
        let color = config.read().settings.theme.lyric_background();
        Self {
            component: CEColorSelect::new(
                " Background ",
                IdCETheme::LyricBackground,
                color,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::Theme(KFMsg::Next)),
                Msg::ConfigEditor(ConfigEditorMsg::Theme(KFMsg::Previous)),
            ),
        }
    }
}

impl Component<Msg, UserEvent> for ConfigLyricBackground {
    fn on(&mut self, ev: Event<UserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct ConfigLyricBorder {
    component: CEColorSelect,
}

impl ConfigLyricBorder {
    pub fn new(config: SharedTuiSettings) -> Self {
        let color = config.read().settings.theme.lyric_border();
        Self {
            component: CEColorSelect::new(
                " Border ",
                IdCETheme::LyricBorder,
                color,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::Theme(KFMsg::Next)),
                Msg::ConfigEditor(ConfigEditorMsg::Theme(KFMsg::Previous)),
            ),
        }
    }
}

impl Component<Msg, UserEvent> for ConfigLyricBorder {
    fn on(&mut self, ev: Event<UserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct ConfigInputHighlight {
    component: Input,
    id: IdConfigEditor,
    config: SharedTuiSettings,
}

impl ConfigInputHighlight {
    pub fn new(name: &str, id: IdConfigEditor, config: SharedTuiSettings) -> Self {
        let config_r = config.read();
        // TODO: this should likely not be here, because it is a runtime error if it is unhandled
        let highlight_str = match id {
            IdConfigEditor::Theme(IdCETheme::LibraryHighlightSymbol) => {
                &config_r.settings.theme.style.library.highlight_symbol
            }
            IdConfigEditor::Theme(IdCETheme::PlaylistHighlightSymbol) => {
                &config_r.settings.theme.style.playlist.highlight_symbol
            }
            IdConfigEditor::Theme(IdCETheme::CurrentlyPlayingTrackSymbol) => {
                &config_r.settings.theme.style.playlist.current_track_symbol
            }
            _ => todo!("Unhandled IdConfigEditor Variant: {:#?}", id),
        };
        let component = Input::default()
            .borders(
                Borders::default()
                    .modifiers(BorderType::Rounded)
                    .color(config_r.settings.theme.library_border()),
            )
            // .foreground(color)
            .input_type(InputType::Text)
            .placeholder(
                "1f984/1f680/1f8a5",
                Style::default().fg(Color::Rgb(128, 128, 128)),
            )
            .title(name, Alignment::Left)
            .value(highlight_str);

        drop(config_r);
        Self {
            component,
            id,
            config,
        }
    }
    fn update_symbol(&mut self, result: CmdResult) -> Msg {
        if let CmdResult::Changed(State::One(StateValue::String(symbol))) = result.clone() {
            if symbol.is_empty() {
                let color = self.config.read().settings.theme.library_border();
                self.update_symbol_after(color);
                return Msg::ForceRedraw;
            }
            if let Some(s) = Self::string_to_unicode_char(&symbol) {
                // success getting a unicode letter
                self.update_symbol_after(Color::Green);
                return Msg::ConfigEditor(ConfigEditorMsg::SymbolChanged(self.id, s.to_string()));
            }
            // fail to get a unicode letter
            self.update_symbol_after(Color::Red);
        }

        // press enter to see preview
        if let CmdResult::Submit(State::One(StateValue::String(symbol))) = result
            && let Some(s) = Self::string_to_unicode_char(&symbol)
        {
            self.attr(Attribute::Value, AttrValue::String(s.to_string()));
            self.update_symbol_after(Color::Green);
            return Msg::ConfigEditor(ConfigEditorMsg::SymbolChanged(self.id, s.to_string()));
        }
        Msg::ForceRedraw
    }
    fn update_symbol_after(&mut self, color: Color) {
        self.attr(Attribute::Foreground, AttrValue::Color(color));
        self.attr(
            Attribute::Borders,
            AttrValue::Borders(
                Borders::default()
                    .modifiers(BorderType::Rounded)
                    .color(color),
            ),
        );
    }
    fn string_to_unicode_char(s: &str) -> Option<char> {
        // Do something more appropriate to find the actual number
        // let number = &s[..];

        u32::from_str_radix(s, 16)
            .ok()
            .and_then(std::char::from_u32)
    }
}

impl Component<Msg, UserEvent> for ConfigInputHighlight {
    fn on(&mut self, ev: Event<UserEvent>) -> Option<Msg> {
        let config = self.config.clone();
        let keys = &config.read().settings.keys;
        match ev {
            // Global Hotkeys
            Event::Keyboard(keyevent) if keyevent == keys.config_keys.save.get() => {
                Some(Msg::ConfigEditor(ConfigEditorMsg::CloseOk))
            }
            Event::Keyboard(KeyEvent { code: Key::Tab, .. }) => {
                Some(Msg::ConfigEditor(ConfigEditorMsg::ChangeLayout))
            }
            Event::Keyboard(keyevent) if keyevent == keys.escape.get() => {
                Some(Msg::ConfigEditor(ConfigEditorMsg::CloseCancel))
            }
            Event::Keyboard(KeyEvent {
                code: Key::Left, ..
            }) => {
                self.perform(Cmd::Move(Direction::Left));
                Some(Msg::ForceRedraw)
            }
            Event::Keyboard(KeyEvent {
                code: Key::Right, ..
            }) => {
                self.perform(Cmd::Move(Direction::Right));
                Some(Msg::ForceRedraw)
            }
            Event::Keyboard(KeyEvent {
                code: Key::Home, ..
            }) => {
                self.perform(Cmd::GoTo(Position::Begin));
                Some(Msg::ForceRedraw)
            }
            Event::Keyboard(KeyEvent { code: Key::End, .. }) => {
                self.perform(Cmd::GoTo(Position::End));
                Some(Msg::ForceRedraw)
            }
            Event::Keyboard(KeyEvent {
                code: Key::Delete, ..
            }) => {
                let result = self.perform(Cmd::Cancel);
                Some(self.update_symbol(result))
            }
            Event::Keyboard(KeyEvent {
                code: Key::Backspace,
                ..
            }) => {
                let result = self.perform(Cmd::Delete);
                Some(self.update_symbol(result))
            }

            Event::Keyboard(KeyEvent {
                code: Key::Char(ch),
                modifiers: KeyModifiers::NONE | KeyModifiers::SHIFT,
            }) => {
                let result = self.perform(Cmd::Type(ch));
                Some(self.update_symbol(result))
            }
            Event::Keyboard(KeyEvent {
                code: Key::Down, ..
            }) => Some(Msg::ConfigEditor(ConfigEditorMsg::Theme(KFMsg::Next))),
            Event::Keyboard(KeyEvent { code: Key::Up, .. }) => {
                Some(Msg::ConfigEditor(ConfigEditorMsg::Theme(KFMsg::Previous)))
            }

            Event::Keyboard(KeyEvent {
                code: Key::Enter, ..
            }) => {
                let result = self.perform(Cmd::Submit);
                Some(self.update_symbol(result))
            }
            _ => None,
        }
    }
}

#[derive(MockComponent)]
pub struct ConfigLibraryHighlightSymbol {
    component: ConfigInputHighlight,
}

impl ConfigLibraryHighlightSymbol {
    pub fn new(config: SharedTuiSettings) -> Self {
        Self {
            component: ConfigInputHighlight::new(
                " Highlight Symbol ",
                IdConfigEditor::Theme(IdCETheme::LibraryHighlightSymbol),
                config,
            ),
        }
    }
}

impl Component<Msg, UserEvent> for ConfigLibraryHighlightSymbol {
    fn on(&mut self, ev: Event<UserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct ConfigPlaylistHighlightSymbol {
    component: ConfigInputHighlight,
}

impl ConfigPlaylistHighlightSymbol {
    pub fn new(config: SharedTuiSettings) -> Self {
        Self {
            component: ConfigInputHighlight::new(
                " Highlight Symbol ",
                IdConfigEditor::Theme(IdCETheme::PlaylistHighlightSymbol),
                config,
            ),
        }
    }
}

impl Component<Msg, UserEvent> for ConfigPlaylistHighlightSymbol {
    fn on(&mut self, ev: Event<UserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct ConfigCurrentlyPlayingTrackSymbol {
    component: ConfigInputHighlight,
}

impl ConfigCurrentlyPlayingTrackSymbol {
    pub fn new(config: SharedTuiSettings) -> Self {
        Self {
            component: ConfigInputHighlight::new(
                " Current Track Symbol ",
                IdConfigEditor::Theme(IdCETheme::CurrentlyPlayingTrackSymbol),
                config,
            ),
        }
    }
}

impl Component<Msg, UserEvent> for ConfigCurrentlyPlayingTrackSymbol {
    fn on(&mut self, ev: Event<UserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[inline]
fn important_popup_title(config: &SharedTuiSettings) -> CEStyleTitle {
    CEStyleTitle::new(config, " Important Popup style ")
}

#[derive(MockComponent)]
pub struct ConfigImportantPopupForeground {
    component: CEColorSelect,
}

impl ConfigImportantPopupForeground {
    pub fn new(config: SharedTuiSettings) -> Self {
        let color = config.read().settings.theme.important_popup_foreground();
        Self {
            component: CEColorSelect::new(
                " Foreground ",
                IdCETheme::ImportantPopupForeground,
                color,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::Theme(KFMsg::Next)),
                Msg::ConfigEditor(ConfigEditorMsg::Theme(KFMsg::Previous)),
            ),
        }
    }
}

impl Component<Msg, UserEvent> for ConfigImportantPopupForeground {
    fn on(&mut self, ev: Event<UserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct ConfigImportantPopupBackground {
    component: CEColorSelect,
}

impl ConfigImportantPopupBackground {
    pub fn new(config: SharedTuiSettings) -> Self {
        let color = config.read().settings.theme.important_popup_background();
        Self {
            component: CEColorSelect::new(
                " Background ",
                IdCETheme::ImportantPopupBackground,
                color,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::Theme(KFMsg::Next)),
                Msg::ConfigEditor(ConfigEditorMsg::Theme(KFMsg::Previous)),
            ),
        }
    }
}

impl Component<Msg, UserEvent> for ConfigImportantPopupBackground {
    fn on(&mut self, ev: Event<UserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct ConfigImportantPopupBorder {
    component: CEColorSelect,
}

impl ConfigImportantPopupBorder {
    pub fn new(config: SharedTuiSettings) -> Self {
        let color = config.read().settings.theme.important_popup_border();
        Self {
            component: CEColorSelect::new(
                " Border ",
                IdCETheme::ImportantPopupBorder,
                color,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::Theme(KFMsg::Next)),
                Msg::ConfigEditor(ConfigEditorMsg::Theme(KFMsg::Previous)),
            ),
        }
    }
}

impl Component<Msg, UserEvent> for ConfigImportantPopupBorder {
    fn on(&mut self, ev: Event<UserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[inline]
fn fallback_title(config: &SharedTuiSettings) -> CEStyleTitle {
    CEStyleTitle::new(config, " Fallback style ")
}

#[derive(MockComponent)]
pub struct ConfigFallbackForeground {
    component: CEColorSelect,
}

impl ConfigFallbackForeground {
    pub fn new(config: SharedTuiSettings) -> Self {
        let color = config.read().settings.theme.fallback_foreground();
        Self {
            component: CEColorSelect::new(
                " Foreground ",
                IdCETheme::FallbackForeground,
                color,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::Theme(KFMsg::Next)),
                Msg::ConfigEditor(ConfigEditorMsg::Theme(KFMsg::Previous)),
            ),
        }
    }
}

impl Component<Msg, UserEvent> for ConfigFallbackForeground {
    fn on(&mut self, ev: Event<UserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct ConfigFallbackBackground {
    component: CEColorSelect,
}

impl ConfigFallbackBackground {
    pub fn new(config: SharedTuiSettings) -> Self {
        let color = config.read().settings.theme.library_background();
        Self {
            component: CEColorSelect::new(
                " Background ",
                IdCETheme::FallbackBackground,
                color,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::Theme(KFMsg::Next)),
                Msg::ConfigEditor(ConfigEditorMsg::Theme(KFMsg::Previous)),
            ),
        }
    }
}

impl Component<Msg, UserEvent> for ConfigFallbackBackground {
    fn on(&mut self, ev: Event<UserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct ConfigFallbackBorder {
    component: CEColorSelect,
}

impl ConfigFallbackBorder {
    pub fn new(config: SharedTuiSettings) -> Self {
        let color = config.read().settings.theme.library_border();
        Self {
            component: CEColorSelect::new(
                " Border ",
                IdCETheme::FallbackBorder,
                color,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::Theme(KFMsg::Next)),
                Msg::ConfigEditor(ConfigEditorMsg::Theme(KFMsg::Previous)),
            ),
        }
    }
}

impl Component<Msg, UserEvent> for ConfigFallbackBorder {
    fn on(&mut self, ev: Event<UserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct ConfigFallbackHighlight {
    component: CEColorSelect,
}

impl ConfigFallbackHighlight {
    pub fn new(config: SharedTuiSettings) -> Self {
        let color = config.read().settings.theme.library_highlight();
        Self {
            component: CEColorSelect::new(
                " Highlight ",
                IdCETheme::FallbackHighlight,
                color,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::Theme(KFMsg::Next)),
                Msg::ConfigEditor(ConfigEditorMsg::Theme(KFMsg::Previous)),
            ),
        }
    }
}

impl Component<Msg, UserEvent> for ConfigFallbackHighlight {
    fn on(&mut self, ev: Event<UserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

impl Model {
    /// Mount / Remount the Config-Editor's Second Page, the Theme, Color & Symbol Options
    #[allow(clippy::too_many_lines)]
    pub(super) fn remount_config_color(
        &mut self,
        config: &SharedTuiSettings,
        theme_idx: Option<usize>,
    ) -> Result<()> {
        // Mount color page
        self.app.remount(
            Id::ConfigEditor(IdConfigEditor::Theme(IdCETheme::ThemeSelectTable)),
            Box::new(CEThemeSelectTable::new(config.clone())),
            Vec::new(),
        )?;

        self.app.remount(
            Id::ConfigEditor(IdConfigEditor::Theme(IdCETheme::LibraryLabel)),
            Box::new(library_title(config)),
            Vec::new(),
        )?;
        self.app.remount(
            Id::ConfigEditor(IdConfigEditor::Theme(IdCETheme::LibraryForeground)),
            Box::new(ConfigLibraryForeground::new(config.clone())),
            Vec::new(),
        )?;
        self.app.remount(
            Id::ConfigEditor(IdConfigEditor::Theme(IdCETheme::LibraryBackground)),
            Box::new(ConfigLibraryBackground::new(config.clone())),
            Vec::new(),
        )?;
        self.app.remount(
            Id::ConfigEditor(IdConfigEditor::Theme(IdCETheme::LibraryBorder)),
            Box::new(ConfigLibraryBorder::new(config.clone())),
            Vec::new(),
        )?;
        self.app.remount(
            Id::ConfigEditor(IdConfigEditor::Theme(IdCETheme::LibraryHighlight)),
            Box::new(ConfigLibraryHighlight::new(config.clone())),
            Vec::new(),
        )?;

        self.app.remount(
            Id::ConfigEditor(IdConfigEditor::Theme(IdCETheme::PlaylistLabel)),
            Box::new(playlist_title(config)),
            Vec::new(),
        )?;

        self.app.remount(
            Id::ConfigEditor(IdConfigEditor::Theme(IdCETheme::PlaylistForeground)),
            Box::new(ConfigPlaylistForeground::new(config.clone())),
            Vec::new(),
        )?;

        self.app.remount(
            Id::ConfigEditor(IdConfigEditor::Theme(IdCETheme::PlaylistBackground)),
            Box::new(ConfigPlaylistBackground::new(config.clone())),
            Vec::new(),
        )?;

        self.app.remount(
            Id::ConfigEditor(IdConfigEditor::Theme(IdCETheme::PlaylistBorder)),
            Box::new(ConfigPlaylistBorder::new(config.clone())),
            Vec::new(),
        )?;

        self.app.remount(
            Id::ConfigEditor(IdConfigEditor::Theme(IdCETheme::PlaylistHighlight)),
            Box::new(ConfigPlaylistHighlight::new(config.clone())),
            Vec::new(),
        )?;

        self.app.remount(
            Id::ConfigEditor(IdConfigEditor::Theme(IdCETheme::ProgressLabel)),
            Box::new(progress_title(config)),
            Vec::new(),
        )?;

        self.app.remount(
            Id::ConfigEditor(IdConfigEditor::Theme(IdCETheme::ProgressForeground)),
            Box::new(ConfigProgressForeground::new(config.clone())),
            Vec::new(),
        )?;

        self.app.remount(
            Id::ConfigEditor(IdConfigEditor::Theme(IdCETheme::ProgressBackground)),
            Box::new(ConfigProgressBackground::new(config.clone())),
            Vec::new(),
        )?;

        self.app.remount(
            Id::ConfigEditor(IdConfigEditor::Theme(IdCETheme::ProgressBorder)),
            Box::new(ConfigProgressBorder::new(config.clone())),
            Vec::new(),
        )?;

        self.app.remount(
            Id::ConfigEditor(IdConfigEditor::Theme(IdCETheme::LyricLabel)),
            Box::new(lyric_title(config)),
            Vec::new(),
        )?;
        self.app.remount(
            Id::ConfigEditor(IdConfigEditor::Theme(IdCETheme::LyricForeground)),
            Box::new(ConfigLyricForeground::new(config.clone())),
            Vec::new(),
        )?;
        self.app.remount(
            Id::ConfigEditor(IdConfigEditor::Theme(IdCETheme::LyricBackground)),
            Box::new(ConfigLyricBackground::new(config.clone())),
            Vec::new(),
        )?;
        self.app.remount(
            Id::ConfigEditor(IdConfigEditor::Theme(IdCETheme::LyricBorder)),
            Box::new(ConfigLyricBorder::new(config.clone())),
            Vec::new(),
        )?;

        self.app.remount(
            Id::ConfigEditor(IdConfigEditor::Theme(IdCETheme::ImportantPopupLabel)),
            Box::new(important_popup_title(config)),
            Vec::new(),
        )?;
        self.app.remount(
            Id::ConfigEditor(IdConfigEditor::Theme(IdCETheme::ImportantPopupForeground)),
            Box::new(ConfigImportantPopupForeground::new(config.clone())),
            Vec::new(),
        )?;
        self.app.remount(
            Id::ConfigEditor(IdConfigEditor::Theme(IdCETheme::ImportantPopupBackground)),
            Box::new(ConfigImportantPopupBackground::new(config.clone())),
            Vec::new(),
        )?;
        self.app.remount(
            Id::ConfigEditor(IdConfigEditor::Theme(IdCETheme::ImportantPopupBorder)),
            Box::new(ConfigImportantPopupBorder::new(config.clone())),
            Vec::new(),
        )?;

        self.app.remount(
            Id::ConfigEditor(IdConfigEditor::Theme(IdCETheme::FallbackLabel)),
            Box::new(fallback_title(config)),
            Vec::new(),
        )?;
        self.app.remount(
            Id::ConfigEditor(IdConfigEditor::Theme(IdCETheme::FallbackForeground)),
            Box::new(ConfigFallbackForeground::new(config.clone())),
            Vec::new(),
        )?;
        self.app.remount(
            Id::ConfigEditor(IdConfigEditor::Theme(IdCETheme::FallbackBackground)),
            Box::new(ConfigFallbackBackground::new(config.clone())),
            Vec::new(),
        )?;
        self.app.remount(
            Id::ConfigEditor(IdConfigEditor::Theme(IdCETheme::FallbackBorder)),
            Box::new(ConfigFallbackBorder::new(config.clone())),
            Vec::new(),
        )?;
        self.app.remount(
            Id::ConfigEditor(IdConfigEditor::Theme(IdCETheme::FallbackHighlight)),
            Box::new(ConfigFallbackHighlight::new(config.clone())),
            Vec::new(),
        )?;

        self.remount_config_color_symbols(config)?;

        self.theme_select_sync(theme_idx);

        Ok(())
    }

    /// Mount / Remount the Config-Editor's Second Page, Symbols
    fn remount_config_color_symbols(&mut self, config: &SharedTuiSettings) -> Result<()> {
        self.app.remount(
            Id::ConfigEditor(IdConfigEditor::Theme(IdCETheme::LibraryHighlightSymbol)),
            Box::new(ConfigLibraryHighlightSymbol::new(config.clone())),
            Vec::new(),
        )?;

        self.app.remount(
            Id::ConfigEditor(IdConfigEditor::Theme(IdCETheme::PlaylistHighlightSymbol)),
            Box::new(ConfigPlaylistHighlightSymbol::new(config.clone())),
            Vec::new(),
        )?;

        self.app.remount(
            Id::ConfigEditor(IdConfigEditor::Theme(
                IdCETheme::CurrentlyPlayingTrackSymbol,
            )),
            Box::new(ConfigCurrentlyPlayingTrackSymbol::new(config.clone())),
            Vec::new(),
        )?;

        Ok(())
    }

    /// Unmount the Config-Editor's Second Page, the Theme, Color & Symbol Options
    pub(super) fn umount_config_color(&mut self) -> Result<()> {
        self.app.umount(&Id::ConfigEditor(IdConfigEditor::Theme(
            IdCETheme::ThemeSelectTable,
        )))?;

        self.app.umount(&Id::ConfigEditor(IdConfigEditor::Theme(
            IdCETheme::LibraryLabel,
        )))?;
        self.app.umount(&Id::ConfigEditor(IdConfigEditor::Theme(
            IdCETheme::LibraryForeground,
        )))?;

        self.app.umount(&Id::ConfigEditor(IdConfigEditor::Theme(
            IdCETheme::LibraryBackground,
        )))?;
        self.app.umount(&Id::ConfigEditor(IdConfigEditor::Theme(
            IdCETheme::LibraryBorder,
        )))?;
        self.app.umount(&Id::ConfigEditor(IdConfigEditor::Theme(
            IdCETheme::LibraryHighlight,
        )))?;

        self.app.umount(&Id::ConfigEditor(IdConfigEditor::Theme(
            IdCETheme::PlaylistLabel,
        )))?;

        self.app.umount(&Id::ConfigEditor(IdConfigEditor::Theme(
            IdCETheme::PlaylistForeground,
        )))?;

        self.app.umount(&Id::ConfigEditor(IdConfigEditor::Theme(
            IdCETheme::PlaylistBackground,
        )))?;
        self.app.umount(&Id::ConfigEditor(IdConfigEditor::Theme(
            IdCETheme::PlaylistBorder,
        )))?;
        self.app.umount(&Id::ConfigEditor(IdConfigEditor::Theme(
            IdCETheme::PlaylistHighlight,
        )))?;
        self.app.umount(&Id::ConfigEditor(IdConfigEditor::Theme(
            IdCETheme::ProgressLabel,
        )))?;

        self.app.umount(&Id::ConfigEditor(IdConfigEditor::Theme(
            IdCETheme::ProgressForeground,
        )))?;
        self.app.umount(&Id::ConfigEditor(IdConfigEditor::Theme(
            IdCETheme::ProgressBackground,
        )))?;
        self.app.umount(&Id::ConfigEditor(IdConfigEditor::Theme(
            IdCETheme::ProgressBorder,
        )))?;

        self.app.umount(&Id::ConfigEditor(IdConfigEditor::Theme(
            IdCETheme::LyricLabel,
        )))?;
        self.app.umount(&Id::ConfigEditor(IdConfigEditor::Theme(
            IdCETheme::LyricForeground,
        )))?;
        self.app.umount(&Id::ConfigEditor(IdConfigEditor::Theme(
            IdCETheme::LyricBackground,
        )))?;
        self.app.umount(&Id::ConfigEditor(IdConfigEditor::Theme(
            IdCETheme::LyricBorder,
        )))?;

        self.app.umount(&Id::ConfigEditor(IdConfigEditor::Theme(
            IdCETheme::ImportantPopupLabel,
        )))?;
        self.app.umount(&Id::ConfigEditor(IdConfigEditor::Theme(
            IdCETheme::ImportantPopupForeground,
        )))?;
        self.app.umount(&Id::ConfigEditor(IdConfigEditor::Theme(
            IdCETheme::ImportantPopupBackground,
        )))?;
        self.app.umount(&Id::ConfigEditor(IdConfigEditor::Theme(
            IdCETheme::ImportantPopupBorder,
        )))?;

        self.app.umount(&Id::ConfigEditor(IdConfigEditor::Theme(
            IdCETheme::FallbackLabel,
        )))?;
        self.app.umount(&Id::ConfigEditor(IdConfigEditor::Theme(
            IdCETheme::FallbackForeground,
        )))?;

        self.app.umount(&Id::ConfigEditor(IdConfigEditor::Theme(
            IdCETheme::FallbackBackground,
        )))?;
        self.app.umount(&Id::ConfigEditor(IdConfigEditor::Theme(
            IdCETheme::FallbackBorder,
        )))?;
        self.app.umount(&Id::ConfigEditor(IdConfigEditor::Theme(
            IdCETheme::FallbackHighlight,
        )))?;

        self.umount_config_color_symbols()?;

        Ok(())
    }

    /// Unmount the Config-Editor's Second Page, Symbols
    pub fn umount_config_color_symbols(&mut self) -> Result<()> {
        self.app.umount(&Id::ConfigEditor(IdConfigEditor::Theme(
            IdCETheme::LibraryHighlightSymbol,
        )))?;
        self.app.umount(&Id::ConfigEditor(IdConfigEditor::Theme(
            IdCETheme::PlaylistHighlightSymbol,
        )))?;
        self.app.umount(&Id::ConfigEditor(IdConfigEditor::Theme(
            IdCETheme::CurrentlyPlayingTrackSymbol,
        )))?;

        Ok(())
    }
}
