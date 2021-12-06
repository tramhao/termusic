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
use crate::ui::Msg;

use tui_realm_stdlib::Input;
use tuirealm::command::{Cmd, CmdResult, Direction, Position};
use tuirealm::event::{Key, KeyEvent, KeyModifiers};
use tuirealm::props::{Alignment, BorderType, Borders, Color, InputType};
use tuirealm::{Component, Event, MockComponent, NoUserEvent, State, StateValue};

#[derive(MockComponent)]
pub struct TEInputTitle {
    component: Input,
}

impl Default for TEInputTitle {
    fn default() -> Self {
        Self {
            component: Input::default()
                .foreground(Color::Cyan)
                // .background(Color::Black)
                .borders(
                    Borders::default()
                        .color(Color::LightYellow)
                        .modifiers(BorderType::Rounded),
                )
                // .invalid_style(Style::default().fg(Color::Red))
                .input_type(InputType::Text)
                .title("Search song name", Alignment::Left),
        }
    }
}

impl Component<Msg, NoUserEvent> for TEInputTitle {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        let cmd_result = match ev {
            Event::Keyboard(KeyEvent { code: Key::Tab, .. }) => return Some(Msg::TEInputTitleBlur),
            Event::Keyboard(KeyEvent { code: Key::Esc, .. }) => {
                return Some(Msg::TagEditorBlur(None))
            }
            Event::Keyboard(KeyEvent {
                code: Key::Char('h'),
                modifiers: KeyModifiers::CONTROL,
            }) => return Some(Msg::TEHelpPopupShow),

            Event::Keyboard(KeyEvent {
                code: Key::Left, ..
            }) => self.perform(Cmd::Move(Direction::Left)),
            Event::Keyboard(KeyEvent {
                code: Key::Right, ..
            }) => self.perform(Cmd::Move(Direction::Right)),
            Event::Keyboard(KeyEvent {
                code: Key::Home, ..
            }) => self.perform(Cmd::GoTo(Position::Begin)),
            Event::Keyboard(KeyEvent { code: Key::End, .. }) => {
                self.perform(Cmd::GoTo(Position::End))
            }
            Event::Keyboard(KeyEvent {
                code: Key::Delete, ..
            }) => self.perform(Cmd::Cancel),
            Event::Keyboard(KeyEvent {
                code: Key::Backspace,
                ..
            }) => self.perform(Cmd::Delete),
            Event::Keyboard(KeyEvent {
                code: Key::Char(ch),
                modifiers: KeyModifiers::SHIFT | KeyModifiers::NONE,
            }) => self.perform(Cmd::Type(ch)),
            Event::Keyboard(KeyEvent {
                code: Key::Enter, ..
            }) => self.perform(Cmd::Submit),
            _ => CmdResult::None,
        };
        match cmd_result {
            CmdResult::Submit(State::One(StateValue::String(_artist))) => Some(Msg::TESearch),
            _ => Some(Msg::None),
        }

        // if cmd_result == CmdResult::Submit(State::One(StateValue::String("DELETE".to_string()))) {
        //     Some(Msg::DeleteConfirmCloseOk)
        // } else {
        //     Some(Msg::DeleteConfirmCloseCancel)
        // }
    }
}
