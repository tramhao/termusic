use anyhow::Result;
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
use std::convert::From;
use termusiclib::config::v2::tui::theme::styles::ColorTermusic;
use termusiclib::config::v2::tui::theme::ThemeWrap;
use termusiclib::config::SharedTuiSettings;
use termusiclib::ids::{Id, IdConfigEditor};
use termusiclib::types::{ConfigEditorMsg, Msg};
use tui_realm_stdlib::{Input, Label, Select, Table};
use tuirealm::command::{Cmd, CmdResult, Direction, Position};
use tuirealm::event::{Key, KeyEvent, KeyModifiers, NoUserEvent};
use tuirealm::props::{
    Alignment, BorderType, Borders, Color, InputType, Style, TableBuilder, TextModifiers, TextSpan,
};
use tuirealm::ratatui::style::Modifier;
use tuirealm::{AttrValue, Attribute, Component, Event, MockComponent, State, StateValue};

use crate::ui::model::Model;

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
                .headers(&["index", "Theme Name"])
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

impl Component<Msg, NoUserEvent> for CEThemeSelectTable {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
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
                return Some(Msg::ConfigEditor(ConfigEditorMsg::ThemeSelectBlurDown));
            }
            Event::Keyboard(KeyEvent {
                code: Key::BackTab,
                modifiers: KeyModifiers::SHIFT,
            }) => {
                return Some(Msg::ConfigEditor(ConfigEditorMsg::ThemeSelectBlurUp));
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
    id: IdConfigEditor,
    config: SharedTuiSettings,
    on_key_shift: Msg,
    on_key_backshift: Msg,
}

impl CEColorSelect {
    pub fn new(
        name: &str,
        id: IdConfigEditor,
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
                .choices(&choices)
                .value(init_value),
            id,
            config,
            on_key_shift,
            on_key_backshift,
        }
    }

    const fn init_color_select(id: IdConfigEditor, theme: &ThemeWrap) -> usize {
        match id {
            IdConfigEditor::LibraryForeground => theme.style.library.foreground_color.as_usize(),
            IdConfigEditor::LibraryBackground => theme.style.library.background_color.as_usize(),
            IdConfigEditor::LibraryBorder => theme.style.library.border_color.as_usize(),
            IdConfigEditor::LibraryHighlight => theme.style.library.highlight_color.as_usize(),

            IdConfigEditor::PlaylistForeground => theme.style.playlist.foreground_color.as_usize(),
            IdConfigEditor::PlaylistBackground => theme.style.playlist.background_color.as_usize(),
            IdConfigEditor::PlaylistBorder => theme.style.playlist.border_color.as_usize(),
            IdConfigEditor::PlaylistHighlight => theme.style.playlist.highlight_color.as_usize(),

            IdConfigEditor::ProgressForeground => theme.style.progress.foreground_color.as_usize(),
            IdConfigEditor::ProgressBackground => theme.style.progress.background_color.as_usize(),
            IdConfigEditor::ProgressBorder => theme.style.progress.border_color.as_usize(),

            IdConfigEditor::LyricForeground => theme.style.lyric.foreground_color.as_usize(),
            IdConfigEditor::LyricBackground => theme.style.lyric.background_color.as_usize(),
            IdConfigEditor::LyricBorder => theme.style.lyric.border_color.as_usize(),

            IdConfigEditor::ImportantPopupForeground => {
                theme.style.important_popup.foreground_color.as_usize()
            }
            IdConfigEditor::ImportantPopupBackground => {
                theme.style.important_popup.background_color.as_usize()
            }
            IdConfigEditor::ImportantPopupBorder => {
                theme.style.important_popup.border_color.as_usize()
            }

            // TODO: add fallback colors
            _ => 0,
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
            Msg::ConfigEditor(ConfigEditorMsg::ColorChanged(self.id, *color_config))
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

impl Component<Msg, NoUserEvent> for CEColorSelect {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
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
            }) => self.perform(Cmd::Submit),
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
pub struct ConfigLibraryTitle {
    component: Label,
}

impl Default for ConfigLibraryTitle {
    fn default() -> Self {
        Self {
            component: Label::default()
                .modifiers(TextModifiers::BOLD)
                .text(" Library style "),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigLibraryTitle {
    fn on(&mut self, _ev: Event<NoUserEvent>) -> Option<Msg> {
        None
    }
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
                IdConfigEditor::LibraryForeground,
                color,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::LibraryForegroundBlurDown),
                Msg::ConfigEditor(ConfigEditorMsg::LibraryForegroundBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigLibraryForeground {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
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
                IdConfigEditor::LibraryBackground,
                color,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::LibraryBackgroundBlurDown),
                Msg::ConfigEditor(ConfigEditorMsg::LibraryBackgroundBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigLibraryBackground {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
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
                IdConfigEditor::LibraryBorder,
                color,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::LibraryBorderBlurDown),
                Msg::ConfigEditor(ConfigEditorMsg::LibraryBorderBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigLibraryBorder {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
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
                IdConfigEditor::LibraryHighlight,
                color,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::LibraryHighlightBlurDown),
                Msg::ConfigEditor(ConfigEditorMsg::LibraryHighlightBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigLibraryHighlight {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct ConfigPlaylistTitle {
    component: Label,
}

impl Default for ConfigPlaylistTitle {
    fn default() -> Self {
        Self {
            component: Label::default()
                .modifiers(TextModifiers::BOLD)
                .text(" Playlist style "),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigPlaylistTitle {
    fn on(&mut self, _ev: Event<NoUserEvent>) -> Option<Msg> {
        None
    }
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
                IdConfigEditor::PlaylistForeground,
                color,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::PlaylistForegroundBlurDown),
                Msg::ConfigEditor(ConfigEditorMsg::PlaylistForegroundBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigPlaylistForeground {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
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
                IdConfigEditor::PlaylistBackground,
                color,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::PlaylistBackgroundBlurDown),
                Msg::ConfigEditor(ConfigEditorMsg::PlaylistBackgroundBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigPlaylistBackground {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
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
                IdConfigEditor::PlaylistBorder,
                color,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::PlaylistBorderBlurDown),
                Msg::ConfigEditor(ConfigEditorMsg::PlaylistBorderBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigPlaylistBorder {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
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
                IdConfigEditor::PlaylistHighlight,
                color,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::PlaylistHighlightBlurDown),
                Msg::ConfigEditor(ConfigEditorMsg::PlaylistHighlightBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigPlaylistHighlight {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct ConfigProgressTitle {
    component: Label,
}

impl Default for ConfigProgressTitle {
    fn default() -> Self {
        Self {
            component: Label::default()
                .modifiers(TextModifiers::BOLD)
                .text(" Progress style "),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigProgressTitle {
    fn on(&mut self, _ev: Event<NoUserEvent>) -> Option<Msg> {
        None
    }
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
                IdConfigEditor::ProgressForeground,
                color,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::ProgressForegroundBlurDown),
                Msg::ConfigEditor(ConfigEditorMsg::ProgressForegroundBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigProgressForeground {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
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
                IdConfigEditor::ProgressBackground,
                color,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::ProgressBackgroundBlurDown),
                Msg::ConfigEditor(ConfigEditorMsg::ProgressBackgroundBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigProgressBackground {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
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
                IdConfigEditor::ProgressBorder,
                color,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::ProgressBorderBlurDown),
                Msg::ConfigEditor(ConfigEditorMsg::ProgressBorderBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigProgressBorder {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct ConfigLyricTitle {
    component: Label,
}

impl Default for ConfigLyricTitle {
    fn default() -> Self {
        Self {
            component: Label::default()
                .modifiers(TextModifiers::BOLD)
                .text(" Lyric style "),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigLyricTitle {
    fn on(&mut self, _ev: Event<NoUserEvent>) -> Option<Msg> {
        None
    }
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
                IdConfigEditor::LyricForeground,
                color,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::LyricForegroundBlurDown),
                Msg::ConfigEditor(ConfigEditorMsg::LyricForegroundBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigLyricForeground {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
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
                IdConfigEditor::LyricBackground,
                color,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::LyricBackgroundBlurDown),
                Msg::ConfigEditor(ConfigEditorMsg::LyricBackgroundBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigLyricBackground {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
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
                IdConfigEditor::LyricBorder,
                color,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::LyricBorderBlurDown),
                Msg::ConfigEditor(ConfigEditorMsg::LyricBorderBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigLyricBorder {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
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
            IdConfigEditor::LibraryHighlightSymbol => {
                &config_r.settings.theme.style.library.highlight_symbol
            }
            IdConfigEditor::PlaylistHighlightSymbol => {
                &config_r.settings.theme.style.playlist.highlight_symbol
            }
            IdConfigEditor::CurrentlyPlayingTrackSymbol => {
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
        if let CmdResult::Submit(State::One(StateValue::String(symbol))) = result {
            if let Some(s) = Self::string_to_unicode_char(&symbol) {
                self.attr(Attribute::Value, AttrValue::String(s.to_string()));
                self.update_symbol_after(Color::Green);
                return Msg::ConfigEditor(ConfigEditorMsg::SymbolChanged(self.id, s.to_string()));
            }
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

impl Component<Msg, NoUserEvent> for ConfigInputHighlight {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
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
            }) => match self.id {
                IdConfigEditor::LibraryHighlightSymbol => Some(Msg::ConfigEditor(
                    ConfigEditorMsg::LibraryHighlightSymbolBlurDown,
                )),
                IdConfigEditor::PlaylistHighlightSymbol => Some(Msg::ConfigEditor(
                    ConfigEditorMsg::PlaylistHighlightSymbolBlurDown,
                )),
                IdConfigEditor::CurrentlyPlayingTrackSymbol => Some(Msg::ConfigEditor(
                    ConfigEditorMsg::CurrentlyPlayingTrackSymbolBlurDown,
                )),
                _ => None,
            },
            Event::Keyboard(KeyEvent { code: Key::Up, .. }) => match self.id {
                IdConfigEditor::LibraryHighlightSymbol => Some(Msg::ConfigEditor(
                    ConfigEditorMsg::LibraryHighlightSymbolBlurUp,
                )),
                IdConfigEditor::PlaylistHighlightSymbol => Some(Msg::ConfigEditor(
                    ConfigEditorMsg::PlaylistHighlightSymbolBlurUp,
                )),
                IdConfigEditor::CurrentlyPlayingTrackSymbol => Some(Msg::ConfigEditor(
                    ConfigEditorMsg::CurrentlyPlayingTrackSymbolBlurUp,
                )),
                _ => None,
            },

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
                IdConfigEditor::LibraryHighlightSymbol,
                config,
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigLibraryHighlightSymbol {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
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
                IdConfigEditor::PlaylistHighlightSymbol,
                config,
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigPlaylistHighlightSymbol {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
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
                IdConfigEditor::CurrentlyPlayingTrackSymbol,
                config,
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigCurrentlyPlayingTrackSymbol {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct ConfigImportantPopupTitle {
    component: Label,
}

impl Default for ConfigImportantPopupTitle {
    fn default() -> Self {
        Self {
            component: Label::default()
                .modifiers(TextModifiers::BOLD)
                .text(" Important Popup style "),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigImportantPopupTitle {
    fn on(&mut self, _ev: Event<NoUserEvent>) -> Option<Msg> {
        None
    }
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
                IdConfigEditor::ImportantPopupForeground,
                color,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::ImportantPopupForegroundBlurDown),
                Msg::ConfigEditor(ConfigEditorMsg::ImportantPopupForegroundBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigImportantPopupForeground {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
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
                IdConfigEditor::ImportantPopupBackground,
                color,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::ImportantPopupBackgroundBlurDown),
                Msg::ConfigEditor(ConfigEditorMsg::ImportantPopupBackgroundBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigImportantPopupBackground {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
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
                IdConfigEditor::ImportantPopupBorder,
                color,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::ImportantPopupBorderBlurDown),
                Msg::ConfigEditor(ConfigEditorMsg::ImportantPopupBorderBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigImportantPopupBorder {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct ConfigFallbackTitle {
    component: Label,
}

impl Default for ConfigFallbackTitle {
    fn default() -> Self {
        Self {
            component: Label::default()
                .modifiers(TextModifiers::BOLD)
                .text(" Fallback style "),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigFallbackTitle {
    fn on(&mut self, _ev: Event<NoUserEvent>) -> Option<Msg> {
        None
    }
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
                IdConfigEditor::FallbackForeground,
                color,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::FallbackForegroundBlurDown),
                Msg::ConfigEditor(ConfigEditorMsg::FallbackForegroundBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigFallbackForeground {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
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
                IdConfigEditor::FallbackBackground,
                color,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::FallbackBackgroundBlurDown),
                Msg::ConfigEditor(ConfigEditorMsg::FallbackBackgroundBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigFallbackBackground {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
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
                IdConfigEditor::FallbackBorder,
                color,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::FallbackBorderBlurDown),
                Msg::ConfigEditor(ConfigEditorMsg::FallbackBorderBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigFallbackBorder {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
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
                IdConfigEditor::FallbackHighlight,
                color,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::FallbackHighlightBlurDown),
                Msg::ConfigEditor(ConfigEditorMsg::FallbackHighlightBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigFallbackHighlight {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
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
            Id::ConfigEditor(IdConfigEditor::CEThemeSelect),
            Box::new(CEThemeSelectTable::new(config.clone())),
            Vec::new(),
        )?;

        self.app.remount(
            Id::ConfigEditor(IdConfigEditor::LibraryLabel),
            Box::<ConfigLibraryTitle>::default(),
            Vec::new(),
        )?;
        self.app.remount(
            Id::ConfigEditor(IdConfigEditor::LibraryForeground),
            Box::new(ConfigLibraryForeground::new(config.clone())),
            Vec::new(),
        )?;
        self.app.remount(
            Id::ConfigEditor(IdConfigEditor::LibraryBackground),
            Box::new(ConfigLibraryBackground::new(config.clone())),
            Vec::new(),
        )?;
        self.app.remount(
            Id::ConfigEditor(IdConfigEditor::LibraryBorder),
            Box::new(ConfigLibraryBorder::new(config.clone())),
            Vec::new(),
        )?;
        self.app.remount(
            Id::ConfigEditor(IdConfigEditor::LibraryHighlight),
            Box::new(ConfigLibraryHighlight::new(config.clone())),
            Vec::new(),
        )?;

        self.app.remount(
            Id::ConfigEditor(IdConfigEditor::PlaylistLabel),
            Box::<ConfigPlaylistTitle>::default(),
            Vec::new(),
        )?;

        self.app.remount(
            Id::ConfigEditor(IdConfigEditor::PlaylistForeground),
            Box::new(ConfigPlaylistForeground::new(config.clone())),
            Vec::new(),
        )?;

        self.app.remount(
            Id::ConfigEditor(IdConfigEditor::PlaylistBackground),
            Box::new(ConfigPlaylistBackground::new(config.clone())),
            Vec::new(),
        )?;

        self.app.remount(
            Id::ConfigEditor(IdConfigEditor::PlaylistBorder),
            Box::new(ConfigPlaylistBorder::new(config.clone())),
            Vec::new(),
        )?;

        self.app.remount(
            Id::ConfigEditor(IdConfigEditor::PlaylistHighlight),
            Box::new(ConfigPlaylistHighlight::new(config.clone())),
            Vec::new(),
        )?;

        self.app.remount(
            Id::ConfigEditor(IdConfigEditor::ProgressLabel),
            Box::<ConfigProgressTitle>::default(),
            Vec::new(),
        )?;

        self.app.remount(
            Id::ConfigEditor(IdConfigEditor::ProgressForeground),
            Box::new(ConfigProgressForeground::new(config.clone())),
            Vec::new(),
        )?;

        self.app.remount(
            Id::ConfigEditor(IdConfigEditor::ProgressBackground),
            Box::new(ConfigProgressBackground::new(config.clone())),
            Vec::new(),
        )?;

        self.app.remount(
            Id::ConfigEditor(IdConfigEditor::ProgressBorder),
            Box::new(ConfigProgressBorder::new(config.clone())),
            Vec::new(),
        )?;

        self.app.remount(
            Id::ConfigEditor(IdConfigEditor::LyricLabel),
            Box::<ConfigLyricTitle>::default(),
            Vec::new(),
        )?;
        self.app.remount(
            Id::ConfigEditor(IdConfigEditor::LyricForeground),
            Box::new(ConfigLyricForeground::new(config.clone())),
            Vec::new(),
        )?;
        self.app.remount(
            Id::ConfigEditor(IdConfigEditor::LyricBackground),
            Box::new(ConfigLyricBackground::new(config.clone())),
            Vec::new(),
        )?;
        self.app.remount(
            Id::ConfigEditor(IdConfigEditor::LyricBorder),
            Box::new(ConfigLyricBorder::new(config.clone())),
            Vec::new(),
        )?;

        self.app.remount(
            Id::ConfigEditor(IdConfigEditor::ImportantPopupLabel),
            Box::<ConfigImportantPopupTitle>::default(),
            Vec::new(),
        )?;
        self.app.remount(
            Id::ConfigEditor(IdConfigEditor::ImportantPopupForeground),
            Box::new(ConfigImportantPopupForeground::new(config.clone())),
            Vec::new(),
        )?;
        self.app.remount(
            Id::ConfigEditor(IdConfigEditor::ImportantPopupBackground),
            Box::new(ConfigImportantPopupBackground::new(config.clone())),
            Vec::new(),
        )?;
        self.app.remount(
            Id::ConfigEditor(IdConfigEditor::ImportantPopupBorder),
            Box::new(ConfigImportantPopupBorder::new(config.clone())),
            Vec::new(),
        )?;

        self.app.remount(
            Id::ConfigEditor(IdConfigEditor::FallbackLabel),
            Box::<ConfigFallbackTitle>::default(),
            Vec::new(),
        )?;
        self.app.remount(
            Id::ConfigEditor(IdConfigEditor::FallbackForeground),
            Box::new(ConfigFallbackForeground::new(config.clone())),
            Vec::new(),
        )?;
        self.app.remount(
            Id::ConfigEditor(IdConfigEditor::FallbackBackground),
            Box::new(ConfigFallbackBackground::new(config.clone())),
            Vec::new(),
        )?;
        self.app.remount(
            Id::ConfigEditor(IdConfigEditor::FallbackBorder),
            Box::new(ConfigFallbackBorder::new(config.clone())),
            Vec::new(),
        )?;
        self.app.remount(
            Id::ConfigEditor(IdConfigEditor::FallbackHighlight),
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
            Id::ConfigEditor(IdConfigEditor::LibraryHighlightSymbol),
            Box::new(ConfigLibraryHighlightSymbol::new(config.clone())),
            Vec::new(),
        )?;

        self.app.remount(
            Id::ConfigEditor(IdConfigEditor::PlaylistHighlightSymbol),
            Box::new(ConfigPlaylistHighlightSymbol::new(config.clone())),
            Vec::new(),
        )?;

        self.app.remount(
            Id::ConfigEditor(IdConfigEditor::CurrentlyPlayingTrackSymbol),
            Box::new(ConfigCurrentlyPlayingTrackSymbol::new(config.clone())),
            Vec::new(),
        )?;

        Ok(())
    }

    /// Unmount the Config-Editor's Second Page, the Theme, Color & Symbol Options
    pub(super) fn umount_config_color(&mut self) -> Result<()> {
        self.app
            .umount(&Id::ConfigEditor(IdConfigEditor::CEThemeSelect))?;

        self.app
            .umount(&Id::ConfigEditor(IdConfigEditor::LibraryLabel))?;
        self.app
            .umount(&Id::ConfigEditor(IdConfigEditor::LibraryForeground))?;

        self.app
            .umount(&Id::ConfigEditor(IdConfigEditor::LibraryBackground))?;
        self.app
            .umount(&Id::ConfigEditor(IdConfigEditor::LibraryBorder))?;
        self.app
            .umount(&Id::ConfigEditor(IdConfigEditor::LibraryHighlight))?;

        self.app
            .umount(&Id::ConfigEditor(IdConfigEditor::PlaylistLabel))?;

        self.app
            .umount(&Id::ConfigEditor(IdConfigEditor::PlaylistForeground))?;

        self.app
            .umount(&Id::ConfigEditor(IdConfigEditor::PlaylistBackground))?;
        self.app
            .umount(&Id::ConfigEditor(IdConfigEditor::PlaylistBorder))?;
        self.app
            .umount(&Id::ConfigEditor(IdConfigEditor::PlaylistHighlight))?;
        self.app
            .umount(&Id::ConfigEditor(IdConfigEditor::ProgressLabel))?;

        self.app
            .umount(&Id::ConfigEditor(IdConfigEditor::ProgressForeground))?;
        self.app
            .umount(&Id::ConfigEditor(IdConfigEditor::ProgressBackground))?;
        self.app
            .umount(&Id::ConfigEditor(IdConfigEditor::ProgressBorder))?;

        self.app
            .umount(&Id::ConfigEditor(IdConfigEditor::LyricLabel))?;
        self.app
            .umount(&Id::ConfigEditor(IdConfigEditor::LyricForeground))?;
        self.app
            .umount(&Id::ConfigEditor(IdConfigEditor::LyricBackground))?;
        self.app
            .umount(&Id::ConfigEditor(IdConfigEditor::LyricBorder))?;

        self.app
            .umount(&Id::ConfigEditor(IdConfigEditor::ImportantPopupLabel))?;
        self.app
            .umount(&Id::ConfigEditor(IdConfigEditor::ImportantPopupForeground))?;
        self.app
            .umount(&Id::ConfigEditor(IdConfigEditor::ImportantPopupBackground))?;
        self.app
            .umount(&Id::ConfigEditor(IdConfigEditor::ImportantPopupBorder))?;

        self.app
            .umount(&Id::ConfigEditor(IdConfigEditor::FallbackLabel))?;
        self.app
            .umount(&Id::ConfigEditor(IdConfigEditor::FallbackForeground))?;

        self.app
            .umount(&Id::ConfigEditor(IdConfigEditor::FallbackBackground))?;
        self.app
            .umount(&Id::ConfigEditor(IdConfigEditor::FallbackBorder))?;
        self.app
            .umount(&Id::ConfigEditor(IdConfigEditor::FallbackHighlight))?;

        self.umount_config_color_symbols()?;

        Ok(())
    }

    /// Unmount the Config-Editor's Second Page, Symbols
    pub fn umount_config_color_symbols(&mut self) -> Result<()> {
        self.app
            .umount(&Id::ConfigEditor(IdConfigEditor::LibraryHighlightSymbol))?;
        self.app
            .umount(&Id::ConfigEditor(IdConfigEditor::PlaylistHighlightSymbol))?;
        self.app.umount(&Id::ConfigEditor(
            IdConfigEditor::CurrentlyPlayingTrackSymbol,
        ))?;

        Ok(())
    }
}
