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
mod color;
mod general;
mod key_combo;
mod update;
mod view;

use crate::config::Settings;
use crate::ui::model::ConfigEditorLayout;
use crate::ui::{ConfigEditorMsg, Msg};
pub use color::*;
pub use general::*;
pub use key_combo::*;

use tui_realm_stdlib::{Radio, Span};
use tuirealm::props::{Alignment, BorderSides, BorderType, Borders, Color, Style, TextSpan};
use tuirealm::{
    command::{Cmd, CmdResult, Direction},
    event::{Key, KeyEvent, NoUserEvent},
    Component, Event, MockComponent, State, StateValue,
};

#[derive(MockComponent)]
pub struct CEHeader {
    component: Radio,
}

impl CEHeader {
    pub fn new(layout: &ConfigEditorLayout, config: &Settings) -> Self {
        Self {
            component: Radio::default()
                .borders(
                    Borders::default()
                        .modifiers(BorderType::Plain)
                        .sides(BorderSides::BOTTOM),
                )
                .choices(&[
                    "General Configuration",
                    "Themes and Colors",
                    "Keys Global",
                    "Keys Other",
                ])
                .foreground(
                    config
                        .style_color_symbol
                        .library_highlight()
                        .unwrap_or(Color::Yellow),
                )
                .inactive(
                    Style::default().fg(config
                        .style_color_symbol
                        .library_highlight()
                        .unwrap_or(Color::Red)),
                )
                .value(match layout {
                    ConfigEditorLayout::General => 0,
                    ConfigEditorLayout::Color => 1,
                    ConfigEditorLayout::Key1 => 2,
                    ConfigEditorLayout::Key2 => 3,
                }),
        }
    }
}

impl Component<Msg, NoUserEvent> for CEHeader {
    fn on(&mut self, _ev: Event<NoUserEvent>) -> Option<Msg> {
        None
    }
}

#[derive(MockComponent)]
pub struct Footer {
    component: Span,
}

impl Footer {
    pub fn new(config: &Settings) -> Self {
        Self {
            component: Span::default().spans(&[
                TextSpan::new("<CTRL+S>").bold().fg(config
                    .style_color_symbol
                    .library_highlight()
                    .unwrap_or(Color::Cyan)),
                TextSpan::new(" Save parameters ").bold(),
                TextSpan::new("<ESC>").bold().fg(config
                    .style_color_symbol
                    .library_highlight()
                    .unwrap_or(Color::Cyan)),
                TextSpan::new(" Exit ").bold(),
                TextSpan::new("<TAB>").bold().fg(config
                    .style_color_symbol
                    .library_highlight()
                    .unwrap_or(Color::Cyan)),
                TextSpan::new(" Change panel ").bold(),
                TextSpan::new("<UP/DOWN>").bold().fg(config
                    .style_color_symbol
                    .library_highlight()
                    .unwrap_or(Color::Cyan)),
                TextSpan::new(" Change field ").bold(),
                TextSpan::new("<ENTER>").bold().fg(config
                    .style_color_symbol
                    .library_highlight()
                    .unwrap_or(Color::Cyan)),
                TextSpan::new(" Select theme/Preview symbol ").bold(),
            ]),
        }
    }
}

impl Component<Msg, NoUserEvent> for Footer {
    fn on(&mut self, _ev: Event<NoUserEvent>) -> Option<Msg> {
        None
    }
}

#[derive(MockComponent)]
pub struct ConfigSavePopup {
    component: Radio,
    config: Settings,
}

impl ConfigSavePopup {
    pub fn new(config: &Settings) -> Self {
        Self {
            component: Radio::default()
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
                .title(" Config changed. Do you want to save? ", Alignment::Center)
                .rewind(true)
                .choices(&["No", "Yes"])
                .value(0),
            config: config.clone(),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigSavePopup {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        let cmd_result = match ev {
            Event::Keyboard(KeyEvent {
                code: Key::Left, ..
            }) => self.perform(Cmd::Move(Direction::Left)),
            Event::Keyboard(KeyEvent {
                code: Key::Right, ..
            }) => self.perform(Cmd::Move(Direction::Right)),

            Event::Keyboard(key) if key == self.config.keys.global_left.key_event() => {
                self.perform(Cmd::Move(Direction::Left))
            }
            Event::Keyboard(key) if key == self.config.keys.global_right.key_event() => {
                self.perform(Cmd::Move(Direction::Right))
            }
            Event::Keyboard(key) if key == self.config.keys.global_up.key_event() => {
                self.perform(Cmd::Move(Direction::Left))
            }
            Event::Keyboard(key) if key == self.config.keys.global_down.key_event() => {
                self.perform(Cmd::Move(Direction::Right))
            }
            Event::Keyboard(key) if key == self.config.keys.global_quit.key_event() => {
                return Some(Msg::ConfigEditor(ConfigEditorMsg::ConfigSaveCancel))
            }
            Event::Keyboard(key) if key == self.config.keys.global_esc.key_event() => {
                return Some(Msg::ConfigEditor(ConfigEditorMsg::ConfigSaveCancel))
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
            Some(Msg::ConfigEditor(ConfigEditorMsg::ConfigSaveCancel))
        } else if matches!(
            cmd_result,
            CmdResult::Submit(State::One(StateValue::Usize(1)))
        ) {
            Some(Msg::ConfigEditor(ConfigEditorMsg::ConfigSaveOk))
        } else {
            Some(Msg::None)
        }
    }
}
