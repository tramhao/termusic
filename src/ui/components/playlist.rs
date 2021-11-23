// use crate::song::Song;
use crate::{
    config::get_app_config_path,
    song::Song,
    ui::{Id, Model, Msg},
};

use anyhow::Result;
use humantime::format_duration;
use rand::seq::SliceRandom;
use rand::thread_rng;
use std::collections::VecDeque;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::str::FromStr;
use std::time::Duration;
use tui_realm_stdlib::Table;
use tuirealm::command::{Cmd, CmdResult, Direction, Position};
use tuirealm::props::{Alignment, BorderType, Borders, Color, TableBuilder, TextSpan};
use tuirealm::{
    event::{Key, KeyEvent, KeyModifiers},
    Component, Event, MockComponent, NoUserEvent, State, StateValue,
};

#[derive(MockComponent)]
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
                .background(Color::Black)
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

impl Component<Msg, NoUserEvent> for Playlist {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        let _drop = match ev {
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
                code: Key::Home, ..
            }) => self.perform(Cmd::GoTo(Position::Begin)),
            Event::Keyboard(KeyEvent { code: Key::End, .. }) => {
                self.perform(Cmd::GoTo(Position::End))
            }
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

            _ => CmdResult::None,
        };
        Some(Msg::None)
    }
}

impl Model {
    pub fn add_playlist(&mut self, current_node: &str) -> Result<()> {
        match Song::from_str(current_node) {
            Ok(item) => {
                self.playlist_items.push_back(item);
                self.sync_playlist();
            }
            Err(e) => return Err(e),
        }
        Ok(())
    }

    pub fn sync_playlist(&mut self) {
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

        self.update_title_playlist();
    }
    pub fn delete_item_playlist(&mut self, index: usize) {
        if self.playlist_items.is_empty() {}
        self.playlist_items.remove(index);
        self.sync_playlist();
    }

    pub fn empty_playlist(&mut self) {
        self.playlist_items.clear();
        self.sync_playlist();
        // self.view.active(COMPONENT_TREEVIEW_LIBRARY);
    }

    pub fn save_playlist(&mut self) -> Result<()> {
        let mut path = get_app_config_path()?;
        path.push("playlist.log");
        let mut file = File::create(path.as_path())?;
        for i in self.playlist_items.iter().rev() {
            if let Some(f) = i.file() {
                writeln!(&mut file, "{}", f)?;
            }
        }

        Ok(())
    }

    pub fn load_playlist(&mut self) -> Result<()> {
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

        // let tx = self.sender_playlist_items.clone();

        // thread::spawn(move || {
        //     let mut playlist_items = VecDeque::new();
        //     for line in &lines {
        //         if let Ok(s) = Song::from_str(line) {
        //             playlist_items.push_back(s);
        //         };
        //     }
        //     tx.send(playlist_items).ok();
        // });

        let mut playlist_items = VecDeque::new();
        for line in &lines {
            if let Ok(s) = Song::from_str(line) {
                playlist_items.push_back(s);
            };
        }

        self.playlist_items = playlist_items;
        Ok(())
    }

    pub fn shuffle(&mut self) {
        let mut rng = thread_rng();
        self.playlist_items.make_contiguous().shuffle(&mut rng);
        self.sync_playlist();
    }

    pub fn update_item_delete(&mut self) {
        self.playlist_items.retain(|x| {
            x.file().map_or(false, |p| {
                let path = Path::new(p);
                path.exists()
            })
        });

        self.sync_playlist();
        self.app.active(&Id::Library).ok();
    }
    fn update_title_playlist(&mut self) {
        let mut duration = Duration::from_secs(0);
        for v in &self.playlist_items {
            duration += v.duration();
        }

        let title = format!(
            "\u{2500} Playlist \u{2500}\u{2500}\u{2500}\u{2524} Total {} songs | {} |  Loop mode: {}  \u{251c}\u{2500}",
            self.playlist_items.len(),
            format_duration(Duration::new(duration.as_secs(), 0)),
            "Random"
            // self.config.loop_mode,
        );
        self.app
            .attr(
                &Id::Playlist,
                tuirealm::Attribute::Title,
                tuirealm::AttrValue::Title((title, Alignment::Left)),
            )
            .ok();

        // if let Some(props) = self.view.get_props(COMPONENT_TABLE_PLAYLIST) {
        //     let props = TablePropsBuilder::from(props)
        //         .with_title(title, tuirealm::tui::layout::Alignment::Left)
        //         .build();
        //     let msg = self.view.update(COMPONENT_TABLE_PLAYLIST, props);
        //     self.update(&msg);
        // }
    }
}
