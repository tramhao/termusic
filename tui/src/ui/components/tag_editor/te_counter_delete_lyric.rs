/*
 * MIT License
 *
 * tui-realm - Copyright (C) 2021 Christian Visintin
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
// use crate::utils::get_block;
use termusicplayback::SharedSettings;
use tui_realm_stdlib::utils::get_block;
use tuirealm::command::{Cmd, CmdResult};
use tuirealm::event::{Key, KeyEvent, KeyModifiers, NoUserEvent};
use tuirealm::props::{Alignment, Borders, Color, Style, TextModifiers};
use tuirealm::tui::layout::Rect;
use tuirealm::tui::widgets::{BorderType, Paragraph};
use tuirealm::{
    AttrValue, Attribute, Component, Event, Frame, MockComponent, Props, State, StateValue,
};

use crate::ui::{Model, Msg, TEMsg, TFMsg};
/// ## Counter
///
/// Counter which increments its value on Submit
#[derive(Default)]
struct Counter {
    props: Props,
    states: OwnStates,
}

#[allow(unused)]
impl Counter {
    pub fn label<S>(mut self, label: S) -> Self
    where
        S: AsRef<str>,
    {
        self.attr(
            Attribute::Title,
            AttrValue::Title((label.as_ref().to_string(), Alignment::Center)),
        );
        self
    }

    pub fn value(mut self, n: isize) -> Self {
        self.attr(Attribute::Value, AttrValue::Number(n));
        self
    }

    pub fn alignment(mut self, a: Alignment) -> Self {
        self.attr(Attribute::TextAlign, AttrValue::Alignment(a));
        self
    }

    pub fn foreground(mut self, c: Color) -> Self {
        self.attr(Attribute::Foreground, AttrValue::Color(c));
        self
    }

    pub fn background(mut self, c: Color) -> Self {
        self.attr(Attribute::Background, AttrValue::Color(c));
        self
    }

    pub fn modifiers(mut self, m: TextModifiers) -> Self {
        self.attr(Attribute::TextProps, AttrValue::TextModifiers(m));
        self
    }

    pub fn borders(mut self, b: Borders) -> Self {
        self.attr(Attribute::Borders, AttrValue::Borders(b));
        self
    }
}

impl MockComponent for Counter {
    fn view(&mut self, frame: &mut Frame<'_>, area: Rect) {
        // Check if visible
        if self.props.get_or(Attribute::Display, AttrValue::Flag(true)) == AttrValue::Flag(true) {
            // Get properties
            let value = self
                .props
                .get_or(Attribute::Value, AttrValue::Number(99))
                .unwrap_number();
            let text = format!("Delete ({value})");

            let alignment = self
                .props
                .get_or(Attribute::TextAlign, AttrValue::Alignment(Alignment::Left))
                .unwrap_alignment();
            let foreground = self
                .props
                .get_or(Attribute::Foreground, AttrValue::Color(Color::Reset))
                .unwrap_color();
            let background = self
                .props
                .get_or(Attribute::Background, AttrValue::Color(Color::Reset))
                .unwrap_color();
            let modifiers = self
                .props
                .get_or(
                    Attribute::TextProps,
                    AttrValue::TextModifiers(TextModifiers::empty()),
                )
                .unwrap_text_modifiers();
            let title = self
                .props
                .get_or(
                    Attribute::Title,
                    AttrValue::Title((String::default(), Alignment::Center)),
                )
                .unwrap_title();
            let borders = self
                .props
                .get_or(Attribute::Borders, AttrValue::Borders(Borders::default()))
                .unwrap_borders();
            let focus = self
                .props
                .get_or(Attribute::Focus, AttrValue::Flag(false))
                .unwrap_flag();

            let inactive_style = self
                .props
                .get(Attribute::FocusStyle)
                .map(tuirealm::AttrValue::unwrap_style);
            frame.render_widget(
                Paragraph::new(text)
                    .block(get_block(borders, Some(title), focus, inactive_style))
                    .style(
                        Style::default()
                            .fg(foreground)
                            .bg(background)
                            .add_modifier(modifiers),
                    )
                    .alignment(alignment),
                area,
            );
        }
    }

    fn query(&self, attr: Attribute) -> Option<AttrValue> {
        self.props.get(attr)
    }

    fn attr(&mut self, attr: Attribute, value: AttrValue) {
        self.props.set(attr, value);
    }

    fn state(&self) -> State {
        State::One(StateValue::Isize(self.states.counter))
    }

    fn perform(&mut self, cmd: Cmd) -> CmdResult {
        match cmd {
            Cmd::Submit => {
                // self.states.incr();
                CmdResult::Changed(self.state())
            }
            _ => CmdResult::None,
        }
    }
}

#[derive(Default)]
struct OwnStates {
    counter: isize,
}

// impl Default for OwnStates {
//     fn default() -> Self {
//         Self { counter: 0 }
//     }
// }
#[allow(unused)]
impl OwnStates {
    fn incr(&mut self) {
        self.counter += 1;
    }
}

// -- Counter components

#[derive(MockComponent)]
pub struct TECounterDelete {
    component: Counter,
    config: SharedSettings,
}

impl TECounterDelete {
    pub fn new(initial_value: isize, config: SharedSettings) -> Self {
        let component = {
            let config = config.read();
            Counter::default()
                .alignment(Alignment::Center)
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
                                .unwrap_or(Color::LightRed),
                        )
                        .modifiers(BorderType::Rounded),
                )
                .foreground(
                    // config
                    //     .style_color_symbol
                    //     .library_highlight()
                    //     .unwrap_or(Color::Cyan),
                    Color::Red,
                )
                .modifiers(TextModifiers::BOLD)
                .value(initial_value)
        };

        Self { component, config }
    }
}

impl Component<Msg, NoUserEvent> for TECounterDelete {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        let keys = &self.config.read().keys;
        // Get command
        let _cmd = match ev {
            Event::Keyboard(keyevent) if keyevent == keys.config_save.key_event() => {
                return Some(Msg::TagEditor(TEMsg::TERename))
            }
            Event::Keyboard(KeyEvent { code: Key::Tab, .. }) => {
                return Some(Msg::TagEditor(TEMsg::TEFocus(TFMsg::CounterDeleteBlurDown)))
            }
            Event::Keyboard(KeyEvent {
                code: Key::BackTab,
                modifiers: KeyModifiers::SHIFT,
            }) => return Some(Msg::TagEditor(TEMsg::TEFocus(TFMsg::CounterDeleteBlurUp))),

            Event::Keyboard(KeyEvent {
                code: Key::Down, ..
            }) => return Some(Msg::TagEditor(TEMsg::TEFocus(TFMsg::CounterDeleteBlurDown))),
            Event::Keyboard(KeyEvent { code: Key::Up, .. }) => {
                return Some(Msg::TagEditor(TEMsg::TEFocus(TFMsg::CounterDeleteBlurUp)))
            }
            Event::Keyboard(keyevent) if keyevent == keys.global_quit.key_event() => {
                return Some(Msg::TagEditor(TEMsg::TagEditorClose(None)))
            }
            Event::Keyboard(keyevent) if keyevent == keys.global_esc.key_event() => {
                return Some(Msg::TagEditor(TEMsg::TagEditorClose(None)))
            }
            Event::Keyboard(keyevent) if keyevent == keys.global_up.key_event() => {
                return Some(Msg::TagEditor(TEMsg::TEFocus(TFMsg::CounterDeleteBlurUp)))
            }

            Event::Keyboard(keyevent) if keyevent == keys.global_down.key_event() => {
                return Some(Msg::TagEditor(TEMsg::TEFocus(TFMsg::CounterDeleteBlurDown)))
            }

            Event::Keyboard(KeyEvent {
                code: Key::Enter, ..
            }) => return Some(Msg::TagEditor(TEMsg::TECounterDeleteOk)),
            _ => Cmd::None,
        };
        None
    }
}
impl Model {
    pub fn te_delete_lyric(&mut self) {
        if let Some(mut song) = self.tageditor_song.clone() {
            if song.lyric_frames_is_empty() {
                song.set_parsed_lyric(None);
                return;
            }
            song.lyric_frames_remove_selected();
            if (song.lyric_selected_index() >= song.lyric_frames_len())
                && (song.lyric_selected_index() > 0)
            {
                song.set_lyric_selected_index(song.lyric_selected_index() - 1);
            }
            match song.save_tag() {
                Ok(()) => self.init_by_song(&song),
                Err(e) => {
                    self.mount_error_popup(e);
                }
            }
        }
    }
}
