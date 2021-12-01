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
use super::Msg;

use tui_realm_stdlib::{Input, Paragraph, Radio, Table};
use tuirealm::command::{Cmd, CmdResult, Direction, Position};
use tuirealm::event::{Key, KeyEvent, KeyModifiers};
use tuirealm::props::{
    Alignment, BorderType, Borders, Color, InputType, TableBuilder, TextModifiers, TextSpan,
};
use tuirealm::{Component, Event, MockComponent, NoUserEvent, State, StateValue};

#[derive(MockComponent)]
pub struct QuitPopup {
    component: Radio,
}

impl Default for QuitPopup {
    fn default() -> Self {
        Self {
            component: Radio::default()
                .foreground(Color::Yellow)
                .background(Color::Black)
                .borders(
                    Borders::default()
                        .color(Color::Yellow)
                        .modifiers(BorderType::Rounded),
                )
                .title("Are sure you want to quit?", Alignment::Center)
                .rewind(true)
                .choices(&["No", "Yes"])
                .value(0),
        }
    }
}

impl Component<Msg, NoUserEvent> for QuitPopup {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        let cmd_result = match ev {
            Event::Keyboard(KeyEvent {
                code: Key::Left | Key::Char('h' | 'j'),
                ..
            }) => self.perform(Cmd::Move(Direction::Left)),
            Event::Keyboard(KeyEvent {
                code: Key::Right | Key::Char('l' | 'k'),
                ..
            }) => self.perform(Cmd::Move(Direction::Right)),
            Event::Keyboard(KeyEvent { code: Key::Esc, .. }) => {
                return Some(Msg::QuitPopupCloseCancel)
            }
            Event::Keyboard(KeyEvent {
                code: Key::Enter, ..
            }) => self.perform(Cmd::Submit),
            _ => return None,
        };
        if matches!(
            cmd_result,
            CmdResult::Submit(State::One(StateValue::Usize(0)))
        ) {
            Some(Msg::QuitPopupCloseCancel)
        } else if matches!(
            cmd_result,
            CmdResult::Submit(State::One(StateValue::Usize(1)))
        ) {
            Some(Msg::QuitPopupCloseOk)
        } else {
            Some(Msg::None)
        }
    }
}

#[derive(MockComponent)]
pub struct ErrorPopup {
    component: Paragraph,
}

impl ErrorPopup {
    pub fn new<S: AsRef<str>>(msg: S) -> Self {
        Self {
            component: Paragraph::default()
                .borders(
                    Borders::default()
                        .color(Color::Red)
                        .modifiers(BorderType::Rounded),
                )
                .foreground(Color::Red)
                .background(Color::Black)
                .modifiers(TextModifiers::BOLD)
                .alignment(Alignment::Center)
                .text(vec![TextSpan::from(msg.as_ref().to_string())].as_slice()),
        }
    }
}

impl Component<Msg, NoUserEvent> for ErrorPopup {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        match ev {
            Event::Keyboard(KeyEvent {
                code: Key::Enter | Key::Esc,
                ..
            }) => Some(Msg::ErrorPopupClose),
            _ => None,
        }
    }
}

#[derive(MockComponent)]
pub struct HelpPopup {
    component: Table,
}

impl Default for HelpPopup {
    fn default() -> Self {
        Self {
            component: Table::default()
                .borders(
                    Borders::default()
                        .modifiers(BorderType::Rounded)
                        .color(Color::Green),
                )
                // .foreground(Color::Yellow)
                .background(Color::Black)
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
                        .add_col(TextSpan::new("<ESC> or <q>").bold().fg(Color::Cyan))
                        .add_col(TextSpan::from("Exit"))
                        .add_row()
                        .add_col(TextSpan::new("<TAB>").bold().fg(Color::Cyan))
                        .add_col(TextSpan::from("Switch focus"))
                        .add_row()
                        .add_col(TextSpan::new("<h,j,k,l,g,G>").bold().fg(Color::Cyan))
                        .add_col(TextSpan::from("Move cursor(vim style)"))
                        .add_row()
                        .add_col(TextSpan::new("<f/b>").bold().fg(Color::Cyan))
                        .add_col(TextSpan::from("Seek forward/backward 5 seconds"))
                        .add_row()
                        .add_col(TextSpan::new("<F/B>").bold().fg(Color::Cyan))
                        .add_col(TextSpan::from("Seek forward/backward 1 second for lyrics"))
                        .add_row()
                        .add_col(TextSpan::new("<F/B>").bold().fg(Color::Cyan))
                        .add_col(TextSpan::from("Before 10 seconds,adjust offset of lyrics"))
                        .add_row()
                        .add_col(TextSpan::new("<T>").bold().fg(Color::Cyan))
                        .add_col(TextSpan::from("Switch lyrics if more than 1 available"))
                        .add_row()
                        .add_col(TextSpan::new("<n/N/space>").bold().fg(Color::Cyan))
                        .add_col(TextSpan::from("Next/Previous/Pause current song"))
                        .add_row()
                        .add_col(TextSpan::new("<+,=/-,_>").bold().fg(Color::Cyan))
                        .add_col(TextSpan::from("Increase/Decrease volume"))
                        .add_row()
                        .add_col(TextSpan::new("Library").bold().fg(Color::LightYellow))
                        .add_row()
                        .add_col(TextSpan::new("<l/L>").bold().fg(Color::Cyan))
                        .add_col(TextSpan::from("Add one/all songs to playlist"))
                        .add_row()
                        .add_col(TextSpan::new("<d>").bold().fg(Color::Cyan))
                        .add_col(TextSpan::from("Delete song or folder"))
                        .add_row()
                        .add_col(TextSpan::new("<s>").bold().fg(Color::Cyan))
                        .add_col(TextSpan::from("Download or search song from youtube"))
                        .add_row()
                        .add_col(TextSpan::new("<t>").bold().fg(Color::Cyan))
                        .add_col(TextSpan::from("Open tag editor for tag and lyric download"))
                        .add_row()
                        .add_col(TextSpan::new("<y/p>").bold().fg(Color::Cyan))
                        .add_col(TextSpan::from("Yank and Paste files"))
                        .add_row()
                        .add_col(TextSpan::new("<Enter>").bold().fg(Color::Cyan))
                        .add_col(TextSpan::from("Open sub directory as root"))
                        .add_row()
                        .add_col(TextSpan::new("<Backspace>").bold().fg(Color::Cyan))
                        .add_col(TextSpan::from("Go back to parent directory"))
                        .add_row()
                        .add_col(TextSpan::new("</>").bold().fg(Color::Cyan))
                        .add_col(TextSpan::from("Search in library"))
                        .add_row()
                        .add_col(TextSpan::new("Playlist").bold().fg(Color::LightYellow))
                        .add_row()
                        .add_col(TextSpan::new("<d/D>").bold().fg(Color::Cyan))
                        .add_col(TextSpan::from("Delete one/all songs from playlist"))
                        .add_row()
                        .add_col(TextSpan::new("<l>").bold().fg(Color::Cyan))
                        .add_col(TextSpan::from("Play selected"))
                        .add_row()
                        .add_col(TextSpan::new("<s>").bold().fg(Color::Cyan))
                        .add_col(TextSpan::from("Shuffle playlist"))
                        .add_row()
                        .add_col(TextSpan::new("<m>").bold().fg(Color::Cyan))
                        .add_col(TextSpan::from("Loop mode toggle"))
                        .add_row()
                        .add_col(TextSpan::new("<a>").bold().fg(Color::Cyan))
                        .add_col(TextSpan::from(
                            "Add a song to the front of playlist or back",
                        ))
                        .build(),
                ),
        }
    }
}

impl Component<Msg, NoUserEvent> for HelpPopup {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        match ev {
            Event::Keyboard(KeyEvent {
                code: Key::Enter | Key::Esc,
                ..
            }) => Some(Msg::HelpPopupClose),
            _ => None,
        }
    }
}

#[derive(MockComponent)]
pub struct DeleteConfirmRadioPopup {
    component: Radio,
}

impl Default for DeleteConfirmRadioPopup {
    fn default() -> Self {
        Self {
            component: Radio::default()
                .foreground(Color::LightRed)
                .background(Color::Black)
                .borders(
                    Borders::default()
                        .color(Color::LightRed)
                        .modifiers(BorderType::Rounded),
                )
                .title("Are sure you want to delete?", Alignment::Left)
                .rewind(true)
                .choices(&["No", "Yes"])
                .value(0),
        }
    }
}

impl Component<Msg, NoUserEvent> for DeleteConfirmRadioPopup {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        let cmd_result = match ev {
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
            Some(Msg::DeleteConfirmCloseCancel)
        } else if matches!(
            cmd_result,
            CmdResult::Submit(State::One(StateValue::Usize(1)))
        ) {
            Some(Msg::DeleteConfirmCloseOk)
        } else {
            Some(Msg::None)
        }
    }
}

#[derive(MockComponent)]
pub struct DeleteConfirmInputPopup {
    component: Input,
}

impl Default for DeleteConfirmInputPopup {
    fn default() -> Self {
        Self {
            component: Input::default()
                .foreground(Color::Yellow)
                .background(Color::Black)
                .borders(
                    Borders::default()
                        .color(Color::Green)
                        .modifiers(BorderType::Rounded),
                )
                // .invalid_style(Style::default().fg(Color::Red))
                .input_type(InputType::Text)
                .title("Type DELETE to confirm:", Alignment::Left),
        }
    }
}

impl Component<Msg, NoUserEvent> for DeleteConfirmInputPopup {
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
                return Some(Msg::DeleteConfirmCloseCancel);
            }
            Event::Keyboard(KeyEvent {
                code: Key::Enter, ..
            }) => self.perform(Cmd::Submit),
            _ => CmdResult::None,
        };
        match cmd_result {
            CmdResult::Submit(State::One(StateValue::String(input_string))) => {
                if input_string == *"DELETE" {
                    return Some(Msg::DeleteConfirmCloseOk);
                }
                Some(Msg::DeleteConfirmCloseCancel)
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
pub struct MessagePopup {
    component: Paragraph,
}

impl MessagePopup {
    pub fn new<S: AsRef<str>>(title: S, msg: S) -> Self {
        Self {
            component: Paragraph::default()
                .borders(
                    Borders::default()
                        .color(Color::Cyan)
                        .modifiers(BorderType::Rounded),
                )
                .foreground(Color::Green)
                .background(Color::Black)
                .modifiers(TextModifiers::BOLD)
                .alignment(Alignment::Center)
                .title(title, Alignment::Center)
                .text(vec![TextSpan::from(msg.as_ref().to_string())].as_slice()),
        }
    }
}

impl Component<Msg, NoUserEvent> for MessagePopup {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        match ev {
            // Event::Keyboard(KeyEvent {
            //     code: Key::Enter | Key::Esc,
            //     ..
            // }) => Some(Msg::ErrorPopupClose),
            _ => None,
        }
    }
}
