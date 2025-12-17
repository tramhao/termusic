use std::path::{Path, PathBuf};

use anyhow::{Result, anyhow, bail};
use termusiclib::config::{SharedTuiSettings, TuiOverlay};
use termusiclib::track::MediaTypes;
use tui_realm_stdlib::Table;
use tuirealm::command::{Cmd, CmdResult, Direction, Position};
use tuirealm::event::{Key, KeyEvent, KeyModifiers};
use tuirealm::props::{Alignment, BorderType, Borders, InputType, TableBuilder, TextSpan};
use tuirealm::{AttrValue, Attribute, Component, Event, MockComponent, State, StateValue};

use crate::ui::Model;
use crate::ui::components::vendored::tui_realm_stdlib_input::Input;
use crate::ui::ids::Id;
use crate::ui::model::UserEvent;
use crate::ui::msg::{GSMsg, Msg};

#[derive(MockComponent)]
pub struct GSInputPopup {
    component: Input,
    source: Source,
}

/// Get a [`Input`] component with the common style applied.
#[inline]
fn common_input_comp(config: &TuiOverlay, title: &str) -> Input {
    Input::default()
        .background(config.settings.theme.fallback_background())
        .foreground(config.settings.theme.fallback_foreground())
        .borders(
            Borders::default()
                .color(config.settings.theme.fallback_border())
                .modifiers(BorderType::Rounded),
        )
        .title(title, Alignment::Left)
}

impl GSInputPopup {
    pub fn new(source: Source, config: &TuiOverlay) -> Self {
        match source {
            Source::Episode => Self {
                component: common_input_comp(
                    config,
                    " Search for all episodes from all feeds: (support * and ?) ",
                )
                .input_type(InputType::Text),
                source,
            },
            _ => Self {
                component: common_input_comp(config, " Search for: (support * and ?) ")
                    .input_type(InputType::Text),
                source,
            },
        }
    }
}

impl Component<Msg, UserEvent> for GSInputPopup {
    fn on(&mut self, ev: Event<UserEvent>) -> Option<Msg> {
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
                return Some(Msg::GeneralSearch(GSMsg::InputBlur));
            }
            _ => CmdResult::None,
        };
        match cmd_result {
            CmdResult::Changed(State::One(StateValue::String(input_string))) => {
                match &self.source {
                    Source::Library(path) => Some(Msg::GeneralSearch(GSMsg::PopupUpdateLibrary(
                        input_string,
                        path.clone(),
                    ))),
                    Source::Playlist => {
                        Some(Msg::GeneralSearch(GSMsg::PopupUpdatePlaylist(input_string)))
                    }
                    Source::Database => {
                        Some(Msg::GeneralSearch(GSMsg::PopupUpdateDatabase(input_string)))
                    }
                    Source::Episode => {
                        Some(Msg::GeneralSearch(GSMsg::PopupUpdateEpisode(input_string)))
                    }
                    Source::Podcast => {
                        Some(Msg::GeneralSearch(GSMsg::PopupUpdatePodcast(input_string)))
                    }
                }
            }
            CmdResult::Submit(_) => Some(Msg::GeneralSearch(GSMsg::InputBlur)),

            CmdResult::None => None,
            _ => Some(Msg::ForceRedraw),
        }
    }
}

#[derive(MockComponent)]
pub struct GSTablePopup {
    component: Table,
    source: Source,
    config: SharedTuiSettings,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Source {
    Library(PathBuf),
    Playlist,
    Database,
    Episode,
    Podcast,
}

/// Get a [`Table`] component with the common style applied.
fn common_table_comp(config: &TuiOverlay, title: String) -> Table {
    Table::default()
        .borders(
            Borders::default()
                .color(config.settings.theme.fallback_border())
                .modifiers(BorderType::Rounded),
        )
        .background(config.settings.theme.fallback_background())
        .foreground(config.settings.theme.fallback_foreground())
        .title(title, Alignment::Left)
        .scroll(true)
        .highlighted_color(config.settings.theme.fallback_highlight())
        .highlighted_str(&config.settings.theme.style.library.highlight_symbol)
        .rewind(false)
        .step(4)
        .row_height(1)
        .column_spacing(3)
        .table(
            TableBuilder::default()
                .add_col(TextSpan::from("Empty result."))
                .add_col(TextSpan::from("Loading..."))
                .build(),
        )
}

impl GSTablePopup {
    #[allow(clippy::too_many_lines)]
    pub fn new(source: Source, config: SharedTuiSettings) -> Self {
        let config_r = config.read();
        // TODO: fix this up to be the proper keys
        let title_library = format!(
            " Results: (Enter: locate/{}: load to playlist) ",
            config_r.settings.keys.navigation_keys.right
        );
        let title_playlist = format!(
            " Results: (Enter: locate/{}: play selected) ",
            config_r.settings.keys.navigation_keys.right
        );
        let title_database = format!(
            " Results: ({}: load to playlist) ",
            config_r.settings.keys.navigation_keys.right
        );
        let title_episode = format!(
            " Results: (Enter: locate/{}: load to playlist) ",
            config_r.settings.keys.navigation_keys.right
        );

        let title_podcast = " Results: (Enter: locate) ";
        let component = match source {
            Source::Library(_) => common_table_comp(&config_r, title_library)
                .headers(["idx", "File name"])
                .widths(&[5, 95]),

            Source::Playlist => common_table_comp(&config_r, title_playlist)
                .headers(["Duration", "Artist", "Title"])
                .widths(&[14, 30, 56]),
            Source::Database => common_table_comp(&config_r, title_database)
                .headers(["Duration", "Artist", "Title"])
                .widths(&[14, 30, 56]),
            Source::Episode => common_table_comp(&config_r, title_episode)
                .headers(["idx", "Episode Title"])
                .widths(&[5, 95]),
            Source::Podcast => common_table_comp(&config_r, title_podcast.to_string())
                .headers(["idx", "Podcast Title"])
                .widths(&[5, 95]),
        };

        drop(config_r);
        Self {
            component,
            source,
            config,
        }
    }
}

impl Component<Msg, UserEvent> for GSTablePopup {
    fn on(&mut self, ev: Event<UserEvent>) -> Option<Msg> {
        let config = self.config.clone();
        let keys = &config.read().settings.keys;
        let cmd_result = match ev {
            Event::Keyboard(KeyEvent { code: Key::Esc, .. }) => {
                return Some(Msg::GeneralSearch(GSMsg::PopupCloseCancel));
            }
            Event::Keyboard(keyevent) if keyevent == keys.quit.get() => {
                return Some(Msg::GeneralSearch(GSMsg::PopupCloseCancel));
            }

            Event::Keyboard(KeyEvent { code: Key::Up, .. }) => {
                self.perform(Cmd::Move(Direction::Up))
            }
            Event::Keyboard(KeyEvent {
                code: Key::Down, ..
            }) => self.perform(Cmd::Move(Direction::Down)),

            Event::Keyboard(keyevent) if keyevent == keys.navigation_keys.down.get() => {
                self.perform(Cmd::Move(Direction::Down))
            }

            Event::Keyboard(keyevent) if keyevent == keys.navigation_keys.up.get() => {
                self.perform(Cmd::Move(Direction::Up))
            }
            Event::Keyboard(KeyEvent {
                code: Key::PageDown,
                ..
            }) => self.perform(Cmd::Scroll(Direction::Down)),
            Event::Keyboard(KeyEvent {
                code: Key::PageUp, ..
            }) => self.perform(Cmd::Scroll(Direction::Up)),
            Event::Keyboard(keyevent) if keyevent == keys.navigation_keys.goto_top.get() => {
                self.perform(Cmd::GoTo(Position::Begin))
            }
            Event::Keyboard(keyevent) if keyevent == keys.navigation_keys.goto_bottom.get() => {
                self.perform(Cmd::GoTo(Position::End))
            }
            Event::Keyboard(KeyEvent {
                code: Key::Home, ..
            }) => self.perform(Cmd::GoTo(Position::Begin)),
            Event::Keyboard(KeyEvent { code: Key::End, .. }) => {
                self.perform(Cmd::GoTo(Position::End))
            }
            Event::Keyboard(KeyEvent { code: Key::Tab, .. }) => {
                return Some(Msg::GeneralSearch(GSMsg::TableBlur));
            }

            Event::Keyboard(keyevent) if keyevent == keys.navigation_keys.right.get() => {
                match self.source {
                    Source::Library(_) => {
                        return Some(Msg::GeneralSearch(GSMsg::PopupCloseLibraryAddPlaylist));
                    }
                    Source::Playlist => {
                        return Some(Msg::GeneralSearch(GSMsg::PopupClosePlaylistPlaySelected));
                    }
                    Source::Database => {
                        return Some(Msg::GeneralSearch(GSMsg::PopupCloseDatabaseAddPlaylist));
                    }
                    Source::Episode => {
                        return Some(Msg::GeneralSearch(GSMsg::PopupCloseEpisodeAddPlaylist));
                    }
                    Source::Podcast => return None,
                }
            }
            Event::Keyboard(KeyEvent {
                code: Key::Enter, ..
            }) => match self.source {
                Source::Library(_) => {
                    return Some(Msg::GeneralSearch(GSMsg::PopupCloseOkLibraryLocate));
                }
                Source::Playlist => {
                    return Some(Msg::GeneralSearch(GSMsg::PopupCloseOkPlaylistLocate));
                }
                Source::Database => return Some(Msg::GeneralSearch(GSMsg::PopupCloseCancel)),
                Source::Episode => {
                    return Some(Msg::GeneralSearch(GSMsg::PopupCloseOkEpisodeLocate));
                }
                Source::Podcast => {
                    return Some(Msg::GeneralSearch(GSMsg::PopupCloseOkPodcastLocate));
                }
            },
            _ => CmdResult::None,
        };
        match cmd_result {
            CmdResult::None => None,
            _ => Some(Msg::ForceRedraw),
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
        if let Ok(State::One(StateValue::Usize(index))) = self.app.state(&Id::GeneralSearchTable)
            && let Ok(Some(AttrValue::Table(table))) =
                self.app.query(&Id::GeneralSearchTable, Attribute::Content)
            && let Some(line) = table.get(index)
            && let Some(text_span) = line.get(1)
        {
            let node = text_span.content.clone();
            self.new_library_scan_dir(PathBuf::from(&node), Some(node));
        }
    }

    pub fn general_search_after_library_add_playlist(&mut self) -> Result<()> {
        if let Ok(State::One(StateValue::Usize(index))) = self.app.state(&Id::GeneralSearchTable)
            && let Ok(Some(AttrValue::Table(table))) =
                self.app.query(&Id::GeneralSearchTable, Attribute::Content)
            && let Some(line) = table.get(index)
            && let Some(text_span) = line.get(1)
        {
            let text = &text_span.content;
            let path = Path::new(text);
            self.playlist_add(path)?;
        }
        Ok(())
    }

    pub fn general_search_after_playlist_select(&mut self) {
        let mut index = 0;
        let mut matched = false;
        if let Ok(State::One(StateValue::Usize(result_index))) =
            self.app.state(&Id::GeneralSearchTable)
            && let Ok(Some(AttrValue::Table(table))) =
                self.app.query(&Id::GeneralSearchTable, Attribute::Content)
            && let Some(line) = table.get(result_index)
            && let Some(file_name_text_span) = line.get(3)
        {
            let file_name = &file_name_text_span.content;
            for (idx, item) in self.playback.playlist.tracks().iter().enumerate() {
                // NOTE: i dont know if this should apply to anything other than "track_data"
                let lower_matched = match item.inner() {
                    MediaTypes::Track(track_data) => {
                        track_data.path().to_string_lossy() == file_name.as_str()
                    }
                    MediaTypes::Radio(radio_track_data) => radio_track_data.url() == file_name,
                    MediaTypes::Podcast(podcast_track_data) => {
                        podcast_track_data.url() == file_name
                    }
                };
                if lower_matched {
                    index = idx;
                    matched = true;
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
            && let Ok(Some(AttrValue::Table(table))) =
                self.app.query(&Id::GeneralSearchTable, Attribute::Content)
            && let Some(line) = table.get(result_index)
            && let Some(file_name_text_span) = line.get(3)
        {
            let file_name = &file_name_text_span.content;
            for (idx, item) in self.playback.playlist.tracks().iter().enumerate() {
                // NOTE: i dont know if this should apply to anything other than "track_data"
                let lower_matched = match item.inner() {
                    MediaTypes::Track(track_data) => {
                        track_data.path().to_string_lossy() == file_name.as_str()
                    }
                    MediaTypes::Radio(radio_track_data) => radio_track_data.url() == file_name,
                    MediaTypes::Podcast(podcast_track_data) => {
                        podcast_track_data.url() == file_name
                    }
                };
                if lower_matched {
                    index = idx;
                    matched = true;
                }
            }
        }
        if !matched {
            return;
        }
        self.playlist_play_selected(index);
    }

    pub fn general_search_after_database_add_playlist(&mut self) -> Result<()> {
        let track = self.general_search_get_info(3)?;
        let path = Path::new(&track);
        self.playlist_add(path)?;
        Ok(())
    }

    #[allow(clippy::cast_possible_wrap)]
    pub fn general_search_after_episode_add_playlist(&mut self) -> Result<()> {
        let episode_id: usize = self.general_search_get_info(2)?.parse()?;
        if let Ok((_podcast_idx, episode_idx)) = self.podcast_find_by_ep_id(episode_id) {
            self.playlist_add_episode(episode_idx)?;
        }
        Ok(())
    }

    pub fn general_search_after_episode_select(&mut self) -> Result<()> {
        let episode_id: usize = self.general_search_get_info(2)?.parse()?;
        if let Ok((podcast_idx, episode_idx)) = self.podcast_find_by_ep_id(episode_id) {
            self.podcast_locate_episode(podcast_idx, episode_idx);
        }
        Ok(())
    }

    pub fn general_search_after_podcast_select(&mut self) -> Result<()> {
        let pod_id: usize = self.general_search_get_info(2)?.parse()?;
        if let Ok(podcast_idx) = self.podcast_find_by_pod_id(pod_id) {
            self.podcast_locate_episode(podcast_idx, 0);
        }
        Ok(())
    }

    pub fn general_search_get_info(&mut self, column: usize) -> Result<String> {
        if let Ok(State::One(StateValue::Usize(index))) = self.app.state(&Id::GeneralSearchTable)
            && let Ok(Some(AttrValue::Table(table))) =
                self.app.query(&Id::GeneralSearchTable, Attribute::Content)
        {
            let line = table
                .get(index)
                .ok_or_else(|| anyhow!("error getting index from table"))?;
            let text_span = line
                .get(column)
                .ok_or_else(|| anyhow!("error getting text span"))?;
            return Ok(text_span.content.clone());
        }
        bail!("column cannot find in general search")
    }
}
