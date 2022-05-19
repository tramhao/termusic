use crate::config::{Keys, Termusic};
use crate::ui::{DBMsg, Id, Model, Msg};
// use anyhow::Result;
// use rand::seq::SliceRandom;
// use rand::thread_rng;
// use std::collections::VecDeque;
// use std::fs::File;
// use std::io::{BufRead, BufReader, Write};
// use std::path::{Path, PathBuf};
// use std::thread;
// use std::time::Duration;
use tui_realm_stdlib::List;
use tuirealm::command::{Cmd, CmdResult, Direction, Position};
use tuirealm::props::{Alignment, BorderType, TableBuilder, TextSpan};
use tuirealm::{
    event::{Key, KeyEvent, KeyModifiers, NoUserEvent},
    AttrValue, Attribute, Component, Event, MockComponent, State, StateValue,
};

use tuirealm::props::{Borders, Color};

#[derive(MockComponent)]
pub struct DBListCriteria {
    component: List,
    on_key_tab: Msg,
    on_key_backtab: Msg,
    keys: Keys,
}

impl DBListCriteria {
    pub fn new(config: &Termusic, on_key_tab: Msg, on_key_backtab: Msg) -> Self {
        Self {
            component: List::default()
                .borders(
                    Borders::default().modifiers(BorderType::Rounded).color(
                        config
                            .style_color_symbol
                            .library_border()
                            .unwrap_or(Color::Blue),
                    ),
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
                        .unwrap_or(Color::Yellow),
                )
                .title("DataBase", Alignment::Left)
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
                .scroll(true)
                .rows(
                    TableBuilder::default()
                        .add_col(TextSpan::from("Artist"))
                        .add_row()
                        .add_col(TextSpan::from("Album"))
                        .add_row()
                        .add_col(TextSpan::from("Genre"))
                        .add_row()
                        .add_col(TextSpan::from("Directory"))
                        .build(),
                ),
            on_key_tab,
            on_key_backtab,
            keys: config.keys.clone(),
        }
    }
}

impl Component<Msg, NoUserEvent> for DBListCriteria {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        let _cmd_result = match ev {
            Event::Keyboard(KeyEvent {
                code: Key::Down, ..
            }) => {
                if let Some(AttrValue::Table(t)) = self.query(Attribute::Content) {
                    if let State::One(StateValue::Usize(index)) = self.state() {
                        if index >= t.len() - 1 {
                            return Some(self.on_key_tab.clone());
                        }
                    }
                }
                self.perform(Cmd::Move(Direction::Down))
            }
            Event::Keyboard(KeyEvent { code: Key::Up, .. }) => {
                self.perform(Cmd::Move(Direction::Up))
            }
            Event::Keyboard(key) if key == self.keys.global_down.key_event() => {
                if let Some(AttrValue::Table(t)) = self.query(Attribute::Content) {
                    if let State::One(StateValue::Usize(index)) = self.state() {
                        if index >= t.len() - 1 {
                            return Some(self.on_key_tab.clone());
                        }
                    }
                }
                self.perform(Cmd::Move(Direction::Down))
            }
            Event::Keyboard(key) if key == self.keys.global_up.key_event() => {
                self.perform(Cmd::Move(Direction::Up))
            }
            Event::Keyboard(KeyEvent {
                code: Key::PageDown,
                ..
            }) => self.perform(Cmd::Scroll(Direction::Down)),
            Event::Keyboard(KeyEvent {
                code: Key::PageUp, ..
            }) => self.perform(Cmd::Scroll(Direction::Up)),
            Event::Keyboard(key) if key == self.keys.global_goto_top.key_event() => {
                self.perform(Cmd::GoTo(Position::Begin))
            }
            Event::Keyboard(key) if key == self.keys.global_goto_bottom.key_event() => {
                self.perform(Cmd::GoTo(Position::End))
            }
            Event::Keyboard(KeyEvent {
                code: Key::Home, ..
            }) => self.perform(Cmd::GoTo(Position::Begin)),

            Event::Keyboard(KeyEvent {
                code: Key::Enter, ..
            }) => {
                if let State::One(StateValue::Usize(index)) = self.state() {
                    return Some(Msg::DataBase(DBMsg::SearchResult(index)));
                }
                CmdResult::None
            }
            Event::Keyboard(KeyEvent { code: Key::End, .. }) => {
                self.perform(Cmd::GoTo(Position::End))
            }
            Event::Keyboard(KeyEvent { code: Key::Tab, .. }) => {
                return Some(Msg::DataBase(DBMsg::SearchTracksBlurDown))
            }
            Event::Keyboard(KeyEvent {
                code: Key::BackTab,
                modifiers: KeyModifiers::SHIFT,
            }) => return Some(self.on_key_backtab.clone()),
            _ => CmdResult::None,
        };
        Some(Msg::None)
    }
}

#[derive(MockComponent)]
pub struct DBListSearchResult {
    component: List,
    on_key_tab: Msg,
    on_key_backtab: Msg,
    keys: Keys,
}

impl DBListSearchResult {
    pub fn new(config: &Termusic, on_key_tab: Msg, on_key_backtab: Msg) -> Self {
        Self {
            component: List::default()
                .borders(
                    Borders::default().modifiers(BorderType::Rounded).color(
                        config
                            .style_color_symbol
                            .library_border()
                            .unwrap_or(Color::Blue),
                    ),
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
                        .unwrap_or(Color::Yellow),
                )
                .title("Result", Alignment::Left)
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
                .scroll(true)
                .rows(
                    TableBuilder::default()
                        .add_col(TextSpan::from("empty"))
                        .build(),
                ),
            on_key_tab,
            on_key_backtab,
            keys: config.keys.clone(),
        }
    }
}

impl Component<Msg, NoUserEvent> for DBListSearchResult {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        let _cmd_result = match ev {
            Event::Keyboard(KeyEvent {
                code: Key::Down, ..
            }) => {
                if let Some(AttrValue::Table(t)) = self.query(Attribute::Content) {
                    if let State::One(StateValue::Usize(index)) = self.state() {
                        if index >= t.len() - 1 {
                            return Some(self.on_key_tab.clone());
                        }
                    }
                }
                self.perform(Cmd::Move(Direction::Down))
            }
            Event::Keyboard(KeyEvent { code: Key::Up, .. }) => {
                if let State::One(StateValue::Usize(index)) = self.state() {
                    if index == 0 {
                        return Some(self.on_key_backtab.clone());
                    }
                }
                self.perform(Cmd::Move(Direction::Up))
            }
            Event::Keyboard(key) if key == self.keys.global_down.key_event() => {
                if let Some(AttrValue::Table(t)) = self.query(Attribute::Content) {
                    if let State::One(StateValue::Usize(index)) = self.state() {
                        if index >= t.len() - 1 {
                            return Some(self.on_key_tab.clone());
                        }
                    }
                }
                self.perform(Cmd::Move(Direction::Down))
            }
            Event::Keyboard(key) if key == self.keys.global_up.key_event() => {
                if let State::One(StateValue::Usize(index)) = self.state() {
                    if index == 0 {
                        return Some(self.on_key_backtab.clone());
                    }
                }
                self.perform(Cmd::Move(Direction::Up))
            }
            Event::Keyboard(KeyEvent {
                code: Key::PageDown,
                ..
            }) => self.perform(Cmd::Scroll(Direction::Down)),
            Event::Keyboard(KeyEvent {
                code: Key::PageUp, ..
            }) => self.perform(Cmd::Scroll(Direction::Up)),
            Event::Keyboard(key) if key == self.keys.global_goto_top.key_event() => {
                self.perform(Cmd::GoTo(Position::Begin))
            }
            Event::Keyboard(key) if key == self.keys.global_goto_bottom.key_event() => {
                self.perform(Cmd::GoTo(Position::End))
            }
            Event::Keyboard(KeyEvent {
                code: Key::Home, ..
            }) => self.perform(Cmd::GoTo(Position::Begin)),
            Event::Keyboard(KeyEvent { code: Key::End, .. }) => {
                self.perform(Cmd::GoTo(Position::End))
            }

            Event::Keyboard(KeyEvent {
                code: Key::Enter, ..
            }) => {
                if let State::One(StateValue::Usize(index)) = self.state() {
                    return Some(Msg::DataBase(DBMsg::SearchTrack(index)));
                }
                CmdResult::None
            }
            Event::Keyboard(KeyEvent { code: Key::Tab, .. }) => {
                return Some(Msg::DataBase(DBMsg::SearchTracksBlurDown))
            }
            Event::Keyboard(KeyEvent {
                code: Key::BackTab,
                modifiers: KeyModifiers::SHIFT,
            }) => return Some(self.on_key_backtab.clone()),

            _ => CmdResult::None,
        };
        Some(Msg::None)
    }
}

#[derive(MockComponent)]
pub struct DBListSearchTracks {
    component: List,
    on_key_tab: Msg,
    on_key_backtab: Msg,
    keys: Keys,
}

impl DBListSearchTracks {
    pub fn new(config: &Termusic, on_key_tab: Msg, on_key_backtab: Msg) -> Self {
        Self {
            component: List::default()
                .borders(
                    Borders::default().modifiers(BorderType::Rounded).color(
                        config
                            .style_color_symbol
                            .library_border()
                            .unwrap_or(Color::Blue),
                    ),
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
                        .unwrap_or(Color::Yellow),
                )
                .title("Tracks", Alignment::Left)
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
                .scroll(true)
                .rows(
                    TableBuilder::default()
                        .add_col(TextSpan::from("empty"))
                        .build(),
                ),
            on_key_tab,
            on_key_backtab,
            keys: config.keys.clone(),
        }
    }
}

impl Component<Msg, NoUserEvent> for DBListSearchTracks {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        let _cmd_result = match ev {
            Event::Keyboard(KeyEvent {
                code: Key::Down, ..
            }) => self.perform(Cmd::Move(Direction::Down)),
            Event::Keyboard(KeyEvent { code: Key::Up, .. }) => {
                if let State::One(StateValue::Usize(index)) = self.state() {
                    if index == 0 {
                        return Some(self.on_key_backtab.clone());
                    }
                }
                self.perform(Cmd::Move(Direction::Up))
            }
            Event::Keyboard(key) if key == self.keys.global_down.key_event() => {
                self.perform(Cmd::Move(Direction::Down))
            }
            Event::Keyboard(key) if key == self.keys.global_up.key_event() => {
                if let State::One(StateValue::Usize(index)) = self.state() {
                    if index == 0 {
                        return Some(self.on_key_backtab.clone());
                    }
                }
                self.perform(Cmd::Move(Direction::Up))
            }
            Event::Keyboard(KeyEvent {
                code: Key::PageDown,
                ..
            }) => self.perform(Cmd::Scroll(Direction::Down)),
            Event::Keyboard(KeyEvent {
                code: Key::PageUp, ..
            }) => self.perform(Cmd::Scroll(Direction::Up)),
            Event::Keyboard(key) if key == self.keys.global_goto_top.key_event() => {
                self.perform(Cmd::GoTo(Position::Begin))
            }
            Event::Keyboard(key) if key == self.keys.global_goto_bottom.key_event() => {
                self.perform(Cmd::GoTo(Position::End))
            }
            Event::Keyboard(KeyEvent {
                code: Key::Home, ..
            }) => self.perform(Cmd::GoTo(Position::Begin)),
            Event::Keyboard(KeyEvent { code: Key::End, .. }) => {
                self.perform(Cmd::GoTo(Position::End))
            }
            Event::Keyboard(KeyEvent { code: Key::Tab, .. }) => {
                return Some(self.on_key_tab.clone())
            }

            Event::Keyboard(KeyEvent {
                code: Key::BackTab,
                modifiers: KeyModifiers::SHIFT,
            }) => return Some(self.on_key_backtab.clone()),

            Event::Keyboard(keyevent) if keyevent == self.keys.global_right.key_event() => {
                if let State::One(StateValue::Usize(index)) = self.state() {
                    return Some(Msg::DataBase(DBMsg::AddPlaylist(index)));
                }
                CmdResult::None
            }
            Event::Keyboard(KeyEvent {
                code: Key::Char('l'),
                modifiers: KeyModifiers::SHIFT,
            }) => return Some(Msg::DataBase(DBMsg::AddAllToPlaylist)),

            _ => CmdResult::None,
        };
        Some(Msg::None)
    }
}

impl Model {
    pub fn database_sync_tracks(&mut self) {
        let mut table: TableBuilder = TableBuilder::default();

        for (idx, record) in self.db_search_tracks.iter().enumerate() {
            if idx > 0 {
                table.add_row();
            }

            table
                .add_col(TextSpan::from(format!("{}", idx + 1)))
                .add_col(TextSpan::from(" "))
                .add_col(TextSpan::from(record.name.to_string()));
        }
        if self.db_search_results.is_empty() {
            table.add_col(TextSpan::from("empty results"));
        }

        let table = table.build();
        self.app
            .attr(
                &Id::DBListSearchTracks,
                tuirealm::Attribute::Content,
                tuirealm::AttrValue::Table(table),
            )
            .ok();

        // self.playlist_update_title();
    }
    pub fn database_sync_results(&mut self) {
        let mut table: TableBuilder = TableBuilder::default();

        for (idx, record) in self.db_search_results.iter().enumerate() {
            if idx > 0 {
                table.add_row();
            }

            table
                .add_col(TextSpan::from(format!("{}", idx + 1)))
                .add_col(TextSpan::from(" "))
                .add_col(TextSpan::from(record));
        }
        if self.db_search_results.is_empty() {
            table.add_col(TextSpan::from("empty results"));
        }

        let table = table.build();
        self.app
            .attr(
                &Id::DBListSearchResult,
                tuirealm::Attribute::Content,
                tuirealm::AttrValue::Table(table),
            )
            .ok();

        // self.playlist_update_title();
    }

    pub fn database_update_search_results(&mut self) {
        self.db_search_results = self.db.get_criterias(&self.db_criteria);
        // eprintln!("{:?}", self.db_search_results);
        self.database_sync_results();
        self.app.active(&Id::DBListSearchResult).ok();
    }

    pub fn database_update_search_tracks(&mut self, index: usize) {
        if let Ok(vec) = self
            .db
            .get_record_by_criteria(&self.db_search_results[index], &self.db_criteria)
        {
            self.db_search_tracks = vec;
        };
        self.database_sync_tracks();
        self.app.active(&Id::DBListSearchTracks).ok();
    }

    #[allow(unused)]
    pub fn database_reload(&mut self) {
        // keep focus
        let mut focus_database = false;
        if let Ok(f) = self.app.query(&Id::DBListCriteria, Attribute::Focus) {
            if Some(AttrValue::Flag(true)) == f {
                focus_database = true;
            }
        }

        if let Ok(f) = self.app.query(&Id::DBListSearchResult, Attribute::Focus) {
            if Some(AttrValue::Flag(true)) == f {
                focus_database = true;
            }
        }

        let mut focus_library = false;
        if let Ok(f) = self.app.query(&Id::Library, Attribute::Focus) {
            if Some(AttrValue::Flag(true)) == f {
                focus_library = true;
            }
        }

        assert!(self.app.umount(&Id::DBListCriteria).is_ok());
        assert!(self.app.umount(&Id::DBListSearchResult).is_ok());
        assert!(self.app.umount(&Id::DBListSearchTracks).is_ok());
        assert!(self
            .app
            .mount(
                Id::DBListCriteria,
                Box::new(DBListCriteria::new(
                    &self.config,
                    Msg::DataBase(DBMsg::CriteriaBlurDown),
                    Msg::DataBase(DBMsg::CriteriaBlurUp)
                )),
                Vec::new()
            )
            .is_ok());

        assert!(self
            .app
            .mount(
                Id::DBListSearchResult,
                Box::new(DBListSearchResult::new(
                    &self.config,
                    Msg::DataBase(DBMsg::SearchResultBlurDown),
                    Msg::DataBase(DBMsg::SearchResultBlurUp)
                )),
                Vec::new()
            )
            .is_ok());
        assert!(self
            .app
            .mount(
                Id::DBListSearchTracks,
                Box::new(DBListSearchTracks::new(
                    &self.config,
                    Msg::DataBase(DBMsg::SearchTracksBlurDown),
                    Msg::DataBase(DBMsg::SearchTracksBlurUp)
                )),
                Vec::new()
            )
            .is_ok());

        self.db_search_results = vec![];
        self.db_search_tracks = vec![];
        self.database_sync_results();
        self.database_sync_results();

        if focus_database {
            assert!(self.app.active(&Id::DBListCriteria).is_ok());
        }
    }
}
