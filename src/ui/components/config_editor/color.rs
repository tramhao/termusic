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
use crate::config::{ColorTermusic, Settings, StyleColorSymbol};
use crate::ui::{CEMsg, ConfigEditorMsg, IdConfigEditor, Msg};
use std::convert::From;
use tui_realm_stdlib::{Label, Select, Table};
use tuirealm::command::{Cmd, CmdResult, Direction, Position};
use tuirealm::event::{Key, KeyEvent, KeyModifiers, NoUserEvent};
use tuirealm::props::{
    Alignment, BorderType, Borders, Color, Style, TableBuilder, TextModifiers, TextSpan,
};
use tuirealm::{AttrValue, Attribute, Component, Event, MockComponent, State, StateValue};

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
}

impl Default for CEThemeSelectTable {
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

impl Component<Msg, NoUserEvent> for CEThemeSelectTable {
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
                return Some(Msg::ConfigEditor(ConfigEditorMsg::ThemeSelectBlurDown));
                // return Some(Msg::ConfigEditor(ConfigEditorMsg::ChangeLayout))
            }
            Event::Keyboard(KeyEvent {
                code: Key::BackTab,
                modifiers: KeyModifiers::SHIFT,
            }) => {
                return Some(Msg::ColorEditor(CEMsg::ThemeSelectBlurUp));
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

#[derive(MockComponent)]
pub struct CEColorSelect {
    component: Select,
    id: IdConfigEditor,
    config: Settings,
    on_key_shift: Msg,
    on_key_backshift: Msg,
}

impl CEColorSelect {
    pub fn new(
        name: &str,
        id: IdConfigEditor,
        color: Color,
        config: &Settings,
        on_key_shift: Msg,
        on_key_backshift: Msg,
    ) -> Self {
        let init_value = Self::init_color_select(&id, &config.style_color_symbol);
        let mut choices = vec![];
        for color in &COLOR_LIST {
            choices.push(String::from(color.clone()));
        }
        Self {
            component: Select::default()
                .borders(
                    Borders::default()
                        .modifiers(BorderType::Rounded)
                        .color(color),
                )
                .foreground(color)
                .title(name, Alignment::Left)
                .rewind(false)
                .inactive(Style::default().bg(color))
                .highlighted_color(Color::LightGreen)
                .highlighted_str(">> ")
                .choices(&choices)
                .value(init_value),
            id,
            config: config.clone(),
            on_key_shift,
            on_key_backshift,
        }
    }

    const fn init_color_select(
        id: &IdConfigEditor,
        _style_color_symbol: &StyleColorSymbol,
    ) -> usize {
        match *id {
            // IdColorEditor::LibraryForeground => style_color_symbol.library_foreground.as_usize(),
            // IdColorEditor::LibraryBackground => style_color_symbol.library_background.as_usize(),
            // IdColorEditor::LibraryBorder => style_color_symbol.library_border.as_usize(),
            // IdColorEditor::LibraryHighlight => style_color_symbol.library_highlight.as_usize(),
            // IdColorEditor::PlaylistForeground => style_color_symbol.playlist_foreground.as_usize(),
            // IdColorEditor::PlaylistBackground => style_color_symbol.playlist_background.as_usize(),
            // IdColorEditor::PlaylistBorder => style_color_symbol.playlist_border.as_usize(),
            // IdColorEditor::PlaylistHighlight => style_color_symbol.playlist_highlight.as_usize(),
            // IdColorEditor::ProgressForeground => style_color_symbol.progress_foreground.as_usize(),
            // IdColorEditor::ProgressBackground => style_color_symbol.progress_background.as_usize(),
            // IdColorEditor::ProgressBorder => style_color_symbol.progress_border.as_usize(),
            // IdColorEditor::LyricForeground => style_color_symbol.lyric_foreground.as_usize(),
            // IdColorEditor::LyricBackground => style_color_symbol.lyric_background.as_usize(),
            // IdColorEditor::LyricBorder => style_color_symbol.lyric_border.as_usize(),
            _ => 0,
        }
    }

    fn update_color(&mut self, index: usize) -> Msg {
        if let Some(color_config) = COLOR_LIST.get(index) {
            let color = color_config
                .color(&self.config.style_color_symbol.alacritty_theme)
                .unwrap_or(Color::Red);
            self.attr(Attribute::Foreground, AttrValue::Color(color));
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
                AttrValue::Style(Style::default().bg(color)),
            );
            Msg::ConfigEditor(ConfigEditorMsg::ColorChanged(
                self.id.clone(),
                color_config.clone(),
            ))
        } else {
            self.attr(Attribute::Foreground, AttrValue::Color(Color::Red));
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
                AttrValue::Style(Style::default().bg(Color::Red)),
            );

            Msg::None
        }
    }
}

impl Component<Msg, NoUserEvent> for CEColorSelect {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        let cmd_result = match ev {
            Event::Keyboard(KeyEvent { code: Key::Tab, .. }) => {
                return Some(Msg::ConfigEditor(ConfigEditorMsg::ChangeLayout))
            }
            Event::Keyboard(KeyEvent {
                code: Key::BackTab,
                modifiers: KeyModifiers::SHIFT,
            }) => return Some(self.on_key_backshift.clone()),

            Event::Keyboard(key) if key == self.config.keys.global_help.key_event() => {
                return Some(Msg::ColorEditor(CEMsg::HelpPopupShow))
            }

            Event::Keyboard(key) if key == self.config.keys.global_up.key_event() => {
                // self.perform(Cmd::Move(Direction::Up))
                return Some(self.on_key_backshift.clone());
            }
            Event::Keyboard(key) if key == self.config.keys.global_down.key_event() => {
                return Some(self.on_key_shift.clone());
                // self.perform(Cmd::Move(Direction::Down))
            }
            Event::Keyboard(key) if key == self.config.keys.global_quit.key_event() => {
                match self.state() {
                    State::One(_) => return Some(Msg::ColorEditor(CEMsg::ColorEditorCloseCancel)),
                    _ => self.perform(Cmd::Cancel),
                }
            }

            Event::Keyboard(key) if key == self.config.keys.global_esc.key_event() => {
                match self.state() {
                    State::One(_) => return Some(Msg::ColorEditor(CEMsg::ColorEditorCloseCancel)),
                    _ => self.perform(Cmd::Cancel),
                }
            }

            Event::Keyboard(KeyEvent {
                code: Key::Enter, ..
            }) => self.perform(Cmd::Submit),
            _ => CmdResult::None,
        };
        match cmd_result {
            CmdResult::Submit(State::One(StateValue::Usize(index))) => {
                Some(self.update_color(index))
                // Some(Msg::TESelectLyricOk(COLOR_LIST[index]))
            }
            _ => Some(Msg::None),
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
                .text("Library style"),
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
    pub fn new(config: &Settings) -> Self {
        Self {
            component: CEColorSelect::new(
                "Foreground",
                IdConfigEditor::LibraryForeground,
                config
                    .style_color_symbol
                    .library_foreground()
                    .unwrap_or(Color::Blue),
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
    pub fn new(config: &Settings) -> Self {
        Self {
            component: CEColorSelect::new(
                "Background",
                IdConfigEditor::LibraryBackground,
                config
                    .style_color_symbol
                    .library_background()
                    .unwrap_or(Color::Blue),
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
    pub fn new(config: &Settings) -> Self {
        Self {
            component: CEColorSelect::new(
                "Border",
                IdConfigEditor::LibraryBorder,
                config
                    .style_color_symbol
                    .library_border()
                    .unwrap_or(Color::Blue),
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
    pub fn new(config: &Settings) -> Self {
        Self {
            component: CEColorSelect::new(
                "Highlight",
                IdConfigEditor::LibraryHighlight,
                config
                    .style_color_symbol
                    .library_highlight()
                    .unwrap_or(Color::Blue),
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
