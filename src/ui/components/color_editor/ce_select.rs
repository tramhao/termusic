//! # Popups
//!
//!
//! Popups components

use super::ColorConfig;
use crate::config::parse_hex_color;
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
use crate::ui::components::ColorMapping;
use crate::ui::{CEMsg, IdColorEditor, Msg};

use tui_realm_stdlib::{Label, Select};
use tuirealm::command::{Cmd, CmdResult, Direction};
use tuirealm::event::{Key, KeyEvent, KeyModifiers};
use tuirealm::props::{Alignment, BorderType, Borders, Color, TextModifiers};
use tuirealm::{
    AttrValue, Attribute, Component, Event, MockComponent, NoUserEvent, State, StateValue,
};
const COLOR_LIST: [&str; 19] = [
    "default",
    "background",
    "foreground",
    "black",
    "red",
    "green",
    "yellow",
    "blue",
    "magenta",
    "cyan",
    "white",
    "bright_black",
    "birght_red",
    "bright_green",
    "bright_yellow",
    "bright_blue",
    "bright_magenta",
    "bright_cyan",
    "bright_white",
];

#[derive(MockComponent)]
pub struct CESelectColor {
    component: Select,
    id: IdColorEditor,
    color_mapping: ColorMapping,
    // on_key_down: Msg,
    // on_key_up: Msg,
}

impl CESelectColor {
    pub fn new(
        name: &str,
        id: IdColorEditor,
        color: Color,
        color_mapping: &ColorMapping,
        // on_key_down: Msg,
        // on_key_up: Msg,
    ) -> Self {
        let init_value = Self::init_color_select(&id, color_mapping);
        Self {
            component: Select::default()
                .borders(
                    Borders::default()
                        .modifiers(BorderType::Rounded)
                        .color(color),
                )
                .foreground(color)
                .title(name, Alignment::Left)
                .rewind(true)
                .highlighted_color(Color::LightGreen)
                .highlighted_str(">> ")
                .choices(&COLOR_LIST)
                .value(init_value),
            id,
            color_mapping: color_mapping.clone(),
            // on_key_down,
            // on_key_up,
        }
    }

    const fn init_color_select(id: &IdColorEditor, color_mapping: &ColorMapping) -> usize {
        match *id {
            IdColorEditor::LibraryForeground => {
                Self::match_color_config(&color_mapping.library_foreground)
            }
            IdColorEditor::LibraryBackground => {
                Self::match_color_config(&color_mapping.library_background)
            }
            IdColorEditor::LibraryBorder => Self::match_color_config(&color_mapping.library_border),

            _ => 0,
        }
    }

    const fn match_color_config(color_config: &ColorConfig) -> usize {
        match color_config {
            ColorConfig::Foreground => 2,
            ColorConfig::Background => 1,
            ColorConfig::Black => 3,
            ColorConfig::Red => 4,
            ColorConfig::Green => 5,
            ColorConfig::Yellow => 6,
            ColorConfig::Blue => 7,
            ColorConfig::Magenta => 8,
            ColorConfig::Cyan => 9,
            ColorConfig::White => 10,
            ColorConfig::LightBlack => 11,
            ColorConfig::LightRed => 12,
            ColorConfig::LightGreen => 13,
            ColorConfig::LightYellow => 14,
            ColorConfig::LightBlue => 15,
            ColorConfig::LightMagenta => 16,
            ColorConfig::LightCyan => 17,
            ColorConfig::LightWhite => 18,
            ColorConfig::Reset | ColorConfig::Text | ColorConfig::Cursor => 0,
        }
    }

    fn update_color(&mut self, index: usize) -> Msg {
        if let Some(color) = COLOR_LIST.get(index) {
            let color = self.parse_color(color).unwrap();
            self.attr(Attribute::Foreground, AttrValue::Color(color));
            self.attr(
                Attribute::Borders,
                AttrValue::Borders(
                    Borders::default()
                        .modifiers(BorderType::Rounded)
                        .color(color),
                ),
            );
            Msg::ColorEditor(CEMsg::ColorChanged(self.id.clone(), color))
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
            Msg::None
        }
    }

    fn parse_color(&self, color_str: &str) -> Option<Color> {
        match color_str {
            "default" => Some(Color::Reset),
            "background" => parse_hex_color(&self.color_mapping.alacritty_theme.background),
            "foreground" => parse_hex_color(&self.color_mapping.alacritty_theme.foreground),
            "black" => parse_hex_color(&self.color_mapping.alacritty_theme.black),
            "red" => parse_hex_color(&self.color_mapping.alacritty_theme.red),
            "green" => parse_hex_color(&self.color_mapping.alacritty_theme.green),
            "yellow" => parse_hex_color(&self.color_mapping.alacritty_theme.yellow),
            "blue" => parse_hex_color(&self.color_mapping.alacritty_theme.blue),
            "magenta" => parse_hex_color(&self.color_mapping.alacritty_theme.magenta),
            "cyan" => parse_hex_color(&self.color_mapping.alacritty_theme.cyan),
            "white" => parse_hex_color(&self.color_mapping.alacritty_theme.white),
            "bright_black" => parse_hex_color(&self.color_mapping.alacritty_theme.light_black),
            "birght_red" => parse_hex_color(&self.color_mapping.alacritty_theme.light_red),
            "bright_green" => parse_hex_color(&self.color_mapping.alacritty_theme.light_green),
            "bright_yellow" => parse_hex_color(&self.color_mapping.alacritty_theme.light_yellow),
            "bright_blue" => parse_hex_color(&self.color_mapping.alacritty_theme.light_blue),
            "bright_magenta" => parse_hex_color(&self.color_mapping.alacritty_theme.light_magenta),
            "bright_cyan" => parse_hex_color(&self.color_mapping.alacritty_theme.light_cyan),
            "bright_white" => parse_hex_color(&self.color_mapping.alacritty_theme.light_white),
            &_ => None,
        }
    }
}

impl Component<Msg, NoUserEvent> for CESelectColor {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        let cmd_result = match ev {
            Event::Keyboard(KeyEvent { code: Key::Tab, .. }) => match self.id {
                IdColorEditor::LibraryForeground => {
                    return Some(Msg::ColorEditor(CEMsg::LibraryForegroundBlur));
                }
                IdColorEditor::LibraryBackground => {
                    return Some(Msg::ColorEditor(CEMsg::LibraryBackgroundBlur));
                }
                IdColorEditor::LibraryBorder => {
                    return Some(Msg::ColorEditor(CEMsg::LibraryBorderBlur));
                }
                IdColorEditor::LibraryHighlight => {
                    return Some(Msg::ColorEditor(CEMsg::LibraryHighlightBlur));
                }

                _ => CmdResult::None,
            },
            Event::Keyboard(KeyEvent { code: Key::Esc, .. }) => {
                return Some(Msg::ColorEditor(CEMsg::ThemeSelectCloseCancel))
            }
            Event::Keyboard(KeyEvent {
                code: Key::Char('h'),
                modifiers: KeyModifiers::CONTROL,
            }) => return Some(Msg::TEHelpPopupShow),

            Event::Keyboard(KeyEvent {
                code: Key::Down | Key::Char('j'),
                ..
            }) => self.perform(Cmd::Move(Direction::Down)),
            Event::Keyboard(KeyEvent {
                code: Key::Up | Key::Char('k'),
                ..
            }) => self.perform(Cmd::Move(Direction::Up)),
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

        // if cmd_result == CmdResult::Submit(State::One(StateValue::String("DELETE".to_string()))) {
        //     Some(Msg::DeleteConfirmCloseOk)
        // } else {
        //     Some(Msg::DeleteConfirmCloseCancel)
        // }
    }
}

#[derive(MockComponent)]
pub struct CELibraryTitle {
    component: Label,
}

impl Default for CELibraryTitle {
    fn default() -> Self {
        Self {
            component: Label::default()
                .modifiers(TextModifiers::BOLD)
                .text("Library styles"),
        }
    }
}

impl Component<Msg, NoUserEvent> for CELibraryTitle {
    fn on(&mut self, _ev: Event<NoUserEvent>) -> Option<Msg> {
        None
    }
}

#[derive(MockComponent)]
pub struct CELibraryForeground {
    component: CESelectColor,
}

impl CELibraryForeground {
    pub fn new(color_mapping: &ColorMapping) -> Self {
        Self {
            component: CESelectColor::new(
                "Foreground",
                IdColorEditor::LibraryForeground,
                color_mapping.library_foreground().unwrap_or(Color::Blue),
                color_mapping,
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for CELibraryForeground {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct CELibraryBackground {
    component: CESelectColor,
}

impl CELibraryBackground {
    pub fn new(color_mapping: &ColorMapping) -> Self {
        Self {
            component: CESelectColor::new(
                "Background",
                IdColorEditor::LibraryBackground,
                color_mapping.library_background().unwrap_or(Color::Blue),
                color_mapping,
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for CELibraryBackground {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct CELibraryBorder {
    component: CESelectColor,
}

impl CELibraryBorder {
    pub fn new(color_mapping: &ColorMapping) -> Self {
        Self {
            component: CESelectColor::new(
                "Border",
                IdColorEditor::LibraryBorder,
                color_mapping.library_border().unwrap_or(Color::Blue),
                color_mapping,
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for CELibraryBorder {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct CELibraryHighlight {
    component: CESelectColor,
}

impl CELibraryHighlight {
    pub fn new(color_mapping: &ColorMapping) -> Self {
        Self {
            component: CESelectColor::new(
                "Highlight",
                IdColorEditor::LibraryHighlight,
                color_mapping.library_highlight().unwrap_or(Color::Blue),
                color_mapping,
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for CELibraryHighlight {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}
