#![allow(clippy::module_name_repetitions)]

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
use termusicplayback::SharedSettings;
use tui_realm_stdlib::{Paragraph, Radio};
use tuirealm::command::{Cmd, CmdResult, Direction};
use tuirealm::event::{Key, KeyEvent, NoUserEvent};
use tuirealm::props::{
    Alignment, BorderType, Borders, Color, PropPayload, PropValue, TextModifiers, TextSpan,
};
use tuirealm::{AttrValue, Attribute, Component, Event, MockComponent, State, StateValue};

mod deleteconfirm;
mod error;
mod help;
mod podcast;
mod saveplaylist;

#[allow(unused_imports)]
pub use deleteconfirm::{DeleteConfirmInputPopup, DeleteConfirmRadioPopup};
pub use error::ErrorPopup;
pub use help::HelpPopup;
#[allow(unused_imports)]
pub use podcast::{
    FeedDeleteConfirmInputPopup, FeedDeleteConfirmRadioPopup, PodcastAddPopup,
    PodcastSearchTablePopup,
};
pub use saveplaylist::{SavePlaylistConfirmPopup, SavePlaylistPopup};

#[derive(MockComponent)]
pub struct QuitPopup {
    component: Radio,
    config: SharedSettings,
}

impl QuitPopup {
    pub fn new(config: SharedSettings) -> Self {
        let component = {
            let config = config.read();
            Radio::default()
                .foreground(
                    config
                        .style_color_symbol
                        .library_foreground()
                        .unwrap_or(Color::Yellow),
                )
                .background(
                    config
                        .style_color_symbol
                        .library_background()
                        .unwrap_or(Color::Reset),
                )
                .borders(
                    Borders::default()
                        .color(
                            config
                                .style_color_symbol
                                .library_border()
                                .unwrap_or(Color::Yellow),
                        )
                        .modifiers(BorderType::Rounded),
                )
                .title(" Are sure you want to quit?", Alignment::Center)
                .rewind(true)
                .choices(&["No", "Yes"])
                .value(0)
        };

        Self { component, config }
    }
}

impl Component<Msg, NoUserEvent> for QuitPopup {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        let config = self.config.clone();
        let keys = &config.read().keys;
        let cmd_result = match ev {
            Event::Keyboard(KeyEvent {
                code: Key::Left, ..
            }) => self.perform(Cmd::Move(Direction::Left)),
            Event::Keyboard(KeyEvent {
                code: Key::Right, ..
            }) => self.perform(Cmd::Move(Direction::Right)),

            Event::Keyboard(key) if key == keys.global_left.key_event() => {
                self.perform(Cmd::Move(Direction::Left))
            }
            Event::Keyboard(key) if key == keys.global_right.key_event() => {
                self.perform(Cmd::Move(Direction::Right))
            }
            Event::Keyboard(key) if key == keys.global_up.key_event() => {
                self.perform(Cmd::Move(Direction::Left))
            }
            Event::Keyboard(key) if key == keys.global_down.key_event() => {
                self.perform(Cmd::Move(Direction::Right))
            }
            Event::Keyboard(key) if key == keys.global_quit.key_event() => {
                return Some(Msg::QuitPopupCloseCancel)
            }
            Event::Keyboard(key) if key == keys.global_esc.key_event() => {
                return Some(Msg::QuitPopupCloseCancel)
            }
            Event::Keyboard(KeyEvent {
                code: Key::Char('y'),
                ..
            }) => {
                // ordering is 0 = No, 1 = Yes
                self.component.attr(
                    Attribute::Value,
                    AttrValue::Payload(PropPayload::One(PropValue::Usize(1))),
                );
                self.perform(Cmd::Submit)
            }
            Event::Keyboard(KeyEvent {
                code: Key::Char('n'),
                ..
            }) => {
                // ordering is 0 = No, 1 = Yes
                self.component.attr(
                    Attribute::Value,
                    AttrValue::Payload(PropPayload::One(PropValue::Usize(0))),
                );
                self.perform(Cmd::Submit)
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
                // .background(Color::Black)
                .modifiers(TextModifiers::BOLD)
                .alignment(Alignment::Center)
                .title(title, Alignment::Center)
                .text(vec![TextSpan::from(msg.as_ref().to_string())].as_slice()),
        }
    }
}

impl Component<Msg, NoUserEvent> for MessagePopup {
    fn on(&mut self, _ev: Event<NoUserEvent>) -> Option<Msg> {
        // match ev {
        //     _ => None,
        // }
        None
    }
}
