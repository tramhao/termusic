//! # Popups
//!
//! Popups components

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
use crate::ui::{CEMsg, IdColorEditor, Msg};

use tui_realm_stdlib::{Label, Select};
use tuirealm::command::{Cmd, CmdResult, Direction};
use tuirealm::event::{Key, KeyEvent, KeyModifiers};
use tuirealm::props::{Alignment, BorderType, Borders, Color, TextModifiers};
use tuirealm::{
    AttrValue, Attribute, Component, Event, MockComponent, NoUserEvent, State, StateValue,
};

const COLOR_LIST: [&str; 19] = [
    "reset",
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
    on_key_down: Msg,
    on_key_up: Msg,
}

impl CESelectColor {
    pub fn new(
        name: &str,
        id: IdColorEditor,
        color: Color,
        on_key_down: Msg,
        on_key_up: Msg,
    ) -> Self {
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
                .choices(&COLOR_LIST),
            id,
            on_key_down,
            on_key_up,
        }
    }

    fn update_color(&mut self, result: CmdResult) -> Option<Msg> {
        if let CmdResult::Changed(State::One(StateValue::String(color))) = result {
            let color = tuirealm::utils::parser::parse_color(&color).unwrap();
            self.attr(Attribute::Foreground, AttrValue::Color(color));
            self.attr(
                Attribute::Borders,
                AttrValue::Borders(
                    Borders::default()
                        .modifiers(BorderType::Rounded)
                        .color(color),
                ),
            );
            Some(Msg::None)
            // Some(Msg::Theme(ThemeMsg::ColorChanged(self.id.clone(), color)))
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
            Some(Msg::None)
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
                _ => CmdResult::None,
            },
            Event::Keyboard(KeyEvent { code: Key::Esc, .. }) => {
                return Some(Msg::TagEditorBlur(None))
            }
            Event::Keyboard(KeyEvent {
                code: Key::Char('h'),
                modifiers: KeyModifiers::CONTROL,
            }) => return Some(Msg::TEHelpPopupShow),

            Event::Keyboard(KeyEvent {
                code: Key::Down, ..
            }) => self.perform(Cmd::Move(Direction::Down)),
            Event::Keyboard(KeyEvent { code: Key::Up, .. }) => {
                self.perform(Cmd::Move(Direction::Up))
            }
            Event::Keyboard(KeyEvent {
                code: Key::Enter, ..
            }) => self.perform(Cmd::Submit),
            _ => CmdResult::None,
        };
        match cmd_result {
            CmdResult::Submit(State::One(StateValue::Usize(index))) => {
                Some(Msg::TESelectLyricOk(index))
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
pub struct CELibraryForeGround {
    component: CESelectColor,
}

impl CELibraryForeGround {
    pub fn new(value: Color) -> Self {
        Self {
            component: CESelectColor::new(
                "Foreground",
                IdColorEditor::LibraryForeground,
                value,
                Msg::ColorEditor(CEMsg::LibraryForegroundBlurDown),
                Msg::ColorEditor(CEMsg::LibraryForegroundBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for CELibraryForeGround {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}
