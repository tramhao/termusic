use std::path::Path;

use termusiclib::config::SharedTuiSettings;
use termusiclib::ids::Id;
use termusiclib::library_db::const_unknown::{UNKNOWN_ARTIST, UNKNOWN_FILE, UNKNOWN_TITLE};
use termusiclib::library_db::{Indexable, SearchCriteria, TrackDB};
use termusiclib::track::DurationFmtShort;
use termusiclib::types::{DBMsg, GSMsg, Msg};
use termusiclib::utils::{is_playlist, playlist_get_vec};
use tui_realm_stdlib::List;
use tuirealm::command::{Cmd, CmdResult, Direction, Position};
use tuirealm::props::Borders;
use tuirealm::props::{Alignment, BorderType, Table, TableBuilder, TextSpan};
use tuirealm::{
    event::{Key, KeyEvent, KeyModifiers, NoUserEvent},
    AttrValue, Attribute, Component, Event, MockComponent, State, StateValue,
};

use super::popups::{YNConfirm, YNConfirmStyle};
use crate::ui::Model;

#[derive(MockComponent)]
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
                .title(" DataBase ", Alignment::Left)
                .scroll(true)
                .highlighted_color(config.settings.theme.library_highlight())
                .highlighted_str(&config.settings.theme.style.library.highlight_symbol)
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
                        .add_row()
                        .add_col(TextSpan::from("Playlists"))
                        .build(),
                )
        };

        Self {
            component,
            on_key_tab,
            on_key_backtab,
            config,
        }
    }
}

impl Component<Msg, NoUserEvent> for DBListCriteria {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        let config = self.config.clone();
        let keys = &config.read().settings.keys;
        let cmd_result = match ev {
            Event::Keyboard(KeyEvent {
                code: Key::Down,
                modifiers: KeyModifiers::NONE,
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
            Event::Keyboard(KeyEvent {
                code: Key::Up,
                modifiers: KeyModifiers::NONE,
            }) => self.perform(Cmd::Move(Direction::Up)),
            Event::Keyboard(key) if key == keys.navigation_keys.down.get() => {
                if let Some(AttrValue::Table(t)) = self.query(Attribute::Content) {
                    if let State::One(StateValue::Usize(index)) = self.state() {
                        if index >= t.len() - 1 {
                            return Some(self.on_key_tab.clone());
                        }
                    }
                }
                self.perform(Cmd::Move(Direction::Down))
            }
            Event::Keyboard(key) if key == keys.navigation_keys.up.get() => {
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
            Event::Keyboard(key) if key == keys.navigation_keys.goto_top.get() => {
                self.perform(Cmd::GoTo(Position::Begin))
            }
            Event::Keyboard(key) if key == keys.navigation_keys.goto_bottom.get() => {
                self.perform(Cmd::GoTo(Position::End))
            }
            Event::Keyboard(KeyEvent {
                code: Key::Home,
                modifiers: KeyModifiers::NONE,
            }) => self.perform(Cmd::GoTo(Position::Begin)),

            Event::Keyboard(KeyEvent {
                code: Key::Enter,
                modifiers: KeyModifiers::NONE,
            }) => {
                if let State::One(StateValue::Usize(index)) = self.state() {
                    return Some(Msg::DataBase(DBMsg::SearchResult(index)));
                }
                CmdResult::None
            }
            Event::Keyboard(KeyEvent {
                code: Key::End,
                modifiers: KeyModifiers::NONE,
            }) => self.perform(Cmd::GoTo(Position::End)),
            Event::Keyboard(KeyEvent {
                code: Key::Tab,
                modifiers: KeyModifiers::NONE,
            }) => return Some(self.on_key_tab.clone()),
            Event::Keyboard(KeyEvent {
                code: Key::BackTab,
                modifiers: KeyModifiers::SHIFT,
            }) => return Some(self.on_key_backtab.clone()),

            Event::Keyboard(keyevent) if keyevent == keys.library_keys.search.get() => {
                return Some(Msg::GeneralSearch(GSMsg::PopupShowDatabase))
            }
            _ => CmdResult::None,
        };
        match cmd_result {
            CmdResult::None => None,
            _ => Some(Msg::ForceRedraw),
        }
    }
}

/// Component for a "Are you sure you want to add ALL found albums? Y/N" popup
#[derive(MockComponent)]
pub struct AddAlbumConfirm {
    component: YNConfirm,
}

impl AddAlbumConfirm {
    pub fn new(config: SharedTuiSettings, criteria: &str) -> Self {
        let component = YNConfirm::new_with_cb(
            config,
            format!(" Are you sure you want to add EVERYTHING from {criteria}? ",),
            |config| YNConfirmStyle {
                foreground_color: config.settings.theme.important_popup_foreground(),
                background_color: config.settings.theme.important_popup_background(),
                border_color: config.settings.theme.important_popup_border(),
                title_alignment: Alignment::Left,
            },
        );

        Self { component }
    }
}

impl Component<Msg, NoUserEvent> for AddAlbumConfirm {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(
            ev,
            Msg::DataBase(DBMsg::AddAllResultsToPlaylist),
            Msg::DataBase(DBMsg::AddAllResultsConfirmCancel),
        )
    }
}

#[derive(MockComponent)]
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
                .title(" Result ", Alignment::Left)
                .scroll(true)
                .highlighted_color(config.settings.theme.library_highlight())
                .highlighted_str(&config.settings.theme.style.library.highlight_symbol)
                .rewind(false)
                .step(4)
                .scroll(true)
                .rows(
                    TableBuilder::default()
                        .add_col(TextSpan::from("Empty"))
                        .build(),
                )
        };

        Self {
            component,
            on_key_tab,
            on_key_backtab,
            config,
        }
    }
}

impl Component<Msg, NoUserEvent> for DBListSearchResult {
    #[allow(clippy::too_many_lines)]
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        let config = self.config.clone();
        let keys = &config.read().settings.keys;
        let cmd_result = match ev {
            Event::Keyboard(KeyEvent {
                code: Key::Down,
                modifiers: KeyModifiers::NONE,
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
            Event::Keyboard(KeyEvent {
                code: Key::Up,
                modifiers: KeyModifiers::NONE,
            }) => {
                if let State::One(StateValue::Usize(index)) = self.state() {
                    if index == 0 {
                        return Some(self.on_key_backtab.clone());
                    }
                }
                self.perform(Cmd::Move(Direction::Up))
            }
            Event::Keyboard(key) if key == keys.navigation_keys.down.get() => {
                if let Some(AttrValue::Table(t)) = self.query(Attribute::Content) {
                    if let State::One(StateValue::Usize(index)) = self.state() {
                        if index >= t.len() - 1 {
                            return Some(self.on_key_tab.clone());
                        }
                    }
                }
                self.perform(Cmd::Move(Direction::Down))
            }
            Event::Keyboard(key) if key == keys.navigation_keys.up.get() => {
                if let State::One(StateValue::Usize(index)) = self.state() {
                    if index == 0 {
                        return Some(self.on_key_backtab.clone());
                    }
                }
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
            Event::Keyboard(key) if key == keys.navigation_keys.goto_top.get() => {
                self.perform(Cmd::GoTo(Position::Begin))
            }
            Event::Keyboard(key) if key == keys.navigation_keys.goto_bottom.get() => {
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
                code: Key::Enter,
                modifiers: KeyModifiers::NONE,
            }) => {
                if let State::One(StateValue::Usize(index)) = self.state() {
                    return Some(Msg::DataBase(DBMsg::SearchTrack(index)));
                }
                CmdResult::None
            }
            Event::Keyboard(KeyEvent {
                code: Key::Tab,
                modifiers: KeyModifiers::NONE,
            }) => return Some(self.on_key_tab.clone()),
            Event::Keyboard(KeyEvent {
                code: Key::BackTab,
                modifiers: KeyModifiers::SHIFT,
            }) => return Some(self.on_key_backtab.clone()),

            Event::Keyboard(keyevent) if keyevent == keys.library_keys.search.get() => {
                return Some(Msg::GeneralSearch(GSMsg::PopupShowDatabase))
            }

            Event::Keyboard(keyevent) if keyevent == keys.database_keys.add_selected.get() => {
                if let State::One(StateValue::Usize(index)) = self.state() {
                    // Maybe we should also have a popup if it is anything other that "Album"?
                    // Because things like "Genre" could be *very* big
                    return Some(Msg::DataBase(DBMsg::AddResultToPlaylist(index)));
                }
                CmdResult::None
            }
            Event::Keyboard(keyevent) if keyevent == keys.database_keys.add_all.get() => {
                return Some(Msg::DataBase(DBMsg::AddAllResultsConfirmShow))
            }

            _ => CmdResult::None,
        };
        match cmd_result {
            CmdResult::None => None,
            _ => Some(Msg::ForceRedraw),
        }
    }
}

#[derive(MockComponent)]
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
                .title(" Tracks ", Alignment::Left)
                .scroll(true)
                .highlighted_color(config.settings.theme.library_highlight())
                .highlighted_str(&config.settings.theme.style.library.highlight_symbol)
                .rewind(false)
                .step(4)
                .scroll(true)
                .rows(
                    TableBuilder::default()
                        .add_col(TextSpan::from("Empty"))
                        .build(),
                )
        };

        Self {
            component,
            on_key_tab,
            on_key_backtab,
            config,
        }
    }
}

impl Component<Msg, NoUserEvent> for DBListSearchTracks {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        let config = self.config.clone();
        let keys = &config.read().settings.keys;
        let cmd_result = match ev {
            Event::Keyboard(KeyEvent {
                code: Key::Down,
                modifiers: KeyModifiers::NONE,
            }) => self.perform(Cmd::Move(Direction::Down)),
            Event::Keyboard(KeyEvent {
                code: Key::Up,
                modifiers: KeyModifiers::NONE,
            }) => {
                if let State::One(StateValue::Usize(index)) = self.state() {
                    if index == 0 {
                        return Some(self.on_key_backtab.clone());
                    }
                }
                self.perform(Cmd::Move(Direction::Up))
            }
            Event::Keyboard(key) if key == keys.navigation_keys.down.get() => {
                self.perform(Cmd::Move(Direction::Down))
            }
            Event::Keyboard(key) if key == keys.navigation_keys.up.get() => {
                if let State::One(StateValue::Usize(index)) = self.state() {
                    if index == 0 {
                        return Some(self.on_key_backtab.clone());
                    }
                }
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
            Event::Keyboard(key) if key == keys.navigation_keys.goto_top.get() => {
                self.perform(Cmd::GoTo(Position::Begin))
            }
            Event::Keyboard(key) if key == keys.navigation_keys.goto_bottom.get() => {
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
            }) => return Some(self.on_key_tab.clone()),

            Event::Keyboard(KeyEvent {
                code: Key::BackTab,
                modifiers: KeyModifiers::SHIFT,
            }) => return Some(self.on_key_backtab.clone()),

            Event::Keyboard(keyevent) if keyevent == keys.database_keys.add_selected.get() => {
                if let State::One(StateValue::Usize(index)) = self.state() {
                    return Some(Msg::DataBase(DBMsg::AddPlaylist(index)));
                }
                CmdResult::None
            }
            Event::Keyboard(keyevent) if keyevent == keys.database_keys.add_all.get() => {
                return Some(Msg::DataBase(DBMsg::AddAllToPlaylist))
            }

            Event::Keyboard(keyevent) if keyevent == keys.library_keys.search.get() => {
                return Some(Msg::GeneralSearch(GSMsg::PopupShowDatabase))
            }

            _ => CmdResult::None,
        };
        match cmd_result {
            CmdResult::None => None,
            _ => Some(Msg::ForceRedraw),
        }
    }
}

impl Model {
    pub fn database_sync_tracks(&mut self) {
        let mut table: TableBuilder = TableBuilder::default();

        for (idx, record) in self.dw.search_tracks.iter().enumerate() {
            if idx > 0 {
                table.add_row();
            }

            let name = {
                // TODO: refactor this once "title" can be optional
                // this check likely is never empty, because "Unknown"(or similar) is stored in the db
                if record.title.is_empty() {
                    record.name.clone()
                } else {
                    record.title.clone()
                }
            };

            table
                .add_col(TextSpan::from(format!("{}", idx + 1)))
                .add_col(TextSpan::from(" "))
                .add_col(TextSpan::from(name));
        }
        if self.dw.search_results.is_empty() {
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
        let mut index = 0;
        for (idx, record) in self.dw.search_results.iter().enumerate() {
            let mut display_name = String::new();
            match self.dw.criteria {
                SearchCriteria::Playlist => {
                    let path = Path::new(record);
                    let path_string = path.to_string_lossy().to_string();
                    let mut vec = path_string.split('/');
                    if let Some(v1) = vec.next_back() {
                        if let Some(v2) = vec.next_back() {
                            display_name = format!("{v2}/{v1}");
                        }
                    }
                }
                SearchCriteria::Directory => {
                    let path = Path::new(record);
                    let path_string = path.to_string_lossy().to_string();
                    let mut vec = path_string.split('/');
                    if let Some(v1) = vec.next_back() {
                        if let Some(v2) = vec.next_back() {
                            display_name = format!("{v2}/{v1}/");
                        }
                    }
                }
                _ => {
                    display_name.clone_from(record);
                }
            }
            if !display_name.is_empty() {
                if idx > 0 {
                    table.add_row();
                }
                index += 1;
                table
                    .add_col(TextSpan::from(format!("{index}")))
                    .add_col(TextSpan::from(" "))
                    .add_col(TextSpan::from(display_name));
            }
        }
        if self.dw.search_results.is_empty() {
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
        match self.dw.criteria {
            SearchCriteria::Playlist => {
                self.dw.search_results = self.database_get_playlist();
            }
            _ => {
                if let Ok(results) = self.db.get_criterias(&self.dw.criteria) {
                    self.dw.search_results = results;
                }
            }
        }
        self.database_sync_results();
        self.app.active(&Id::DBListSearchResult).ok();
    }

    fn database_get_playlist(&self) -> Vec<String> {
        let mut vec = Vec::new();

        let root = self.library.tree.root();
        let p: &Path = Path::new(root.id());
        let all_items = walkdir::WalkDir::new(p).follow_links(true);
        for record in all_items
            .into_iter()
            .filter_map(std::result::Result::ok)
            .filter(|p| is_playlist(&p.path().to_string_lossy()))
        {
            let full_path_name = record.path().to_string_lossy().to_string();
            vec.push(full_path_name);
        }
        vec
    }

    /// Find all tracks for the given [`criteria`](SearchCriteria) which matches `val`.
    ///
    /// Or for the [`Playlist`](SearchCriteria::Playlist) case, `val` is the path of the playlist.
    pub fn database_get_tracks_by_criteria(
        &mut self,
        criteria: SearchCriteria,
        val: &str,
    ) -> Option<Vec<TrackDB>> {
        match criteria {
            SearchCriteria::Playlist => {
                if let Ok(vec) = playlist_get_vec(val) {
                    let mut vec_db = Vec::with_capacity(vec.len());
                    for item in vec {
                        if let Ok(i) = self.db.get_record_by_path(&item) {
                            vec_db.push(i);
                        }
                    }
                    return Some(vec_db);
                }
            }
            _ => {
                return self.db.get_record_by_criteria(val, &criteria).ok();
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

        self.database_sync_tracks();
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

    pub fn database_reload(&mut self) {
        assert!(self
            .app
            .remount(
                Id::DBListCriteria,
                Box::new(DBListCriteria::new(
                    self.config_tui.clone(),
                    Msg::DataBase(DBMsg::CriteriaBlurDown),
                    Msg::DataBase(DBMsg::CriteriaBlurUp)
                )),
                Vec::new()
            )
            .is_ok());

        assert!(self
            .app
            .remount(
                Id::DBListSearchResult,
                Box::new(DBListSearchResult::new(
                    self.config_tui.clone(),
                    Msg::DataBase(DBMsg::SearchResultBlurDown),
                    Msg::DataBase(DBMsg::SearchResultBlurUp)
                )),
                Vec::new()
            )
            .is_ok());
        assert!(self
            .app
            .remount(
                Id::DBListSearchTracks,
                Box::new(DBListSearchTracks::new(
                    self.config_tui.clone(),
                    Msg::DataBase(DBMsg::SearchTracksBlurDown),
                    Msg::DataBase(DBMsg::SearchTracksBlurUp)
                )),
                Vec::new()
            )
            .is_ok());

        self.dw.reset_search_results();
        self.database_sync_tracks();
        self.database_sync_results();
    }

    fn match_record<T: Indexable>(record: &T, search: &str) -> bool {
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

    pub fn update_search<'a, T: Indexable>(
        indexable_songs: &'a [T],
        input: &'a str,
    ) -> impl Iterator<Item = &'a T> {
        let search = format!("*{}*", input.to_lowercase());
        indexable_songs
            .iter()
            .filter(move |&record| Model::match_record(record, &search))
    }

    pub fn build_table<T: Indexable, I: Iterator<Item = T>>(data: I) -> Table {
        let mut peekable_data = data.peekable();
        let mut table: TableBuilder = TableBuilder::default();
        if peekable_data.peek().is_none() {
            table.add_col(TextSpan::from("0"));
            table.add_col(TextSpan::from("empty tracks from db/playlist"));
            table.add_col(TextSpan::from(""));
            return table.build();
        }

        for (idx, record) in peekable_data.enumerate() {
            if idx > 0 {
                table.add_row();
            }

            let duration = DurationFmtShort(record.meta_duration());
            let duration_string = format!("[{duration:^6.6}]");

            table
                .add_col(TextSpan::new(duration_string))
                .add_col(
                    TextSpan::new(record.meta_artist().unwrap_or(UNKNOWN_ARTIST))
                        .fg(tuirealm::ratatui::style::Color::LightYellow),
                )
                .add_col(TextSpan::new(record.meta_title().unwrap_or(UNKNOWN_TITLE)).bold())
                .add_col(TextSpan::new(record.meta_file().unwrap_or(UNKNOWN_FILE)));
        }
        table.build()
    }

    pub fn database_update_search(&mut self, input: &str) {
        let mut db_tracks = vec![];
        if let Ok(tracks) = self.db.get_all_records() {
            db_tracks.clone_from(&tracks);
        }

        let filtered_music = Model::update_search(&db_tracks, input);
        self.general_search_update_show(Model::build_table(filtered_music));
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
