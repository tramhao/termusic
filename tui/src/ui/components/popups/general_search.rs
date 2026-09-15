use std::borrow::Cow;
use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::{Result, anyhow, bail};
use termusiclib::common::const_unknown::{UNKNOWN_ARTIST, UNKNOWN_FILE, UNKNOWN_TITLE};
use termusiclib::config::{SharedTuiSettings, TuiOverlay};
use termusiclib::new_database::track_ops;
use termusiclib::track::{DurationFmtShort, MediaTypes, Track};
use tui_realm_stdlib::components::{Input, Table};
use tui_realm_stdlib::prop_ext::CommonHighlight;
use tuirealm::command::{Cmd, CmdResult, Direction, Position};
use tuirealm::component::{AppComponent, Component};
use tuirealm::event::{Event, Key, KeyEvent, KeyModifiers};
use tuirealm::props::{
    AttrValue, AttrValueRef, Attribute, BorderType, Borders, HorizontalAlignment, InputType,
    LineStatic, QueryResult, Style, Table as PropTable, TableBuilder, Title,
};
use tuirealm::state::{State, StateValue};

use crate::ui::Model;
use crate::ui::ids::Id;
use crate::ui::model::UserEvent;
use crate::ui::msg::{GSMsg, Msg};
use crate::ui::utils::STYLE_REMOVE_REVERSE;

#[derive(Component)]
pub struct GSInputPopup {
    component: Input,
    source: Source,
}

/// Get a [`Input`] component with the common style applied.
#[inline]
fn common_input_comp<T: Into<Title>>(config: &TuiOverlay, title: T) -> Input {
    Input::default()
        .foreground(config.settings.theme.fallback_foreground())
        .background(config.settings.theme.fallback_background())
        .inactive(Style::new().bg(config.settings.theme.fallback_background()))
        .borders(
            Borders::default()
                .color(config.settings.theme.fallback_border())
                .modifiers(BorderType::Rounded),
        )
        .title(title.into().alignment(HorizontalAlignment::Left))
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

impl AppComponent<Msg, UserEvent> for GSInputPopup {
    fn on(&mut self, ev: &Event<UserEvent>) -> Option<Msg> {
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
            }) => self.perform(Cmd::Type(*ch)),
            Event::Keyboard(KeyEvent { code: Key::Esc, .. }) => {
                return Some(Msg::GeneralSearch(GSMsg::PopupCloseCancel));
            }
            Event::Keyboard(KeyEvent {
                code: Key::Enter, ..
            }) => self.perform(Cmd::Submit),
            Event::Keyboard(KeyEvent { code: Key::Tab, .. }) => {
                return Some(Msg::GeneralSearch(GSMsg::InputBlur));
            }
            _ => CmdResult::NoChange,
        };
        match cmd_result {
            CmdResult::Changed(State::Single(StateValue::String(input_string))) => {
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

            CmdResult::NoChange => None,
            _ => Some(Msg::ForceRedraw),
        }
    }
}

#[derive(Component)]
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
        .foreground(config.settings.theme.fallback_foreground())
        .background(config.settings.theme.fallback_background())
        .inactive(Style::new().bg(config.settings.theme.fallback_background()))
        .title(Title::from(title).alignment(HorizontalAlignment::Left))
        .scroll(true)
        .highlight_style(
            CommonHighlight::default()
                .style
                .fg(config.settings.theme.fallback_highlight()),
        )
        .highlight_style_inactive(STYLE_REMOVE_REVERSE)
        .highlight_str(config.settings.theme.style.library.highlight_symbol.clone())
        .rewind(false)
        .step(4)
        .row_height(1)
        .column_spacing(3)
        .table(
            TableBuilder::default()
                .add_col(LineStatic::from("Empty result."))
                .add_col(LineStatic::from("Loading..."))
                .build(),
        )
}

impl GSTablePopup {
    #[allow(clippy::too_many_lines)]
    pub fn new(source: Source, config: SharedTuiSettings) -> Self {
        let config_r = config.read();
        let title_library = format!(
            " Results: (Enter: locate/{}: load to playlist) ",
            config_r.settings.keys.library_keys.load_track
        );
        let title_playlist = format!(
            " Results: (Enter: locate/{}: play selected) ",
            config_r.settings.keys.library_keys.load_track
        );
        let title_database = format!(
            " Results: ({}: load to playlist) ",
            config_r.settings.keys.library_keys.load_track
        );
        let title_episode = format!(
            " Results: (Enter: locate/{}: load to playlist) ",
            config_r.settings.keys.library_keys.load_track
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

impl AppComponent<Msg, UserEvent> for GSTablePopup {
    fn on(&mut self, ev: &Event<UserEvent>) -> Option<Msg> {
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

            Event::Keyboard(keyevent) if keyevent == keys.library_keys.load_track.get() => {
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
                    Source::Podcast => {
                        return Some(Msg::GeneralSearch(GSMsg::PopupCloseOkPodcastLocate));
                    }
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
            _ => CmdResult::NoChange,
        };
        match cmd_result {
            CmdResult::NoChange => None,
            _ => Some(Msg::ForceRedraw),
        }
    }
}

impl Model {
    /// Mount / Remount a search popup for the provided source
    pub fn mount_general_search(&mut self, source: Source) -> Result<()> {
        self.app.remount(
            Id::GeneralSearchInput,
            Box::new(GSInputPopup::new(source.clone(), &self.config_tui.read())),
            Vec::new(),
        )?;
        self.app.remount(
            Id::GeneralSearchTable,
            Box::new(GSTablePopup::new(source, self.config_tui.clone())),
            Vec::new(),
        )?;

        self.app.active(&Id::GeneralSearchInput)?;
        if let Err(e) = self.update_photo() {
            self.mount_error_popup(e.context("update_photo"));
        }

        Ok(())
    }

    /// Unmount the General Search Popup.
    pub fn umount_general_search(&mut self) -> Result<()> {
        self.app.umount(&Id::GeneralSearchInput)?;
        self.app.umount(&Id::GeneralSearchTable)?;
        if let Err(e) = self.update_photo() {
            self.mount_error_popup(e.context("update_photo"));
        }

        Ok(())
    }

    pub fn general_search_update_show(&mut self, table: tuirealm::props::Table) {
        self.app
            .attr(
                &Id::GeneralSearchTable,
                Attribute::Content,
                AttrValue::Table(table),
            )
            .ok();
    }

    pub fn general_search_after_library_select(&mut self) {
        if let Ok(State::Single(StateValue::Usize(index))) = self.app.state(&Id::GeneralSearchTable)
            && let Some(table) = self
                .app
                .query(&Id::GeneralSearchTable, Attribute::Content)
                .ok()
                .flatten()
                .as_ref()
                .map(QueryResult::as_ref)
                .and_then(AttrValueRef::as_table)
            && let Some(line) = table.get(index)
            && let Some(text_span) = line.get(1)
        {
            let node = text_span.to_string();
            self.new_library_scan_dir(PathBuf::from(&node), Some(node));
        }
    }

    pub fn general_search_after_library_add_playlist(&mut self) -> Result<()> {
        if let Ok(State::Single(StateValue::Usize(index))) = self.app.state(&Id::GeneralSearchTable)
            && let Some(table) = self
                .app
                .query(&Id::GeneralSearchTable, Attribute::Content)
                .ok()
                .flatten()
                .as_ref()
                .map(QueryResult::as_ref)
                .and_then(AttrValueRef::as_table)
            && let Some(line) = table.get(index)
            && let Some(text_span) = line.get(1)
        {
            let text = text_span.to_string();
            let path = Path::new(&text);
            self.playlist_add(path)?;
        }
        Ok(())
    }

    pub fn general_search_after_playlist_select(&mut self) {
        let mut index = 0;
        let mut matched = false;
        if let Ok(State::Single(StateValue::Usize(result_index))) =
            self.app.state(&Id::GeneralSearchTable)
            && let Some(table) = self
                .app
                .query(&Id::GeneralSearchTable, Attribute::Content)
                .ok()
                .flatten()
                .as_ref()
                .map(QueryResult::as_ref)
                .and_then(AttrValueRef::as_table)
            && let Some(line) = table.get(result_index)
            && let Some(file_name_text_span) = line.get(3)
        {
            let file_name = file_name_text_span.to_string();
            for (idx, item) in self.playback.playlist.read().tracks().iter().enumerate() {
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
        if let Ok(State::Single(StateValue::Usize(result_index))) =
            self.app.state(&Id::GeneralSearchTable)
            && let Some(table) = self
                .app
                .query(&Id::GeneralSearchTable, Attribute::Content)
                .ok()
                .flatten()
                .as_ref()
                .map(QueryResult::as_ref)
                .and_then(AttrValueRef::as_table)
            && let Some(line) = table.get(result_index)
            && let Some(file_name_text_span) = line.get(3)
        {
            let file_name = file_name_text_span.to_string();
            for (idx, item) in self.playback.playlist.read().tracks().iter().enumerate() {
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
        if let Ok(State::Single(StateValue::Usize(index))) = self.app.state(&Id::GeneralSearchTable)
            && let Some(table) = self
                .app
                .query(&Id::GeneralSearchTable, Attribute::Content)
                .ok()
                .flatten()
                .as_ref()
                .map(QueryResult::as_ref)
                .and_then(AttrValueRef::as_table)
        {
            let line = table
                .get(index)
                .ok_or_else(|| anyhow!("error getting index from table"))?;
            let text_span = line
                .get(column)
                .ok_or_else(|| anyhow!("error getting text span"))?;
            return Ok(text_span.to_string());
        }
        bail!("column cannot find in general search")
    }
}

/// Get various values for matching.
///
/// [`wildmatch`] requires matching against strings.
/// Aside from just matching, it is also used to display the found matches.
pub trait Matchable {
    fn meta_file(&self) -> Option<Cow<'_, str>>;
    fn meta_title(&self) -> Option<&str>;
    fn meta_album(&self) -> Option<&str>;
    fn meta_artist(&self) -> Option<&str>;
    fn meta_duration(&self) -> Option<Duration>;
}

impl Matchable for Track {
    fn meta_file(&self) -> Option<Cow<'_, str>> {
        self.as_track()
            .and_then(|v| v.path().to_str())
            .map(Cow::from)
    }

    fn meta_title(&self) -> Option<&str> {
        self.title()
    }

    fn meta_album(&self) -> Option<&str> {
        self.as_track().and_then(|v| v.album())
    }

    fn meta_artist(&self) -> Option<&str> {
        self.artist()
    }

    fn meta_duration(&self) -> Option<Duration> {
        self.duration()
    }
}

impl Matchable for &Track {
    fn meta_file(&self) -> Option<Cow<'_, str>> {
        self.as_track()
            .and_then(|v| v.path().to_str())
            .map(Cow::from)
    }

    fn meta_title(&self) -> Option<&str> {
        self.title()
    }

    fn meta_album(&self) -> Option<&str> {
        self.as_track().and_then(|v| v.album())
    }

    fn meta_artist(&self) -> Option<&str> {
        self.artist()
    }

    fn meta_duration(&self) -> Option<Duration> {
        self.duration()
    }
}

impl Matchable for track_ops::TrackRead {
    fn meta_file(&self) -> Option<Cow<'_, str>> {
        let pathbuf = self.as_pathbuf();
        let _ = pathbuf.to_str()?;
        Some(pathbuf.into_os_string().into_string().unwrap().into())
    }

    fn meta_title(&self) -> Option<&str> {
        self.title.as_deref()
    }

    fn meta_album(&self) -> Option<&str> {
        self.album.as_ref().map(|v| v.title.as_str())
    }

    fn meta_artist(&self) -> Option<&str> {
        self.artist_display.as_deref()
    }

    fn meta_duration(&self) -> Option<Duration> {
        self.duration
    }
}

impl Matchable for &track_ops::TrackRead {
    fn meta_file(&self) -> Option<Cow<'_, str>> {
        let pathbuf = self.as_pathbuf();
        let _ = pathbuf.to_str()?;
        Some(pathbuf.into_os_string().into_string().unwrap().into())
    }

    fn meta_title(&self) -> Option<&str> {
        self.title.as_deref()
    }

    fn meta_album(&self) -> Option<&str> {
        self.album.as_ref().map(|v| v.title.as_str())
    }

    fn meta_artist(&self) -> Option<&str> {
        self.artist_display.as_deref()
    }

    fn meta_duration(&self) -> Option<Duration> {
        self.duration
    }
}

fn match_record<T: Matchable>(record: &T, search: &str) -> bool {
    let artist_match: bool = if let Some(artist) = record.meta_artist() {
        wildmatch::WildMatch::new(search).matches(&artist.to_lowercase())
    } else {
        false
    };
    let title_match: bool = if let Some(title) = record.meta_title() {
        wildmatch::WildMatch::new(search).matches(&title.to_lowercase())
    } else {
        false
    };
    let album_match: bool = if let Some(album) = record.meta_album() {
        wildmatch::WildMatch::new(search).matches(&album.to_lowercase())
    } else {
        false
    };
    artist_match || title_match || album_match
}

pub fn update_search<'a, T: Matchable>(
    data: &'a [T],
    pattern: &'a str,
) -> impl Iterator<Item = &'a T> {
    let search = format!("*{}*", pattern.to_lowercase());
    data.iter()
        .filter(move |&record| match_record(record, &search))
}

pub fn build_table<T: Matchable, I: Iterator<Item = T>>(
    data: I,
    config: &SharedTuiSettings,
) -> PropTable {
    let mut peekable_data = data.peekable();
    let mut table: TableBuilder = TableBuilder::default();
    if peekable_data.peek().is_none() {
        table.add_col(LineStatic::from("0"));
        table.add_col(LineStatic::from("empty tracks from db/playlist"));
        table.add_col(LineStatic::from(""));
        return table.build();
    }

    let artist_color = config.read_recursive().settings.theme.library_highlight();

    for (idx, record) in peekable_data.enumerate() {
        if idx > 0 {
            table.add_row();
        }

        let duration_string = if let Some(dur) = record.meta_duration() {
            let duration = DurationFmtShort(dur);
            format!("[{duration:^6.6}]")
        } else {
            "[--:--]".to_string()
        };

        table
            .add_col(LineStatic::from(duration_string))
            .add_col(LineStatic::styled(
                record.meta_artist().unwrap_or(UNKNOWN_ARTIST).to_string(),
                Style::new().fg(artist_color),
            ))
            .add_col(LineStatic::styled(
                record.meta_title().unwrap_or(UNKNOWN_TITLE).to_string(),
                Style::new().bold(),
            ))
            .add_col(LineStatic::from(
                record
                    .meta_file()
                    .unwrap_or(Cow::Borrowed(UNKNOWN_FILE))
                    .to_string(),
            ));
    }
    table.build()
}
