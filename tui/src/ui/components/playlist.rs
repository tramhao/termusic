use crate::ui::model::TermusicLayout;
use crate::ui::Model;
use anyhow::{anyhow, bail, Result};
use rand::seq::SliceRandom;
use std::path::Path;
use termusiclib::sqlite::SearchCriteria;
use termusiclib::sqlite::TrackForDB;
use termusiclib::types::{GSMsg, Id, Msg, PLMsg};
use termusiclib::utils::{filetype_supported, get_parent_folder, is_playlist, playlist_get_vec};
use termusiclib::{
    config::{Keys, Settings},
    track::Track,
};
use termusicplayback::PlayerCmd;

use tui_realm_stdlib::Table;
use tuirealm::props::{Alignment, BorderType, PropPayload, PropValue, TableBuilder, TextSpan};
use tuirealm::props::{Borders, Color};
use tuirealm::{
    command::{Cmd, CmdResult, Direction, Position},
    event::KeyModifiers,
};
use tuirealm::{
    event::{Key, KeyEvent, NoUserEvent},
    AttrValue, Attribute, Component, Event, MockComponent, State, StateValue,
};

#[derive(MockComponent)]
pub struct Playlist {
    component: Table,
    keys: Keys,
}

impl Playlist {
    pub fn new(config: &Settings) -> Self {
        Self {
            component: Table::default()
                .borders(
                    Borders::default().modifiers(BorderType::Rounded).color(
                        config
                            .style_color_symbol
                            .playlist_border()
                            .unwrap_or(Color::Blue),
                    ),
                )
                .background(
                    config
                        .style_color_symbol
                        .playlist_background()
                        .unwrap_or(Color::Reset),
                )
                .foreground(
                    config
                        .style_color_symbol
                        .playlist_foreground()
                        .unwrap_or(Color::Yellow),
                )
                .title(" Playlist ", Alignment::Left)
                .scroll(true)
                .highlighted_color(
                    config
                        .style_color_symbol
                        .playlist_highlight()
                        .unwrap_or(Color::LightBlue),
                )
                .highlighted_str(&config.style_color_symbol.playlist_highlight_symbol)
                .rewind(false)
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
            keys: config.keys.clone(),
        }
    }
}

impl Component<Msg, NoUserEvent> for Playlist {
    #[allow(clippy::too_many_lines)]
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        let _cmd_result = match ev {
            Event::Keyboard(KeyEvent {
                code: Key::Down,
                modifiers: KeyModifiers::NONE,
            }) => self.perform(Cmd::Move(Direction::Down)),
            Event::Keyboard(KeyEvent {
                code: Key::Up,
                modifiers: KeyModifiers::NONE,
            }) => self.perform(Cmd::Move(Direction::Up)),
            Event::Keyboard(key) if key == self.keys.global_down.key_event() => {
                self.perform(Cmd::Move(Direction::Down))
            }
            Event::Keyboard(key) if key == self.keys.global_up.key_event() => {
                self.perform(Cmd::Move(Direction::Up))
            }
            Event::Keyboard(KeyEvent {
                code: Key::PageDown,
                modifiers: KeyModifiers::NONE,
            }) => self.perform(Cmd::Scroll(Direction::Down)),
            Event::Keyboard(KeyEvent {
                code: Key::PageUp,
                modifiers: KeyModifiers::NONE,
            }) => self.perform(Cmd::Scroll(Direction::Up)),
            Event::Keyboard(key) if key == self.keys.global_goto_top.key_event() => {
                self.perform(Cmd::GoTo(Position::Begin))
            }
            Event::Keyboard(key) if key == self.keys.global_goto_bottom.key_event() => {
                self.perform(Cmd::GoTo(Position::End))
            }
            Event::Keyboard(KeyEvent {
                code: Key::Home,
                modifiers: KeyModifiers::NONE,
            }) => self.perform(Cmd::GoTo(Position::Begin)),
            Event::Keyboard(KeyEvent {
                code: Key::End,
                modifiers: KeyModifiers::NONE,
            }) => self.perform(Cmd::GoTo(Position::End)),
            Event::Keyboard(KeyEvent {
                code: Key::Tab,
                modifiers: KeyModifiers::NONE,
            }) => return Some(Msg::Playlist(PLMsg::PlaylistTableBlurDown)),
            Event::Keyboard(KeyEvent {
                code: Key::BackTab,
                modifiers: KeyModifiers::SHIFT,
            }) => return Some(Msg::Playlist(PLMsg::PlaylistTableBlurUp)),
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
            Event::Keyboard(key) if key == self.keys.playlist_search.key_event() => {
                return Some(Msg::GeneralSearch(GSMsg::PopupShowPlaylist))
            }
            Event::Keyboard(key) if key == self.keys.playlist_swap_down.key_event() => {
                match self.component.state() {
                    State::One(StateValue::Usize(index_selected)) => {
                        self.perform(Cmd::Move(Direction::Down));
                        return Some(Msg::Playlist(PLMsg::SwapDown(index_selected)));
                    }
                    _ => return Some(Msg::None),
                }
            }
            Event::Keyboard(key) if key == self.keys.playlist_swap_up.key_event() => {
                match self.component.state() {
                    State::One(StateValue::Usize(index_selected)) => {
                        self.perform(Cmd::Move(Direction::Up));
                        return Some(Msg::Playlist(PLMsg::SwapUp(index_selected)));
                    }
                    _ => return Some(Msg::None),
                }
            }
            Event::Keyboard(key) if key == self.keys.playlist_cmus_lqueue.key_event() => {
                return Some(Msg::Playlist(PLMsg::CmusLQueue));
            }
            Event::Keyboard(key) if key == self.keys.playlist_cmus_tqueue.key_event() => {
                return Some(Msg::Playlist(PLMsg::CmusTQueue));
            }
            _ => CmdResult::None,
        };
        Some(Msg::None)
    }
}

impl Model {
    pub fn playlist_reload(&mut self) {
        assert!(self
            .app
            .remount(
                Id::Playlist,
                Box::new(Playlist::new(&self.config)),
                Vec::new()
            )
            .is_ok());
        self.playlist_switch_layout();
        self.playlist_sync();
    }

    pub fn playlist_switch_layout(&mut self) {
        if self.layout == TermusicLayout::Podcast {
            let headers = &["Duration", "Episodes"];
            self.app
                .attr(
                    &Id::Playlist,
                    Attribute::Text,
                    AttrValue::Payload(PropPayload::Vec(
                        headers
                            .iter()
                            .map(|x| PropValue::Str((*x).to_string()))
                            // .map(|x| PropValue::Str(x.as_ref().to_string()))
                            .collect(),
                    )),
                )
                .ok();

            let widths = &[12, 88];
            self.app
                .attr(
                    &Id::Playlist,
                    Attribute::Width,
                    AttrValue::Payload(PropPayload::Vec(
                        widths.iter().map(|x| PropValue::U16(*x)).collect(),
                    )),
                )
                .ok();
            self.playlist_sync();
            return;
        }

        let headers = &["Duration", "Artist", "Title", "Album"];
        self.app
            .attr(
                &Id::Playlist,
                Attribute::Text,
                AttrValue::Payload(PropPayload::Vec(
                    headers
                        .iter()
                        .map(|x| PropValue::Str((*x).to_string()))
                        // .map(|x| PropValue::Str(x.as_ref().to_string()))
                        .collect(),
                )),
            )
            .ok();

        let widths = &[12, 20, 25, 43];
        self.app
            .attr(
                &Id::Playlist,
                Attribute::Width,
                AttrValue::Payload(PropPayload::Vec(
                    widths.iter().map(|x| PropValue::U16(*x)).collect(),
                )),
            )
            .ok();
        self.playlist_sync();
    }

    fn playlist_add_playlist(&mut self, current_node: &str) -> Result<()> {
        let vec = playlist_get_vec(current_node)?;
        let vec_str: Vec<&str> = vec.iter().map(std::convert::AsRef::as_ref).collect();
        self.playlist.add_playlist(vec_str)?;
        self.player_sync_playlist()?;
        self.playlist_sync();
        Ok(())
    }

    pub fn playlist_add_episode(&mut self, episode_index: usize) -> Result<()> {
        if self.podcasts.is_empty() {
            return Ok(());
        }
        let podcast_selected = self
            .podcasts
            .get(self.podcasts_index)
            .ok_or_else(|| anyhow!("get podcast selected failed."))?;
        let episode_selected = podcast_selected
            .episodes
            .get(episode_index)
            .ok_or_else(|| anyhow!("get episode selected failed."))?;
        self.playlist.add_episode(episode_selected);
        self.player_sync_playlist()?;
        self.playlist_sync();
        Ok(())
    }

    pub fn playlist_add(&mut self, current_node: &str) -> Result<()> {
        let p: &Path = Path::new(&current_node);
        if !p.exists() {
            return Ok(());
        }
        if p.is_dir() {
            let new_items_vec = Self::library_dir_children(p);
            let new_items_str_vec: Vec<&str> = new_items_vec
                .iter()
                .map(std::convert::AsRef::as_ref)
                .collect();
            self.playlist.add_playlist(new_items_str_vec)?;
            self.player_sync_playlist()?;
            self.playlist_sync();
            return Ok(());
        }
        self.playlist_add_item(current_node)?;
        self.playlist_sync();
        Ok(())
    }

    fn playlist_add_item(&mut self, current_node: &str) -> Result<()> {
        if is_playlist(current_node) {
            self.playlist_add_playlist(current_node)?;
            return Ok(());
        }
        let vec = vec![current_node];
        self.playlist.add_playlist(vec)?;
        self.player_sync_playlist()?;
        Ok(())
    }

    pub fn playlist_add_all_from_db(&mut self, vec: &[TrackForDB]) {
        let vec2: Vec<String> = vec.iter().map(|f| f.file.clone()).collect();
        let vec3: Vec<&str> = vec2.iter().map(std::convert::AsRef::as_ref).collect();
        if let Err(e) = self.playlist.add_playlist(vec3) {
            self.mount_error_popup(format!("Error add all from db: {e}"));
        }
        if let Err(e) = self.player_sync_playlist() {
            self.mount_error_popup(format!("sync playlist error: {e}"));
        }
        self.playlist_sync();
    }

    pub fn playlist_add_cmus_lqueue(&mut self) {
        let vec = self.playlist_get_records_for_cmus_lqueue(
            self.config.playlist_select_random_album_quantity,
        );
        self.playlist_add_all_from_db(&vec);
    }

    pub fn playlist_add_cmus_tqueue(&mut self) {
        let vec = self.playlist_get_records_for_cmus_tqueue(
            self.config.playlist_select_random_track_quantity,
        );
        self.playlist_add_all_from_db(&vec);
    }

    fn playlist_sync_podcasts(&mut self) {
        let mut table: TableBuilder = TableBuilder::default();

        for (idx, record) in self.playlist.tracks().iter().enumerate() {
            if idx > 0 {
                table.add_row();
            }

            let duration = record.duration_formatted().to_string();
            let duration_string = format!("[{duration:^7.7}]");

            let mut title = record.title().unwrap_or("Unknown Title").to_string();
            if record.podcast_localfile.is_some() {
                title = format!("[D] {title}");
            }
            if Some(idx) == self.playlist.get_current_track_index() {
                title = format!(
                    "{}{title}",
                    self.ce_style_color_symbol.currently_playing_track_symbol
                );
            };
            table
                .add_col(TextSpan::new(duration_string.as_str()))
                .add_col(TextSpan::new(title).bold());
        }
        if self.playlist.is_empty() {
            table.add_col(TextSpan::from("0"));
            table.add_col(TextSpan::from("empty playlist"));
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

    pub fn playlist_sync(&mut self) {
        if self.layout == TermusicLayout::Podcast {
            self.playlist_sync_podcasts();
            return;
        }

        let mut table: TableBuilder = TableBuilder::default();

        for (idx, record) in self.playlist.tracks().iter().enumerate() {
            if idx > 0 {
                table.add_row();
            }

            let duration = record.duration_formatted().to_string();
            let duration_string = format!("[{duration:^7.7}]");

            let noname_string = "No Name".to_string();
            let name = record.name().unwrap_or(&noname_string);
            let artist = record.artist().unwrap_or(name);
            let mut title = String::from(record.title().unwrap_or("Unknown Title"));
            let album = record.album().unwrap_or("Unknown Album");

            if Some(idx) == self.playlist.get_current_track_index() {
                title = format!(
                    "{}{title}",
                    self.ce_style_color_symbol.currently_playing_track_symbol
                );
            };

            table
                .add_col(TextSpan::new(duration_string.as_str()))
                .add_col(TextSpan::new(artist).fg(tuirealm::tui::style::Color::LightYellow))
                .add_col(TextSpan::new(title).bold())
                .add_col(TextSpan::new(album));
        }
        if self.playlist.is_empty() {
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
        if self.playlist.is_empty() {
            return;
        }
        self.playlist.remove(index);
        if let Err(e) = self.player_sync_playlist() {
            self.mount_error_popup(format!("sync playlist error: {e}"));
        }
        self.playlist_sync();
    }

    pub fn playlist_clear(&mut self) {
        self.playlist.clear();
        if let Err(e) = self.player_sync_playlist() {
            self.mount_error_popup(format!("sync playlist error: {e}"));
        }
        self.playlist_sync();
    }

    pub fn playlist_shuffle(&mut self) {
        self.playlist.shuffle();
        if let Err(e) = self.player_sync_playlist() {
            self.mount_error_popup(format!("sync playlist error: {e}"));
        }
        self.playlist_sync();
    }

    pub fn playlist_update_library_delete(&mut self) {
        self.playlist.remove_deleted_items();
        if let Err(e) = self.player_sync_playlist() {
            self.mount_error_popup(format!("sync playlist error: {e}"));
        }
        self.playlist_sync();
    }

    pub fn playlist_update_title(&mut self) {
        let duration = self.playlist.tracks().iter().map(Track::duration).sum();
        let title = format!(
            "\u{2500} Playlist \u{2500}\u{2500}\u{2524} Total {} tracks | {} | Mode: {} \u{251c}\u{2500}",
            self.playlist.len(),
            Track::duration_formatted_short(&duration),
            self.config.player_loop_mode.display(self.config.playlist_display_symbol),
        );
        self.app
            .attr(
                &Id::Playlist,
                tuirealm::Attribute::Title,
                tuirealm::AttrValue::Title((title, Alignment::Left)),
            )
            .ok();
    }
    pub fn playlist_play_selected(&mut self, index: usize) {
        self.playlist.set_current_track_index(Some(index));
        if let Err(e) = self.player_sync_playlist() {
            self.mount_error_popup(format!("sync playlist error: {e}"));
        }
        self.command(&PlayerCmd::PlaySelected);
        // TODO: Could running playlist_sync after PlaySelected cause race conditions depending on whether
        // PlaySelected runs fast or slow? Check this.
        self.playlist_sync();
    }

    pub fn playlist_update_search(&mut self, input: &str) {
        let mut table: TableBuilder = TableBuilder::default();
        let mut idx = 0;
        let search = format!("*{}*", input.to_lowercase());
        for record in self.playlist.tracks() {
            let artist = record.artist().unwrap_or("Unknown artist");
            let title = record.title().unwrap_or("Unknown title");
            if wildmatch::WildMatch::new(&search).matches(&artist.to_lowercase())
                | wildmatch::WildMatch::new(&search).matches(&title.to_lowercase())
            {
                if idx > 0 {
                    table.add_row();
                }

                let duration = record.duration_formatted().to_string();
                let duration_string = format!("[{duration:^6.6}]");

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
        if self.playlist.is_empty() {
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

    pub fn playlist_get_records_for_cmus_tqueue(&mut self, quantity: u32) -> Vec<TrackForDB> {
        let mut result = vec![];
        if let Ok(vec) = self.db.get_all_records() {
            let mut i = 0;
            loop {
                if let Some(record) = vec.choose(&mut rand::thread_rng()) {
                    if record.title.contains("Unknown Title") {
                        continue;
                    }
                    if filetype_supported(&record.file) {
                        result.push(record.clone());
                        i += 1;
                        if i > quantity - 1 {
                            break;
                        }
                    }
                }
            }
        }
        result
    }

    pub fn playlist_get_records_for_cmus_lqueue(&mut self, quantity: u32) -> Vec<TrackForDB> {
        let mut result = vec![];
        if let Ok(vec) = self.db.get_all_records() {
            loop {
                if let Some(v) = vec.choose(&mut rand::thread_rng()) {
                    if v.album.contains("empty") {
                        continue;
                    }
                    if let Ok(mut vec2) = self
                        .db
                        .get_record_by_criteria(&v.album, &SearchCriteria::Album)
                    {
                        if vec2.len() < quantity as usize {
                            continue;
                        }
                        result.append(&mut vec2);
                        break;
                    }
                }
            }
        }
        result
    }

    pub fn playlist_save_m3u_before(&mut self, filename: &str) -> Result<()> {
        let current_node: String = match self.app.state(&Id::Library).ok().unwrap() {
            State::One(StateValue::String(id)) => id,
            _ => bail!("Invalid node selected in library"),
        };

        let parent_folder = get_parent_folder(&current_node);

        let full_filename = format!("{parent_folder}/{filename}.m3u");

        let path_m3u = Path::new(&full_filename);

        if path_m3u.exists() {
            self.mount_save_playlist_confirm(&full_filename);
            return Ok(());
        }

        self.playlist_save_m3u(&full_filename)
    }

    pub fn playlist_save_m3u(&mut self, filename: &str) -> Result<()> {
        self.playlist.save_m3u(filename)?;

        self.library_reload_with_node_focus(Some(filename));

        Ok(())
    }
}
