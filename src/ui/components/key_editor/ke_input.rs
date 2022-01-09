//! # Popups
//!
//!
//! Popups components

use super::Keys;
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
use crate::ui::{IdKeyEditor, KEMsg, Msg};

use std::str;
use tui_realm_stdlib::Input;
use tuirealm::command::{Cmd, CmdResult, Direction, Position};
use tuirealm::event::{Key, KeyEvent, KeyModifiers, NoUserEvent};
use tuirealm::props::{Alignment, BorderType, Borders, Color, InputType, Style};
use tuirealm::{AttrValue, Attribute, Component, Event, MockComponent};

#[derive(MockComponent)]
pub struct KEInput {
    component: Input,
    id: IdKeyEditor,
    on_key_shift: Msg,
    on_key_backshift: Msg,
    keys: Keys,
}

impl KEInput {
    pub fn new(
        name: &str,
        id: IdKeyEditor,
        keys: Keys,
        on_key_shift: Msg,
        on_key_backshift: Msg,
    ) -> Self {
        let init_value = keys.global_quit.key();
        Self {
            component: Input::default()
                .borders(
                    Borders::default()
                        .modifiers(BorderType::Rounded)
                        .color(Color::Blue),
                )
                // .foreground(color)
                .input_type(InputType::Text)
                .placeholder("a/b/c", Style::default().fg(Color::Rgb(128, 128, 128)))
                .title(name, Alignment::Left)
                .value(init_value),
            id,
            keys,
            on_key_shift,
            on_key_backshift,
        }
    }
    fn update_key(&mut self, result: CmdResult) -> Msg {
        // if let CmdResult::Changed(State::One(StateValue::String(symbol))) = result.clone() {
        //     if symbol.is_empty() {
        //         self.update_symbol_after(Color::Blue);
        //         return Msg::None;
        //     }
        //     if let Some(s) = Self::string_to_unicode_char(&symbol) {
        //         // success getting a unicode letter
        //         self.update_symbol_after(Color::Green);
        //         return Msg::ColorEditor(CEMsg::SymbolChanged(self.id.clone(), s.to_string()));
        //     }
        //     // fail to get a unicode letter
        //     self.update_symbol_after(Color::Red);
        //     // return Msg::ColorEditor(CEMsg::SymbolChanged(self.id.clone(), symbol));
        //     // return Msg::None;
        // }

        // // press enter to see preview
        // if let CmdResult::Submit(State::One(StateValue::String(symbol))) = result {
        //     if let Some(s) = Self::string_to_unicode_char(&symbol) {
        //         self.attr(Attribute::Value, AttrValue::String(s.to_string()));
        //         self.update_symbol_after(Color::Green);
        //         return Msg::ColorEditor(CEMsg::SymbolChanged(self.id.clone(), s.to_string()));
        //     }
        // }
        Msg::None
    }
    fn update_symbol_after(&mut self, color: Color) {
        self.attr(Attribute::Foreground, AttrValue::Color(color));
        self.attr(
            Attribute::Borders,
            AttrValue::Borders(
                Borders::default()
                    .modifiers(BorderType::Rounded)
                    .color(color),
            ),
        );
    }
}

impl Component<Msg, NoUserEvent> for KEInput {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        match ev {
            Event::Keyboard(KeyEvent {
                code: Key::Left, ..
            }) => {
                self.perform(Cmd::Move(Direction::Left));
                Some(Msg::None)
            }
            Event::Keyboard(KeyEvent {
                code: Key::Right, ..
            }) => {
                self.perform(Cmd::Move(Direction::Right));
                Some(Msg::None)
            }
            Event::Keyboard(KeyEvent {
                code: Key::Home, ..
            }) => {
                self.perform(Cmd::GoTo(Position::Begin));
                Some(Msg::None)
            }
            Event::Keyboard(KeyEvent { code: Key::End, .. }) => {
                self.perform(Cmd::GoTo(Position::End));
                Some(Msg::None)
            }
            Event::Keyboard(KeyEvent {
                code: Key::Delete, ..
            }) => {
                let result = self.perform(Cmd::Cancel);
                Some(self.update_key(result))
            }
            Event::Keyboard(KeyEvent {
                code: Key::Backspace,
                ..
            }) => {
                let result = self.perform(Cmd::Delete);
                Some(self.update_key(result))
            }
            Event::Keyboard(KeyEvent {
                code: Key::Char('h'),
                modifiers: KeyModifiers::CONTROL,
            }) => Some(Msg::KeyEditor(KEMsg::HelpPopupShow)),

            Event::Keyboard(KeyEvent {
                code: Key::Char(ch),
                modifiers: KeyModifiers::NONE,
            }) => {
                let result = self.perform(Cmd::Type(ch));
                Some(self.update_key(result))
            }
            Event::Keyboard(KeyEvent { code: Key::Tab, .. }) => Some(self.on_key_shift.clone()),
            Event::Keyboard(KeyEvent {
                code: Key::BackTab,
                modifiers: KeyModifiers::SHIFT,
            }) => Some(self.on_key_backshift.clone()),

            Event::Keyboard(KeyEvent { code: Key::Esc, .. }) => {
                Some(Msg::KeyEditor(KEMsg::KeyEditorCloseCancel))
            }
            Event::Keyboard(KeyEvent {
                code: Key::Enter, ..
            }) => {
                let result = self.perform(Cmd::Submit);
                Some(self.update_key(result))
            }
            _ => Some(Msg::None),
        }
    }
}

#[derive(MockComponent)]
pub struct KEGlobalQuitInput {
    component: KEInput,
}

impl KEGlobalQuitInput {
    pub fn new(keys: &Keys) -> Self {
        Self {
            component: KEInput::new(
                "HotKey:",
                IdKeyEditor::GlobalQuitInput,
                keys.clone(),
                Msg::KeyEditor(KEMsg::GlobalQuitInputBlurDown),
                Msg::KeyEditor(KEMsg::GlobalQuitInputBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for KEGlobalQuitInput {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

// #[derive(MockComponent)]
// pub struct CEPlaylistHighlightSymbol {
//     component: CEInputHighlight,
// }

// impl CEPlaylistHighlightSymbol {
//     pub fn new(style_color_symbol: &StyleColorSymbol) -> Self {
//         Self {
//             component: CEInputHighlight::new(
//                 "Highlight Symbol",
//                 IdColorEditor::PlaylistHighlightSymbol,
//                 &style_color_symbol.playlist_highlight_symbol,
//                 style_color_symbol,
//             ),
//         }
//     }
// }

// impl Component<Msg, NoUserEvent> for CEPlaylistHighlightSymbol {
//     fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
//         self.component.on(ev)
//     }
// }
