//! # Popups
//!
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
use crate::ui::components::StyleColorSymbol;
use crate::ui::{CEMsg, IdColorEditor, Msg};

use std::str;
use tui_realm_stdlib::Input;
use tuirealm::command::{Cmd, CmdResult, Direction, Position};
use tuirealm::event::{Key, KeyEvent, KeyModifiers};
use tuirealm::props::{Alignment, BorderType, Borders, Color, InputType, Style};
use tuirealm::{
    AttrValue, Attribute, Component, Event, MockComponent, NoUserEvent, State, StateValue,
};

#[derive(MockComponent)]
pub struct CEInputHighlight {
    component: Input,
    id: IdColorEditor,
    // highlight_symbol: String,
    // color_mapping: StyleColorSymbol,
    // on_key_down: Msg,
    // on_key_up: Msg,
}

impl CEInputHighlight {
    pub fn new(
        name: &str,
        id: IdColorEditor,
        highlight_str: &str,
        _color_mapping: &StyleColorSymbol,
    ) -> Self {
        Self {
            component: Input::default()
                .borders(
                    Borders::default()
                        .modifiers(BorderType::Rounded)
                        .color(Color::Blue),
                )
                // .foreground(color)
                .input_type(InputType::Text)
                .placeholder(
                    "1f984/1f680/27a4",
                    Style::default().fg(Color::Rgb(128, 128, 128)),
                )
                .title(name, Alignment::Left)
                .value(highlight_str),
            id,
            // highlight_symbol: highlight_str.to_string(),
            // color_mapping: color_mapping.clone(),
        }
    }
    fn update_symbol(&mut self, result: CmdResult) -> Msg {
        if let CmdResult::Changed(State::One(StateValue::String(symbol))) = result.clone() {
            if symbol.is_empty() {
                self.update_symbol_after(Color::Blue);
                return Msg::None;
            }
            if let Some(s) = Self::string_to_unicode_char(&symbol) {
                // success getting a unicode letter
                self.update_symbol_after(Color::Green);
                return Msg::ColorEditor(CEMsg::SymbolChanged(self.id.clone(), s.to_string()));
            }
            // fail to get a unicode letter
            self.update_symbol_after(Color::Red);
            return Msg::ColorEditor(CEMsg::SymbolChanged(self.id.clone(), symbol));
            // return Msg::None;
        }

        // press enter to see preview
        if let CmdResult::Submit(State::One(StateValue::String(symbol))) = result {
            if let Some(s) = Self::string_to_unicode_char(&symbol) {
                self.attr(Attribute::Value, AttrValue::String(s.to_string()));
                self.update_symbol_after(Color::Green);
                return Msg::ColorEditor(CEMsg::SymbolChanged(self.id.clone(), s.to_string()));
            }
        }
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
    fn string_to_unicode_char(s: &str) -> Option<char> {
        // Do something more appropriate to find the actual number
        // let number = &s[..];

        u32::from_str_radix(s, 16)
            .ok()
            .and_then(std::char::from_u32)
    }
}

impl Component<Msg, NoUserEvent> for CEInputHighlight {
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
                Some(self.update_symbol(result))
            }
            Event::Keyboard(KeyEvent {
                code: Key::Backspace,
                ..
            }) => {
                let result = self.perform(Cmd::Delete);
                Some(self.update_symbol(result))
            }
            Event::Keyboard(KeyEvent {
                code: Key::Char('h'),
                modifiers: KeyModifiers::CONTROL,
            }) => Some(Msg::ColorEditor(CEMsg::HelpPopupShow)),

            Event::Keyboard(KeyEvent {
                code: Key::Char(ch),
                modifiers: KeyModifiers::NONE,
            }) => {
                let result = self.perform(Cmd::Type(ch));
                Some(self.update_symbol(result))
            }
            Event::Keyboard(KeyEvent { code: Key::Tab, .. }) => match self.id {
                IdColorEditor::LibraryHighlightSymbol => {
                    Some(Msg::ColorEditor(CEMsg::LibraryHighlightSymbolBlur))
                }
                IdColorEditor::PlaylistHighlightSymbol => {
                    Some(Msg::ColorEditor(CEMsg::PlaylistHighlightSymbolBlur))
                }
                _ => Some(Msg::None),
            },
            Event::Keyboard(KeyEvent { code: Key::Esc, .. }) => {
                Some(Msg::ColorEditor(CEMsg::ColorEditorCloseCancel))
            }
            Event::Keyboard(KeyEvent {
                code: Key::Enter, ..
            }) => {
                let result = self.perform(Cmd::Submit);
                Some(self.update_symbol(result))
            }
            _ => Some(Msg::None),
        }
    }
}

#[derive(MockComponent)]
pub struct CELibraryHighlightSymbol {
    component: CEInputHighlight,
}

impl CELibraryHighlightSymbol {
    pub fn new(style_color_symbol: &StyleColorSymbol) -> Self {
        Self {
            component: CEInputHighlight::new(
                "Highlight String",
                IdColorEditor::LibraryHighlightSymbol,
                &style_color_symbol.library_highlight_symbol,
                style_color_symbol,
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for CELibraryHighlightSymbol {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct CEPlaylistHighlightSymbol {
    component: CEInputHighlight,
}

impl CEPlaylistHighlightSymbol {
    pub fn new(style_color_symbol: &StyleColorSymbol) -> Self {
        Self {
            component: CEInputHighlight::new(
                "Highlight String",
                IdColorEditor::PlaylistHighlightSymbol,
                &style_color_symbol.playlist_highlight_symbol,
                style_color_symbol,
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for CEPlaylistHighlightSymbol {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}
