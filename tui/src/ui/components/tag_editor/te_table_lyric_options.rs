use anyhow::{Context, Result, anyhow};
use termusiclib::config::SharedTuiSettings;
use termusiclib::ids::{Id, IdTagEditor};
use termusiclib::songtag::{SongTag, SongtagSearchResult, search};
use tokio::runtime::Handle;
use tui_realm_stdlib::Table;
use tuirealm::command::{Cmd, CmdResult, Direction, Position};
use tuirealm::event::{Key, KeyEvent, KeyModifiers};
use tuirealm::props::{Alignment, BorderType, Borders, TableBuilder, TextSpan};
use tuirealm::{Component, Event, MockComponent, State, StateValue};

use crate::ui::Model;
use crate::ui::model::UserEvent;
use crate::ui::msg::{Msg, TEMsg, TFMsg};

#[derive(MockComponent)]
pub struct TETableLyricOptions {
    component: Table,
    config: SharedTuiSettings,
}

impl TETableLyricOptions {
    pub fn new(config: SharedTuiSettings) -> Self {
        let component = {
            let config = config.read();
            Table::default()
                .borders(
                    Borders::default()
                        .modifiers(BorderType::Rounded)
                        .color(config.settings.theme.library_border()),
                )
                .foreground(config.settings.theme.library_foreground())
                .background(config.settings.theme.library_background())
                .title(" Search Results ", Alignment::Left)
                .scroll(true)
                .highlighted_color(config.settings.theme.library_highlight())
                .highlighted_str("\u{1f680}")
                // .highlighted_str("🚀")
                .rewind(false)
                .step(4)
                .row_height(1)
                .headers(["Artist", "Title", "Album", "api", "Copyright Info"])
                .column_spacing(1)
                .widths(&[20, 20, 20, 10, 30])
                .table(
                    TableBuilder::default()
                        .add_col(TextSpan::from("0"))
                        .add_col(TextSpan::from(" "))
                        .add_col(TextSpan::from("No Results."))
                        .build(),
                )
        };

        Self { component, config }
    }
}

impl Component<Msg, UserEvent> for TETableLyricOptions {
    fn on(&mut self, ev: Event<UserEvent>) -> Option<Msg> {
        let config = self.config.clone();
        let keys = &config.read().settings.keys;
        let cmd_result = match ev {
            Event::Keyboard(KeyEvent { code: Key::Tab, .. }) => {
                return Some(Msg::TagEditor(TEMsg::Focus(
                    TFMsg::TableLyricOptionsBlurDown,
                )));
            }
            Event::Keyboard(KeyEvent {
                code: Key::BackTab,
                modifiers: KeyModifiers::SHIFT,
            }) => {
                return Some(Msg::TagEditor(TEMsg::Focus(TFMsg::TableLyricOptionsBlurUp)));
            }

            Event::Keyboard(keyevent) if keyevent == keys.config_keys.save.get() => {
                return Some(Msg::TagEditor(TEMsg::Save));
            }

            Event::Keyboard(k) if k == keys.quit.get() => {
                return Some(Msg::TagEditor(TEMsg::Close));
            }
            Event::Keyboard(k) if k == keys.escape.get() => {
                return Some(Msg::TagEditor(TEMsg::Close));
            }
            Event::Keyboard(KeyEvent {
                code: Key::Down, ..
            }) => self.perform(Cmd::Move(Direction::Down)),
            Event::Keyboard(KeyEvent { code: Key::Up, .. }) => {
                self.perform(Cmd::Move(Direction::Up))
            }
            Event::Keyboard(k) if k == keys.navigation_keys.down.get() => {
                self.perform(Cmd::Move(Direction::Down))
            }
            Event::Keyboard(k) if k == keys.navigation_keys.up.get() => {
                self.perform(Cmd::Move(Direction::Up))
            }
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

            Event::Keyboard(k) if k == keys.navigation_keys.goto_top.get() => {
                self.perform(Cmd::GoTo(Position::Begin))
            }

            Event::Keyboard(k) if k == keys.navigation_keys.goto_bottom.get() => {
                self.perform(Cmd::GoTo(Position::End))
            }
            Event::Keyboard(k) if k == keys.library_keys.youtube_search.get() => {
                if let State::One(StateValue::Usize(index)) = self.state() {
                    return Some(Msg::TagEditor(TEMsg::Download(index)));
                }
                CmdResult::None
            }
            Event::Keyboard(KeyEvent {
                code: Key::Enter, ..
            }) => {
                if let State::One(StateValue::Usize(index)) = self.state() {
                    return Some(Msg::TagEditor(TEMsg::Embed(index)));
                }
                CmdResult::None
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
    /// Apply the given items as the songtag results
    pub fn te_set_songtag_lyric_options(&mut self, items: Vec<SongTag>) {
        // dont do anything if not mounted
        if !self
            .app
            .mounted(&Id::TagEditor(IdTagEditor::TableLyricOptions))
        {
            return;
        }
        self.songtag_options = items;
        self.te_sync_songtag_options();
        assert!(
            self.app
                .active(&Id::TagEditor(IdTagEditor::TableLyricOptions))
                .is_ok()
        );
    }

    /// Build the Songtag Results table and assign it
    fn te_sync_songtag_options(&mut self) {
        let mut table: TableBuilder = TableBuilder::default();

        for (idx, record) in self.songtag_options.iter().enumerate() {
            if idx > 0 {
                table.add_row();
            }
            let artist = record.artist().unwrap_or("Nobody");
            let title = record.title().unwrap_or("Unknown Title");
            let album = record.album().unwrap_or("Unknown Album");
            let api = record.service_provider().to_string();

            let url = match record.url() {
                Some(termusiclib::songtag::UrlTypes::Protected) => "Copyright Protected",
                Some(_) => "Downloadable",
                None => "No URL",
            };

            table
                .add_col(TextSpan::new(artist).fg(tuirealm::ratatui::style::Color::LightYellow))
                .add_col(TextSpan::new(title).bold())
                .add_col(TextSpan::new(album))
                .add_col(TextSpan::new(api))
                .add_col(TextSpan::new(url));
        }
        let table = table.build();
        self.te_set_results(table).unwrap();
    }

    /// Set TagEditor "Search Results" attribute consistently
    fn te_set_results(&mut self, table: tuirealm::props::Table) -> Result<()> {
        self.app.attr(
            &Id::TagEditor(IdTagEditor::TableLyricOptions),
            tuirealm::Attribute::Content,
            tuirealm::AttrValue::Table(table),
        )?;

        Ok(())
    }

    /// Set TagEditor "Search Results" to "Loading"
    fn te_set_loading_results(&mut self) {
        let table = TableBuilder::default()
            .add_col(TextSpan::from("0"))
            .add_col(TextSpan::from(" "))
            .add_col(TextSpan::from("Loading..."))
            .build();

        self.te_set_results(table).unwrap();
    }

    pub fn te_songtag_search(&mut self) {
        let mut search_str = String::new();
        if let Ok(State::One(StateValue::String(artist))) =
            self.app.state(&Id::TagEditor(IdTagEditor::InputArtist))
        {
            search_str.push_str(&artist);
        }
        search_str.push(' ');
        if let Ok(State::One(StateValue::String(title))) =
            self.app.state(&Id::TagEditor(IdTagEditor::InputTitle))
        {
            search_str.push_str(&title);
        }

        if search_str.len() < 4 {
            if let Some(song) = &self.tageditor_song {
                if let Some(stem) = song.path().file_stem() {
                    search_str = stem.to_string_lossy().to_string();
                }
            }
        }

        let handle = Handle::current();

        self.te_set_loading_results();
        self.download_tracker.increase_one(&search_str);

        let songtag_tx = self.tx_to_main.clone();
        let tracker_handle = self.download_tracker.clone();

        handle.spawn(async move {
            search(&search_str, move |msg| {
                let _ = songtag_tx.send(Msg::TagEditor(TEMsg::SearchLyricResult(msg)));
            })
            .await;
            tracker_handle.decrease_one(&search_str);
        });
    }

    /// Handle [`SongtagSearchResult`] events
    pub fn te_update_lyric_results(&mut self, result: SongtagSearchResult) {
        match result {
            SongtagSearchResult::Finish(list) => {
                self.te_set_songtag_lyric_options(list);
            }
        }
    }

    pub fn te_songtag_download(&mut self, index: usize) -> Result<()> {
        let song_tag = self
            .songtag_options
            .get(index)
            .with_context(|| format!("no song_tag with index {index} found"))?;
        if let Some(song) = &self.tageditor_song {
            let file = song.path();
            // this needs to be wrapped as this is not running another thread but some main-runtime thread and so needs to inform the runtime to hand-off other tasks
            // though i am not fully sure if that is 100% the case, this avoid the panic though
            let tx_to_main = self.tx_to_main.clone();
            tokio::task::block_in_place(move || {
                Handle::current().block_on(song_tag.download(file, move |msg| {
                    let _ = tx_to_main.send(Msg::Download(msg));
                }))
            })?;
        }
        Ok(())
    }
    pub fn te_rename_song_by_tag(&mut self) -> Result<()> {
        if let Some(mut song) = self.tageditor_song.clone() {
            if let Ok(State::One(StateValue::String(artist))) =
                self.app.state(&Id::TagEditor(IdTagEditor::InputArtist))
            {
                song.set_artist(&artist);
            }
            if let Ok(State::One(StateValue::String(title))) =
                self.app.state(&Id::TagEditor(IdTagEditor::InputTitle))
            {
                song.set_title(&title);
            }

            if let Ok(State::One(StateValue::String(album))) =
                self.app.state(&Id::TagEditor(IdTagEditor::InputAlbum))
            {
                song.set_album(&album);
            }
            if let Ok(State::One(StateValue::String(genre))) =
                self.app.state(&Id::TagEditor(IdTagEditor::InputGenre))
            {
                song.set_genre(&genre);
            }
            song.save_tag()?;
            // the unwrap should also never happen as all components should be properly mounted
            self.init_by_song(song).unwrap();
            self.playlist_update_library_delete();
        }
        Ok(())
    }

    pub fn te_load_lyric_and_photo(&mut self, index: usize) -> Result<()> {
        if self.songtag_options.is_empty() {
            return Ok(());
        }
        if let Some(mut song) = self.tageditor_song.clone() {
            let song_tag = self
                .songtag_options
                .get(index)
                .ok_or_else(|| anyhow!("cannot get songtag"))?;
            let lang_ext = song_tag.lang_ext().unwrap_or("eng");
            if let Some(artist) = song_tag.artist() {
                song.set_artist(artist);
            }
            if let Some(title) = song_tag.title() {
                song.set_title(title);
            }
            if let Some(album) = song_tag.album() {
                song.set_album(album);
            }

            let tracker_id = song_tag.id();
            self.download_tracker.increase_one(tracker_id);

            // TODO: consider a way to not do it "block_on"
            // this needs to be wrapped as this is not running another thread but some main-runtime thread and so needs to inform the runtime to hand-off other tasks
            // though i am not fully sure if that is 100% the case, this avoid the panic though
            let (lyric_string, artwork) = tokio::task::block_in_place(move || {
                Handle::current().block_on(async {
                    tokio::join!(song_tag.fetch_lyric(), song_tag.fetch_photo())
                })
            });

            self.download_tracker.decrease_one(tracker_id);

            if let Ok(Some(lyric_string)) = lyric_string {
                song.set_lyric(&lyric_string, lang_ext, None::<String>);
            }
            if let Ok(artwork) = artwork {
                song.set_picture(artwork);
            }

            song.save_tag()?;
            // the unwrap should also never happen as all components should be properly mounted
            self.init_by_song(song).unwrap();
            self.playlist_update_library_delete();
            // self.library_sync(song.file());
        }
        Ok(())
    }
}
