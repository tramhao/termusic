use crate::ui::Model;
use std::path::Path;
use termusiclib::config::SharedTuiSettings;
use termusiclib::library_db::const_unknown::{UNKNOWN_ARTIST, UNKNOWN_FILE, UNKNOWN_TITLE};
use termusiclib::library_db::{Indexable, SearchCriteria};
use termusiclib::types::{DBMsg, Id, Msg};
use termusiclib::utils::{is_playlist, playlist_get_vec};
use tui_realm_stdlib::List;
use tuirealm::command::{Cmd, CmdResult, Direction, Position};
use tuirealm::props::Borders;
use tuirealm::props::{Alignment, BorderType, Table, TableBuilder, TextSpan};
use tuirealm::{
    event::{Key, KeyEvent, KeyModifiers, NoUserEvent},
    AttrValue, Attribute, Component, Event, MockComponent, State, StateValue,
};

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
                return Some(Msg::GeneralSearch(crate::ui::GSMsg::PopupShowDatabase))
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
                return Some(Msg::GeneralSearch(crate::ui::GSMsg::PopupShowDatabase))
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
                return Some(Msg::GeneralSearch(crate::ui::GSMsg::PopupShowDatabase))
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
            };
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

    pub fn database_update_search_tracks(&mut self, index: usize) {
        match self.dw.criteria {
            SearchCriteria::Playlist => {
                if let Some(result) = self.dw.search_results.get(index) {
                    if let Ok(vec) = playlist_get_vec(result) {
                        let mut vec_db = Vec::new();
                        for item in vec {
                            if let Ok(i) = self.db.get_record_by_path(&item) {
                                vec_db.push(i);
                            }
                        }
                        self.dw.search_tracks = vec_db;
                    }
                }
            }
            _ => {
                if let Ok(vec) = self
                    .db
                    .get_record_by_criteria(&self.dw.search_results[index], &self.dw.criteria)
                {
                    self.dw.search_tracks = vec;
                };
            }
        }

        self.database_sync_tracks();
        self.app.active(&Id::DBListSearchTracks).ok();
    }

    #[allow(unused)]
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

    pub fn update_search<'a, T: Indexable>(indexable_songs: &'a Vec<T>, input: &str) -> Vec<&'a T> {
        let mut filtered_records = vec![];
        let search = format!("*{}*", input.to_lowercase());
        for record in indexable_songs {
            let artist_match: bool = if let Some(artist) = record.meta_artist() {
                wildmatch::WildMatch::new(&search).matches(&artist.to_lowercase())
            } else {
                false
            };
            let title_match: bool = if let Some(title) = record.meta_title() {
                wildmatch::WildMatch::new(&search).matches(&title.to_lowercase())
            } else {
                false
            };
            let album_match: bool = if let Some(album) = record.meta_album() {
                wildmatch::WildMatch::new(&search).matches(&album.to_lowercase())
            } else {
                false
            };
            if artist_match || title_match || album_match {
                filtered_records.push(record);
            }
        }
        filtered_records
    }

    pub fn build_table<T: Indexable>(data: &[&T]) -> Table {
        let mut table: TableBuilder = TableBuilder::default();
        if data.is_empty() {
            table.add_col(TextSpan::from("0"));
            table.add_col(TextSpan::from("empty tracks from db/playlist"));
            table.add_col(TextSpan::from(""));
            return table.build();
        }

        for (idx, record) in data.iter().enumerate() {
            if idx > 0 {
                table.add_row();
            }

            let duration = termusiclib::track::Track::duration_formatted_short(&record.duration());
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
        self.general_search_update_show(Model::build_table(&filtered_music));
    }
}
