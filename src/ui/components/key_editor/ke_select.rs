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
// use crate::ui::components::StyleColorSymbol;
use crate::ui::{IdKeyEditor, KEMsg, Msg};
// use lazy_static::lazy_static;
// use regex::Regex;
// use std::convert::From;
use super::Keys;
use tui_realm_stdlib::{Label, Select};
use tuirealm::command::{Cmd, CmdResult, Direction};
use tuirealm::event::{Key, KeyEvent, KeyModifiers, NoUserEvent};
use tuirealm::props::{Alignment, BorderType, Borders, Color, Style, TextModifiers};
use tuirealm::{Component, Event, MockComponent, State, StateValue};

#[derive(Debug, Clone, PartialEq)]
pub enum MyModifiers {
    NONE,
    SHIFT,
    CONTROL,
    ALT,
}
impl From<MyModifiers> for &'static str {
    fn from(modifier: MyModifiers) -> Self {
        match modifier {
            MyModifiers::NONE => "none",
            MyModifiers::SHIFT => "shift",
            MyModifiers::CONTROL => "control",
            MyModifiers::ALT => "alt",
        }
    }
}

impl From<MyModifiers> for String {
    fn from(modifier: MyModifiers) -> Self {
        <MyModifiers as Into<&'static str>>::into(modifier).to_owned()
    }
}

impl MyModifiers {
    const fn as_usize(&self) -> usize {
        match self {
            MyModifiers::NONE => 0,
            MyModifiers::SHIFT => 1,
            MyModifiers::CONTROL => 2,
            MyModifiers::ALT => 3,
        }
    }
}
const MODIFIER_LIST: [MyModifiers; 4] = [
    MyModifiers::NONE,
    MyModifiers::SHIFT,
    MyModifiers::CONTROL,
    MyModifiers::ALT,
];

#[derive(MockComponent)]
pub struct KESelectModifier {
    component: Select,
    id: IdKeyEditor,
    keys: Keys,
    on_key_shift: Msg,
    on_key_backshift: Msg,
}

impl KESelectModifier {
    pub fn new(
        name: &str,
        id: IdKeyEditor,
        keys: Keys,
        on_key_shift: Msg,
        on_key_backshift: Msg,
    ) -> Self {
        let init_value = Self::init_modifier_select(&id, &keys);
        let mut choices = vec![];
        for modifier in &MODIFIER_LIST {
            choices.push(String::from(modifier.clone()));
        }
        Self {
            component: Select::default()
                .borders(
                    Borders::default()
                        .modifiers(BorderType::Rounded)
                        .color(Color::Green),
                )
                .foreground(Color::Green)
                .title(name, Alignment::Left)
                .rewind(false)
                .inactive(Style::default().bg(Color::Green))
                .highlighted_color(Color::LightGreen)
                .highlighted_str(">> ")
                .choices(&choices)
                .value(init_value),
            id,
            keys,
            on_key_shift,
            on_key_backshift,
        }
    }

    const fn init_modifier_select(id: &IdKeyEditor, keys: &Keys) -> usize {
        match *id {
            // IdKeyEditor::GlobalQuit => keys.global_quit.as_usize(),
            // IdColorEditor::LibraryForeground => style_color_symbol.library_foreground.as_usize(),
            // IdColorEditor::LibraryBackground => style_color_symbol.library_background.as_usize(),
            // IdColorEditor::LibraryBorder => style_color_symbol.library_border.as_usize(),
            // IdColorEditor::LibraryHighlight => style_color_symbol.library_highlight.as_usize(),
            // IdColorEditor::PlaylistForeground => style_color_symbol.playlist_foreground.as_usize(),
            // IdColorEditor::PlaylistBackground => style_color_symbol.playlist_background.as_usize(),
            // IdColorEditor::PlaylistBorder => style_color_symbol.playlist_border.as_usize(),
            // IdColorEditor::PlaylistHighlight => style_color_symbol.playlist_highlight.as_usize(),
            // IdColorEditor::ProgressForeground => style_color_symbol.progress_foreground.as_usize(),
            // IdColorEditor::ProgressBackground => style_color_symbol.progress_background.as_usize(),
            // IdColorEditor::ProgressBorder => style_color_symbol.progress_border.as_usize(),
            // IdColorEditor::LyricForeground => style_color_symbol.lyric_foreground.as_usize(),
            // IdColorEditor::LyricBackground => style_color_symbol.lyric_background.as_usize(),
            // IdColorEditor::LyricBorder => style_color_symbol.lyric_border.as_usize(),
            _ => 0,
        }
    }

    fn update_key(&mut self, index: usize) -> Msg {
        // if let Some(color_config) = COLOR_LIST.get(index) {
        //     let color = color_config
        //         .color(&self.style_color_symbol.alacritty_theme)
        //         .unwrap_or(Color::Red);
        //     self.attr(Attribute::Foreground, AttrValue::Color(color));
        //     self.attr(
        //         Attribute::Borders,
        //         AttrValue::Borders(
        //             Borders::default()
        //                 .modifiers(BorderType::Rounded)
        //                 .color(color),
        //         ),
        //     );
        //     self.attr(
        //         Attribute::FocusStyle,
        //         AttrValue::Style(Style::default().bg(color)),
        //     );
        //     Msg::ColorEditor(CEMsg::ColorChanged(self.id.clone(), color_config.clone()))
        // } else {
        //     self.attr(Attribute::Foreground, AttrValue::Color(Color::Red));
        //     self.attr(
        //         Attribute::Borders,
        //         AttrValue::Borders(
        //             Borders::default()
        //                 .modifiers(BorderType::Rounded)
        //                 .color(Color::Red),
        //         ),
        //     );
        //     self.attr(
        //         Attribute::FocusStyle,
        //         AttrValue::Style(Style::default().bg(Color::Red)),
        //     );

        Msg::None
        // }
    }
}

impl Component<Msg, NoUserEvent> for KESelectModifier {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        let cmd_result = match ev {
            Event::Keyboard(KeyEvent { code: Key::Tab, .. }) => {
                return Some(self.on_key_shift.clone())
            }
            Event::Keyboard(KeyEvent {
                code: Key::BackTab,
                modifiers: KeyModifiers::SHIFT,
            }) => return Some(self.on_key_backshift.clone()),
            Event::Keyboard(KeyEvent {
                code: Key::Esc | Key::Char('q'),
                ..
            }) => return Some(Msg::KeyEditor(KEMsg::KeyEditorCloseCancel)),
            Event::Keyboard(KeyEvent {
                code: Key::Char('h'),
                modifiers: KeyModifiers::CONTROL,
            }) => return Some(Msg::KeyEditor(KEMsg::HelpPopupShow)),
            Event::Keyboard(KeyEvent {
                code: Key::Down | Key::Char('j'),
                ..
            }) => self.perform(Cmd::Move(Direction::Down)),
            Event::Keyboard(KeyEvent {
                code: Key::Up | Key::Char('k'),
                ..
            }) => self.perform(Cmd::Move(Direction::Up)),
            Event::Keyboard(KeyEvent {
                code: Key::Enter, ..
            }) => self.perform(Cmd::Submit),
            _ => CmdResult::None,
        };
        match cmd_result {
            CmdResult::Submit(State::One(StateValue::Usize(index))) => {
                Some(self.update_key(index))
                // Some(Msg::TESelectLyricOk(COLOR_LIST[index]))
            }
            _ => Some(Msg::None),
        }
    }
}

#[derive(MockComponent)]
pub struct KEGlobalLabel {
    component: Label,
}

impl Default for KEGlobalLabel {
    fn default() -> Self {
        Self {
            component: Label::default()
                .modifiers(TextModifiers::BOLD)
                .text("Global Hotkeys"),
        }
    }
}

impl Component<Msg, NoUserEvent> for KEGlobalLabel {
    fn on(&mut self, _ev: Event<NoUserEvent>) -> Option<Msg> {
        None
    }
}

#[derive(MockComponent)]
pub struct KEGlobalQuit {
    component: KESelectModifier,
}

impl KEGlobalQuit {
    pub fn new(keys: &Keys) -> Self {
        Self {
            component: KESelectModifier::new(
                "Quit",
                IdKeyEditor::GlobalQuit,
                keys.clone(),
                Msg::KeyEditor(KEMsg::GlobalQuitBlurDown),
                Msg::KeyEditor(KEMsg::GlobalQuitBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for KEGlobalQuit {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}
