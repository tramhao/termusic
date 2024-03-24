/*
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
use crate::ui::{Msg, TEMsg, TFMsg};
use termusicplayback::SharedSettings;
use tui_realm_stdlib::Select;
use tuirealm::command::{Cmd, CmdResult, Direction};
use tuirealm::event::{Key, KeyEvent, KeyModifiers, NoUserEvent};
use tuirealm::props::{Alignment, BorderType, Borders, Color};
use tuirealm::{Component, Event, MockComponent, State, StateValue};

#[derive(MockComponent)]
pub struct TESelectLyric {
    component: Select,
    config: SharedSettings,
}

impl TESelectLyric {
    pub fn new(config: SharedSettings) -> Self {
        let component = {
            let config = config.read();
            Select::default()
                .borders(
                    Borders::default().modifiers(BorderType::Rounded).color(
                        config
                            .style_color_symbol
                            .library_border()
                            .unwrap_or(Color::Blue),
                    ),
                )
                .foreground(
                    config
                        .style_color_symbol
                        .library_foreground()
                        .unwrap_or(Color::LightRed),
                )
                .title(" Select a lyric ", Alignment::Center)
                .rewind(true)
                .highlighted_color(
                    config
                        .style_color_symbol
                        .library_highlight()
                        .unwrap_or(Color::LightGreen),
                )
                .highlighted_str(&config.style_color_symbol.library_highlight_symbol)
                .choices(&["No Lyric"])
        };

        Self { component, config }
    }
}

impl Component<Msg, NoUserEvent> for TESelectLyric {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        let config = self.config.clone();
        let keys = &config.read().keys;
        let cmd_result = match ev {
            Event::Keyboard(keyevent) if keyevent == keys.config_save.key_event() => {
                return Some(Msg::TagEditor(TEMsg::TERename))
            }
            Event::Keyboard(keyevent) if keyevent == keys.global_quit.key_event() => {
                match self.state() {
                    State::One(_) => return Some(Msg::TagEditor(TEMsg::TagEditorClose(None))),
                    _ => self.perform(Cmd::Cancel),
                }
            }
            Event::Keyboard(keyevent) if keyevent == keys.global_esc.key_event() => {
                match self.state() {
                    State::One(_) => return Some(Msg::TagEditor(TEMsg::TagEditorClose(None))),
                    _ => self.perform(Cmd::Cancel),
                }
            }
            Event::Keyboard(KeyEvent { code: Key::Tab, .. }) => {
                return Some(Msg::TagEditor(TEMsg::TEFocus(TFMsg::SelectLyricBlurDown)))
            }
            Event::Keyboard(KeyEvent {
                code: Key::BackTab,
                modifiers: KeyModifiers::SHIFT,
            }) => return Some(Msg::TagEditor(TEMsg::TEFocus(TFMsg::SelectLyricBlurUp))),

            Event::Keyboard(keyevent) if keyevent == keys.global_up.key_event() => {
                match self.state() {
                    State::One(_) => {
                        return Some(Msg::TagEditor(TEMsg::TEFocus(TFMsg::SelectLyricBlurUp)))
                    }
                    _ => self.perform(Cmd::Move(Direction::Up)),
                }
            }
            Event::Keyboard(keyevent) if keyevent == keys.global_down.key_event() => {
                match self.state() {
                    State::One(_) => {
                        return Some(Msg::TagEditor(TEMsg::TEFocus(TFMsg::SelectLyricBlurDown)))
                    }
                    _ => self.perform(Cmd::Move(Direction::Down)),
                }
            }
            Event::Keyboard(KeyEvent {
                code: Key::Down, ..
            }) => match self.state() {
                State::One(_) => {
                    return Some(Msg::TagEditor(TEMsg::TEFocus(TFMsg::SelectLyricBlurDown)))
                }
                _ => self.perform(Cmd::Move(Direction::Down)),
            },
            Event::Keyboard(KeyEvent { code: Key::Up, .. }) => match self.state() {
                State::One(_) => {
                    return Some(Msg::TagEditor(TEMsg::TEFocus(TFMsg::SelectLyricBlurUp)))
                }
                _ => self.perform(Cmd::Move(Direction::Up)),
            },
            Event::Keyboard(KeyEvent {
                code: Key::Enter, ..
            }) => self.perform(Cmd::Submit),
            _ => CmdResult::None,
        };
        match cmd_result {
            CmdResult::Submit(State::One(StateValue::Usize(index))) => {
                Some(Msg::TagEditor(TEMsg::TESelectLyricOk(index)))
            }
            _ => Some(Msg::None),
        }
    }
}
