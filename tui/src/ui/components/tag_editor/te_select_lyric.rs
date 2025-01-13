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
use termusiclib::config::SharedTuiSettings;
use tui_realm_stdlib::Select;
use tuirealm::command::{Cmd, CmdResult, Direction};
use tuirealm::event::{Key, KeyEvent, KeyModifiers, NoUserEvent};
use tuirealm::props::{Alignment, BorderType, Borders};
use tuirealm::{Component, Event, MockComponent, State, StateValue};

#[derive(MockComponent)]
pub struct TESelectLyric {
    component: Select,
    config: SharedTuiSettings,
}

impl TESelectLyric {
    pub fn new(config: SharedTuiSettings) -> Self {
        let component = {
            let config = config.read();
            Select::default()
                .borders(
                    Borders::default()
                        .modifiers(BorderType::Rounded)
                        .color(config.settings.theme.library_border()),
                )
                .foreground(config.settings.theme.library_foreground())
                .title(" Select a lyric ", Alignment::Center)
                .rewind(true)
                .highlighted_color(config.settings.theme.library_highlight())
                .highlighted_str(&config.settings.theme.style.library.highlight_symbol)
                .choices(&["No Lyric"])
        };

        Self { component, config }
    }
}

impl Component<Msg, NoUserEvent> for TESelectLyric {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        let config = self.config.clone();
        let keys = &config.read().settings.keys;
        let cmd_result = match ev {
            Event::Keyboard(keyevent) if keyevent == keys.config_keys.save.get() => {
                return Some(Msg::TagEditor(TEMsg::TERename))
            }
            Event::Keyboard(keyevent) if keyevent == keys.quit.get() => match self.state() {
                State::One(_) => return Some(Msg::TagEditor(TEMsg::TagEditorClose(None))),
                _ => self.perform(Cmd::Cancel),
            },
            Event::Keyboard(keyevent) if keyevent == keys.escape.get() => match self.state() {
                State::One(_) => return Some(Msg::TagEditor(TEMsg::TagEditorClose(None))),
                _ => self.perform(Cmd::Cancel),
            },
            Event::Keyboard(KeyEvent { code: Key::Tab, .. }) => {
                return Some(Msg::TagEditor(TEMsg::TEFocus(TFMsg::SelectLyricBlurDown)))
            }
            Event::Keyboard(KeyEvent {
                code: Key::BackTab,
                modifiers: KeyModifiers::SHIFT,
            }) => return Some(Msg::TagEditor(TEMsg::TEFocus(TFMsg::SelectLyricBlurUp))),

            Event::Keyboard(keyevent) if keyevent == keys.navigation_keys.up.get() => {
                match self.state() {
                    State::One(_) => {
                        return Some(Msg::TagEditor(TEMsg::TEFocus(TFMsg::SelectLyricBlurUp)))
                    }
                    _ => self.perform(Cmd::Move(Direction::Up)),
                }
            }
            Event::Keyboard(keyevent) if keyevent == keys.navigation_keys.down.get() => {
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
            CmdResult::None => None,
            _ => Some(Msg::ForceRedraw),
        }
    }
}
