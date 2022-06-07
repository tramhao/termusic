/**
 * MIT License
 *
 * termusic - Copyright (c) 2021 Larry Hao
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

use crate::config::{Keys, Termusic};
use crate::ui::Model;
use anyhow::{anyhow, Result};
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
    pub fn new(source: Source, config: &Termusic) -> Self {
        Self {
            component: Input::default()
                .background(
                    config
                        .style_color_symbol
                        .library_background()
                        .unwrap_or(Color::Reset),
                )
                .foreground(
                    config
                        .style_color_symbol
                        .library_foreground()
                        .unwrap_or(Color::Magenta),
                )
                .borders(
                    Borders::default()
                        .color(
                            config
                                .style_color_symbol
                                .library_border()
                                .unwrap_or(Color::Magenta),
                        )
                        .modifiers(BorderType::Rounded),
                )
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
                Source::Database => {
                    Some(Msg::GeneralSearch(GSMsg::PopupUpdateDatabase(input_string)))
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
    keys: Keys,
}
pub enum Source {
    Library,
    Playlist,
    Database,
}
impl GSTablePopup {
    pub fn new(source: Source, config: &Termusic) -> Self {
        let title_library = format!(
            "Results:(Enter: locate/{}: load to playlist)",
            config.keys.global_right
        );
        let title_playlist = format!(
            "Results:(Enter: locate/{}: play selected)",
            config.keys.global_right
        );
        let title_database = format!("Results:( {}: load to playlist)", config.keys.global_right);
        match source {
            Source::Library => Self {
                component: Table::default()
                    .borders(
                        Borders::default()
                            .color(
                                config
                                    .style_color_symbol
                                    .library_border()
                                    .unwrap_or(Color::Magenta),
                            )
                            .modifiers(BorderType::Rounded),
                    )
                    .background(
                        config
                            .style_color_symbol
                            .library_background()
                            .unwrap_or(Color::Reset),
                    )
                    .foreground(
                        config
                            .style_color_symbol
                            .library_foreground()
                            .unwrap_or(Color::Magenta),
                    )
                    .title(title_library, Alignment::Left)
                    .scroll(true)
                    .highlighted_color(
                        config
                            .style_color_symbol
                            .library_highlight()
                            .unwrap_or(Color::LightBlue),
                    )
                    .highlighted_str(&config.style_color_symbol.library_highlight_symbol)
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
                keys: config.keys.clone(),
            },

            Source::Playlist => Self {
                component: Table::default()
                    .borders(
                        Borders::default()
                            .color(
                                config
                                    .style_color_symbol
                                    .library_border()
                                    .unwrap_or(Color::Magenta),
                            )
                            .modifiers(BorderType::Rounded),
                    )
                    .background(
                        config
                            .style_color_symbol
                            .library_background()
                            .unwrap_or(Color::Reset),
                    )
                    .foreground(
                        config
                            .style_color_symbol
                            .library_foreground()
                            .unwrap_or(Color::Magenta),
                    )
                    .title(title_playlist, Alignment::Left)
                    .scroll(true)
                    .highlighted_color(
                        config
                            .style_color_symbol
                            .library_highlight()
                            .unwrap_or(Color::LightBlue),
                    )
                    .highlighted_str(&config.style_color_symbol.library_highlight_symbol)
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
                keys: config.keys.clone(),
            },
            Source::Database => Self {
                component: Table::default()
                    .borders(
                        Borders::default()
                            .color(
                                config
                                    .style_color_symbol
                                    .library_border()
                                    .unwrap_or(Color::Magenta),
                            )
                            .modifiers(BorderType::Rounded),
                    )
                    .background(
                        config
                            .style_color_symbol
                            .library_background()
                            .unwrap_or(Color::Reset),
                    )
                    .foreground(
                        config
                            .style_color_symbol
                            .library_foreground()
                            .unwrap_or(Color::Magenta),
                    )
                    .title(title_database, Alignment::Left)
                    .scroll(true)
                    .highlighted_color(
                        config
                            .style_color_symbol
                            .library_highlight()
                            .unwrap_or(Color::LightBlue),
                    )
                    .highlighted_str(&config.style_color_symbol.library_highlight_symbol)
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
                keys: config.keys.clone(),
            },
        }
    }
}

impl Component<Msg, NoUserEvent> for GSTablePopup {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        let _cmd_result = match ev {
            Event::Keyboard(KeyEvent { code: Key::Esc, .. }) => {
                return Some(Msg::GeneralSearch(GSMsg::PopupCloseCancel))
            }
            Event::Keyboard(keyevent) if keyevent == self.keys.global_quit.key_event() => {
                return Some(Msg::GeneralSearch(GSMsg::PopupCloseCancel))
            }

            Event::Keyboard(KeyEvent { code: Key::Up, .. }) => {
                self.perform(Cmd::Move(Direction::Up))
            }
            Event::Keyboard(KeyEvent {
                code: Key::Down, ..
            }) => self.perform(Cmd::Move(Direction::Down)),

            Event::Keyboard(keyevent) if keyevent == self.keys.global_down.key_event() => {
                self.perform(Cmd::Move(Direction::Down))
            }

            Event::Keyboard(keyevent) if keyevent == self.keys.global_up.key_event() => {
                self.perform(Cmd::Move(Direction::Up))
            }
            Event::Keyboard(KeyEvent {
                code: Key::PageDown,
                ..
            }) => self.perform(Cmd::Scroll(Direction::Down)),
            Event::Keyboard(KeyEvent {
                code: Key::PageUp, ..
            }) => self.perform(Cmd::Scroll(Direction::Up)),
            Event::Keyboard(keyevent) if keyevent == self.keys.global_goto_top.key_event() => {
                self.perform(Cmd::GoTo(Position::Begin))
            }
            Event::Keyboard(keyevent) if keyevent == self.keys.global_goto_bottom.key_event() => {
                self.perform(Cmd::GoTo(Position::End))
            }
            Event::Keyboard(KeyEvent { code: Key::End, .. }) => {
                self.perform(Cmd::GoTo(Position::End))
            }
            Event::Keyboard(KeyEvent { code: Key::Tab, .. }) => {
                return Some(Msg::GeneralSearch(GSMsg::TableBlur))
            }

            Event::Keyboard(keyevent) if keyevent == self.keys.global_right.key_event() => {
                match self.source {
                    Source::Library => {
                        return Some(Msg::GeneralSearch(GSMsg::PopupCloseLibraryAddPlaylist))
                    }
                    Source::Playlist => {
                        return Some(Msg::GeneralSearch(GSMsg::PopupClosePlaylistPlaySelected))
                    }
                    Source::Database => {
                        return Some(Msg::GeneralSearch(GSMsg::PopupCloseDatabaseAddPlaylist))
                    }
                }
            }
            Event::Keyboard(KeyEvent {
                code: Key::Enter, ..
            }) => match self.source {
                Source::Library => {
                    return Some(Msg::GeneralSearch(GSMsg::PopupCloseOkLibraryLocate))
                }
                Source::Playlist => {
                    return Some(Msg::GeneralSearch(GSMsg::PopupCloseOkPlaylistLocate))
                }
                Source::Database => return Some(Msg::GeneralSearch(GSMsg::PopupCloseCancel)),
            },
            _ => CmdResult::None,
        };
        Some(Msg::None)
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
        if let Ok(State::One(StateValue::Usize(index))) = self.app.state(&Id::GeneralSearchTable) {
            if let Ok(Some(AttrValue::Table(table))) =
                self.app.query(&Id::GeneralSearchTable, Attribute::Content)
            {
                if let Some(line) = table.get(index) {
                    if let Some(text_span) = line.get(1) {
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
                }
            }
        }
    }

    pub fn general_search_after_library_add_playlist(&mut self) {
        if let Ok(State::One(StateValue::Usize(index))) = self.app.state(&Id::GeneralSearchTable) {
            if let Ok(Some(AttrValue::Table(table))) =
                self.app.query(&Id::GeneralSearchTable, Attribute::Content)
            {
                if let Some(line) = table.get(index) {
                    if let Some(text_span) = line.get(1) {
                        let text = &text_span.content;
                        self.playlist_add(text);
                    }
                }
            }
        }
    }

    pub fn general_search_after_playlist_select(&mut self) {
        let mut index = 0;
        let mut matched = false;
        if let Ok(State::One(StateValue::Usize(result_index))) =
            self.app.state(&Id::GeneralSearchTable)
        {
            if let Ok(Some(AttrValue::Table(table))) =
                self.app.query(&Id::GeneralSearchTable, Attribute::Content)
            {
                if let Some(line) = table.get(result_index) {
                    if let Some(file_name_text_span) = line.get(3) {
                        let file_name = &file_name_text_span.content;
                        for (idx, item) in self.player.playlist.tracks.iter().enumerate() {
                            if item.file() == Some(file_name) {
                                index = idx;
                                matched = true;
                            }
                        }
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
        if let Ok(State::One(StateValue::Usize(result_index))) =
            self.app.state(&Id::GeneralSearchTable)
        {
            if let Ok(Some(AttrValue::Table(table))) =
                self.app.query(&Id::GeneralSearchTable, Attribute::Content)
            {
                if let Some(line) = table.get(result_index) {
                    if let Some(file_name_text_span) = line.get(3) {
                        let file_name = &file_name_text_span.content;
                        for (idx, item) in self.player.playlist.tracks.iter().enumerate() {
                            if item.file() == Some(file_name) {
                                index = idx;
                                matched = true;
                            }
                        }
                    }
                }
            }
        }
        if !matched {
            return;
        }
        self.playlist_play_selected(index);
    }

    pub fn general_search_after_database_add_playlist(&mut self) -> Result<()> {
        if let Ok(State::One(StateValue::Usize(index))) = self.app.state(&Id::GeneralSearchTable) {
            if let Ok(Some(AttrValue::Table(table))) =
                self.app.query(&Id::GeneralSearchTable, Attribute::Content)
            {
                let line = table
                    .get(index)
                    .ok_or_else(|| anyhow!("error getting index from table"))?;
                let text_span = line
                    .get(3)
                    .ok_or_else(|| anyhow!("error getting text span"))?;
                self.playlist_add(&text_span.content);
            }
        }
        Ok(())
    }
}
