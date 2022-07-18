use crate::config::Settings;
use crate::ui::components::{
    DBListCriteria, DBListSearchResult, DBListSearchTracks, DeleteConfirmInputPopup,
    DeleteConfirmRadioPopup, DownloadSpinner, ErrorPopup, GSInputPopup, GSTablePopup,
    GlobalListener, HelpPopup, LabelGeneric, LabelSpan, Lyric, MessagePopup, MusicLibrary,
    Playlist, Progress, QuitPopup, Source, TECounterDelete, TEHelpPopup, TEInputArtist,
    TEInputTitle, TERadioTag, TESelectLyric, TETableLyricOptions, TETextareaLyric, YSInputPopup,
    YSTablePopup,
};
use crate::utils::{draw_area_in_absolute, draw_area_in_relative, draw_area_top_right_absolute};

use crate::ui::model::{ConfigEditorLayout, Model, TermusicLayout};
use crate::{
    track::Track,
    ui::{Application, DBMsg, Id, IdConfigEditor, IdTagEditor, Msg},
    VERSION,
};
use std::convert::TryFrom;
use std::path::Path;
use std::time::{Duration, Instant};
use tui_realm_treeview::Tree;
use tuirealm::event::NoUserEvent;
use tuirealm::props::{Alignment, AttrValue, Attribute, Color, PropPayload, PropValue, TextSpan};
use tuirealm::tui::layout::{Constraint, Direction, Layout};
use tuirealm::tui::widgets::Clear;
use tuirealm::EventListenerCfg;
use tuirealm::Frame;

impl Model {
    pub fn init_app(tree: &Tree, config: &Settings) -> Application<Id, Msg, NoUserEvent> {
        // Setup application
        // NOTE: NoUserEvent is a shorthand to tell tui-realm we're not going to use any custom user event
        // NOTE: the event listener is configured to use the default crossterm input listener and to raise a Tick event each second
        // which we will use to update the clock

        let mut app: Application<Id, Msg, NoUserEvent> = Application::init(
            EventListenerCfg::default()
                .default_input_listener(Duration::from_millis(20))
                .poll_timeout(Duration::from_millis(10))
                .tick_interval(Duration::from_secs(1)),
        );
        assert!(app
            .mount(
                Id::Library,
                Box::new(MusicLibrary::new(tree, None, config)),
                vec![]
            )
            .is_ok());
        assert!(app
            .mount(
                Id::DBListCriteria,
                Box::new(DBListCriteria::new(
                    config,
                    Msg::DataBase(DBMsg::CriteriaBlurDown),
                    Msg::DataBase(DBMsg::CriteriaBlurUp)
                )),
                vec![]
            )
            .is_ok());

        assert!(app
            .mount(
                Id::DBListSearchResult,
                Box::new(DBListSearchResult::new(
                    config,
                    Msg::DataBase(DBMsg::SearchResultBlurDown),
                    Msg::DataBase(DBMsg::SearchResultBlurUp)
                )),
                vec![]
            )
            .is_ok());
        assert!(app
            .mount(
                Id::DBListSearchTracks,
                Box::new(DBListSearchTracks::new(
                    config,
                    Msg::DataBase(DBMsg::SearchTracksBlurDown),
                    Msg::DataBase(DBMsg::SearchTracksBlurUp)
                )),
                vec![]
            )
            .is_ok());
        assert!(app
            .mount(Id::Playlist, Box::new(Playlist::new(config)), vec![])
            .is_ok());
        assert!(app
            .mount(Id::Progress, Box::new(Progress::new(config)), vec![])
            .is_ok());
        assert!(app
            .mount(Id::Lyric, Box::new(Lyric::new(config)), vec![])
            .is_ok());

        assert!(app
            .mount(
                Id::DownloadSpinner,
                Box::new(DownloadSpinner::new(config)),
                vec![]
            )
            .is_ok());

        // Mount global hotkey listener
        assert!(app
            .mount(
                Id::GlobalListener,
                Box::new(GlobalListener::new(&config.keys)),
                Self::subscribe(&config.keys),
            )
            .is_ok());
        // Active library
        assert!(app.active(&Id::Library).is_ok());
        app
    }

    pub fn view(&mut self) {
        if self.redraw {
            self.redraw = false;
            self.last_redraw = Instant::now();
            if self
                .app
                .mounted(&Id::TagEditor(IdTagEditor::TableLyricOptions))
            {
                self.view_tag_editor();
                return;
            } else if self.app.mounted(&Id::ConfigEditor(IdConfigEditor::Header)) {
                match self.config_layout {
                    ConfigEditorLayout::General => self.view_config_editor_general(),
                    ConfigEditorLayout::Color => self.view_config_editor_color(),
                    ConfigEditorLayout::Key1 => self.view_config_editor_key1(),
                    ConfigEditorLayout::Key2 => self.view_config_editor_key2(),
                }
                return;
            }

            match self.layout {
                TermusicLayout::TreeView => self.view_layout_treeview(),
                TermusicLayout::DataBase => self.view_layout_database(),
            }
        }
    }

    pub fn view_layout_database(&mut self) {
        assert!(self
            .terminal
            .raw_mut()
            .draw(|f| {
                let chunks_main = Layout::default()
                    .direction(Direction::Vertical)
                    .margin(0)
                    .constraints([Constraint::Min(2), Constraint::Length(1)].as_ref())
                    .split(f.size());
                let chunks_left = Layout::default()
                    .direction(Direction::Horizontal)
                    .margin(0)
                    .constraints([Constraint::Ratio(1, 3), Constraint::Ratio(2, 3)].as_ref())
                    .split(chunks_main[0]);

                let chunks_left_sections = Layout::default()
                    .direction(Direction::Vertical)
                    .margin(0)
                    .constraints(
                        [
                            Constraint::Length(10),
                            Constraint::Length(10),
                            Constraint::Min(2),
                        ]
                        .as_ref(),
                    )
                    .split(chunks_left[0]);
                let chunks_right = Layout::default()
                    .direction(Direction::Vertical)
                    .margin(0)
                    .constraints(
                        [
                            Constraint::Min(2),
                            Constraint::Length(3),
                            Constraint::Length(4),
                        ]
                        .as_ref(),
                    )
                    .split(chunks_left[1]);

                self.app
                    .view(&Id::DBListCriteria, f, chunks_left_sections[0]);
                self.app
                    .view(&Id::DBListSearchResult, f, chunks_left_sections[1]);
                self.app
                    .view(&Id::DBListSearchTracks, f, chunks_left_sections[2]);

                self.app.view(&Id::Playlist, f, chunks_right[0]);
                self.app.view(&Id::Progress, f, chunks_right[1]);
                self.app.view(&Id::Lyric, f, chunks_right[2]);
                Self::view_layout_commons(f, &mut self.app, self.downloading_item_quantity);
            })
            .is_ok());
    }

    pub fn view_layout_treeview(&mut self) {
        assert!(self
            .terminal
            .raw_mut()
            .draw(|f| {
                let chunks_main = Layout::default()
                    .direction(Direction::Vertical)
                    .margin(0)
                    .constraints([Constraint::Min(2), Constraint::Length(1)].as_ref())
                    .split(f.size());
                let chunks_left = Layout::default()
                    .direction(Direction::Horizontal)
                    .margin(0)
                    .constraints([Constraint::Ratio(1, 3), Constraint::Ratio(2, 3)].as_ref())
                    .split(chunks_main[0]);
                let chunks_right = Layout::default()
                    .direction(Direction::Vertical)
                    .margin(0)
                    .constraints(
                        [
                            Constraint::Min(2),
                            Constraint::Length(3),
                            Constraint::Length(4),
                        ]
                        .as_ref(),
                    )
                    .split(chunks_left[1]);

                self.app.view(&Id::Library, f, chunks_left[0]);
                self.app.view(&Id::Playlist, f, chunks_right[0]);
                self.app.view(&Id::Progress, f, chunks_right[1]);
                self.app.view(&Id::Lyric, f, chunks_right[2]);
                self.app.view(&Id::Label, f, chunks_main[1]);

                Self::view_layout_commons(f, &mut self.app, self.downloading_item_quantity);
            })
            .is_ok());
    }

    fn view_layout_commons(
        f: &mut Frame<'_>,
        app: &mut Application<Id, Msg, NoUserEvent>,
        downloading_item_quantity: usize,
    ) {
        // -- footer
        if downloading_item_quantity > 0 {
            let chunks_main = Layout::default()
                .direction(Direction::Vertical)
                .margin(0)
                .constraints([Constraint::Min(2), Constraint::Length(1)].as_ref())
                .split(f.size());
            let chunks_footer = Layout::default()
                .direction(Direction::Horizontal)
                .margin(0)
                .constraints(
                    [
                        Constraint::Length(1),
                        Constraint::Length(1),
                        Constraint::Min(10),
                    ]
                    .as_ref(),
                )
                .split(chunks_main[1]);

            app.view(&Id::DownloadSpinner, f, chunks_footer[1]);
            // app.view(&Id::LabelCounter, f, chunks_footer[2]);
            app.view(&Id::Label, f, chunks_footer[2]);
        } else {
            let chunks_main = Layout::default()
                .direction(Direction::Vertical)
                .margin(0)
                .constraints([Constraint::Min(2), Constraint::Length(1)].as_ref())
                .split(f.size());
            app.view(&Id::Label, f, chunks_main[1]);
        }

        // -- popups
        if app.mounted(&Id::QuitPopup) {
            let popup = draw_area_in_absolute(f.size(), 30, 3);
            f.render_widget(Clear, popup);
            app.view(&Id::QuitPopup, f, popup);
        } else if app.mounted(&Id::HelpPopup) {
            let popup = draw_area_in_relative(f.size(), 60, 91);
            f.render_widget(Clear, popup);
            app.view(&Id::HelpPopup, f, popup);
        } else if app.mounted(&Id::DeleteConfirmRadioPopup) {
            let popup = draw_area_in_absolute(f.size(), 30, 3);
            f.render_widget(Clear, popup);
            app.view(&Id::DeleteConfirmRadioPopup, f, popup);
        } else if app.mounted(&Id::DeleteConfirmInputPopup) {
            let popup = draw_area_in_absolute(f.size(), 30, 3);
            f.render_widget(Clear, popup);
            app.view(&Id::DeleteConfirmInputPopup, f, popup);
        } else if app.mounted(&Id::GeneralSearchInput) {
            let popup = draw_area_in_relative(f.size(), 65, 68);
            f.render_widget(Clear, popup);
            let popup_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints(
                    [
                        Constraint::Length(3), // Input form
                        Constraint::Min(2),    // Yes/No
                    ]
                    .as_ref(),
                )
                .split(popup);
            app.view(&Id::GeneralSearchInput, f, popup_chunks[0]);
            app.view(&Id::GeneralSearchTable, f, popup_chunks[1]);
        } else if app.mounted(&Id::YoutubeSearchInputPopup) {
            let popup = draw_area_in_absolute(f.size(), 50, 3);
            f.render_widget(Clear, popup);
            app.view(&Id::YoutubeSearchInputPopup, f, popup);
        } else if app.mounted(&Id::YoutubeSearchTablePopup) {
            let popup = draw_area_in_relative(f.size(), 65, 68);
            f.render_widget(Clear, popup);
            app.view(&Id::YoutubeSearchTablePopup, f, popup);
        }
        if app.mounted(&Id::MessagePopup) {
            let popup = draw_area_top_right_absolute(f.size(), 25, 4);
            f.render_widget(Clear, popup);
            app.view(&Id::MessagePopup, f, popup);
        }
        if app.mounted(&Id::ErrorPopup) {
            let popup = draw_area_in_absolute(f.size(), 50, 4);
            f.render_widget(Clear, popup);
            app.view(&Id::ErrorPopup, f, popup);
        }
    }
    // Mount error and give focus to it
    pub fn mount_error_popup(&mut self, err: &str) {
        // pub fn mount_error_popup(&mut self, err: impl ToString) {
        assert!(self
            .app
            .remount(Id::ErrorPopup, Box::new(ErrorPopup::new(err)), vec![])
            .is_ok());
        assert!(self.app.active(&Id::ErrorPopup).is_ok());
        // self.app.lock_subs();
    }
    /// Mount quit popup
    pub fn mount_quit_popup(&mut self) {
        assert!(self
            .app
            .remount(
                Id::QuitPopup,
                Box::new(QuitPopup::new(&self.config)),
                vec![]
            )
            .is_ok());
        assert!(self.app.active(&Id::QuitPopup).is_ok());
        self.app.lock_subs();
    }
    /// Mount help popup
    pub fn mount_help_popup(&mut self) {
        assert!(self
            .app
            .remount(
                Id::HelpPopup,
                Box::new(HelpPopup::new(&self.config)),
                vec![]
            )
            .is_ok());
        assert!(self.app.active(&Id::HelpPopup).is_ok());
        self.app.lock_subs();
    }

    pub fn mount_confirm_radio(&mut self) {
        assert!(self
            .app
            .remount(
                Id::DeleteConfirmRadioPopup,
                Box::new(DeleteConfirmRadioPopup::new(&self.config)),
                vec![]
            )
            .is_ok());
        assert!(self.app.active(&Id::DeleteConfirmRadioPopup).is_ok());
        self.app.lock_subs();
    }

    pub fn mount_confirm_input(&mut self) {
        assert!(self
            .app
            .remount(
                Id::DeleteConfirmInputPopup,
                Box::new(DeleteConfirmInputPopup::new(
                    &self.config.style_color_symbol
                )),
                vec![]
            )
            .is_ok());
        assert!(self.app.active(&Id::DeleteConfirmInputPopup).is_ok());
        self.app.lock_subs();
    }

    pub fn mount_search_library(&mut self) {
        assert!(self
            .app
            .remount(
                Id::GeneralSearchInput,
                Box::new(GSInputPopup::new(Source::Library, &self.config)),
                vec![]
            )
            .is_ok());
        assert!(self
            .app
            .remount(
                Id::GeneralSearchTable,
                Box::new(GSTablePopup::new(Source::Library, &self.config)),
                vec![]
            )
            .is_ok());

        assert!(self.app.active(&Id::GeneralSearchInput).is_ok());
        self.app.lock_subs();
        if let Err(e) = self.update_photo() {
            self.mount_error_popup(format!("update photo error: {}", e).as_ref());
        }
    }

    pub fn mount_search_playlist(&mut self) {
        assert!(self
            .app
            .remount(
                Id::GeneralSearchInput,
                Box::new(GSInputPopup::new(Source::Playlist, &self.config)),
                vec![]
            )
            .is_ok());
        assert!(self
            .app
            .remount(
                Id::GeneralSearchTable,
                Box::new(GSTablePopup::new(Source::Playlist, &self.config)),
                vec![]
            )
            .is_ok());
        assert!(self.app.active(&Id::GeneralSearchInput).is_ok());
        self.app.lock_subs();
        if let Err(e) = self.update_photo() {
            self.mount_error_popup(format!("update photo error: {}", e).as_ref());
        }
    }

    pub fn mount_search_database(&mut self) {
        assert!(self
            .app
            .remount(
                Id::GeneralSearchInput,
                Box::new(GSInputPopup::new(Source::Database, &self.config)),
                vec![]
            )
            .is_ok());
        assert!(self
            .app
            .remount(
                Id::GeneralSearchTable,
                Box::new(GSTablePopup::new(Source::Database, &self.config)),
                vec![]
            )
            .is_ok());
        assert!(self.app.active(&Id::GeneralSearchInput).is_ok());
        self.app.lock_subs();
        if let Err(e) = self.update_photo() {
            self.mount_error_popup(format!("update photo error: {}", e).as_ref());
        }
    }

    pub fn mount_youtube_search_input(&mut self) {
        assert!(self
            .app
            .remount(
                Id::YoutubeSearchInputPopup,
                Box::new(YSInputPopup::new(&self.config)),
                vec![]
            )
            .is_ok());
        assert!(self.app.active(&Id::YoutubeSearchInputPopup).is_ok());
        self.app.lock_subs();
    }

    pub fn mount_youtube_search_table(&mut self) {
        assert!(self
            .app
            .remount(
                Id::YoutubeSearchTablePopup,
                Box::new(YSTablePopup::new(&self.config)),
                vec![]
            )
            .is_ok());
        assert!(self.app.active(&Id::YoutubeSearchTablePopup).is_ok());
        self.app.lock_subs();
        if let Err(e) = self.update_photo() {
            self.mount_error_popup(format!("update photo error: {}", e).as_ref());
        }
    }
    pub fn mount_message(&mut self, title: &str, text: &str) {
        assert!(self
            .app
            .remount(
                Id::MessagePopup,
                Box::new(MessagePopup::new(title, text)),
                vec![]
            )
            .is_ok());
    }

    /// ### `umount_message`
    ///
    /// Umount error message
    pub fn umount_message(&mut self, _title: &str, text: &str) {
        if let Ok(Some(AttrValue::Payload(PropPayload::Vec(spans)))) =
            self.app.query(&Id::MessagePopup, Attribute::Text)
        {
            if let Some(display_text) = spans.get(0) {
                let d = display_text.clone().unwrap_text_span().content;
                if text.eq(&d) {
                    self.app.umount(&Id::MessagePopup).ok();
                }
            }
        }
    }
    pub fn mount_tageditor(&mut self, node_id: &str) {
        let p: &Path = Path::new(node_id);
        if p.is_dir() {
            self.mount_error_popup("directory doesn't have tag!");
            return;
        }

        let p = p.to_string_lossy();
        match Track::read_from_path(p.as_ref()) {
            Ok(s) => {
                assert!(self
                    .app
                    .remount(
                        Id::TagEditor(IdTagEditor::LabelHint),
                        Box::new(LabelGeneric::new(&self.config, "Press <ENTER> to search:")),
                        vec![]
                    )
                    .is_ok());
                assert!(self
                    .app
                    .remount(
                        Id::TagEditor(IdTagEditor::InputArtist),
                        Box::new(TEInputArtist::default()),
                        vec![]
                    )
                    .is_ok());
                assert!(self
                    .app
                    .remount(
                        Id::TagEditor(IdTagEditor::InputTitle),
                        Box::new(TEInputTitle::default()),
                        vec![]
                    )
                    .is_ok());
                assert!(self
                    .app
                    .remount(
                        Id::TagEditor(IdTagEditor::RadioTag),
                        Box::new(TERadioTag::default()),
                        vec![]
                    )
                    .is_ok());
                assert!(self
                    .app
                    .remount(
                        Id::TagEditor(IdTagEditor::TableLyricOptions),
                        Box::new(TETableLyricOptions::default()),
                        vec![]
                    )
                    .is_ok());
                assert!(self
                    .app
                    .remount(
                        Id::TagEditor(IdTagEditor::SelectLyric),
                        Box::new(TESelectLyric::default()),
                        vec![]
                    )
                    .is_ok());
                assert!(self
                    .app
                    .remount(
                        Id::TagEditor(IdTagEditor::CounterDelete),
                        Box::new(TECounterDelete::new(5)),
                        vec![]
                    )
                    .is_ok());
                assert!(self
                    .app
                    .remount(
                        Id::TagEditor(IdTagEditor::TextareaLyric),
                        Box::new(TETextareaLyric::default()),
                        vec![]
                    )
                    .is_ok());

                self.app
                    .active(&Id::TagEditor(IdTagEditor::InputArtist))
                    .ok();
                self.app.lock_subs();
                self.init_by_song(&s);
            }
            Err(e) => {
                self.mount_error_popup(format!("song load error: {}", e).as_ref());
            }
        };
        if let Err(e) = self.update_photo() {
            self.mount_error_popup(format!("clear photo error: {}", e).as_str());
        }
    }
    pub fn umount_tageditor(&mut self) {
        self.app.umount(&Id::TagEditor(IdTagEditor::LabelHint)).ok();
        self.app
            .umount(&Id::TagEditor(IdTagEditor::InputArtist))
            .ok();
        self.app
            .umount(&Id::TagEditor(IdTagEditor::InputTitle))
            .ok();
        self.app.umount(&Id::TagEditor(IdTagEditor::RadioTag)).ok();
        self.app
            .umount(&Id::TagEditor(IdTagEditor::TableLyricOptions))
            .ok();
        self.app
            .umount(&Id::TagEditor(IdTagEditor::SelectLyric))
            .ok();
        self.app
            .umount(&Id::TagEditor(IdTagEditor::CounterDelete))
            .ok();
        self.app
            .umount(&Id::TagEditor(IdTagEditor::TextareaLyric))
            .ok();
        if let Err(e) = self.update_photo() {
            self.mount_error_popup(format!("update photo error: {}", e).as_ref());
        }
        self.app.unlock_subs();
    }
    // initialize the value in tageditor based on info from Song
    pub fn init_by_song(&mut self, s: &Track) {
        self.tageditor_song = Some(s.clone());
        if let Some(artist) = s.artist() {
            assert!(self
                .app
                .attr(
                    &Id::TagEditor(IdTagEditor::InputArtist),
                    Attribute::Value,
                    AttrValue::String(artist.to_string()),
                )
                .is_ok());
        }

        if let Some(title) = s.title() {
            assert!(self
                .app
                .attr(
                    &Id::TagEditor(IdTagEditor::InputTitle),
                    Attribute::Value,
                    AttrValue::String(title.to_string()),
                )
                .is_ok());
        }

        if s.lyric_frames_is_empty() {
            self.init_by_song_no_lyric();
            return;
        }

        let mut vec_lang: Vec<String> = vec![];
        if let Some(lf) = s.lyric_frames() {
            for l in lf {
                vec_lang.push(l.description.clone());
            }
        }
        vec_lang.sort();

        assert!(self
            .app
            .attr(
                &Id::TagEditor(IdTagEditor::SelectLyric),
                Attribute::Content,
                AttrValue::Payload(PropPayload::Vec(
                    vec_lang
                        .iter()
                        .map(|x| PropValue::Str((*x).to_string()))
                        .collect(),
                )),
            )
            .is_ok());
        if let Ok(vec_lang_len_isize) = isize::try_from(vec_lang.len()) {
            assert!(self
                .app
                .attr(
                    &Id::TagEditor(IdTagEditor::CounterDelete),
                    Attribute::Value,
                    AttrValue::Number(vec_lang_len_isize),
                )
                .is_ok());
        }
        let mut vec_lyric: Vec<TextSpan> = vec![];
        if let Some(f) = s.lyric_selected() {
            for line in f.text.split('\n') {
                vec_lyric.push(TextSpan::from(line));
            }
        }
        assert!(self
            .app
            .attr(
                &Id::TagEditor(IdTagEditor::TextareaLyric),
                Attribute::Title,
                AttrValue::Title((
                    format!("{} Lyrics", vec_lang[s.lyric_selected_index()]),
                    Alignment::Left
                ))
            )
            .is_ok());

        assert!(self
            .app
            .attr(
                &Id::TagEditor(IdTagEditor::TextareaLyric),
                Attribute::Text,
                AttrValue::Payload(PropPayload::Vec(
                    vec_lyric.iter().cloned().map(PropValue::TextSpan).collect()
                ))
            )
            .is_ok());
    }

    fn init_by_song_no_lyric(&mut self) {
        assert!(self
            .app
            .attr(
                &Id::TagEditor(IdTagEditor::SelectLyric),
                Attribute::Content,
                AttrValue::Payload(PropPayload::Vec(
                    ["Empty"]
                        .iter()
                        .map(|x| PropValue::Str((*x).to_string()))
                        .collect(),
                )),
            )
            .is_ok());
        assert!(self
            .app
            .attr(
                &Id::TagEditor(IdTagEditor::CounterDelete),
                Attribute::Value,
                AttrValue::Number(0),
            )
            .is_ok());

        assert!(self
            .app
            .attr(
                &Id::TagEditor(IdTagEditor::TextareaLyric),
                Attribute::Title,
                AttrValue::Title(("Empty Lyric".to_string(), Alignment::Left))
            )
            .is_ok());
        assert!(self
            .app
            .attr(
                &Id::TagEditor(IdTagEditor::TextareaLyric),
                Attribute::Text,
                AttrValue::Payload(PropPayload::Vec(vec![PropValue::TextSpan(TextSpan::from(
                    "No Lyrics."
                )),]))
            )
            .is_ok());
    }

    pub fn mount_tageditor_help(&mut self) {
        assert!(self
            .app
            .remount(
                Id::TagEditor(IdTagEditor::HelpPopup),
                Box::new(TEHelpPopup::default()),
                vec![]
            )
            .is_ok());
        // Active help
        assert!(self
            .app
            .active(&Id::TagEditor(IdTagEditor::HelpPopup))
            .is_ok());
    }

    #[allow(clippy::too_many_lines)]
    fn view_tag_editor(&mut self) {
        assert!(self
            .terminal
            .raw_mut()
            .draw(|f| {
                if self.app.mounted(&Id::TagEditor(IdTagEditor::LabelHint)) {
                    f.render_widget(Clear, f.size());
                    let chunks_main = Layout::default()
                        .direction(Direction::Vertical)
                        .margin(0)
                        .constraints(
                            [
                                Constraint::Length(1),
                                Constraint::Length(3),
                                Constraint::Min(2),
                                Constraint::Length(1),
                            ]
                            .as_ref(),
                        )
                        .split(f.size());

                    let chunks_middle1 = Layout::default()
                        .direction(Direction::Horizontal)
                        .margin(0)
                        .constraints(
                            [
                                Constraint::Ratio(1, 4),
                                Constraint::Ratio(2, 4),
                                Constraint::Ratio(1, 4),
                            ]
                            .as_ref(),
                        )
                        .split(chunks_main[1]);
                    let chunks_middle2 = Layout::default()
                        .direction(Direction::Horizontal)
                        .margin(0)
                        .constraints([Constraint::Ratio(3, 5), Constraint::Ratio(2, 5)].as_ref())
                        .split(chunks_main[2]);

                    let chunks_middle2_right = Layout::default()
                        .direction(Direction::Vertical)
                        .margin(0)
                        .constraints([Constraint::Length(6), Constraint::Min(2)].as_ref())
                        .split(chunks_middle2[1]);

                    let chunks_middle2_right_top = Layout::default()
                        .direction(Direction::Horizontal)
                        .margin(0)
                        .constraints([Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)].as_ref())
                        .split(chunks_middle2_right[0]);

                    self.app
                        .view(&Id::TagEditor(IdTagEditor::LabelHint), f, chunks_main[0]);
                    self.app.view(&Id::Label, f, chunks_main[3]);
                    self.app.view(
                        &Id::TagEditor(IdTagEditor::InputArtist),
                        f,
                        chunks_middle1[0],
                    );
                    self.app.view(
                        &Id::TagEditor(IdTagEditor::InputTitle),
                        f,
                        chunks_middle1[1],
                    );
                    self.app
                        .view(&Id::TagEditor(IdTagEditor::RadioTag), f, chunks_middle1[2]);
                    self.app.view(
                        &Id::TagEditor(IdTagEditor::TableLyricOptions),
                        f,
                        chunks_middle2[0],
                    );
                    self.app.view(
                        &Id::TagEditor(IdTagEditor::SelectLyric),
                        f,
                        chunks_middle2_right_top[0],
                    );
                    self.app.view(
                        &Id::TagEditor(IdTagEditor::CounterDelete),
                        f,
                        chunks_middle2_right_top[1],
                    );
                    self.app.view(
                        &Id::TagEditor(IdTagEditor::TextareaLyric),
                        f,
                        chunks_middle2_right[1],
                    );

                    if self.app.mounted(&Id::TagEditor(IdTagEditor::HelpPopup)) {
                        let popup = draw_area_in_relative(f.size(), 50, 70);
                        f.render_widget(Clear, popup);
                        self.app
                            .view(&Id::TagEditor(IdTagEditor::HelpPopup), f, popup);
                    }
                    if self.app.mounted(&Id::MessagePopup) {
                        let popup = draw_area_top_right_absolute(f.size(), 25, 4);
                        f.render_widget(Clear, popup);
                        self.app.view(&Id::MessagePopup, f, popup);
                    }
                    if self.app.mounted(&Id::ErrorPopup) {
                        let popup = draw_area_in_relative(f.size(), 50, 4);
                        f.render_widget(Clear, popup);
                        self.app.view(&Id::ErrorPopup, f, popup);
                    }
                }
            })
            .is_ok());
    }

    #[allow(clippy::too_many_lines)]
    pub fn remount_label_help(
        &mut self,
        optional_text: Option<&str>,
        foreground: Option<Color>,
        background: Option<Color>,
    ) {
        if optional_text.is_none() {
            assert!(self
                .app
                .remount(
                    Id::Label,
                    Box::new(LabelSpan::new(
                        &self.config,
                        &[
                            TextSpan::new(" Version: ")
                                .fg(self
                                    .config
                                    .style_color_symbol
                                    .library_foreground()
                                    .unwrap_or(Color::Blue))
                                .bold(),
                            TextSpan::new(VERSION)
                                .fg(self
                                    .config
                                    .style_color_symbol
                                    .library_highlight()
                                    .unwrap_or(Color::Cyan))
                                .bold(),
                            TextSpan::new(" Help: ")
                                .fg(self
                                    .config
                                    .style_color_symbol
                                    .library_foreground()
                                    .unwrap_or(Color::Blue))
                                .bold(),
                            TextSpan::new(format!("<{}>", self.config.keys.global_help))
                                .fg(self
                                    .config
                                    .style_color_symbol
                                    .library_highlight()
                                    .unwrap_or(Color::Cyan))
                                .bold(),
                            TextSpan::new(" Config: ")
                                .fg(self
                                    .config
                                    .style_color_symbol
                                    .library_foreground()
                                    .unwrap_or(Color::Blue))
                                .bold(),
                            TextSpan::new(format!("<{}>", self.config.keys.global_config_open))
                                .fg(self
                                    .config
                                    .style_color_symbol
                                    .library_highlight()
                                    .unwrap_or(Color::Cyan))
                                .bold(),
                            TextSpan::new(" Library: ")
                                .fg(self
                                    .config
                                    .style_color_symbol
                                    .library_foreground()
                                    .unwrap_or(Color::Blue))
                                .bold(),
                            TextSpan::new(format!("<{}>", self.config.keys.global_layout_treeview))
                                .fg(self
                                    .config
                                    .style_color_symbol
                                    .library_highlight()
                                    .unwrap_or(Color::Cyan))
                                .bold(),
                            TextSpan::new(" Database: ")
                                .fg(self
                                    .config
                                    .style_color_symbol
                                    .library_foreground()
                                    .unwrap_or(Color::Blue))
                                .bold(),
                            TextSpan::new(format!("<{}>", self.config.keys.global_layout_database))
                                .fg(self
                                    .config
                                    .style_color_symbol
                                    .library_highlight()
                                    .unwrap_or(Color::Cyan))
                                .bold(),
                        ]
                    )),
                    Vec::default(),
                )
                .is_ok());
            return;
        }
        if let Some(text) = optional_text {
            assert!(self
                .app
                .remount(
                    Id::Label,
                    Box::new(LabelSpan::new(
                        &self.config,
                        &[TextSpan::new(text)
                            .fg(foreground.unwrap_or(Color::Cyan))
                            .bold()
                            .bg(background.unwrap_or(Color::Reset)),]
                    )),
                    Vec::default(),
                )
                .is_ok());
        }
    }
}
