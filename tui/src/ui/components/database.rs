use std::borrow::Cow;
use std::path::Path;
use std::time::Duration;

use anyhow::Result;
use either::Either;
use termusiclib::common::const_unknown::{UNKNOWN_ARTIST, UNKNOWN_FILE, UNKNOWN_TITLE};
use termusiclib::config::SharedTuiSettings;
use termusiclib::config::v2::tui::keys::Keys;
use termusiclib::new_database::track_ops::TrackRead;
use termusiclib::new_database::{album_ops, artist_ops, track_ops};
use termusiclib::track::{DurationFmtShort, Track};
use termusiclib::utils::{is_playlist, playlist_get_vec};
use tui_realm_stdlib::components::List;
use tui_realm_stdlib::prop_ext::CommonHighlight;
use tuirealm::command::{Cmd, CmdResult, Direction, Position};
use tuirealm::component::{AppComponent, Component};
use tuirealm::event::Event;
use tuirealm::event::{Key, KeyEvent, KeyModifiers};
use tuirealm::props::{
    AttrValue, AttrValueRef, Attribute, BorderType, HorizontalAlignment, LineStatic, PropPayload,
    PropPayloadRef, PropValue, QueryResult, Table, TableBuilder, Title,
};
use tuirealm::props::{Borders, Style};
use tuirealm::state::{State, StateValue};

use super::popups::{YNConfirm, YNConfirmStyle};
use crate::ui::Model;
use crate::ui::ids::Id;
use crate::ui::model::UserEvent;
use crate::ui::msg::{DBMsg, GSMsg, Msg, SearchCriteria};
use crate::ui::utils::STYLE_REMOVE_REVERSE;

/// Helper trait to accomedate mutable access to `self` while also allowing access to other `self` properties for [`common_list_movement`].
trait OnKeyDB {
    fn on_key_tab(&self) -> Msg;
    fn on_key_backtab(&self) -> Msg;
}

/// Common matches for [`List`] component movement and events
fn common_list_movement<C: Component + OnKeyDB>(
    comp: &mut C,
    keys: &Keys,
    ev: &Event<UserEvent>,
) -> Option<Either<CmdResult, Msg>> {
    let res = match ev {
        Event::Keyboard(KeyEvent {
            code: Key::Up,
            modifiers: KeyModifiers::NONE,
        }) => comp.perform(Cmd::Move(Direction::Up)),
        Event::Keyboard(key) if key == keys.navigation_keys.up.get() => {
            comp.perform(Cmd::Move(Direction::Up))
        }

        Event::Keyboard(KeyEvent {
            code: Key::Down,
            modifiers: KeyModifiers::NONE,
        }) => {
            if let Some(t) = comp
                .query(Attribute::Text)
                .as_ref()
                .map(QueryResult::as_ref)
                .and_then(AttrValueRef::as_payload)
                .and_then(PropPayloadRef::as_vec)
                && let State::Single(StateValue::Usize(index)) = comp.state()
                && index >= t.len() - 1
            {
                return Some(Either::Right(comp.on_key_tab()));
            }
            comp.perform(Cmd::Move(Direction::Down))
        }
        Event::Keyboard(key) if key == keys.navigation_keys.down.get() => {
            if let Some(t) = comp
                .query(Attribute::Text)
                .as_ref()
                .map(QueryResult::as_ref)
                .and_then(AttrValueRef::as_payload)
                .and_then(PropPayloadRef::as_vec)
                && let State::Single(StateValue::Usize(index)) = comp.state()
                && index >= t.len() - 1
            {
                return Some(Either::Right(comp.on_key_tab()));
            }
            comp.perform(Cmd::Move(Direction::Down))
        }

        Event::Keyboard(KeyEvent {
            code: Key::PageUp,
            modifiers: KeyModifiers::NONE,
        }) => comp.perform(Cmd::Scroll(Direction::Up)),
        Event::Keyboard(KeyEvent {
            code: Key::PageDown,
            modifiers: KeyModifiers::NONE,
        }) => comp.perform(Cmd::Scroll(Direction::Down)),

        Event::Keyboard(KeyEvent {
            code: Key::Home,
            modifiers: KeyModifiers::NONE,
        }) => comp.perform(Cmd::GoTo(Position::Begin)),
        Event::Keyboard(key) if key == keys.navigation_keys.goto_top.get() => {
            comp.perform(Cmd::GoTo(Position::Begin))
        }

        Event::Keyboard(KeyEvent {
            code: Key::End,
            modifiers: KeyModifiers::NONE,
        }) => comp.perform(Cmd::GoTo(Position::End)),
        Event::Keyboard(key) if key == keys.navigation_keys.goto_bottom.get() => {
            comp.perform(Cmd::GoTo(Position::End))
        }

        Event::Keyboard(KeyEvent {
            code: Key::Tab,
            modifiers: KeyModifiers::NONE,
        }) => return Some(Either::Right(comp.on_key_tab())),
        Event::Keyboard(KeyEvent {
            code: Key::BackTab,
            modifiers: KeyModifiers::SHIFT,
        }) => return Some(Either::Right(comp.on_key_backtab())),

        _ => return None,
    };

    Some(Either::Left(res))
}

/// Like [`SearchCriteria`], but specific to TUI and mapping to & from a [`Table`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DBCriteria {
    Artists,
    Albums,
    Genres,
    Directories,
    Playlists,
}

impl DBCriteria {
    /// Number of elements in the table.
    /// This is for example used to get exact space allocation for the layout.
    ///
    /// Note: keep this in-sync with [`Self::build_table`]
    const NUM_OPTIONS: u16 = 5;

    fn build_table() -> [LineStatic; 5] {
        [
            LineStatic::from("Artist"),
            LineStatic::from("Album"),
            LineStatic::from("Genre"),
            LineStatic::from("Directory"),
            LineStatic::from("Playlists"),
        ]
    }

    /// Try to map the given table index to a variant, returns [`None`] if index is out of bounds.
    fn from_table_index(idx: usize) -> Option<Self> {
        // NOTE: this has to match whatever `build_table` produces
        let res = match idx {
            0 => Self::Artists,
            1 => Self::Albums,
            2 => Self::Genres,
            3 => Self::Directories,
            4 => Self::Playlists,
            _ => return None,
        };

        Some(res)
    }
}

impl From<DBCriteria> for SearchCriteria {
    fn from(value: DBCriteria) -> Self {
        match value {
            DBCriteria::Artists => Self::Artist,
            DBCriteria::Albums => Self::Album,
            DBCriteria::Genres => Self::Genre,
            DBCriteria::Directories => Self::Directory,
            DBCriteria::Playlists => Self::Playlist,
        }
    }
}

#[derive(Component)]
pub struct DBListCriteria {
    component: List,
    on_key_tab: Msg,
    on_key_backtab: Msg,
    config: SharedTuiSettings,
}

impl DBListCriteria {
    pub fn new(config: SharedTuiSettings, on_key_tab: Msg, on_key_backtab: Msg) -> Self {
        let component = {
            let config = config.read();
            List::default()
                .borders(
                    Borders::default()
                        .modifiers(BorderType::Rounded)
                        .color(config.settings.theme.library_border()),
                )
                .background(config.settings.theme.library_background())
                .foreground(config.settings.theme.library_foreground())
                .inactive(Style::new().bg(config.settings.theme.library_background()))
                .title(Title::from(" DataBase ").alignment(HorizontalAlignment::Left))
                .scroll(true)
                .highlight_style(
                    CommonHighlight::default()
                        .style
                        .fg(config.settings.theme.library_highlight()),
                )
                .highlight_style_inactive(STYLE_REMOVE_REVERSE)
                .highlight_str(config.settings.theme.style.library.highlight_symbol.clone())
                .rewind(false)
                .step(4)
                .scroll(true)
                .rows(DBCriteria::build_table())
        };

        Self {
            component,
            on_key_tab,
            on_key_backtab,
            config,
        }
    }

    /// Get the number of static options in the list.
    // See `DBCriteria::num_option` for actual implementation.
    pub const fn num_options() -> u16 {
        DBCriteria::NUM_OPTIONS
    }
}

impl OnKeyDB for DBListCriteria {
    fn on_key_tab(&self) -> Msg {
        self.on_key_tab.clone()
    }

    fn on_key_backtab(&self) -> Msg {
        self.on_key_backtab.clone()
    }
}

impl AppComponent<Msg, UserEvent> for DBListCriteria {
    fn on(&mut self, ev: &Event<UserEvent>) -> Option<Msg> {
        let config = self.config.clone();
        let keys = &config.read().settings.keys;

        let cmd_result = common_list_movement(self, keys, ev).unwrap_or_else(|| {
            let res = match ev {
                Event::Keyboard(KeyEvent {
                    code: Key::Enter,
                    modifiers: KeyModifiers::NONE,
                }) => {
                    if let State::Single(StateValue::Usize(index)) = self.state() {
                        let criteria = DBCriteria::from_table_index(index)
                            .expect("All table options to be mapped");
                        return Either::Right(Msg::DataBase(DBMsg::SearchResult(criteria.into())));
                    }
                    CmdResult::NoChange
                }

                Event::Keyboard(keyevent) if keyevent == keys.library_keys.search.get() => {
                    return Either::Right(Msg::GeneralSearch(GSMsg::PopupShowDatabase));
                }
                _ => CmdResult::NoChange,
            };

            Either::Left(res)
        });

        match cmd_result {
            Either::Left(CmdResult::NoChange) => None,
            Either::Right(msg) => Some(msg),
            Either::Left(_) => Some(Msg::ForceRedraw),
        }
    }
}

/// Component for a "Are you sure you want to add ALL found albums? Y/N" popup
#[derive(Component)]
pub struct AddAlbumConfirm {
    component: YNConfirm,
}

impl AddAlbumConfirm {
    pub fn new(config: SharedTuiSettings, criteria: &str) -> Self {
        let component = YNConfirm::new_with_cb(
            config,
            format!(" Are you sure you want to add EVERYTHING from {criteria}? "),
            |config| YNConfirmStyle {
                foreground_color: config.settings.theme.important_popup_foreground(),
                background_color: config.settings.theme.important_popup_background(),
                border_color: config.settings.theme.important_popup_border(),
                title_alignment: HorizontalAlignment::Left,
            },
        );

        Self { component }
    }
}

impl AppComponent<Msg, UserEvent> for AddAlbumConfirm {
    fn on(&mut self, ev: &Event<UserEvent>) -> Option<Msg> {
        self.component.on(
            ev,
            Msg::DataBase(DBMsg::AddAllResultsToPlaylist),
            Msg::DataBase(DBMsg::AddAllResultsConfirmCancel),
        )
    }
}

#[derive(Component)]
pub struct DBListSearchResult {
    component: List,
    on_key_tab: Msg,
    on_key_backtab: Msg,
    config: SharedTuiSettings,
}

impl DBListSearchResult {
    pub fn new(config: SharedTuiSettings, on_key_tab: Msg, on_key_backtab: Msg) -> Self {
        let component = {
            let config = config.read();
            List::default()
                .borders(
                    Borders::default()
                        .modifiers(BorderType::Rounded)
                        .color(config.settings.theme.library_border()),
                )
                .background(config.settings.theme.library_background())
                .foreground(config.settings.theme.library_foreground())
                .inactive(Style::new().bg(config.settings.theme.library_background()))
                .title(Title::from(" Result ").alignment(HorizontalAlignment::Left))
                .scroll(true)
                .highlight_style(
                    CommonHighlight::default()
                        .style
                        .fg(config.settings.theme.library_highlight()),
                )
                .highlight_style_inactive(STYLE_REMOVE_REVERSE)
                .highlight_str(config.settings.theme.style.library.highlight_symbol.clone())
                .rewind(false)
                .step(4)
                .scroll(true)
                .rows([LineStatic::from("Empty")])
        };

        Self {
            component,
            on_key_tab,
            on_key_backtab,
            config,
        }
    }
}

impl OnKeyDB for DBListSearchResult {
    fn on_key_tab(&self) -> Msg {
        self.on_key_tab.clone()
    }

    fn on_key_backtab(&self) -> Msg {
        self.on_key_backtab.clone()
    }
}

impl AppComponent<Msg, UserEvent> for DBListSearchResult {
    #[allow(clippy::too_many_lines)]
    fn on(&mut self, ev: &Event<UserEvent>) -> Option<Msg> {
        let config = self.config.clone();
        let keys = &config.read().settings.keys;

        let cmd_result = common_list_movement(self, keys, ev).unwrap_or_else(|| {
            let res = match ev {
                Event::Keyboard(KeyEvent {
                    code: Key::Enter,
                    modifiers: KeyModifiers::NONE,
                }) => {
                    if let State::Single(StateValue::Usize(index)) = self.state() {
                        return Either::Right(Msg::DataBase(DBMsg::SearchTrack(index)));
                    }
                    CmdResult::NoChange
                }

                Event::Keyboard(keyevent) if keyevent == keys.library_keys.search.get() => {
                    return Either::Right(Msg::GeneralSearch(GSMsg::PopupShowDatabase));
                }

                Event::Keyboard(keyevent) if keyevent == keys.database_keys.add_selected.get() => {
                    if let State::Single(StateValue::Usize(index)) = self.state() {
                        // Maybe we should also have a popup if it is anything other that "Album"?
                        // Because things like "Genre" could be *very* big
                        return Either::Right(Msg::DataBase(DBMsg::AddResultToPlaylist(index)));
                    }
                    CmdResult::NoChange
                }
                Event::Keyboard(keyevent) if keyevent == keys.database_keys.add_all.get() => {
                    return Either::Right(Msg::DataBase(DBMsg::AddAllResultsConfirmShow));
                }

                _ => CmdResult::NoChange,
            };

            Either::Left(res)
        });

        match cmd_result {
            Either::Left(CmdResult::NoChange) => None,
            Either::Right(msg) => Some(msg),
            Either::Left(_) => Some(Msg::ForceRedraw),
        }
    }
}

#[derive(Component)]
pub struct DBListSearchTracks {
    component: List,
    on_key_tab: Msg,
    on_key_backtab: Msg,
    config: SharedTuiSettings,
}

impl DBListSearchTracks {
    pub fn new(config: SharedTuiSettings, on_key_tab: Msg, on_key_backtab: Msg) -> Self {
        let component = {
            let config = config.read();
            List::default()
                .borders(
                    Borders::default()
                        .modifiers(BorderType::Rounded)
                        .color(config.settings.theme.library_border()),
                )
                .background(config.settings.theme.library_background())
                .foreground(config.settings.theme.library_foreground())
                .inactive(Style::new().bg(config.settings.theme.library_background()))
                .title(Title::from(" Tracks ").alignment(HorizontalAlignment::Left))
                .scroll(true)
                .highlight_style(
                    CommonHighlight::default()
                        .style
                        .fg(config.settings.theme.library_highlight()),
                )
                .highlight_style_inactive(STYLE_REMOVE_REVERSE)
                .highlight_str(config.settings.theme.style.library.highlight_symbol.clone())
                .rewind(false)
                .step(4)
                .scroll(true)
                .rows([LineStatic::from("Empty")])
        };

        Self {
            component,
            on_key_tab,
            on_key_backtab,
            config,
        }
    }
}

impl OnKeyDB for DBListSearchTracks {
    fn on_key_tab(&self) -> Msg {
        self.on_key_tab.clone()
    }

    fn on_key_backtab(&self) -> Msg {
        self.on_key_backtab.clone()
    }
}

impl AppComponent<Msg, UserEvent> for DBListSearchTracks {
    fn on(&mut self, ev: &Event<UserEvent>) -> Option<Msg> {
        let config = self.config.clone();
        let keys = &config.read().settings.keys;

        let cmd_result = common_list_movement(self, keys, ev).unwrap_or_else(|| {
            let res = match ev {
                Event::Keyboard(keyevent) if keyevent == keys.database_keys.add_selected.get() => {
                    if let State::Single(StateValue::Usize(index)) = self.state() {
                        return Either::Right(Msg::DataBase(DBMsg::AddPlaylist(index)));
                    }
                    CmdResult::NoChange
                }
                Event::Keyboard(keyevent) if keyevent == keys.database_keys.add_all.get() => {
                    return Either::Right(Msg::DataBase(DBMsg::AddAllToPlaylist));
                }

                Event::Keyboard(keyevent) if keyevent == keys.library_keys.search.get() => {
                    return Either::Right(Msg::GeneralSearch(GSMsg::PopupShowDatabase));
                }

                _ => CmdResult::NoChange,
            };

            Either::Left(res)
        });

        match cmd_result {
            Either::Left(CmdResult::NoChange) => None,
            Either::Right(msg) => Some(msg),
            Either::Left(_) => Some(Msg::ForceRedraw),
        }
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

impl Model {
    /// Build & Apply the `Tracks` Database component table data.
    pub fn database_sync_tracks_results(&mut self) {
        // let mut table: TableBuilder = TableBuilder::default();
        let mut lines = Vec::new();

        for (idx, record) in self.dw.search_tracks.iter().enumerate() {
            let name = record
                .title
                .as_ref()
                .map_or_else(|| record.file_stem.to_string_lossy(), Cow::from);

            lines.push(PropValue::TextLine(LineStatic::from(format!(
                "{} {name}",
                idx + 1
            ))));
        }
        if self.dw.search_results.is_empty() {
            lines.push(PropValue::TextLine(LineStatic::from("empty results")));
        }

        self.app
            .attr(
                &Id::DBListSearchTracks,
                Attribute::Text,
                AttrValue::Payload(PropPayload::Vec(lines)),
            )
            .ok();

        // self.playlist_update_title();
    }

    /// Build & Apply the `Results` Database component table data.
    pub fn database_sync_results(&mut self) {
        let mut lines = Vec::new();
        for (idx, record) in self.dw.search_results.iter().enumerate() {
            let mut display_name = String::new();
            match self.dw.criteria {
                SearchCriteria::Playlist => {
                    let path = Path::new(record);
                    let path_str = path.to_string_lossy();
                    let mut vec = path_str.split('/').rev();
                    if let (Some(v1), Some(v2)) = (vec.next(), vec.next()) {
                        display_name = format!("{v2}/{v1}");
                    }
                }
                SearchCriteria::Directory => {
                    let path = Path::new(record);
                    let path_string = path.to_string_lossy();
                    let mut vec = path_string.split('/').rev();
                    if let (Some(v1), Some(v2)) = (vec.next(), vec.next()) {
                        display_name = format!("{v2}/{v1}/");
                    }
                }
                _ => {
                    display_name.clone_from(record);
                }
            }
            if !display_name.is_empty() {
                lines.push(PropValue::TextLine(LineStatic::from(format!(
                    "{} {display_name}",
                    idx + 1
                ))));
            }
        }
        if self.dw.search_results.is_empty() {
            lines.push(PropValue::TextLine(LineStatic::from("empty results")));
        }

        // let table = table.build();
        self.app
            .attr(
                &Id::DBListSearchResult,
                Attribute::Text,
                AttrValue::Payload(PropPayload::Vec(lines)),
            )
            .ok();

        // self.playlist_update_title();
    }

    /// Update [`DBListSearchResult`] by querying the database or getting all playlists.
    pub fn database_update_search_results(&mut self) {
        let mut res = match self.dw.criteria {
            SearchCriteria::Playlist => self.database_get_playlist(),
            SearchCriteria::Artist => {
                let mut result = Vec::new();
                let all_artists = artist_ops::get_all_artists(
                    &self.db.get_connection(),
                    artist_ops::RowOrdering::IdAsc,
                );
                if let Ok(all_artists) = all_artists {
                    result.extend(all_artists.into_iter().map(|v| v.name));
                }

                result
            }
            SearchCriteria::Album => {
                let mut result = Vec::new();
                let all_albums = album_ops::get_all_albums(
                    &self.db.get_connection(),
                    album_ops::RowOrdering::IdAsc,
                );
                if let Ok(all_albums) = all_albums {
                    result.extend(all_albums.into_iter().map(|v| v.title));
                }

                result
            }
            SearchCriteria::Genre => {
                let mut result = Vec::new();
                let all_genres = track_ops::all_distinct_genres(&self.db.get_connection());
                if let Ok(all_genres) = all_genres {
                    result.extend(all_genres);
                }

                result
            }
            SearchCriteria::Directory => {
                let mut result = Vec::new();
                let all_dirs = track_ops::all_distinct_directories(&self.db.get_connection());
                if let Ok(all_dirs) = all_dirs {
                    result.extend(all_dirs);
                }

                result
            }
        };

        res.sort_by(|a, b| alphanumeric_sort::compare_str(a, b));

        self.dw.search_results = res;
        self.database_sync_results();
        self.app.active(&Id::DBListSearchResult).ok();
    }

    /// Scan all Music Roots for all playlists.
    fn database_get_playlist(&self) -> Vec<String> {
        let mut vec = Vec::new();

        for dir in &self.config_server.read().settings.player.music_dirs {
            let absolute_dir = shellexpand::path::tilde(dir);

            let all_items = walkdir::WalkDir::new(absolute_dir).follow_links(true);
            for record in all_items
                .into_iter()
                .filter_map(std::result::Result::ok)
                .filter(|p| is_playlist(p.path()))
            {
                let full_path_name = record.path().to_string_lossy().to_string();
                vec.push(full_path_name);
            }
        }

        vec
    }

    /// Find all tracks for the given [`criteria`](SearchCriteria) which matches `val`.
    ///
    /// Or for the [`Playlist`](SearchCriteria::Playlist) case, `val` is the path of the playlist.
    #[expect(clippy::too_many_lines)]
    pub fn database_get_tracks_by_criteria(
        &mut self,
        criteria: SearchCriteria,
        val: &str,
    ) -> Option<Vec<TrackRead>> {
        match criteria {
            SearchCriteria::Playlist => {
                let path = Path::new(val);
                if let Ok(vec) = playlist_get_vec(path) {
                    let mut vec_db = Vec::with_capacity(vec.len());
                    let conn = self.db.get_connection();
                    for item in vec {
                        let path = Path::new(&item);
                        // TODO: do we really need to lookup each value in a playlist in the database first?
                        let track = track_ops::get_track_from_path(&conn, path);
                        if let Ok(i) = track {
                            vec_db.push(i);
                        }
                    }
                    return Some(vec_db);
                }
            }
            SearchCriteria::Artist => {
                let mut result = Vec::new();
                let conn = self.db.get_connection();
                let all_artists = artist_ops::get_all_artists_like(
                    &conn,
                    &format!("%{val}%"),
                    artist_ops::RowOrdering::IdAsc,
                );
                if let Ok(all_artists) = all_artists {
                    for artist in all_artists {
                        let all_tracks = track_ops::get_tracks_from_artist(
                            &conn,
                            &artist.name,
                            track_ops::RowOrdering::IdAsc,
                        );
                        if let Ok(all_tracks) = all_tracks {
                            result.extend(all_tracks);
                        }
                    }
                }

                result.sort_by(|a, b| {
                    alphanumeric_sort::compare_path(a.as_pathbuf(), b.as_pathbuf())
                });

                return Some(result);
            }
            SearchCriteria::Album => {
                let mut result = Vec::new();
                let conn = self.db.get_connection();
                let all_albums = album_ops::get_all_albums_like(
                    &conn,
                    &format!("%{val}%"),
                    album_ops::RowOrdering::IdAsc,
                );
                if let Ok(all_albums) = all_albums {
                    for album in all_albums {
                        let all_tracks = track_ops::get_tracks_from_album(
                            &conn,
                            &album.title,
                            &album.artist_display,
                            track_ops::RowOrdering::IdAsc,
                        );
                        if let Ok(all_tracks) = all_tracks {
                            result.extend(all_tracks);
                        }
                    }
                }

                result.sort_by(|a, b| {
                    alphanumeric_sort::compare_path(a.as_pathbuf(), b.as_pathbuf())
                });

                return Some(result);
            }
            SearchCriteria::Genre => {
                let mut result = Vec::new();
                let conn = self.db.get_connection();
                let all_tracks = if val == "[unknown]" {
                    track_ops::get_tracks_from_genre(&conn, None, track_ops::RowOrdering::IdAsc)
                } else {
                    track_ops::get_tracks_from_genre_like(
                        &conn,
                        &format!("%{val}%"),
                        track_ops::RowOrdering::IdAsc,
                    )
                };
                if let Ok(all_tracks) = all_tracks {
                    result.extend(all_tracks);
                }

                result.sort_by(|a, b| {
                    alphanumeric_sort::compare_path(a.as_pathbuf(), b.as_pathbuf())
                });

                return Some(result);
            }
            SearchCriteria::Directory => {
                let mut result = Vec::new();
                let conn = self.db.get_connection();
                let dir = Path::new(val);
                let all_tracks =
                    track_ops::get_tracks_from_directory(&conn, dir, track_ops::RowOrdering::IdAsc);
                if let Ok(all_tracks) = all_tracks {
                    result.extend(all_tracks);
                }

                result.sort_by(|a, b| {
                    alphanumeric_sort::compare_path(a.as_pathbuf(), b.as_pathbuf())
                });

                return Some(result);
            }
        }

        None
    }

    /// Update view `Tracks` by populating it with items from the selected `Result`(view) index.
    pub fn database_update_search_tracks(&mut self, index: usize) {
        self.dw.search_tracks.clear();
        let Some(at_index) = self.dw.search_results.get(index).cloned() else {
            return;
        };

        let Some(result) = self.database_get_tracks_by_criteria(self.dw.criteria, &at_index) else {
            return;
        };

        self.dw.search_tracks = result;

        self.database_sync_tracks_results();
        self.app.active(&Id::DBListSearchTracks).ok();
    }

    /// Add all Results (from view `Result`) to the playlist.
    pub fn database_add_all_results(&mut self) {
        self.umount_results_add_confirm_database();
        if !self.dw.search_results.is_empty() {
            let mut tracks = Vec::new();
            // clone once instead every value in every iteration
            let search_results = self.dw.search_results.clone();
            for result in search_results {
                if let Some(mut res) =
                    self.database_get_tracks_by_criteria(self.dw.criteria, &result)
                {
                    tracks.append(&mut res);
                }
            }

            self.playlist_add_all_from_db(&tracks);
        }
    }

    /// Mount/Remount the Database search result components.
    pub fn remount_database_search(&mut self) -> Result<()> {
        self.app.remount(
            Id::DBListCriteria,
            Box::new(DBListCriteria::new(
                self.config_tui.clone(),
                Msg::DataBase(DBMsg::CriteriaBlurDown),
                Msg::DataBase(DBMsg::CriteriaBlurUp),
            )),
            Vec::new(),
        )?;

        self.app.remount(
            Id::DBListSearchResult,
            Box::new(DBListSearchResult::new(
                self.config_tui.clone(),
                Msg::DataBase(DBMsg::SearchResultBlurDown),
                Msg::DataBase(DBMsg::SearchResultBlurUp),
            )),
            Vec::new(),
        )?;
        self.app.remount(
            Id::DBListSearchTracks,
            Box::new(DBListSearchTracks::new(
                self.config_tui.clone(),
                Msg::DataBase(DBMsg::SearchTracksBlurDown),
                Msg::DataBase(DBMsg::SearchTracksBlurUp),
            )),
            Vec::new(),
        )?;

        Ok(())
    }

    /// Reload database component data.
    pub fn database_reload(&mut self) {
        self.remount_database_search().unwrap();

        self.dw.reset_search_results();
        self.database_sync_tracks_results();
        self.database_sync_results();
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
        indexable_songs: &'a [T],
        input: &'a str,
    ) -> impl Iterator<Item = &'a T> {
        let search = format!("*{}*", input.to_lowercase());
        indexable_songs
            .iter()
            .filter(move |&record| Model::match_record(record, &search))
    }

    pub fn build_table<T: Matchable, I: Iterator<Item = T>>(
        data: I,
        config: &SharedTuiSettings,
    ) -> Table {
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

    pub fn database_update_search(&mut self, input: &str) {
        let mut db_tracks = Vec::new();
        let all_tracks =
            track_ops::get_all_tracks(&self.db.get_connection(), track_ops::RowOrdering::IdAsc);
        if let Ok(all_tracks) = all_tracks {
            db_tracks = all_tracks;
        }

        let filtered_music = Model::update_search(&db_tracks, input);
        self.general_search_update_show(Model::build_table(filtered_music, &self.config_tui));
    }

    /// Mount the [`AddAlbumConfirm`] popup
    pub fn mount_results_add_confirm_database(&mut self, criteria: SearchCriteria) {
        self.app
            .remount(
                Id::DatabaseAddConfirmPopup,
                Box::new(AddAlbumConfirm::new(
                    self.config_tui.clone(),
                    criteria.as_str(),
                )),
                Vec::new(),
            )
            .unwrap();

        self.app.active(&Id::DatabaseAddConfirmPopup).unwrap();
    }

    /// Unmount the [`AddAlbumConfirm`] popup
    pub fn umount_results_add_confirm_database(&mut self) {
        let _ = self.app.umount(&Id::DatabaseAddConfirmPopup);
    }
}
