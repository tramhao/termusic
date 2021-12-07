// use crate::song::Song;
use crate::{
    config::get_app_config_path,
    song::Song,
    ui::{Id, Model, Msg},
};

use crate::ui::Loop;
use anyhow::Result;
use humantime::format_duration;
use rand::seq::SliceRandom;
use rand::thread_rng;
use std::collections::VecDeque;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::str::FromStr;
use std::thread;
use std::time::Duration;
use tui_realm_stdlib::Table;
use tuirealm::command::{Cmd, CmdResult, Direction, Position};
use tuirealm::props::{Alignment, BorderType, TableBuilder, TextSpan};
use tuirealm::{
    event::{Key, KeyEvent, KeyModifiers},
    Component, Event, MockComponent, NoUserEvent, State, StateValue,
};

use tuirealm::props::{AttrValue, Attribute, Borders, Color, Style, TextModifiers};
use tuirealm::tui::{
    layout::{Constraint, Rect},
    text::Span,
    widgets::{Cell, Row, Table as TuiTable, TableState},
};
use tuirealm::Frame;

pub struct Playlist {
    component: Table,
}

impl Default for Playlist {
    fn default() -> Self {
        Self {
            component: Table::default()
                .borders(
                    Borders::default()
                        .modifiers(BorderType::Thick)
                        .color(Color::Blue),
                )
                // .foreground(Color::Yellow)
                .background(Color::Reset)
                .title("Playlist", Alignment::Left)
                .scroll(true)
                .highlighted_color(Color::LightBlue)
                .highlighted_str("\u{1f680}")
                // .highlighted_str("🚀")
                .rewind(true)
                .step(4)
                .row_height(1)
                .headers(&["Duration", "Artist", "Title", "Album"])
                .column_spacing(3)
                .widths(&[10, 20, 25, 45])
                .table(
                    TableBuilder::default()
                        .add_col(TextSpan::from("Empty"))
                        .add_col(TextSpan::from("Empty Queue"))
                        .add_col(TextSpan::from("Empty"))
                        .build(),
                ),
        }
    }
}
impl MockComponent for Playlist {
    fn view(&mut self, render: &mut Frame, area: Rect) {
        self.component.view(render, area);
    }

    fn query(&self, attr: Attribute) -> Option<AttrValue> {
        self.component.query(attr)
    }

    fn attr(&mut self, attr: Attribute, value: AttrValue) {
        self.component.attr(attr, value);
    }

    fn state(&self) -> State {
        self.component.state()
    }

    fn perform(&mut self, cmd: Cmd) -> CmdResult {
        match cmd {
            // Cmd::Move(Direction::Down) => {
            //     let prev = self.states.list_index;
            //     self.states.incr_list_index(self.rewindable());
            //     if prev != self.states.list_index {
            //         CmdResult::Changed(self.state())
            //     } else {
            //         CmdResult::None
            //     }
            // }
            // Cmd::Move(Direction::Up) => {
            //     let prev = self.states.list_index;
            //     self.states.decr_list_index(self.rewindable());
            //     if prev != self.states.list_index {
            //         CmdResult::Changed(self.state())
            //     } else {
            //         CmdResult::None
            //     }
            // }
            // Cmd::Scroll(Direction::Down) => {
            //     let prev = self.states.list_index;
            //     let step = self
            //         .props
            //         .get_or(Attribute::ScrollStep, AttrValue::Length(8))
            //         .unwrap_length();
            //     let step: usize = self.states.calc_max_step_ahead(step);
            //     (0..step).for_each(|_| self.states.incr_list_index(false));
            //     if prev != self.states.list_index {
            //         CmdResult::Changed(self.state())
            //     } else {
            //         CmdResult::None
            //     }
            // }
            // Cmd::Scroll(Direction::Up) => {
            //     let prev = self.states.list_index;
            //     let step = self
            //         .props
            //         .get_or(Attribute::ScrollStep, AttrValue::Length(8))
            //         .unwrap_length();
            //     let step: usize = self.states.calc_max_step_behind(step);
            //     (0..step).for_each(|_| self.states.decr_list_index(false));
            //     if prev != self.states.list_index {
            //         CmdResult::Changed(self.state())
            //     } else {
            //         CmdResult::None
            //     }
            // }
            // Cmd::GoTo(Position::Begin) => {
            //     let prev = self.states.list_index;
            //     self.states.list_index_at_first();
            //     if prev != self.states.list_index {
            //         CmdResult::Changed(self.state())
            //     } else {
            //         CmdResult::None
            //     }
            // }
            // Cmd::GoTo(Position::End) => {
            //     let prev = self.states.list_index;
            //     self.states.list_index_at_last();
            //     if prev != self.states.list_index {
            //         CmdResult::Changed(self.state())
            //     } else {
            //         CmdResult::None
            //     }
            // }
            Cmd::GoTo(Position::At(index)) => {
                let prev = self.component.states.list_index;
                self.component.states.list_index = index;
                if prev == self.component.states.list_index {
                    CmdResult::None
                } else {
                    CmdResult::Changed(self.state())
                }
            }
            _ => self.component.perform(cmd),
        }
    }
}

impl Component<Msg, NoUserEvent> for Playlist {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        let _cmd_result = match ev {
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
                return Some(Msg::PlaylistTableBlur)
            }
            Event::Keyboard(KeyEvent {
                code: Key::Char('d'),
                ..
            }) => match self.component.state() {
                State::One(StateValue::Usize(index_selected)) => {
                    return Some(Msg::PlaylistDelete(index_selected))
                }
                _ => return Some(Msg::None),
            },
            Event::Keyboard(KeyEvent {
                code: Key::Char('D'),
                modifiers: KeyModifiers::SHIFT,
            }) => return Some(Msg::PlaylistDeleteAll),
            Event::Keyboard(KeyEvent {
                code: Key::Char('s'),
                ..
            }) => return Some(Msg::PlaylistShuffle),
            Event::Keyboard(KeyEvent {
                code: Key::Char('m'),
                ..
            }) => return Some(Msg::PlaylistLoopModeCycle),
            Event::Keyboard(KeyEvent {
                code: Key::Char('l'),
                ..
            }) => {
                if let State::One(StateValue::Usize(index)) = self.state() {
                    return Some(Msg::PlaylistPlaySelected(index));
                }
                CmdResult::None
            }
            Event::Keyboard(KeyEvent {
                code: Key::Char('a'),
                ..
            }) => return Some(Msg::PlaylistAddFront),
            Event::Keyboard(KeyEvent {
                code: Key::Char('/'),
                ..
            }) => return Some(Msg::GeneralSearchPopupShowPlaylist),
            _ => CmdResult::None,
        };
        // match cmd_result {
        // CmdResult::Submit(State::One(StateValue::Usize(_index))) => {
        //     return Some(Msg::PlaylistPlaySelected);
        // }
        //_ =>
        Some(Msg::None)
        // }
    }
}

impl Model {
    pub fn playlist_add_item(
        &mut self,
        current_node: &str,
        add_playlist_front: bool,
    ) -> Result<()> {
        match Song::from_str(current_node) {
            Ok(item) => {
                if add_playlist_front {
                    self.playlist_items.push_front(item);
                } else {
                    self.playlist_items.push_back(item);
                }
                self.playlist_sync();
            }
            Err(e) => return Err(e),
        }
        Ok(())
    }
    pub fn playlist_add(&mut self, current_node: &str) {
        let p: &Path = Path::new(&current_node);
        if p.exists() {
            if p.is_dir() {
                let new_items = Self::library_dir_children(p);
                for s in &new_items {
                    if let Err(e) = self.playlist_add_item(s, false) {
                        self.mount_error_popup(format!("Add playlist error: {}", e).as_str());
                    }
                }
            } else if let Err(e) =
                self.playlist_add_item(current_node, self.config.add_playlist_front)
            {
                self.mount_error_popup(format!("Add Playlist error: {}", e).as_str());
            }
        }
    }

    pub fn playlist_sync(&mut self) {
        let mut table: TableBuilder = TableBuilder::default();

        for (idx, record) in self.playlist_items.iter().enumerate() {
            if idx > 0 {
                table.add_row();
            }

            let duration = record.duration_formatted().to_string();
            let duration_string = format!("[{:^6.6}]", duration);

            let noname_string = "No Name".to_string();
            let name = record.name().unwrap_or(&noname_string);
            let artist = record.artist().unwrap_or(name);
            let title = record.title().unwrap_or("Unknown Title");

            table
                .add_col(TextSpan::new(duration_string.as_str()))
                .add_col(TextSpan::new(artist).fg(tuirealm::tui::style::Color::LightYellow))
                .add_col(TextSpan::new(title).bold())
                .add_col(TextSpan::new(record.album().unwrap_or("Unknown Album")));
        }
        if self.playlist_items.is_empty() {
            table.add_col(TextSpan::from("0"));
            table.add_col(TextSpan::from("empty playlist"));
            table.add_col(TextSpan::from(""));
            table.add_col(TextSpan::from(""));
        }

        let table = table.build();
        self.app
            .attr(
                &Id::Playlist,
                tuirealm::Attribute::Content,
                tuirealm::AttrValue::Table(table),
            )
            .ok();

        self.playlist_update_title();
    }
    pub fn playlist_delete_item(&mut self, index: usize) {
        if self.playlist_items.is_empty() {}
        self.playlist_items.remove(index);
        self.playlist_sync();
    }

    pub fn playlist_empty(&mut self) {
        self.playlist_items.clear();
        self.playlist_sync();
        self.app.active(&Id::Library).ok();
    }

    pub fn playlist_save(&mut self) -> Result<()> {
        let mut path = get_app_config_path()?;
        path.push("playlist.log");
        let mut file = File::create(path.as_path())?;
        for i in &self.playlist_items {
            if let Some(f) = i.file() {
                writeln!(&mut file, "{}", f)?;
            }
        }

        Ok(())
    }

    pub fn playlist_load(&mut self) -> Result<()> {
        let mut path = get_app_config_path()?;
        path.push("playlist.log");

        let file = if let Ok(f) = File::open(path.as_path()) {
            f
        } else {
            File::create(path.as_path())?;
            File::open(path)?
        };
        let reader = BufReader::new(file);
        let lines: Vec<_> = reader
            .lines()
            .map(|line| line.unwrap_or_else(|_| "Error".to_string()))
            .collect();

        let tx = self.sender_playlist_items.clone();

        thread::spawn(move || {
            let mut playlist_items = VecDeque::new();
            for line in &lines {
                if let Ok(s) = Song::from_str(line) {
                    playlist_items.push_back(s);
                };
            }
            tx.send(playlist_items).ok();
        });

        // let mut playlist_items = VecDeque::new();
        // for line in &lines {
        //     if let Ok(s) = Song::from_str(line) {
        //         playlist_items.push_back(s);
        //     };
        // }

        // self.playlist_items = playlist_items;
        Ok(())
    }

    pub fn playlist_shuffle(&mut self) {
        let mut rng = thread_rng();
        self.playlist_items.make_contiguous().shuffle(&mut rng);
        self.playlist_sync();
    }

    pub fn playlist_update_library_delete(&mut self) {
        self.playlist_items.retain(|x| {
            x.file().map_or(false, |p| {
                let path = Path::new(p);
                path.exists()
            })
        });

        self.playlist_sync();
        // assert!(self.app.active(&Id::Library).is_ok());
    }
    pub fn playlist_update_title(&mut self) {
        let mut duration = Duration::from_secs(0);
        for v in &self.playlist_items {
            duration += v.duration();
        }
        let add_queue = if self.config.add_playlist_front {
            "\u{fb22}"
            // "ﬢ"
            // "front"
        } else {
            "\u{fb20}"
            // "ﬠ"
            // "back"
        };

        let title = format!(
            "\u{2500} Playlist \u{2500}\u{2500}\u{2524} Total {} songs | {} | Loop: {} | Add:{} \u{251c}\u{2500}",
            self.playlist_items.len(),
            format_duration(Duration::new(duration.as_secs(), 0)),
            self.config.loop_mode,
            add_queue
        );
        self.app
            .attr(
                &Id::Playlist,
                tuirealm::Attribute::Title,
                tuirealm::AttrValue::Title((title, Alignment::Left)),
            )
            .ok();
    }
    pub fn playlist_cycle_loop_mode(&mut self) {
        match self.config.loop_mode {
            Loop::Queue => {
                self.config.loop_mode = Loop::Playlist;
            }
            Loop::Playlist => {
                self.config.loop_mode = Loop::Single;
                if let Some(song) = self.playlist_items.pop_back() {
                    self.playlist_items.push_front(song);
                }
            }
            Loop::Single => {
                self.config.loop_mode = Loop::Queue;
                if let Some(song) = self.playlist_items.pop_front() {
                    self.playlist_items.push_back(song);
                }
            }
        };
        self.playlist_sync();
        self.playlist_update_title();
    }
    pub fn playlist_play_selected(&mut self, index: usize) {
        // self.time_pos = 0;
        if let Some(song) = self.playlist_items.remove(index) {
            self.playlist_items.push_front(song);
            self.playlist_sync();
            // self.status = Some(Status::Stopped);
            self.player_next();
        }
    }
    pub fn playlist_update_search(&mut self, input: &str) {
        let mut table: TableBuilder = TableBuilder::default();
        let mut idx = 0;
        let search = format!("*{}*", input.to_lowercase());
        for record in &self.playlist_items {
            // for record in all_items.into_iter().filter_map(std::result::Result::ok) {
            if let Some(file_name) = record.name() {
                if wildmatch::WildMatch::new(&search).matches(&file_name.to_string().to_lowercase())
                {
                    if idx > 0 {
                        table.add_row();
                    }
                    idx += 1;
                    table
                        .add_col(TextSpan::new(idx.to_string()))
                        .add_col(TextSpan::new(file_name.to_string()));
                }
            }
        }
        let table = table.build();

        self.general_search_update_library(table);
    }
}
