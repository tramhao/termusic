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
use super::{GSMsg, Id, Msg};

use crate::ui::Model;
use if_chain::if_chain;
use tui_realm_stdlib::{Input, Table};
use tui_realm_treeview::TREE_INITIAL_NODE;
use tuirealm::command::{Cmd, CmdResult, Direction, Position};
use tuirealm::event::{Key, KeyEvent, KeyModifiers, NoUserEvent};
use tuirealm::props::{Alignment, BorderType, Borders, Color, InputType, TableBuilder, TextSpan};
use tuirealm::{AttrValue, Attribute, Component, Event, MockComponent, State, StateValue};

#[derive(MockComponent)]
pub struct GSInputPopup {
    component: Input,
    source: Source,
}

impl GSInputPopup {
    pub fn new(source: Source) -> Self {
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
                .title("Search for: (support * and ?)", Alignment::Left),
            source,
        }
    }
}

impl Component<Msg, NoUserEvent> for GSInputPopup {
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
                return Some(Msg::GeneralSearch(GSMsg::PopupCloseCancel));
            }
            Event::Keyboard(KeyEvent {
                code: Key::Enter, ..
            }) => self.perform(Cmd::Submit),
            Event::Keyboard(KeyEvent { code: Key::Tab, .. }) => {
                return Some(Msg::GeneralSearch(GSMsg::InputBlur))
            }
            _ => CmdResult::None,
        };
        match cmd_result {
            CmdResult::Changed(State::One(StateValue::String(input_string))) => match self.source {
                Source::Library => {
                    Some(Msg::GeneralSearch(GSMsg::PopupUpdateLibrary(input_string)))
                }
                Source::Playlist => {
                    Some(Msg::GeneralSearch(GSMsg::PopupUpdatePlaylist(input_string)))
                }
            },
            CmdResult::Submit(_) => Some(Msg::GeneralSearch(GSMsg::InputBlur)),

            _ => Some(Msg::None),
        }
    }
}

#[derive(MockComponent)]
pub struct GSTablePopup {
    component: Table,
    source: Source,
}
pub enum Source {
    Library,
    Playlist,
}
impl GSTablePopup {
    pub fn new(source: Source) -> Self {
        match source {
            Source::Library => Self {
                component: Table::default()
                    .borders(
                        Borders::default()
                            .modifiers(BorderType::Rounded)
                            .color(Color::Green),
                    )
                    // .foreground(Color::Yellow)
                    .background(Color::Black)
                    .title(
                        "Results:(Enter: locate/l: load to playlist)",
                        Alignment::Left,
                    )
                    .scroll(true)
                    .highlighted_color(Color::LightBlue)
                    .highlighted_str("\u{1f680}")
                    // .highlighted_str("🚀")
                    .rewind(false)
                    .step(4)
                    .row_height(1)
                    .headers(&["index", "File name"])
                    .column_spacing(3)
                    .widths(&[5, 95])
                    .table(
                        TableBuilder::default()
                            .add_col(TextSpan::from("Empty result."))
                            .add_col(TextSpan::from("Loading..."))
                            .build(),
                    ),
                source,
            },

            Source::Playlist => Self {
                component: Table::default()
                    .borders(
                        Borders::default()
                            .modifiers(BorderType::Rounded)
                            .color(Color::Green),
                    )
                    // .foreground(Color::Yellow)
                    .background(Color::Black)
                    .title("Results:(Enter: locate/l: play selected)", Alignment::Left)
                    .scroll(true)
                    .highlighted_color(Color::LightBlue)
                    .highlighted_str("\u{1f680}")
                    // .highlighted_str("🚀")
                    .rewind(false)
                    .step(4)
                    .row_height(1)
                    .headers(&["Duration", "Artist", "Title"])
                    .column_spacing(3)
                    .widths(&[14, 30, 56])
                    .table(
                        TableBuilder::default()
                            .add_col(TextSpan::from("Empty result."))
                            .add_col(TextSpan::from("Loading..."))
                            .build(),
                    ),
                source,
            },
        }
    }
}

impl Component<Msg, NoUserEvent> for GSTablePopup {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        let cmd_result = match ev {
            Event::Keyboard(KeyEvent { code: Key::Esc, .. }) => {
                return Some(Msg::GeneralSearch(GSMsg::PopupCloseCancel))
            }

            Event::Keyboard(KeyEvent {
                code: Key::Down | Key::Char('j'),
                ..
            }) => self.perform(Cmd::Move(Direction::Down)),
            Event::Keyboard(KeyEvent {
                code: Key::Up | Key::Char('k'),
                ..
            }) => self.perform(Cmd::Move(Direction::Up)),
            Event::Keyboard(KeyEvent {
                code: Key::PageDown,
                ..
            }) => self.perform(Cmd::Scroll(Direction::Down)),
            Event::Keyboard(KeyEvent {
                code: Key::PageUp, ..
            }) => self.perform(Cmd::Scroll(Direction::Up)),
            Event::Keyboard(KeyEvent {
                code: Key::Home | Key::Char('g'),
                ..
            }) => self.perform(Cmd::GoTo(Position::Begin)),
            Event::Keyboard(
                KeyEvent { code: Key::End, .. }
                | KeyEvent {
                    code: Key::Char('G'),
                    modifiers: KeyModifiers::SHIFT,
                },
            ) => self.perform(Cmd::GoTo(Position::End)),
            Event::Keyboard(KeyEvent { code: Key::Tab, .. }) => {
                return Some(Msg::GeneralSearch(GSMsg::TableBlur))
            }
            Event::Keyboard(KeyEvent {
                code: Key::Char('l'),
                ..
            }) => match self.source {
                Source::Library => {
                    return Some(Msg::GeneralSearch(GSMsg::PopupCloseLibraryAddPlaylist))
                }
                Source::Playlist => {
                    return Some(Msg::GeneralSearch(GSMsg::PopupClosePlaylistPlaySelected))
                }
            },
            Event::Keyboard(KeyEvent {
                code: Key::Enter, ..
            }) => match self.source {
                Source::Library => {
                    return Some(Msg::GeneralSearch(GSMsg::PopupCloseOkLibraryLocate))
                }
                Source::Playlist => {
                    return Some(Msg::GeneralSearch(GSMsg::PopupCloseOkPlaylistLocate))
                }
            },
            _ => CmdResult::None,
        };
        match cmd_result {
            // CmdResult::Submit(State::One(StateValue::Usize(index))) => {
            //     Some(Msg::LibrarySearchPopupCloseOkLocate(index))
            // }
            _ => Some(Msg::None),
        }
    }
}

impl Model {
    pub fn general_search_update_show(&mut self, table: Vec<Vec<TextSpan>>) {
        self.app
            .attr(
                &Id::GeneralSearchTable,
                tuirealm::Attribute::Content,
                tuirealm::AttrValue::Table(table),
            )
            .ok();
    }
    pub fn general_search_after_library_select(&mut self) {
        if_chain!(
            if let Ok(State::One(StateValue::Usize(index))) = self.app.state(&Id::GeneralSearchTable);
            if let Ok(Some(AttrValue::Table(table))) =
                self.app.query(&Id::GeneralSearchTable, Attribute::Content);
            if let Some(line) = table.get(index);
            if let Some(text_span) = line.get(1);
            then {
                let node = &text_span.content;
                assert!(self
                    .app
                    .attr(
                        &Id::Library,
                        Attribute::Custom(TREE_INITIAL_NODE),
                        AttrValue::String(node.to_string()),
                    )
                    .is_ok());
            }
        );
    }

    pub fn general_search_after_library_add_playlist(&mut self) {
        if_chain! {
            if let Ok(State::One(StateValue::Usize(index))) = self.app.state(&Id::GeneralSearchTable);
            if let Ok(Some(AttrValue::Table(table))) =
                self.app.query(&Id::GeneralSearchTable, Attribute::Content);
            if let Some(line) = table.get(index);
            if let Some(text_span) = line.get(1);
            let text = &text_span.content;
            then {
                self.playlist_add(text);
            }
        }
    }

    pub fn general_search_after_playlist_select(&mut self) {
        let mut index = 0;
        let mut matched = false;
        if_chain! {
            if let Ok(State::One(StateValue::Usize(result_index))) =
                self.app.state(&Id::GeneralSearchTable);
            if let Ok(Some(AttrValue::Table(table))) =
                self.app.query(&Id::GeneralSearchTable, Attribute::Content);
            if let Some(line) = table.get(result_index);
            if let Some(file_name_text_span) = line.get(3);
            let file_name = &file_name_text_span.content;
            then {
                for (idx, item) in self.playlist_items.iter().enumerate() {
                    if item.file() == Some(file_name) {
                        index = idx;
                        matched = true;
                    }
                }
            }
        }
        if !matched {
            return;
        }
        self.playlist_locate(index);
    }

    pub fn general_search_after_playlist_play_selected(&mut self) {
        let mut index = 0;
        let mut matched = false;
        if_chain! {
            if let Ok(State::One(StateValue::Usize(result_index))) =
                self.app.state(&Id::GeneralSearchTable);
            if let Ok(Some(AttrValue::Table(table))) =
                    self.app.query(&Id::GeneralSearchTable, Attribute::Content);
            if let Some(line) = table.get(result_index);
            if let Some(file_name_text_span) = line.get(3);
            let file_name = &file_name_text_span.content;
            then {
                for (idx, item) in self.playlist_items.iter().enumerate() {
                    if item.file() == Some(file_name) {
                        index = idx;
                        matched = true;
                    }
                }

            }
        }
        if !matched {
            return;
        }
        self.playlist_play_selected(index);
    }
}
