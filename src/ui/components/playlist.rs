// use crate::song::Song;
use crate::{
    config::get_app_config_path,
    song::Song,
    ui::{
        components::{Keys, StyleColorSymbol},
        GSMsg, Id, Loop, Model, Msg, PLMsg,
    },
};
use anyhow::Result;
use rand::seq::SliceRandom;
use rand::thread_rng;
use std::collections::VecDeque;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::thread;
use std::time::Duration;
use tui_realm_stdlib::Table;
use tuirealm::command::{Cmd, CmdResult, Direction, Position};
use tuirealm::props::{Alignment, BorderType, PropPayload, PropValue, TableBuilder, TextSpan};
use tuirealm::{
    event::{Key, KeyEvent, NoUserEvent},
    AttrValue, Attribute, Component, Event, MockComponent, State, StateValue,
};

use tuirealm::props::{Borders, Color};

#[derive(MockComponent)]
pub struct Playlist {
    component: Table,
    keys: Keys,
}

impl Playlist {
    pub fn new(color_mapping: &StyleColorSymbol, keys: &Keys) -> Self {
        Self {
            component: Table::default()
                .borders(
                    Borders::default()
                        .modifiers(BorderType::Rounded)
                        .color(color_mapping.playlist_border().unwrap_or(Color::Blue)),
                )
                .background(color_mapping.playlist_background().unwrap_or(Color::Reset))
                .foreground(color_mapping.playlist_foreground().unwrap_or(Color::Yellow))
                .title("Playlist", Alignment::Left)
                .scroll(true)
                .highlighted_color(
                    color_mapping
                        .playlist_highlight()
                        .unwrap_or(Color::LightBlue),
                )
                .highlighted_str(&color_mapping.playlist_highlight_symbol)
                // .highlighted_str("\u{1f680}")
                // .highlighted_str("🚀")
                .rewind(true)
                .step(4)
                .row_height(1)
                .headers(&["Duration", "Artist", "Title", "Album"])
                .column_spacing(2)
                .widths(&[12, 20, 25, 43])
                .table(
                    TableBuilder::default()
                        .add_col(TextSpan::from("Empty"))
                        .add_col(TextSpan::from("Empty Queue"))
                        .add_col(TextSpan::from("Empty"))
                        .build(),
                ),
            keys: keys.clone(),
        }
    }
}

impl Component<Msg, NoUserEvent> for Playlist {
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
                return Some(Msg::Playlist(PLMsg::TableBlur))
            }
            Event::Keyboard(key) if key == self.keys.playlist_delete.key_event() => {
                match self.component.state() {
                    State::One(StateValue::Usize(index_selected)) => {
                        return Some(Msg::Playlist(PLMsg::Delete(index_selected)))
                    }
                    _ => return Some(Msg::None),
                }
            }
            Event::Keyboard(key) if key == self.keys.playlist_delete_all.key_event() => {
                return Some(Msg::Playlist(PLMsg::DeleteAll))
            }
            Event::Keyboard(key) if key == self.keys.playlist_shuffle.key_event() => {
                return Some(Msg::Playlist(PLMsg::Shuffle))
            }
            Event::Keyboard(key) if key == self.keys.playlist_mode_cycle.key_event() => {
                return Some(Msg::Playlist(PLMsg::LoopModeCycle))
            }
            Event::Keyboard(key) if key == self.keys.playlist_play_selected.key_event() => {
                if let State::One(StateValue::Usize(index)) = self.state() {
                    return Some(Msg::Playlist(PLMsg::PlaySelected(index)));
                }
                CmdResult::None
            }
            Event::Keyboard(key) if key == self.keys.playlist_add_front.key_event() => {
                return Some(Msg::Playlist(PLMsg::AddFront))
            }
            Event::Keyboard(key) if key == self.keys.playlist_search.key_event() => {
                return Some(Msg::GeneralSearch(GSMsg::PopupShowPlaylist))
            }
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
    pub fn playlist_reload(&mut self) {
        // keep focus
        let mut focus_playlist = false;
        if let Ok(f) = self.app.query(&Id::Playlist, Attribute::Focus) {
            if Some(AttrValue::Flag(true)) == f {
                focus_playlist = true;
            }
        }

        let mut focus_library = false;
        if let Ok(f) = self.app.query(&Id::Library, Attribute::Focus) {
            if Some(AttrValue::Flag(true)) == f {
                focus_library = true;
            }
        }

        assert!(self.app.umount(&Id::Playlist).is_ok());
        assert!(self
            .app
            .mount(
                Id::Playlist,
                Box::new(Playlist::new(
                    &self.config.style_color_symbol,
                    &self.config.keys
                )),
                Vec::new()
            )
            .is_ok());
        self.playlist_sync();
        if focus_playlist {
            assert!(self.app.active(&Id::Playlist).is_ok());
            return;
        }

        if focus_library {
            return;
            // assert!(self.app.active(&Id::Library).is_ok());
        }

        assert!(self.app.active(&Id::Library).is_ok());
    }
    fn playlist_add_item(&mut self, current_node: &str, add_playlist_front: bool) -> Result<()> {
        match Song::read_from_path(current_node) {
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
    fn playlist_add_items(&mut self, p: &Path) {
        let new_items = Self::library_dir_children(p);
        let mut index = 0;
        for s in &new_items {
            if self.config.add_playlist_front {
                match Song::read_from_path(s) {
                    Ok(item) => {
                        self.playlist_items.insert(index, item);
                        index += 1;
                    }
                    Err(_e) => {
                        index -= 1;
                    }
                }
                continue;
            }

            self.playlist_add_item(s, false).ok();
        }
        self.playlist_sync();
    }
    pub fn playlist_add(&mut self, current_node: &str) {
        let p: &Path = Path::new(&current_node);
        if !p.exists() {
            return;
        }

        if p.is_dir() {
            self.playlist_add_items(p);
        } else if let Err(e) = self.playlist_add_item(current_node, self.config.add_playlist_front)
        {
            self.mount_error_popup(format!("Add Playlist error: {}", e).as_str());
        }
    }

    pub fn playlist_sync(&mut self) {
        let mut table: TableBuilder = TableBuilder::default();

        for (idx, record) in self.playlist_items.iter().enumerate() {
            if idx > 0 {
                table.add_row();
            }

            let duration = record.duration_formatted().to_string();
            let duration_string = format!("[{:^7.7}]", duration);

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
        // self.app.active(&Id::Library).ok();
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
                if let Ok(s) = Song::read_from_path(line) {
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
            if self.config.playlist_display_symbol {
                // "\u{1f51d}"
                "\u{fb22}"
                // "ﬢ"
            } else {
                "next"
            }
        } else if self.config.playlist_display_symbol {
            "\u{fb20}"
            // "ﬠ"
        } else {
            "last"
        };
        let title = format!(
            "\u{2500} Playlist \u{2500}\u{2500}\u{2524} Total {} tracks | {} | Loop: {} | Add:{} \u{251c}\u{2500}",
            self.playlist_items.len(),
            Song::duration_formatted_short(&duration),
            self.config.loop_mode.display(self.config.playlist_display_symbol),
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
            let artist = record.artist().unwrap_or("Unknown artist");
            let title = record.title().unwrap_or("Unknown title");
            if wildmatch::WildMatch::new(&search).matches(&artist.to_lowercase())
                | wildmatch::WildMatch::new(&search).matches(&title.to_lowercase())
            {
                if idx > 0 {
                    table.add_row();
                }

                let duration = record.duration_formatted().to_string();
                let duration_string = format!("[{:^6.6}]", duration);

                let noname_string = "No Name".to_string();
                let name = record.name().unwrap_or(&noname_string);
                let artist = record.artist().unwrap_or(name);
                let title = record.title().unwrap_or("Unknown Title");
                let file_name = record.file().unwrap_or("no file");

                table
                    .add_col(TextSpan::new(duration_string.as_str()))
                    .add_col(TextSpan::new(artist).fg(tuirealm::tui::style::Color::LightYellow))
                    .add_col(TextSpan::new(title).bold())
                    .add_col(TextSpan::new(file_name));
                // .add_col(TextSpan::new(record.album().unwrap_or("Unknown Album")));
                idx += 1;
            }
        }
        if self.playlist_items.is_empty() {
            table.add_col(TextSpan::from("0"));
            table.add_col(TextSpan::from("empty playlist"));
            table.add_col(TextSpan::from(""));
        }
        let table = table.build();

        self.general_search_update_show(table);
    }

    pub fn playlist_locate(&mut self, index: usize) {
        assert!(self
            .app
            .attr(
                &Id::Playlist,
                Attribute::Value,
                AttrValue::Payload(PropPayload::One(PropValue::Usize(index))),
            )
            .is_ok());
    }
}
