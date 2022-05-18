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
    event::{Key, KeyEvent, NoUserEvent},
    AttrValue, Attribute, Component, Event, MockComponent, State, StateValue,
};

use crate::sqlite::SearchCriteria;
use tuirealm::props::{Borders, Color};

#[derive(MockComponent)]
pub struct DBListCriteria {
    component: List,
    keys: Keys,
}

impl DBListCriteria {
    pub fn new(config: &Termusic) -> Self {
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
            keys: config.keys.clone(),
        }
    }
}

impl Component<Msg, NoUserEvent> for DBListCriteria {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        let _cmd_result = match ev {
            Event::Keyboard(KeyEvent {
                code: Key::Down, ..
            }) => self.perform(Cmd::Move(Direction::Down)),
            Event::Keyboard(KeyEvent { code: Key::Up, .. }) => {
                self.perform(Cmd::Move(Direction::Up))
            }
            Event::Keyboard(key) if key == self.keys.global_down.key_event() => {
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
                return Some(Msg::DataBase(DBMsg::CriteriaBlur))
            }
            _ => CmdResult::None,
        };
        Some(Msg::None)
    }
}

#[derive(MockComponent)]
pub struct DBListSearchResult {
    component: List,
    keys: Keys,
}

impl DBListSearchResult {
    pub fn new(config: &Termusic) -> Self {
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
                        .add_col(TextSpan::from("Artist"))
                        .add_row()
                        .add_col(TextSpan::from("Album"))
                        .add_row()
                        .add_col(TextSpan::from("Genre"))
                        .build(),
                ),
            keys: config.keys.clone(),
        }
    }
}

impl Component<Msg, NoUserEvent> for DBListSearchResult {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        let _cmd_result = match ev {
            Event::Keyboard(KeyEvent {
                code: Key::Down, ..
            }) => self.perform(Cmd::Move(Direction::Down)),
            Event::Keyboard(KeyEvent { code: Key::Up, .. }) => {
                self.perform(Cmd::Move(Direction::Up))
            }
            Event::Keyboard(key) if key == self.keys.global_down.key_event() => {
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
                return Some(Msg::DataBase(DBMsg::SearchResultBlur))
            }
            _ => CmdResult::None,
        };
        Some(Msg::None)
    }
}

#[derive(MockComponent)]
pub struct DBListSearchTracks {
    component: List,
    keys: Keys,
}

impl DBListSearchTracks {
    pub fn new(config: &Termusic) -> Self {
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
                        .add_col(TextSpan::from("Artist"))
                        .add_row()
                        .add_col(TextSpan::from("Album"))
                        .add_row()
                        .add_col(TextSpan::from("Genre"))
                        .build(),
                ),
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
                self.perform(Cmd::Move(Direction::Up))
            }
            Event::Keyboard(key) if key == self.keys.global_down.key_event() => {
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
            Event::Keyboard(KeyEvent { code: Key::End, .. }) => {
                self.perform(Cmd::GoTo(Position::End))
            }
            Event::Keyboard(KeyEvent { code: Key::Tab, .. }) => {
                return Some(Msg::DataBase(DBMsg::SearchTracksBlur))
            }

            Event::Keyboard(keyevent) if keyevent == self.keys.global_right.key_event() => {
                if let State::One(StateValue::Usize(index)) = self.state() {
                    return Some(Msg::DataBase(DBMsg::AddPlaylist(index)));
                }
                CmdResult::None
                // let current_node = self.component.tree_state().selected().unwrap();
                // let p: &Path = Path::new(current_node);
                // if p.is_dir() {
                //     self.perform(Cmd::Custom(TREE_CMD_OPEN))
                // } else {
                //     return Some(Msg::Playlist(crate::ui::PLMsg::Add(
                //         current_node.to_string(),
                //     )));
                // }
            }
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

    pub fn database_update_search_results(&mut self, index: usize) {
        self.db_search_results = self.db.get_criterias(index);
        // eprintln!("{:?}", self.db_search_results);
        self.database_sync_results();
        self.app.active(&Id::DBListSearchResult).ok();
    }

    pub fn database_update_search_tracks(&mut self, index: usize) {
        // FIXME: index is wrong here
        let mut crit = SearchCriteria::from(index);

        if let Ok(State::One(StateValue::Usize(crit_index))) = self.app.state(&Id::DBListCriteria) {
            crit = SearchCriteria::from(crit_index);
        }
        if let Ok(vec) = self
            .db
            .get_record_by_criteria(&self.db_search_results[index], &crit)
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
                Id::Playlist,
                Box::new(DBListCriteria::new(&self.config)),
                Vec::new()
            )
            .is_ok());
        self.playlist_sync();
        if focus_database {
            assert!(self.app.active(&Id::DBListCriteria).is_ok());
            return;
        }

        if focus_library {
            return;
            // assert!(self.app.active(&Id::Library).is_ok());
        }

        assert!(self.app.active(&Id::Library).is_ok());
    }

    #[allow(unused)]
    pub fn database_add_to_playlist(&mut self) {}
}
