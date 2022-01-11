//! # Popups
//!
//!
//! Popups components

use super::{KeyBind, Keys};
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
use tuirealm::{AttrValue, Attribute, Component, Event, MockComponent, State, StateValue};

#[derive(MockComponent)]
pub struct KEInput {
    component: Input,
    id: IdKeyEditor,
    on_key_shift: Msg,
    on_key_backshift: Msg,
    // keys: Keys,
}

impl KEInput {
    pub fn new(
        name: &str,
        id: IdKeyEditor,
        keys: &Keys,
        on_key_shift: Msg,
        on_key_backshift: Msg,
    ) -> Self {
        let init_value = Self::init_key(&id, keys);
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
            // keys,
            on_key_shift,
            on_key_backshift,
        }
    }

    fn init_key(id: &IdKeyEditor, keys: &Keys) -> String {
        match *id {
            IdKeyEditor::GlobalQuitInput => keys.global_quit.key(),
            IdKeyEditor::GlobalLeftInput => keys.global_left.key(),
            IdKeyEditor::GlobalRightInput => keys.global_right.key(),
            IdKeyEditor::GlobalUpInput => keys.global_up.key(),
            IdKeyEditor::GlobalDownInput => keys.global_down.key(),
            _ => "".to_string(),
        }
    }

    fn update_key(&mut self, result: CmdResult) -> Msg {
        if let CmdResult::Changed(State::One(StateValue::String(codes))) = result {
            if codes.is_empty() {
                self.update_symbol_after(Color::Blue);
                return Msg::None;
            }
            if KeyBind::key_from_str(&codes).is_ok() {
                // success getting a unicode letter
                self.update_symbol_after(Color::Green);
                return Msg::KeyEditor(KEMsg::KeyChanged(self.id.clone()));
            }
            // fail to get a good code
            self.update_symbol_after(Color::Red);
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
                ..
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
                "",
                IdKeyEditor::GlobalQuitInput,
                keys,
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

#[derive(MockComponent)]
pub struct KEGlobalLeftInput {
    component: KEInput,
}

impl KEGlobalLeftInput {
    pub fn new(keys: &Keys) -> Self {
        Self {
            component: KEInput::new(
                "",
                IdKeyEditor::GlobalLeftInput,
                keys,
                Msg::KeyEditor(KEMsg::GlobalLeftInputBlurDown),
                Msg::KeyEditor(KEMsg::GlobalLeftInputBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for KEGlobalLeftInput {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct KEGlobalRightInput {
    component: KEInput,
}

impl KEGlobalRightInput {
    pub fn new(keys: &Keys) -> Self {
        Self {
            component: KEInput::new(
                "",
                IdKeyEditor::GlobalRightInput,
                keys,
                Msg::KeyEditor(KEMsg::GlobalRightInputBlurDown),
                Msg::KeyEditor(KEMsg::GlobalRightInputBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for KEGlobalRightInput {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}
#[derive(MockComponent)]
pub struct KEGlobalUpInput {
    component: KEInput,
}

impl KEGlobalUpInput {
    pub fn new(keys: &Keys) -> Self {
        Self {
            component: KEInput::new(
                "",
                IdKeyEditor::GlobalUpInput,
                keys,
                Msg::KeyEditor(KEMsg::GlobalUpInputBlurDown),
                Msg::KeyEditor(KEMsg::GlobalUpInputBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for KEGlobalUpInput {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}
#[derive(MockComponent)]
pub struct KEGlobalDownInput {
    component: KEInput,
}

impl KEGlobalDownInput {
    pub fn new(keys: &Keys) -> Self {
        Self {
            component: KEInput::new(
                "",
                IdKeyEditor::GlobalDownInput,
                keys,
                Msg::KeyEditor(KEMsg::GlobalDownInputBlurDown),
                Msg::KeyEditor(KEMsg::GlobalDownInputBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for KEGlobalDownInput {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}
