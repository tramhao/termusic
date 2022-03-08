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

use super::{Msg, YSMsg};
use tui_realm_stdlib::{Input, Table};
use tuirealm::command::{Cmd, CmdResult, Direction, Position};
use tuirealm::event::{Key, KeyEvent, KeyModifiers, NoUserEvent};
use tuirealm::props::{Alignment, BorderType, Borders, Color, InputType, TableBuilder, TextSpan};
use tuirealm::{Component, Event, MockComponent, State, StateValue};

#[derive(MockComponent)]
pub struct YSInputPopup {
    component: Input,
}

impl Default for YSInputPopup {
    fn default() -> Self {
        Self {
            component: Input::default()
                .foreground(Color::Yellow)
                .background(Color::Reset)
                .borders(
                    Borders::default()
                        .color(Color::Green)
                        .modifiers(BorderType::Rounded),
                )
                // .invalid_style(Style::default().fg(Color::Red))
                .input_type(InputType::Text)
                .title("Download url or search:", Alignment::Left),
        }
    }
}

impl Component<Msg, NoUserEvent> for YSInputPopup {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        let cmd_result = match ev {
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
            Event::Keyboard(KeyEvent { code: Key::Esc, .. }) => {
                return Some(Msg::YoutubeSearch(YSMsg::InputPopupCloseCancel));
            }
            Event::Keyboard(KeyEvent {
                code: Key::Enter, ..
            }) => self.perform(Cmd::Submit),
            _ => CmdResult::None,
        };
        match cmd_result {
            CmdResult::Submit(State::One(StateValue::String(input_string))) => {
                Some(Msg::YoutubeSearch(YSMsg::InputPopupCloseOk(input_string)))
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
pub struct YSTablePopup {
    component: Table,
}

impl Default for YSTablePopup {
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
                .title("Tab/Shift+Tab for next and previous page", Alignment::Left)
                .scroll(true)
                .highlighted_color(Color::LightBlue)
                .highlighted_str("\u{1f680}")
                // .highlighted_str("🚀")
                .rewind(false)
                .step(4)
                .row_height(1)
                .headers(&["Duration", "Name"])
                .column_spacing(3)
                .widths(&[20, 80])
                .table(
                    TableBuilder::default()
                        .add_col(TextSpan::from("Empty result."))
                        .add_col(TextSpan::from("Loading..."))
                        .build(),
                ),
        }
    }
}

impl Component<Msg, NoUserEvent> for YSTablePopup {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        let cmd_result = match ev {
            Event::Keyboard(KeyEvent { code: Key::Esc, .. }) => {
                return Some(Msg::YoutubeSearch(YSMsg::TablePopupCloseCancel))
            }

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
                code: Key::Tab,
                modifiers: KeyModifiers::NONE,
            }) => return Some(Msg::YoutubeSearch(YSMsg::TablePopupNext)),
            Event::Keyboard(KeyEvent {
                code: Key::BackTab,
                modifiers: KeyModifiers::SHIFT,
            }) => return Some(Msg::YoutubeSearch(YSMsg::TablePopupPrevious)),
            Event::Keyboard(KeyEvent {
                code: Key::Enter, ..
            }) => {
                if let State::One(StateValue::Usize(index)) = self.state() {
                    return Some(Msg::YoutubeSearch(YSMsg::TablePopupCloseOk(index)));
                }
                CmdResult::None
            }
            _ => CmdResult::None,
        };
        match cmd_result {
            // CmdResult::Submit(State::One(StateValue::Usize(index))) => {
            //     Some(Msg::YoutubeSearchTablePopupCloseOk(index))
            // }
            _ => Some(Msg::None),
        }
    }
}
