use crate::config::Termusic;
use crate::ui::components::{
    draw_area_in_absolute, draw_area_in_relative, draw_area_top_right_absolute, CEHelpPopup,
    CELibraryBackground, CELibraryBorder, CELibraryForeground, CELibraryHighlight,
    CELibraryHighlightSymbol, CELibraryTitle, CELyricBackground, CELyricBorder, CELyricForeground,
    CELyricTitle, CEPlaylistBackground, CEPlaylistBorder, CEPlaylistForeground,
    CEPlaylistHighlight, CEPlaylistHighlightSymbol, CEPlaylistTitle, CEProgressBackground,
    CEProgressBorder, CEProgressForeground, CEProgressTitle, CERadioOk, DBListCriteria,
    DBListSearchResult, DBListSearchTracks, DeleteConfirmInputPopup, DeleteConfirmRadioPopup,
    ErrorPopup, GSInputPopup, GSTablePopup, GlobalListener, HelpPopup, KEDatabaseAddAll,
    KEDatabaseAddAllInput, KEGlobalColorEditor, KEGlobalColorEditorInput, KEGlobalDown,
    KEGlobalDownInput, KEGlobalGotoBottom, KEGlobalGotoBottomInput, KEGlobalGotoTop,
    KEGlobalGotoTopInput, KEGlobalHelp, KEGlobalHelpInput, KEGlobalKeyEditor,
    KEGlobalKeyEditorInput, KEGlobalLayoutDatabase, KEGlobalLayoutDatabaseInput,
    KEGlobalLayoutTreeview, KEGlobalLayoutTreeviewInput, KEGlobalLeft, KEGlobalLeftInput,
    KEGlobalLyricAdjustBackward, KEGlobalLyricAdjustBackwardInput, KEGlobalLyricAdjustForward,
    KEGlobalLyricAdjustForwardInput, KEGlobalLyricCycle, KEGlobalLyricCycleInput,
    KEGlobalPlayerNext, KEGlobalPlayerNextInput, KEGlobalPlayerPrevious,
    KEGlobalPlayerPreviousInput, KEGlobalPlayerSeekBackward, KEGlobalPlayerSeekBackwardInput,
    KEGlobalPlayerSeekForward, KEGlobalPlayerSeekForwardInput, KEGlobalPlayerSpeedDown,
    KEGlobalPlayerSpeedDownInput, KEGlobalPlayerSpeedUp, KEGlobalPlayerSpeedUpInput,
    KEGlobalPlayerTogglePause, KEGlobalPlayerTogglePauseInput, KEGlobalQuit, KEGlobalQuitInput,
    KEGlobalRight, KEGlobalRightInput, KEGlobalUp, KEGlobalUpInput, KEGlobalVolumeDown,
    KEGlobalVolumeDownInput, KEGlobalVolumeUp, KEGlobalVolumeUpInput, KEHelpPopup, KELibraryDelete,
    KELibraryDeleteInput, KELibraryLoadDir, KELibraryLoadDirInput, KELibraryPaste,
    KELibraryPasteInput, KELibrarySearch, KELibrarySearchInput, KELibrarySearchYoutube,
    KELibrarySearchYoutubeInput, KELibraryTagEditor, KELibraryTagEditorInput, KELibraryYank,
    KELibraryYankInput, KEPlaylistAddFront, KEPlaylistAddFrontInput, KEPlaylistDelete,
    KEPlaylistDeleteAll, KEPlaylistDeleteAllInput, KEPlaylistDeleteInput, KEPlaylistModeCycle,
    KEPlaylistModeCycleInput, KEPlaylistPlaySelected, KEPlaylistPlaySelectedInput,
    KEPlaylistSearch, KEPlaylistSearchInput, KEPlaylistShuffle, KEPlaylistShuffleInput,
    KEPlaylistSwapDown, KEPlaylistSwapDownInput, KEPlaylistSwapUp, KEPlaylistSwapUpInput,
    KERadioOk, Label, Lyric, MessagePopup, MusicLibrary, Playlist, Progress, QuitPopup, Source,
    TECounterDelete, TEHelpPopup, TEInputArtist, TEInputTitle, TERadioTag, TESelectLyric,
    TETableLyricOptions, TETextareaLyric, ThemeSelectTable, YSInputPopup, YSTablePopup,
};

use crate::ui::model::{Model, TermusicLayout};
use crate::{
    track::Track,
    ui::{Application, DBMsg, Id, IdColorEditor, IdKeyEditor, IdTagEditor, Msg},
    VERSION,
};
use std::convert::TryFrom;
use std::path::Path;
use std::time::{Duration, Instant};
use tui_realm_treeview::Tree;
use tuirealm::event::NoUserEvent;
use tuirealm::props::{
    Alignment, AttrValue, Attribute, Color, PropPayload, PropValue, TextModifiers, TextSpan,
};
use tuirealm::tui::layout::{Constraint, Direction, Layout};
use tuirealm::tui::widgets::Clear;
use tuirealm::Frame;
use tuirealm::{EventListenerCfg, State};

impl Model {
    pub fn init_app(tree: &Tree, config: &Termusic) -> Application<Id, Msg, NoUserEvent> {
        // Setup application
        // NOTE: NoUserEvent is a shorthand to tell tui-realm we're not going to use any custom user event
        // NOTE: the event listener is configured to use the default crossterm input listener and to raise a Tick event each second
        // which we will use to update the clock

        let mut app: Application<Id, Msg, NoUserEvent> = Application::init(
            EventListenerCfg::default()
                .default_input_listener(Duration::from_millis(20))
                .poll_timeout(Duration::from_millis(40))
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
                Id::Label,
                Box::new(
                    Label::default()
                        .text(format!(
                            "Press <{}> for help. Version: {}",
                            config.keys.global_help, VERSION,
                        ))
                        .alignment(Alignment::Left)
                        .background(Color::Reset)
                        .foreground(Color::Cyan)
                        .modifiers(TextModifiers::BOLD),
                ),
                Vec::default(),
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

    #[allow(clippy::too_many_lines)]
    pub fn view(&mut self) {
        if self.redraw {
            self.redraw = false;
            self.last_redraw = Instant::now();
            if self
                .app
                .mounted(&Id::ColorEditor(IdColorEditor::ThemeSelect))
            {
                self.view_color_editor();
                return;
            } else if self
                .app
                .mounted(&Id::TagEditor(IdTagEditor::TableLyricOptions))
            {
                self.view_tag_editor();
                return;
            } else if self.app.mounted(&Id::KeyEditor(IdKeyEditor::LabelHint)) {
                self.view_key_editor();
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
                self.app.view(&Id::Label, f, chunks_main[1]);

                Self::view_layout_commons(f, &mut self.app);
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

                Self::view_layout_commons(f, &mut self.app);
            })
            .is_ok());
    }

    fn view_layout_commons(f: &mut Frame, app: &mut Application<Id, Msg, NoUserEvent>) {
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
                Box::new(GSInputPopup::new(
                    Source::Library,
                    &self.config.style_color_symbol,
                    &self.config.keys
                )),
                vec![]
            )
            .is_ok());
        assert!(self
            .app
            .remount(
                Id::GeneralSearchTable,
                Box::new(GSTablePopup::new(
                    Source::Library,
                    &self.config.style_color_symbol,
                    &self.config.keys
                )),
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
                Box::new(GSInputPopup::new(
                    Source::Playlist,
                    &self.config.style_color_symbol,
                    &self.config.keys
                )),
                vec![]
            )
            .is_ok());
        assert!(self
            .app
            .remount(
                Id::GeneralSearchTable,
                Box::new(GSTablePopup::new(
                    Source::Playlist,
                    &self.config.style_color_symbol,
                    &self.config.keys
                )),
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
                Box::new(YSInputPopup::new(
                    &self.config.style_color_symbol,
                    &self.config.keys
                )),
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
                Box::new(YSTablePopup::new(
                    &self.config.style_color_symbol,
                    &self.config.keys
                )),
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
        // assert!(self.app.active(&Id::ErrorPopup).is_ok());
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
                        Box::new(
                            Label::default()
                                .text("Press <ENTER> to search:")
                                .alignment(Alignment::Left)
                                .background(Color::Reset)
                                .foreground(Color::Magenta)
                                .modifiers(TextModifiers::BOLD),
                        ),
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
        // self.app.umount(&Id::TELabelHelp).ok();
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
    fn view_color_editor(&mut self) {
        assert!(self
            .terminal
            .raw_mut()
            .draw(|f| {
                if self
                    .app
                    .mounted(&Id::ColorEditor(IdColorEditor::ThemeSelect))
                {
                    f.render_widget(Clear, f.size());
                    let chunks_main = Layout::default()
                        .direction(Direction::Vertical)
                        .margin(0)
                        .constraints(
                            [
                                Constraint::Length(1),
                                Constraint::Min(2),
                                Constraint::Length(1),
                            ]
                            .as_ref(),
                        )
                        .split(f.size());

                    let chunks_middle = Layout::default()
                        .direction(Direction::Horizontal)
                        .margin(0)
                        .constraints([Constraint::Ratio(1, 4), Constraint::Ratio(3, 4)].as_ref())
                        .split(chunks_main[1]);

                    let chunks_middle_left = Layout::default()
                        .direction(Direction::Vertical)
                        .margin(0)
                        .constraints([Constraint::Min(7), Constraint::Length(3)].as_ref())
                        .split(chunks_middle[0]);

                    let chunks_middle_right = Layout::default()
                        .direction(Direction::Vertical)
                        .margin(0)
                        .constraints(
                            [
                                Constraint::Length(7),
                                Constraint::Length(7),
                                Constraint::Length(7),
                                Constraint::Length(7),
                                Constraint::Length(7),
                            ]
                            .as_ref(),
                        )
                        .split(chunks_middle[1]);
                    let chunks_middle_right_library = Layout::default()
                        .direction(Direction::Vertical)
                        .margin(0)
                        .constraints([Constraint::Length(1), Constraint::Length(6)].as_ref())
                        .split(chunks_middle_right[0]);

                    let chunks_middle_right_library_items = Layout::default()
                        .direction(Direction::Horizontal)
                        .margin(0)
                        .constraints(
                            [
                                Constraint::Ratio(1, 5),
                                Constraint::Ratio(1, 5),
                                Constraint::Ratio(1, 5),
                                Constraint::Ratio(1, 5),
                                Constraint::Ratio(1, 5),
                            ]
                            .as_ref(),
                        )
                        .split(chunks_middle_right_library[1]);
                    let chunks_middle_right_playlist = Layout::default()
                        .direction(Direction::Vertical)
                        .margin(0)
                        .constraints([Constraint::Length(1), Constraint::Length(6)].as_ref())
                        .split(chunks_middle_right[1]);

                    let chunks_middle_right_playlist_items = Layout::default()
                        .direction(Direction::Horizontal)
                        .margin(0)
                        .constraints(
                            [
                                Constraint::Ratio(1, 5),
                                Constraint::Ratio(1, 5),
                                Constraint::Ratio(1, 5),
                                Constraint::Ratio(1, 5),
                                Constraint::Ratio(1, 5),
                            ]
                            .as_ref(),
                        )
                        .split(chunks_middle_right_playlist[1]);
                    let chunks_middle_right_progress = Layout::default()
                        .direction(Direction::Vertical)
                        .margin(0)
                        .constraints([Constraint::Length(1), Constraint::Length(6)].as_ref())
                        .split(chunks_middle_right[2]);

                    let chunks_middle_right_progress_items = Layout::default()
                        .direction(Direction::Horizontal)
                        .margin(0)
                        .constraints(
                            [
                                Constraint::Ratio(1, 5),
                                Constraint::Ratio(1, 5),
                                Constraint::Ratio(1, 5),
                                Constraint::Ratio(1, 5),
                                Constraint::Ratio(1, 5),
                            ]
                            .as_ref(),
                        )
                        .split(chunks_middle_right_progress[1]);
                    let chunks_middle_right_lyric = Layout::default()
                        .direction(Direction::Vertical)
                        .margin(0)
                        .constraints([Constraint::Length(1), Constraint::Length(6)].as_ref())
                        .split(chunks_middle_right[3]);

                    let chunks_middle_right_lyric_items = Layout::default()
                        .direction(Direction::Horizontal)
                        .margin(0)
                        .constraints(
                            [
                                Constraint::Ratio(1, 5),
                                Constraint::Ratio(1, 5),
                                Constraint::Ratio(1, 5),
                                Constraint::Ratio(1, 5),
                                Constraint::Ratio(1, 5),
                            ]
                            .as_ref(),
                        )
                        .split(chunks_middle_right_lyric[1]);

                    self.app.view(
                        &Id::ColorEditor(IdColorEditor::LabelHint),
                        f,
                        chunks_main[0],
                    );
                    self.app.view(&Id::Label, f, chunks_main[2]);

                    self.app.view(
                        &Id::ColorEditor(IdColorEditor::ThemeSelect),
                        f,
                        chunks_middle_left[0],
                    );
                    self.app.view(
                        &Id::ColorEditor(IdColorEditor::RadioOk),
                        f,
                        chunks_middle_left[1],
                    );

                    self.app.view(
                        &Id::ColorEditor(IdColorEditor::LibraryLabel),
                        f,
                        chunks_middle_right_library[0],
                    );
                    self.app.view(
                        &Id::ColorEditor(IdColorEditor::LibraryForeground),
                        f,
                        chunks_middle_right_library_items[0],
                    );
                    self.app.view(
                        &Id::ColorEditor(IdColorEditor::LibraryBackground),
                        f,
                        chunks_middle_right_library_items[1],
                    );
                    self.app.view(
                        &Id::ColorEditor(IdColorEditor::LibraryBorder),
                        f,
                        chunks_middle_right_library_items[2],
                    );
                    self.app.view(
                        &Id::ColorEditor(IdColorEditor::LibraryHighlight),
                        f,
                        chunks_middle_right_library_items[3],
                    );
                    self.app.view(
                        &Id::ColorEditor(IdColorEditor::LibraryHighlightSymbol),
                        f,
                        chunks_middle_right_library_items[4],
                    );
                    self.app.view(
                        &Id::ColorEditor(IdColorEditor::PlaylistLabel),
                        f,
                        chunks_middle_right_playlist[0],
                    );
                    self.app.view(
                        &Id::ColorEditor(IdColorEditor::PlaylistForeground),
                        f,
                        chunks_middle_right_playlist_items[0],
                    );
                    self.app.view(
                        &Id::ColorEditor(IdColorEditor::PlaylistBackground),
                        f,
                        chunks_middle_right_playlist_items[1],
                    );
                    self.app.view(
                        &Id::ColorEditor(IdColorEditor::PlaylistBorder),
                        f,
                        chunks_middle_right_playlist_items[2],
                    );
                    self.app.view(
                        &Id::ColorEditor(IdColorEditor::PlaylistHighlight),
                        f,
                        chunks_middle_right_playlist_items[3],
                    );
                    self.app.view(
                        &Id::ColorEditor(IdColorEditor::PlaylistHighlightSymbol),
                        f,
                        chunks_middle_right_playlist_items[4],
                    );
                    self.app.view(
                        &Id::ColorEditor(IdColorEditor::ProgressLabel),
                        f,
                        chunks_middle_right_progress[0],
                    );
                    self.app.view(
                        &Id::ColorEditor(IdColorEditor::ProgressForeground),
                        f,
                        chunks_middle_right_progress_items[0],
                    );
                    self.app.view(
                        &Id::ColorEditor(IdColorEditor::ProgressBackground),
                        f,
                        chunks_middle_right_progress_items[1],
                    );
                    self.app.view(
                        &Id::ColorEditor(IdColorEditor::ProgressBorder),
                        f,
                        chunks_middle_right_progress_items[2],
                    );
                    self.app.view(
                        &Id::ColorEditor(IdColorEditor::LyricLabel),
                        f,
                        chunks_middle_right_lyric[0],
                    );
                    self.app.view(
                        &Id::ColorEditor(IdColorEditor::LyricForeground),
                        f,
                        chunks_middle_right_lyric_items[0],
                    );
                    self.app.view(
                        &Id::ColorEditor(IdColorEditor::LyricBackground),
                        f,
                        chunks_middle_right_lyric_items[1],
                    );
                    self.app.view(
                        &Id::ColorEditor(IdColorEditor::LyricBorder),
                        f,
                        chunks_middle_right_lyric_items[2],
                    );
                    if self.app.mounted(&Id::ColorEditor(IdColorEditor::HelpPopup)) {
                        let popup = draw_area_in_relative(f.size(), 50, 70);
                        f.render_widget(Clear, popup);
                        self.app
                            .view(&Id::ColorEditor(IdColorEditor::HelpPopup), f, popup);
                    }
                }
            })
            .is_ok());
    }

    #[allow(clippy::too_many_lines, clippy::cognitive_complexity)]
    pub fn mount_color_editor(&mut self) {
        let mut config = self.config.clone();
        // This is for preview the theme colors
        config.style_color_symbol = self.ce_style_color_symbol.clone();

        assert!(self
            .app
            .remount(
                Id::ColorEditor(IdColorEditor::LabelHint),
                Box::new(
                    Label::default()
                        .text("  Color Editor. You can select theme to change the general style, or you can change specific color.")
                        .alignment(Alignment::Left)
                        .background(Color::Reset)
                        .foreground(Color::Magenta)
                        .modifiers(TextModifiers::BOLD),
                ),
                vec![]
            )
            .is_ok());

        assert!(self
            .app
            .remount(
                Id::ColorEditor(IdColorEditor::ThemeSelect),
                Box::new(ThemeSelectTable::default()),
                vec![]
            )
            .is_ok());
        assert!(self
            .app
            .remount(
                Id::ColorEditor(IdColorEditor::LibraryLabel),
                Box::new(CELibraryTitle::default()),
                vec![]
            )
            .is_ok());
        assert!(self
            .app
            .remount(
                Id::ColorEditor(IdColorEditor::LibraryForeground),
                Box::new(CELibraryForeground::new(&config)),
                vec![]
            )
            .is_ok());

        assert!(self
            .app
            .remount(
                Id::ColorEditor(IdColorEditor::LibraryBackground),
                Box::new(CELibraryBackground::new(&config)),
                vec![]
            )
            .is_ok());
        assert!(self
            .app
            .remount(
                Id::ColorEditor(IdColorEditor::LibraryBorder),
                Box::new(CELibraryBorder::new(&config)),
                vec![]
            )
            .is_ok());
        assert!(self
            .app
            .remount(
                Id::ColorEditor(IdColorEditor::LibraryHighlight),
                Box::new(CELibraryHighlight::new(&config)),
                vec![]
            )
            .is_ok());
        assert!(self
            .app
            .remount(
                Id::ColorEditor(IdColorEditor::LibraryHighlightSymbol),
                Box::new(CELibraryHighlightSymbol::new(&config.style_color_symbol)),
                vec![]
            )
            .is_ok());
        assert!(self
            .app
            .remount(
                Id::ColorEditor(IdColorEditor::PlaylistLabel),
                Box::new(CEPlaylistTitle::default()),
                vec![]
            )
            .is_ok());
        assert!(self
            .app
            .remount(
                Id::ColorEditor(IdColorEditor::PlaylistForeground),
                Box::new(CEPlaylistForeground::new(&config)),
                vec![]
            )
            .is_ok());

        assert!(self
            .app
            .remount(
                Id::ColorEditor(IdColorEditor::PlaylistBackground),
                Box::new(CEPlaylistBackground::new(&config)),
                vec![]
            )
            .is_ok());
        assert!(self
            .app
            .remount(
                Id::ColorEditor(IdColorEditor::PlaylistBorder),
                Box::new(CEPlaylistBorder::new(&config)),
                vec![]
            )
            .is_ok());
        assert!(self
            .app
            .remount(
                Id::ColorEditor(IdColorEditor::PlaylistHighlight),
                Box::new(CEPlaylistHighlight::new(&config)),
                vec![]
            )
            .is_ok());
        assert!(self
            .app
            .remount(
                Id::ColorEditor(IdColorEditor::PlaylistHighlightSymbol),
                Box::new(CEPlaylistHighlightSymbol::new(&config.style_color_symbol)),
                vec![]
            )
            .is_ok());
        assert!(self
            .app
            .remount(
                Id::ColorEditor(IdColorEditor::ProgressLabel),
                Box::new(CEProgressTitle::default()),
                vec![]
            )
            .is_ok());
        assert!(self
            .app
            .remount(
                Id::ColorEditor(IdColorEditor::ProgressForeground),
                Box::new(CEProgressForeground::new(&config)),
                vec![]
            )
            .is_ok());

        assert!(self
            .app
            .remount(
                Id::ColorEditor(IdColorEditor::ProgressBackground),
                Box::new(CEProgressBackground::new(&config)),
                vec![]
            )
            .is_ok());
        assert!(self
            .app
            .remount(
                Id::ColorEditor(IdColorEditor::ProgressBorder),
                Box::new(CEProgressBorder::new(&config)),
                vec![]
            )
            .is_ok());
        assert!(self
            .app
            .remount(
                Id::ColorEditor(IdColorEditor::LyricLabel),
                Box::new(CELyricTitle::default()),
                vec![]
            )
            .is_ok());
        assert!(self
            .app
            .remount(
                Id::ColorEditor(IdColorEditor::LyricForeground),
                Box::new(CELyricForeground::new(&config)),
                vec![]
            )
            .is_ok());

        assert!(self
            .app
            .remount(
                Id::ColorEditor(IdColorEditor::LyricBackground),
                Box::new(CELyricBackground::new(&config)),
                vec![]
            )
            .is_ok());
        assert!(self
            .app
            .remount(
                Id::ColorEditor(IdColorEditor::LyricBorder),
                Box::new(CELyricBorder::new(&config)),
                vec![]
            )
            .is_ok());

        assert!(self
            .app
            .remount(
                Id::ColorEditor(IdColorEditor::RadioOk),
                Box::new(CERadioOk::default()),
                vec![]
            )
            .is_ok());

        // focus theme
        assert!(self
            .app
            .active(&Id::ColorEditor(IdColorEditor::ThemeSelect))
            .is_ok());
        self.theme_select_sync();
        self.app.lock_subs();
        if let Err(e) = self.update_photo() {
            self.mount_error_popup(format!("clear photo error: {}", e).as_str());
        }
    }

    pub fn umount_color_editor(&mut self) {
        self.app
            .umount(&Id::ColorEditor(IdColorEditor::ThemeSelect))
            .ok();
        self.app
            .umount(&Id::ColorEditor(IdColorEditor::LibraryLabel))
            .ok();
        self.app
            .umount(&Id::ColorEditor(IdColorEditor::LibraryForeground))
            .ok();
        self.app
            .umount(&Id::ColorEditor(IdColorEditor::LibraryBackground))
            .ok();
        self.app
            .umount(&Id::ColorEditor(IdColorEditor::LibraryBorder))
            .ok();
        self.app
            .umount(&Id::ColorEditor(IdColorEditor::LibraryHighlight))
            .ok();
        self.app
            .umount(&Id::ColorEditor(IdColorEditor::LibraryHighlightSymbol))
            .ok();
        self.app
            .umount(&Id::ColorEditor(IdColorEditor::PlaylistLabel))
            .ok();
        self.app
            .umount(&Id::ColorEditor(IdColorEditor::PlaylistForeground))
            .ok();
        self.app
            .umount(&Id::ColorEditor(IdColorEditor::PlaylistBackground))
            .ok();
        self.app
            .umount(&Id::ColorEditor(IdColorEditor::PlaylistBorder))
            .ok();
        self.app
            .umount(&Id::ColorEditor(IdColorEditor::PlaylistHighlight))
            .ok();
        self.app
            .umount(&Id::ColorEditor(IdColorEditor::PlaylistHighlightSymbol))
            .ok();
        self.app
            .umount(&Id::ColorEditor(IdColorEditor::ProgressLabel))
            .ok();
        self.app
            .umount(&Id::ColorEditor(IdColorEditor::ProgressForeground))
            .ok();
        self.app
            .umount(&Id::ColorEditor(IdColorEditor::ProgressBackground))
            .ok();
        self.app
            .umount(&Id::ColorEditor(IdColorEditor::ProgressBorder))
            .ok();
        self.app
            .umount(&Id::ColorEditor(IdColorEditor::LyricLabel))
            .ok();
        self.app
            .umount(&Id::ColorEditor(IdColorEditor::LyricForeground))
            .ok();
        self.app
            .umount(&Id::ColorEditor(IdColorEditor::LyricBackground))
            .ok();
        self.app
            .umount(&Id::ColorEditor(IdColorEditor::LyricBorder))
            .ok();

        self.app
            .umount(&Id::ColorEditor(IdColorEditor::RadioOk))
            .ok();

        self.app.unlock_subs();
        self.library_reload_tree();
        self.playlist_reload();
        self.database_reload();
        self.progress_reload();
        self.global_fix_focus();
        self.lyric_reload();
        self.update_lyric();
        if let Err(e) = self.update_photo() {
            self.mount_error_popup(format!("update photo error: {}", e).as_ref());
        }
    }

    pub fn mount_color_editor_help(&mut self) {
        assert!(self
            .app
            .remount(
                Id::ColorEditor(IdColorEditor::HelpPopup),
                Box::new(CEHelpPopup::default()),
                vec![]
            )
            .is_ok());
        // Active help
        assert!(self
            .app
            .active(&Id::ColorEditor(IdColorEditor::HelpPopup))
            .is_ok());
    }

    #[allow(clippy::too_many_lines, clippy::cognitive_complexity)]
    pub fn mount_key_editor(&mut self) {
        assert!(self
            .app
            .remount(
                Id::KeyEditor(IdKeyEditor::LabelHint),
                Box::new(
                    Label::default()
                        .text("  Key Editor. ")
                        .alignment(Alignment::Left)
                        .background(Color::Reset)
                        .foreground(Color::Magenta)
                        .modifiers(TextModifiers::BOLD),
                ),
                vec![]
            )
            .is_ok());

        assert!(self
            .app
            .remount(
                Id::KeyEditor(IdKeyEditor::RadioOk),
                Box::new(KERadioOk::default()),
                vec![]
            )
            .is_ok());

        assert!(self
            .app
            .remount(
                Id::KeyEditor(IdKeyEditor::GlobalQuit),
                Box::new(KEGlobalQuit::new(&self.config.keys)),
                vec![],
            )
            .is_ok());
        assert!(self
            .app
            .remount(
                Id::KeyEditor(IdKeyEditor::GlobalQuitInput),
                Box::new(KEGlobalQuitInput::new(&self.config.keys)),
                vec![],
            )
            .is_ok());
        assert!(self
            .app
            .remount(
                Id::KeyEditor(IdKeyEditor::GlobalLeft),
                Box::new(KEGlobalLeft::new(&self.config.keys)),
                vec![],
            )
            .is_ok());
        assert!(self
            .app
            .remount(
                Id::KeyEditor(IdKeyEditor::GlobalLeftInput),
                Box::new(KEGlobalLeftInput::new(&self.config.keys)),
                vec![],
            )
            .is_ok());
        assert!(self
            .app
            .remount(
                Id::KeyEditor(IdKeyEditor::GlobalRight),
                Box::new(KEGlobalRight::new(&self.config.keys)),
                vec![],
            )
            .is_ok());
        assert!(self
            .app
            .remount(
                Id::KeyEditor(IdKeyEditor::GlobalRightInput),
                Box::new(KEGlobalRightInput::new(&self.config.keys)),
                vec![],
            )
            .is_ok());
        assert!(self
            .app
            .remount(
                Id::KeyEditor(IdKeyEditor::GlobalUp),
                Box::new(KEGlobalUp::new(&self.config.keys)),
                vec![],
            )
            .is_ok());
        assert!(self
            .app
            .remount(
                Id::KeyEditor(IdKeyEditor::GlobalUpInput),
                Box::new(KEGlobalUpInput::new(&self.config.keys)),
                vec![],
            )
            .is_ok());

        assert!(self
            .app
            .remount(
                Id::KeyEditor(IdKeyEditor::GlobalDown),
                Box::new(KEGlobalDown::new(&self.config.keys)),
                vec![],
            )
            .is_ok());
        assert!(self
            .app
            .remount(
                Id::KeyEditor(IdKeyEditor::GlobalDownInput),
                Box::new(KEGlobalDownInput::new(&self.config.keys)),
                vec![],
            )
            .is_ok());

        assert!(self
            .app
            .remount(
                Id::KeyEditor(IdKeyEditor::GlobalGotoTop),
                Box::new(KEGlobalGotoTop::new(&self.config.keys)),
                vec![],
            )
            .is_ok());
        assert!(self
            .app
            .remount(
                Id::KeyEditor(IdKeyEditor::GlobalGotoTopInput),
                Box::new(KEGlobalGotoTopInput::new(&self.config.keys)),
                vec![],
            )
            .is_ok());
        assert!(self
            .app
            .remount(
                Id::KeyEditor(IdKeyEditor::GlobalGotoBottom),
                Box::new(KEGlobalGotoBottom::new(&self.config.keys)),
                vec![],
            )
            .is_ok());
        assert!(self
            .app
            .remount(
                Id::KeyEditor(IdKeyEditor::GlobalGotoBottomInput),
                Box::new(KEGlobalGotoBottomInput::new(&self.config.keys)),
                vec![],
            )
            .is_ok());
        assert!(self
            .app
            .remount(
                Id::KeyEditor(IdKeyEditor::GlobalPlayerTogglePause),
                Box::new(KEGlobalPlayerTogglePause::new(&self.config.keys)),
                vec![],
            )
            .is_ok());
        assert!(self
            .app
            .remount(
                Id::KeyEditor(IdKeyEditor::GlobalPlayerTogglePauseInput),
                Box::new(KEGlobalPlayerTogglePauseInput::new(&self.config.keys)),
                vec![],
            )
            .is_ok());
        assert!(self
            .app
            .remount(
                Id::KeyEditor(IdKeyEditor::GlobalPlayerNext),
                Box::new(KEGlobalPlayerNext::new(&self.config.keys)),
                vec![],
            )
            .is_ok());
        assert!(self
            .app
            .remount(
                Id::KeyEditor(IdKeyEditor::GlobalPlayerNextInput),
                Box::new(KEGlobalPlayerNextInput::new(&self.config.keys)),
                vec![],
            )
            .is_ok());
        assert!(self
            .app
            .remount(
                Id::KeyEditor(IdKeyEditor::GlobalPlayerPrevious),
                Box::new(KEGlobalPlayerPrevious::new(&self.config.keys)),
                vec![],
            )
            .is_ok());
        assert!(self
            .app
            .remount(
                Id::KeyEditor(IdKeyEditor::GlobalPlayerPreviousInput),
                Box::new(KEGlobalPlayerPreviousInput::new(&self.config.keys)),
                vec![],
            )
            .is_ok());

        assert!(self
            .app
            .remount(
                Id::KeyEditor(IdKeyEditor::GlobalHelp),
                Box::new(KEGlobalHelp::new(&self.config.keys)),
                vec![],
            )
            .is_ok());
        assert!(self
            .app
            .remount(
                Id::KeyEditor(IdKeyEditor::GlobalHelpInput),
                Box::new(KEGlobalHelpInput::new(&self.config.keys)),
                vec![],
            )
            .is_ok());

        assert!(self
            .app
            .remount(
                Id::KeyEditor(IdKeyEditor::GlobalVolumeUp),
                Box::new(KEGlobalVolumeUp::new(&self.config.keys)),
                vec![],
            )
            .is_ok());
        assert!(self
            .app
            .remount(
                Id::KeyEditor(IdKeyEditor::GlobalVolumeUpInput),
                Box::new(KEGlobalVolumeUpInput::new(&self.config.keys)),
                vec![],
            )
            .is_ok());

        assert!(self
            .app
            .remount(
                Id::KeyEditor(IdKeyEditor::GlobalVolumeDown),
                Box::new(KEGlobalVolumeDown::new(&self.config.keys)),
                vec![],
            )
            .is_ok());
        assert!(self
            .app
            .remount(
                Id::KeyEditor(IdKeyEditor::GlobalVolumeDownInput),
                Box::new(KEGlobalVolumeDownInput::new(&self.config.keys)),
                vec![],
            )
            .is_ok());

        assert!(self
            .app
            .remount(
                Id::KeyEditor(IdKeyEditor::GlobalPlayerSeekForward),
                Box::new(KEGlobalPlayerSeekForward::new(&self.config.keys)),
                vec![],
            )
            .is_ok());
        assert!(self
            .app
            .remount(
                Id::KeyEditor(IdKeyEditor::GlobalPlayerSeekForwardInput),
                Box::new(KEGlobalPlayerSeekForwardInput::new(&self.config.keys)),
                vec![],
            )
            .is_ok());

        assert!(self
            .app
            .remount(
                Id::KeyEditor(IdKeyEditor::GlobalPlayerSeekBackward),
                Box::new(KEGlobalPlayerSeekBackward::new(&self.config.keys)),
                vec![],
            )
            .is_ok());
        assert!(self
            .app
            .remount(
                Id::KeyEditor(IdKeyEditor::GlobalPlayerSeekBackwardInput),
                Box::new(KEGlobalPlayerSeekBackwardInput::new(&self.config.keys)),
                vec![],
            )
            .is_ok());

        assert!(self
            .app
            .remount(
                Id::KeyEditor(IdKeyEditor::GlobalPlayerSpeedUp),
                Box::new(KEGlobalPlayerSpeedUp::new(&self.config.keys)),
                vec![],
            )
            .is_ok());

        assert!(self
            .app
            .remount(
                Id::KeyEditor(IdKeyEditor::GlobalPlayerSpeedUpInput),
                Box::new(KEGlobalPlayerSpeedUpInput::new(&self.config.keys)),
                vec![],
            )
            .is_ok());
        assert!(self
            .app
            .remount(
                Id::KeyEditor(IdKeyEditor::GlobalPlayerSpeedDown),
                Box::new(KEGlobalPlayerSpeedDown::new(&self.config.keys)),
                vec![],
            )
            .is_ok());

        assert!(self
            .app
            .remount(
                Id::KeyEditor(IdKeyEditor::GlobalPlayerSpeedDownInput),
                Box::new(KEGlobalPlayerSpeedDownInput::new(&self.config.keys)),
                vec![],
            )
            .is_ok());

        assert!(self
            .app
            .remount(
                Id::KeyEditor(IdKeyEditor::GlobalLyricAdjustForward),
                Box::new(KEGlobalLyricAdjustForward::new(&self.config.keys)),
                vec![],
            )
            .is_ok());
        assert!(self
            .app
            .remount(
                Id::KeyEditor(IdKeyEditor::GlobalLyricAdjustForwardInput),
                Box::new(KEGlobalLyricAdjustForwardInput::new(&self.config.keys)),
                vec![],
            )
            .is_ok());

        assert!(self
            .app
            .remount(
                Id::KeyEditor(IdKeyEditor::GlobalLyricAdjustBackward),
                Box::new(KEGlobalLyricAdjustBackward::new(&self.config.keys)),
                vec![],
            )
            .is_ok());
        assert!(self
            .app
            .remount(
                Id::KeyEditor(IdKeyEditor::GlobalLyricAdjustBackwardInput),
                Box::new(KEGlobalLyricAdjustBackwardInput::new(&self.config.keys)),
                vec![],
            )
            .is_ok());

        assert!(self
            .app
            .remount(
                Id::KeyEditor(IdKeyEditor::GlobalLyricCycle),
                Box::new(KEGlobalLyricCycle::new(&self.config.keys)),
                vec![],
            )
            .is_ok());
        assert!(self
            .app
            .remount(
                Id::KeyEditor(IdKeyEditor::GlobalLyricCycleInput),
                Box::new(KEGlobalLyricCycleInput::new(&self.config.keys)),
                vec![],
            )
            .is_ok());
        assert!(self
            .app
            .remount(
                Id::KeyEditor(IdKeyEditor::GlobalColorEditor),
                Box::new(KEGlobalColorEditor::new(&self.config.keys)),
                vec![],
            )
            .is_ok());
        assert!(self
            .app
            .remount(
                Id::KeyEditor(IdKeyEditor::GlobalColorEditorInput),
                Box::new(KEGlobalColorEditorInput::new(&self.config.keys)),
                vec![],
            )
            .is_ok());
        assert!(self
            .app
            .remount(
                Id::KeyEditor(IdKeyEditor::GlobalKeyEditor),
                Box::new(KEGlobalKeyEditor::new(&self.config.keys)),
                vec![],
            )
            .is_ok());
        assert!(self
            .app
            .remount(
                Id::KeyEditor(IdKeyEditor::GlobalKeyEditorInput),
                Box::new(KEGlobalKeyEditorInput::new(&self.config.keys)),
                vec![],
            )
            .is_ok());
        assert!(self
            .app
            .remount(
                Id::KeyEditor(IdKeyEditor::LibraryDelete),
                Box::new(KELibraryDelete::new(&self.config.keys)),
                vec![],
            )
            .is_ok());
        assert!(self
            .app
            .remount(
                Id::KeyEditor(IdKeyEditor::LibraryDeleteInput),
                Box::new(KELibraryDeleteInput::new(&self.config.keys)),
                vec![],
            )
            .is_ok());
        assert!(self
            .app
            .remount(
                Id::KeyEditor(IdKeyEditor::LibraryLoadDir),
                Box::new(KELibraryLoadDir::new(&self.config.keys)),
                vec![],
            )
            .is_ok());
        assert!(self
            .app
            .remount(
                Id::KeyEditor(IdKeyEditor::LibraryLoadDirInput),
                Box::new(KELibraryLoadDirInput::new(&self.config.keys)),
                vec![],
            )
            .is_ok());
        assert!(self
            .app
            .remount(
                Id::KeyEditor(IdKeyEditor::LibraryYank),
                Box::new(KELibraryYank::new(&self.config.keys)),
                vec![],
            )
            .is_ok());
        assert!(self
            .app
            .remount(
                Id::KeyEditor(IdKeyEditor::LibraryYankInput),
                Box::new(KELibraryYankInput::new(&self.config.keys)),
                vec![],
            )
            .is_ok());
        assert!(self
            .app
            .remount(
                Id::KeyEditor(IdKeyEditor::LibraryPaste),
                Box::new(KELibraryPaste::new(&self.config.keys)),
                vec![],
            )
            .is_ok());
        assert!(self
            .app
            .remount(
                Id::KeyEditor(IdKeyEditor::LibraryPasteInput),
                Box::new(KELibraryPasteInput::new(&self.config.keys)),
                vec![],
            )
            .is_ok());
        assert!(self
            .app
            .remount(
                Id::KeyEditor(IdKeyEditor::LibrarySearch),
                Box::new(KELibrarySearch::new(&self.config.keys)),
                vec![],
            )
            .is_ok());
        assert!(self
            .app
            .remount(
                Id::KeyEditor(IdKeyEditor::LibrarySearchInput),
                Box::new(KELibrarySearchInput::new(&self.config.keys)),
                vec![],
            )
            .is_ok());
        assert!(self
            .app
            .remount(
                Id::KeyEditor(IdKeyEditor::LibrarySearchYoutube),
                Box::new(KELibrarySearchYoutube::new(&self.config.keys)),
                vec![],
            )
            .is_ok());
        assert!(self
            .app
            .remount(
                Id::KeyEditor(IdKeyEditor::LibrarySearchYoutubeInput),
                Box::new(KELibrarySearchYoutubeInput::new(&self.config.keys)),
                vec![],
            )
            .is_ok());
        assert!(self
            .app
            .remount(
                Id::KeyEditor(IdKeyEditor::LibraryTagEditor),
                Box::new(KELibraryTagEditor::new(&self.config.keys)),
                vec![],
            )
            .is_ok());
        assert!(self
            .app
            .remount(
                Id::KeyEditor(IdKeyEditor::LibraryTagEditorInput),
                Box::new(KELibraryTagEditorInput::new(&self.config.keys)),
                vec![],
            )
            .is_ok());

        assert!(self
            .app
            .remount(
                Id::KeyEditor(IdKeyEditor::PlaylistDelete),
                Box::new(KEPlaylistDelete::new(&self.config.keys)),
                vec![],
            )
            .is_ok());
        assert!(self
            .app
            .remount(
                Id::KeyEditor(IdKeyEditor::PlaylistDeleteInput),
                Box::new(KEPlaylistDeleteInput::new(&self.config.keys)),
                vec![],
            )
            .is_ok());
        assert!(self
            .app
            .remount(
                Id::KeyEditor(IdKeyEditor::PlaylistDeleteAll),
                Box::new(KEPlaylistDeleteAll::new(&self.config.keys)),
                vec![],
            )
            .is_ok());
        assert!(self
            .app
            .remount(
                Id::KeyEditor(IdKeyEditor::PlaylistDeleteAllInput),
                Box::new(KEPlaylistDeleteAllInput::new(&self.config.keys)),
                vec![],
            )
            .is_ok());
        assert!(self
            .app
            .remount(
                Id::KeyEditor(IdKeyEditor::PlaylistAddFront),
                Box::new(KEPlaylistAddFront::new(&self.config.keys)),
                vec![],
            )
            .is_ok());
        assert!(self
            .app
            .remount(
                Id::KeyEditor(IdKeyEditor::PlaylistAddFrontInput),
                Box::new(KEPlaylistAddFrontInput::new(&self.config.keys)),
                vec![],
            )
            .is_ok());
        assert!(self
            .app
            .remount(
                Id::KeyEditor(IdKeyEditor::PlaylistShuffle),
                Box::new(KEPlaylistShuffle::new(&self.config.keys)),
                vec![],
            )
            .is_ok());
        assert!(self
            .app
            .remount(
                Id::KeyEditor(IdKeyEditor::PlaylistShuffleInput),
                Box::new(KEPlaylistShuffleInput::new(&self.config.keys)),
                vec![],
            )
            .is_ok());
        assert!(self
            .app
            .remount(
                Id::KeyEditor(IdKeyEditor::PlaylistSearch),
                Box::new(KEPlaylistSearch::new(&self.config.keys)),
                vec![],
            )
            .is_ok());
        assert!(self
            .app
            .remount(
                Id::KeyEditor(IdKeyEditor::PlaylistSearchInput),
                Box::new(KEPlaylistSearchInput::new(&self.config.keys)),
                vec![],
            )
            .is_ok());
        assert!(self
            .app
            .remount(
                Id::KeyEditor(IdKeyEditor::PlaylistPlaySelected),
                Box::new(KEPlaylistPlaySelected::new(&self.config.keys)),
                vec![],
            )
            .is_ok());
        assert!(self
            .app
            .remount(
                Id::KeyEditor(IdKeyEditor::PlaylistPlaySelectedInput),
                Box::new(KEPlaylistPlaySelectedInput::new(&self.config.keys)),
                vec![],
            )
            .is_ok());
        assert!(self
            .app
            .remount(
                Id::KeyEditor(IdKeyEditor::PlaylistModeCycle),
                Box::new(KEPlaylistModeCycle::new(&self.config.keys)),
                vec![],
            )
            .is_ok());
        assert!(self
            .app
            .remount(
                Id::KeyEditor(IdKeyEditor::PlaylistModeCycleInput),
                Box::new(KEPlaylistModeCycleInput::new(&self.config.keys)),
                vec![],
            )
            .is_ok());

        assert!(self
            .app
            .remount(
                Id::KeyEditor(IdKeyEditor::PlaylistSwapDown),
                Box::new(KEPlaylistSwapDown::new(&self.config.keys)),
                vec![],
            )
            .is_ok());

        assert!(self
            .app
            .remount(
                Id::KeyEditor(IdKeyEditor::PlaylistSwapDownInput),
                Box::new(KEPlaylistSwapDownInput::new(&self.config.keys)),
                vec![],
            )
            .is_ok());

        assert!(self
            .app
            .remount(
                Id::KeyEditor(IdKeyEditor::PlaylistSwapUp),
                Box::new(KEPlaylistSwapUp::new(&self.config.keys)),
                vec![],
            )
            .is_ok());

        assert!(self
            .app
            .remount(
                Id::KeyEditor(IdKeyEditor::PlaylistSwapUpInput),
                Box::new(KEPlaylistSwapUpInput::new(&self.config.keys)),
                vec![],
            )
            .is_ok());

        assert!(self
            .app
            .remount(
                Id::KeyEditor(IdKeyEditor::GlobalLayoutTreeview),
                Box::new(KEGlobalLayoutTreeview::new(&self.config.keys)),
                vec![],
            )
            .is_ok());

        assert!(self
            .app
            .remount(
                Id::KeyEditor(IdKeyEditor::GlobalLayoutTreeviewInput),
                Box::new(KEGlobalLayoutTreeviewInput::new(&self.config.keys)),
                vec![],
            )
            .is_ok());

        assert!(self
            .app
            .remount(
                Id::KeyEditor(IdKeyEditor::GlobalLayoutDatabase),
                Box::new(KEGlobalLayoutDatabase::new(&self.config.keys)),
                vec![],
            )
            .is_ok());

        assert!(self
            .app
            .remount(
                Id::KeyEditor(IdKeyEditor::GlobalLayoutDatabaseInput),
                Box::new(KEGlobalLayoutDatabaseInput::new(&self.config.keys)),
                vec![],
            )
            .is_ok());

        assert!(self
            .app
            .remount(
                Id::KeyEditor(IdKeyEditor::DatabaseAddAll),
                Box::new(KEDatabaseAddAll::new(&self.config.keys)),
                vec![],
            )
            .is_ok());

        assert!(self
            .app
            .remount(
                Id::KeyEditor(IdKeyEditor::DatabaseAddAllInput),
                Box::new(KEDatabaseAddAllInput::new(&self.config.keys)),
                vec![],
            )
            .is_ok());

        // focus
        assert!(self
            .app
            .active(&Id::KeyEditor(IdKeyEditor::GlobalQuit))
            .is_ok());
        // self.theme_select_sync();
        self.app.lock_subs();
        if let Err(e) = self.update_photo() {
            self.mount_error_popup(format!("clear photo error: {}", e).as_str());
        }
    }

    #[allow(clippy::too_many_lines)]
    pub fn umount_key_editor(&mut self) {
        self.app.umount(&Id::KeyEditor(IdKeyEditor::LabelHint)).ok();
        self.app
            .umount(&Id::KeyEditor(IdKeyEditor::GlobalQuit))
            .ok();
        self.app
            .umount(&Id::KeyEditor(IdKeyEditor::GlobalQuitInput))
            .ok();
        self.app
            .umount(&Id::KeyEditor(IdKeyEditor::GlobalLeft))
            .ok();
        self.app
            .umount(&Id::KeyEditor(IdKeyEditor::GlobalLeftInput))
            .ok();
        self.app
            .umount(&Id::KeyEditor(IdKeyEditor::GlobalRight))
            .ok();
        self.app
            .umount(&Id::KeyEditor(IdKeyEditor::GlobalRightInput))
            .ok();
        self.app.umount(&Id::KeyEditor(IdKeyEditor::GlobalUp)).ok();
        self.app
            .umount(&Id::KeyEditor(IdKeyEditor::GlobalUpInput))
            .ok();
        self.app
            .umount(&Id::KeyEditor(IdKeyEditor::GlobalDown))
            .ok();
        self.app
            .umount(&Id::KeyEditor(IdKeyEditor::GlobalDownInput))
            .ok();
        self.app
            .umount(&Id::KeyEditor(IdKeyEditor::GlobalGotoTop))
            .ok();
        self.app
            .umount(&Id::KeyEditor(IdKeyEditor::GlobalGotoTopInput))
            .ok();
        self.app
            .umount(&Id::KeyEditor(IdKeyEditor::GlobalGotoBottom))
            .ok();
        self.app
            .umount(&Id::KeyEditor(IdKeyEditor::GlobalGotoBottomInput))
            .ok();
        self.app
            .umount(&Id::KeyEditor(IdKeyEditor::GlobalPlayerTogglePause))
            .ok();
        self.app
            .umount(&Id::KeyEditor(IdKeyEditor::GlobalPlayerTogglePauseInput))
            .ok();
        self.app
            .umount(&Id::KeyEditor(IdKeyEditor::GlobalPlayerNext))
            .ok();
        self.app
            .umount(&Id::KeyEditor(IdKeyEditor::GlobalPlayerNextInput))
            .ok();
        self.app
            .umount(&Id::KeyEditor(IdKeyEditor::GlobalPlayerPrevious))
            .ok();
        self.app
            .umount(&Id::KeyEditor(IdKeyEditor::GlobalPlayerPreviousInput))
            .ok();

        self.app
            .umount(&Id::KeyEditor(IdKeyEditor::GlobalHelp))
            .ok();
        self.app
            .umount(&Id::KeyEditor(IdKeyEditor::GlobalHelpInput))
            .ok();
        self.app
            .umount(&Id::KeyEditor(IdKeyEditor::GlobalVolumeUp))
            .ok();
        self.app
            .umount(&Id::KeyEditor(IdKeyEditor::GlobalVolumeUpInput))
            .ok();
        self.app
            .umount(&Id::KeyEditor(IdKeyEditor::GlobalVolumeDown))
            .ok();
        self.app
            .umount(&Id::KeyEditor(IdKeyEditor::GlobalVolumeDownInput))
            .ok();

        self.app
            .umount(&Id::KeyEditor(IdKeyEditor::GlobalPlayerSeekForward))
            .ok();
        self.app
            .umount(&Id::KeyEditor(IdKeyEditor::GlobalPlayerSeekForwardInput))
            .ok();
        self.app
            .umount(&Id::KeyEditor(IdKeyEditor::GlobalPlayerSeekBackward))
            .ok();
        self.app
            .umount(&Id::KeyEditor(IdKeyEditor::GlobalPlayerSeekBackwardInput))
            .ok();
        self.app
            .umount(&Id::KeyEditor(IdKeyEditor::GlobalPlayerSpeedUp))
            .ok();
        self.app
            .umount(&Id::KeyEditor(IdKeyEditor::GlobalPlayerSpeedUpInput))
            .ok();
        self.app
            .umount(&Id::KeyEditor(IdKeyEditor::GlobalPlayerSpeedDown))
            .ok();
        self.app
            .umount(&Id::KeyEditor(IdKeyEditor::GlobalPlayerSpeedDownInput))
            .ok();

        self.app
            .umount(&Id::KeyEditor(IdKeyEditor::GlobalLyricAdjustForward))
            .ok();
        self.app
            .umount(&Id::KeyEditor(IdKeyEditor::GlobalLyricAdjustForwardInput))
            .ok();
        self.app
            .umount(&Id::KeyEditor(IdKeyEditor::GlobalLyricAdjustBackward))
            .ok();
        self.app
            .umount(&Id::KeyEditor(IdKeyEditor::GlobalLyricAdjustBackwardInput))
            .ok();
        self.app
            .umount(&Id::KeyEditor(IdKeyEditor::GlobalLyricCycle))
            .ok();
        self.app
            .umount(&Id::KeyEditor(IdKeyEditor::GlobalLyricCycleInput))
            .ok();
        self.app
            .umount(&Id::KeyEditor(IdKeyEditor::GlobalColorEditor))
            .ok();
        self.app
            .umount(&Id::KeyEditor(IdKeyEditor::GlobalColorEditorInput))
            .ok();
        self.app
            .umount(&Id::KeyEditor(IdKeyEditor::GlobalKeyEditor))
            .ok();
        self.app
            .umount(&Id::KeyEditor(IdKeyEditor::GlobalKeyEditorInput))
            .ok();
        self.app
            .umount(&Id::KeyEditor(IdKeyEditor::LibraryDelete))
            .ok();
        self.app
            .umount(&Id::KeyEditor(IdKeyEditor::LibraryDeleteInput))
            .ok();
        self.app
            .umount(&Id::KeyEditor(IdKeyEditor::LibraryLoadDir))
            .ok();
        self.app
            .umount(&Id::KeyEditor(IdKeyEditor::LibraryLoadDirInput))
            .ok();
        self.app
            .umount(&Id::KeyEditor(IdKeyEditor::LibraryYank))
            .ok();
        self.app
            .umount(&Id::KeyEditor(IdKeyEditor::LibraryYankInput))
            .ok();
        self.app
            .umount(&Id::KeyEditor(IdKeyEditor::LibraryPaste))
            .ok();
        self.app
            .umount(&Id::KeyEditor(IdKeyEditor::LibraryPasteInput))
            .ok();
        self.app
            .umount(&Id::KeyEditor(IdKeyEditor::LibrarySearch))
            .ok();
        self.app
            .umount(&Id::KeyEditor(IdKeyEditor::LibrarySearchInput))
            .ok();
        self.app
            .umount(&Id::KeyEditor(IdKeyEditor::LibrarySearchYoutube))
            .ok();
        self.app
            .umount(&Id::KeyEditor(IdKeyEditor::LibrarySearchYoutubeInput))
            .ok();
        self.app
            .umount(&Id::KeyEditor(IdKeyEditor::LibraryTagEditor))
            .ok();
        self.app
            .umount(&Id::KeyEditor(IdKeyEditor::LibraryTagEditorInput))
            .ok();
        self.app
            .umount(&Id::KeyEditor(IdKeyEditor::PlaylistDelete))
            .ok();
        self.app
            .umount(&Id::KeyEditor(IdKeyEditor::PlaylistDeleteInput))
            .ok();

        self.app
            .umount(&Id::KeyEditor(IdKeyEditor::PlaylistDeleteAll))
            .ok();
        self.app
            .umount(&Id::KeyEditor(IdKeyEditor::PlaylistDeleteAllInput))
            .ok();
        self.app
            .umount(&Id::KeyEditor(IdKeyEditor::PlaylistShuffle))
            .ok();
        self.app
            .umount(&Id::KeyEditor(IdKeyEditor::PlaylistShuffleInput))
            .ok();
        self.app
            .umount(&Id::KeyEditor(IdKeyEditor::PlaylistModeCycle))
            .ok();
        self.app
            .umount(&Id::KeyEditor(IdKeyEditor::PlaylistModeCycleInput))
            .ok();
        self.app
            .umount(&Id::KeyEditor(IdKeyEditor::PlaylistPlaySelected))
            .ok();
        self.app
            .umount(&Id::KeyEditor(IdKeyEditor::PlaylistPlaySelectedInput))
            .ok();
        self.app
            .umount(&Id::KeyEditor(IdKeyEditor::PlaylistAddFront))
            .ok();
        self.app
            .umount(&Id::KeyEditor(IdKeyEditor::PlaylistAddFrontInput))
            .ok();
        self.app
            .umount(&Id::KeyEditor(IdKeyEditor::PlaylistSearch))
            .ok();
        self.app
            .umount(&Id::KeyEditor(IdKeyEditor::PlaylistSearchInput))
            .ok();

        self.app
            .umount(&Id::KeyEditor(IdKeyEditor::PlaylistSwapDown))
            .ok();
        self.app
            .umount(&Id::KeyEditor(IdKeyEditor::PlaylistSwapDownInput))
            .ok();

        self.app
            .umount(&Id::KeyEditor(IdKeyEditor::PlaylistSwapUp))
            .ok();
        self.app
            .umount(&Id::KeyEditor(IdKeyEditor::PlaylistSwapUpInput))
            .ok();

        self.app
            .umount(&Id::KeyEditor(IdKeyEditor::GlobalLayoutDatabase))
            .ok();

        self.app
            .umount(&Id::KeyEditor(IdKeyEditor::GlobalLayoutDatabaseInput))
            .ok();

        self.app
            .umount(&Id::KeyEditor(IdKeyEditor::GlobalLayoutTreeview))
            .ok();

        self.app
            .umount(&Id::KeyEditor(IdKeyEditor::GlobalLayoutTreeviewInput))
            .ok();

        self.app
            .umount(&Id::KeyEditor(IdKeyEditor::DatabaseAddAll))
            .ok();

        self.app
            .umount(&Id::KeyEditor(IdKeyEditor::DatabaseAddAllInput))
            .ok();

        self.app.umount(&Id::KeyEditor(IdKeyEditor::RadioOk)).ok();
        self.app.unlock_subs();
        self.library_reload_tree();
        self.playlist_reload();
        self.database_reload();
        self.global_fix_focus();
        assert!(self
            .app
            .remount(
                Id::GlobalListener,
                Box::new(GlobalListener::new(&self.config.keys)),
                Self::subscribe(&self.config.keys),
            )
            .is_ok());

        assert!(self
            .app
            .remount(
                Id::Label,
                Box::new(
                    Label::default()
                        .text(format!(
                            "Press <{}> for help. Version: {}",
                            self.config.keys.global_help, VERSION,
                        ))
                        .alignment(Alignment::Left)
                        .background(Color::Reset)
                        .foreground(Color::Cyan)
                        .modifiers(TextModifiers::BOLD),
                ),
                Vec::default(),
            )
            .is_ok());
        if let Err(e) = self.update_photo() {
            self.mount_error_popup(format!("clear photo error: {}", e).as_str());
        }
    }

    pub fn mount_key_editor_help(&mut self) {
        assert!(self
            .app
            .remount(
                Id::KeyEditor(IdKeyEditor::HelpPopup),
                Box::new(KEHelpPopup::new(&self.config.keys)),
                vec![]
            )
            .is_ok());
        // Active help
        assert!(self
            .app
            .active(&Id::KeyEditor(IdKeyEditor::HelpPopup))
            .is_ok());
    }

    #[allow(clippy::too_many_lines, clippy::cognitive_complexity)]
    fn view_key_editor(&mut self) {
        let select_global_quit_len = match self.app.state(&Id::KeyEditor(IdKeyEditor::GlobalQuit)) {
            Ok(State::One(_)) => 3,
            _ => 8,
        };
        let select_global_left_len = match self.app.state(&Id::KeyEditor(IdKeyEditor::GlobalLeft)) {
            Ok(State::One(_)) => 3,
            _ => 8,
        };
        let select_global_right_len = match self.app.state(&Id::KeyEditor(IdKeyEditor::GlobalRight))
        {
            Ok(State::One(_)) => 3,
            _ => 8,
        };
        let select_global_up_len = match self.app.state(&Id::KeyEditor(IdKeyEditor::GlobalUp)) {
            Ok(State::One(_)) => 3,
            _ => 8,
        };
        let select_global_down_len = match self.app.state(&Id::KeyEditor(IdKeyEditor::GlobalDown)) {
            Ok(State::One(_)) => 3,
            _ => 8,
        };
        let select_global_goto_top_len =
            match self.app.state(&Id::KeyEditor(IdKeyEditor::GlobalGotoTop)) {
                Ok(State::One(_)) => 3,
                _ => 8,
            };
        let select_global_goto_bottom_len = match self
            .app
            .state(&Id::KeyEditor(IdKeyEditor::GlobalGotoBottom))
        {
            Ok(State::One(_)) => 3,
            _ => 8,
        };
        let select_global_player_toggle_pause_len = match self
            .app
            .state(&Id::KeyEditor(IdKeyEditor::GlobalPlayerTogglePause))
        {
            Ok(State::One(_)) => 3,
            _ => 8,
        };
        let select_global_player_next_len = match self
            .app
            .state(&Id::KeyEditor(IdKeyEditor::GlobalPlayerNext))
        {
            Ok(State::One(_)) => 3,
            _ => 8,
        };
        let select_global_player_previous_len = match self
            .app
            .state(&Id::KeyEditor(IdKeyEditor::GlobalPlayerPrevious))
        {
            Ok(State::One(_)) => 3,
            _ => 8,
        };

        let select_global_help_len = match self.app.state(&Id::KeyEditor(IdKeyEditor::GlobalHelp)) {
            Ok(State::One(_)) => 3,
            _ => 8,
        };

        let select_global_volume_up_len =
            match self.app.state(&Id::KeyEditor(IdKeyEditor::GlobalVolumeUp)) {
                Ok(State::One(_)) => 3,
                _ => 8,
            };

        let select_global_volume_down_len = match self
            .app
            .state(&Id::KeyEditor(IdKeyEditor::GlobalVolumeDown))
        {
            Ok(State::One(_)) => 3,
            _ => 8,
        };

        let select_global_player_seek_forward_len = match self
            .app
            .state(&Id::KeyEditor(IdKeyEditor::GlobalPlayerSeekForward))
        {
            Ok(State::One(_)) => 3,
            _ => 8,
        };
        let select_global_player_seek_backward_len = match self
            .app
            .state(&Id::KeyEditor(IdKeyEditor::GlobalPlayerSeekBackward))
        {
            Ok(State::One(_)) => 3,
            _ => 8,
        };
        let select_global_player_speed_up_len = match self
            .app
            .state(&Id::KeyEditor(IdKeyEditor::GlobalPlayerSpeedUp))
        {
            Ok(State::One(_)) => 3,
            _ => 8,
        };
        let select_global_player_speed_down_len = match self
            .app
            .state(&Id::KeyEditor(IdKeyEditor::GlobalPlayerSpeedDown))
        {
            Ok(State::One(_)) => 3,
            _ => 8,
        };

        let select_global_lyric_adjust_forward_len = match self
            .app
            .state(&Id::KeyEditor(IdKeyEditor::GlobalLyricAdjustForward))
        {
            Ok(State::One(_)) => 3,
            _ => 8,
        };
        let select_global_lyric_adjust_backward_len = match self
            .app
            .state(&Id::KeyEditor(IdKeyEditor::GlobalLyricAdjustBackward))
        {
            Ok(State::One(_)) => 3,
            _ => 8,
        };
        let select_global_lyric_cycle_len = match self
            .app
            .state(&Id::KeyEditor(IdKeyEditor::GlobalLyricCycle))
        {
            Ok(State::One(_)) => 3,
            _ => 8,
        };
        let select_global_color_editor_len = match self
            .app
            .state(&Id::KeyEditor(IdKeyEditor::GlobalColorEditor))
        {
            Ok(State::One(_)) => 3,
            _ => 8,
        };
        let select_global_key_editor_len =
            match self.app.state(&Id::KeyEditor(IdKeyEditor::GlobalKeyEditor)) {
                Ok(State::One(_)) => 3,
                _ => 8,
            };

        let select_library_delete_len =
            match self.app.state(&Id::KeyEditor(IdKeyEditor::LibraryDelete)) {
                Ok(State::One(_)) => 3,
                _ => 8,
            };
        let select_library_load_dir_len =
            match self.app.state(&Id::KeyEditor(IdKeyEditor::LibraryLoadDir)) {
                Ok(State::One(_)) => 3,
                _ => 8,
            };
        let select_library_yank_len = match self.app.state(&Id::KeyEditor(IdKeyEditor::LibraryYank))
        {
            Ok(State::One(_)) => 3,
            _ => 8,
        };
        let select_library_paste_len =
            match self.app.state(&Id::KeyEditor(IdKeyEditor::LibraryPaste)) {
                Ok(State::One(_)) => 3,
                _ => 8,
            };
        let select_library_search_len =
            match self.app.state(&Id::KeyEditor(IdKeyEditor::LibrarySearch)) {
                Ok(State::One(_)) => 3,
                _ => 8,
            };
        let select_library_search_youtube_len = match self
            .app
            .state(&Id::KeyEditor(IdKeyEditor::LibrarySearchYoutube))
        {
            Ok(State::One(_)) => 3,
            _ => 8,
        };
        let select_library_tag_editor_len = match self
            .app
            .state(&Id::KeyEditor(IdKeyEditor::LibraryTagEditor))
        {
            Ok(State::One(_)) => 3,
            _ => 8,
        };
        let select_playlist_delete_len =
            match self.app.state(&Id::KeyEditor(IdKeyEditor::PlaylistDelete)) {
                Ok(State::One(_)) => 3,
                _ => 8,
            };
        let select_playlist_delete_all_len = match self
            .app
            .state(&Id::KeyEditor(IdKeyEditor::PlaylistDeleteAll))
        {
            Ok(State::One(_)) => 3,
            _ => 8,
        };

        let select_playlist_shuffle_len =
            match self.app.state(&Id::KeyEditor(IdKeyEditor::PlaylistShuffle)) {
                Ok(State::One(_)) => 3,
                _ => 8,
            };

        let select_playlist_mode_cycle_len = match self
            .app
            .state(&Id::KeyEditor(IdKeyEditor::PlaylistModeCycle))
        {
            Ok(State::One(_)) => 3,
            _ => 8,
        };
        let select_playlist_add_front_len = match self
            .app
            .state(&Id::KeyEditor(IdKeyEditor::PlaylistAddFront))
        {
            Ok(State::One(_)) => 3,
            _ => 8,
        };
        let select_playlist_search_len =
            match self.app.state(&Id::KeyEditor(IdKeyEditor::PlaylistSearch)) {
                Ok(State::One(_)) => 3,
                _ => 8,
            };
        let select_playlist_play_selected_len = match self
            .app
            .state(&Id::KeyEditor(IdKeyEditor::PlaylistPlaySelected))
        {
            Ok(State::One(_)) => 3,
            _ => 8,
        };

        let select_playlist_swap_down_len = match self
            .app
            .state(&Id::KeyEditor(IdKeyEditor::PlaylistSwapDown))
        {
            Ok(State::One(_)) => 3,
            _ => 8,
        };

        let select_playlist_swap_up_len =
            match self.app.state(&Id::KeyEditor(IdKeyEditor::PlaylistSwapUp)) {
                Ok(State::One(_)) => 3,
                _ => 8,
            };

        let select_global_layout_treeview_len = match self
            .app
            .state(&Id::KeyEditor(IdKeyEditor::GlobalLayoutTreeview))
        {
            Ok(State::One(_)) => 3,
            _ => 8,
        };

        let select_global_layout_database_len = match self
            .app
            .state(&Id::KeyEditor(IdKeyEditor::GlobalLayoutDatabase))
        {
            Ok(State::One(_)) => 3,
            _ => 8,
        };

        let select_database_add_all_len =
            match self.app.state(&Id::KeyEditor(IdKeyEditor::DatabaseAddAll)) {
                Ok(State::One(_)) => 3,
                _ => 8,
            };

        assert!(self
            .terminal
            .raw_mut()
            .draw(|f| {
                if self.app.mounted(&Id::KeyEditor(IdKeyEditor::LabelHint)) {
                    f.render_widget(Clear, f.size());
                    let chunks_main = Layout::default()
                        .direction(Direction::Vertical)
                        .margin(0)
                        .constraints(
                            [
                                Constraint::Length(1),
                                Constraint::Min(2),
                                Constraint::Length(3),
                                Constraint::Length(1),
                            ]
                            .as_ref(),
                        )
                        .split(f.size());

                    let chunks_middle = Layout::default()
                        .direction(Direction::Horizontal)
                        .margin(0)
                        .constraints(
                            [
                                Constraint::Ratio(1, 7),
                                Constraint::Ratio(2, 35),
                                Constraint::Ratio(1, 7),
                                Constraint::Ratio(2, 35),
                                Constraint::Ratio(1, 7),
                                Constraint::Ratio(2, 35),
                                Constraint::Ratio(1, 7),
                                Constraint::Ratio(2, 35),
                                Constraint::Ratio(1, 7),
                                Constraint::Ratio(2, 35),
                            ]
                            .as_ref(),
                        )
                        .split(chunks_main[1]);

                    let chunks_middle_column1 = Layout::default()
                        .direction(Direction::Vertical)
                        .margin(0)
                        .constraints(
                            [
                                Constraint::Length(select_global_quit_len),
                                Constraint::Length(select_global_left_len),
                                Constraint::Length(select_global_down_len),
                                Constraint::Length(select_global_up_len),
                                Constraint::Length(select_global_right_len),
                                Constraint::Length(select_global_goto_top_len),
                                Constraint::Length(select_global_goto_bottom_len),
                                Constraint::Length(select_global_player_toggle_pause_len),
                                Constraint::Length(select_global_player_next_len),
                                Constraint::Min(0),
                            ]
                            .as_ref(),
                        )
                        .split(chunks_middle[0]);
                    let chunks_middle_column2 = Layout::default()
                        .direction(Direction::Vertical)
                        .margin(0)
                        .constraints(
                            [
                                Constraint::Length(select_global_quit_len),
                                Constraint::Length(select_global_left_len),
                                Constraint::Length(select_global_down_len),
                                Constraint::Length(select_global_up_len),
                                Constraint::Length(select_global_right_len),
                                Constraint::Length(select_global_goto_top_len),
                                Constraint::Length(select_global_goto_bottom_len),
                                Constraint::Length(select_global_player_toggle_pause_len),
                                Constraint::Length(select_global_player_next_len),
                                Constraint::Min(0),
                            ]
                            .as_ref(),
                        )
                        .split(chunks_middle[1]);
                    let chunks_middle_column3 = Layout::default()
                        .direction(Direction::Vertical)
                        .margin(0)
                        .constraints(
                            [
                                Constraint::Length(select_global_player_previous_len),
                                Constraint::Length(select_global_help_len),
                                Constraint::Length(select_global_volume_up_len),
                                Constraint::Length(select_global_volume_down_len),
                                Constraint::Length(select_global_player_seek_forward_len),
                                Constraint::Length(select_global_player_seek_backward_len),
                                Constraint::Length(select_global_player_speed_up_len),
                                Constraint::Length(select_global_player_speed_down_len),
                                Constraint::Length(select_global_lyric_adjust_forward_len),
                                Constraint::Min(0),
                            ]
                            .as_ref(),
                        )
                        .split(chunks_middle[2]);
                    let chunks_middle_column4 = Layout::default()
                        .direction(Direction::Vertical)
                        .margin(0)
                        .constraints(
                            [
                                Constraint::Length(select_global_player_previous_len),
                                Constraint::Length(select_global_help_len),
                                Constraint::Length(select_global_volume_up_len),
                                Constraint::Length(select_global_volume_down_len),
                                Constraint::Length(select_global_player_seek_forward_len),
                                Constraint::Length(select_global_player_seek_backward_len),
                                Constraint::Length(select_global_player_speed_up_len),
                                Constraint::Length(select_global_player_speed_down_len),
                                Constraint::Length(select_global_lyric_adjust_forward_len),
                                Constraint::Min(0),
                            ]
                            .as_ref(),
                        )
                        .split(chunks_middle[3]);
                    let chunks_middle_column5 = Layout::default()
                        .direction(Direction::Vertical)
                        .margin(0)
                        .constraints(
                            [
                                Constraint::Length(select_global_lyric_adjust_backward_len),
                                Constraint::Length(select_global_lyric_cycle_len),
                                Constraint::Length(select_global_color_editor_len),
                                Constraint::Length(select_global_key_editor_len),
                                Constraint::Length(select_library_tag_editor_len),
                                Constraint::Length(select_library_delete_len),
                                Constraint::Length(select_library_load_dir_len),
                                Constraint::Length(select_library_yank_len),
                                Constraint::Length(select_library_paste_len),
                                Constraint::Min(0),
                            ]
                            .as_ref(),
                        )
                        .split(chunks_middle[4]);
                    let chunks_middle_column6 = Layout::default()
                        .direction(Direction::Vertical)
                        .margin(0)
                        .constraints(
                            [
                                Constraint::Length(select_global_lyric_adjust_backward_len),
                                Constraint::Length(select_global_lyric_cycle_len),
                                Constraint::Length(select_global_color_editor_len),
                                Constraint::Length(select_global_key_editor_len),
                                Constraint::Length(select_library_tag_editor_len),
                                Constraint::Length(select_library_delete_len),
                                Constraint::Length(select_library_load_dir_len),
                                Constraint::Length(select_library_yank_len),
                                Constraint::Length(select_library_paste_len),
                                Constraint::Min(0),
                            ]
                            .as_ref(),
                        )
                        .split(chunks_middle[5]);

                    let chunks_middle_column7 = Layout::default()
                        .direction(Direction::Vertical)
                        .margin(0)
                        .constraints(
                            [
                                Constraint::Length(select_library_search_len),
                                Constraint::Length(select_library_search_youtube_len),
                                Constraint::Length(select_playlist_delete_len),
                                Constraint::Length(select_playlist_delete_all_len),
                                Constraint::Length(select_playlist_search_len),
                                Constraint::Length(select_playlist_shuffle_len),
                                Constraint::Length(select_playlist_add_front_len),
                                Constraint::Length(select_playlist_mode_cycle_len),
                                Constraint::Length(select_playlist_play_selected_len),
                                Constraint::Min(0),
                            ]
                            .as_ref(),
                        )
                        .split(chunks_middle[6]);
                    let chunks_middle_column8 = Layout::default()
                        .direction(Direction::Vertical)
                        .margin(0)
                        .constraints(
                            [
                                Constraint::Length(select_library_search_len),
                                Constraint::Length(select_library_search_youtube_len),
                                Constraint::Length(select_playlist_delete_len),
                                Constraint::Length(select_playlist_delete_all_len),
                                Constraint::Length(select_playlist_search_len),
                                Constraint::Length(select_playlist_shuffle_len),
                                Constraint::Length(select_playlist_add_front_len),
                                Constraint::Length(select_playlist_mode_cycle_len),
                                Constraint::Length(select_playlist_play_selected_len),
                                Constraint::Min(0),
                            ]
                            .as_ref(),
                        )
                        .split(chunks_middle[7]);

                    let chunks_middle_column9 = Layout::default()
                        .direction(Direction::Vertical)
                        .margin(0)
                        .constraints(
                            [
                                Constraint::Length(select_playlist_swap_down_len),
                                Constraint::Length(select_playlist_swap_up_len),
                                Constraint::Length(select_global_layout_treeview_len),
                                Constraint::Length(select_global_layout_database_len),
                                Constraint::Length(select_database_add_all_len),
                                Constraint::Min(0),
                            ]
                            .as_ref(),
                        )
                        .split(chunks_middle[8]);
                    let chunks_middle_column10 = Layout::default()
                        .direction(Direction::Vertical)
                        .margin(0)
                        .constraints(
                            [
                                Constraint::Length(select_playlist_swap_down_len),
                                Constraint::Length(select_playlist_swap_up_len),
                                Constraint::Length(select_global_layout_treeview_len),
                                Constraint::Length(select_global_layout_database_len),
                                Constraint::Length(select_database_add_all_len),
                                Constraint::Min(0),
                            ]
                            .as_ref(),
                        )
                        .split(chunks_middle[9]);

                    self.app
                        .view(&Id::KeyEditor(IdKeyEditor::LabelHint), f, chunks_main[0]);
                    self.app
                        .view(&Id::KeyEditor(IdKeyEditor::RadioOk), f, chunks_main[2]);
                    self.app.view(&Id::Label, f, chunks_main[3]);
                    self.app.view(
                        &Id::KeyEditor(IdKeyEditor::GlobalQuit),
                        f,
                        chunks_middle_column1[0],
                    );
                    self.app.view(
                        &Id::KeyEditor(IdKeyEditor::GlobalQuitInput),
                        f,
                        chunks_middle_column2[0],
                    );
                    self.app.view(
                        &Id::KeyEditor(IdKeyEditor::GlobalLeft),
                        f,
                        chunks_middle_column1[1],
                    );
                    self.app.view(
                        &Id::KeyEditor(IdKeyEditor::GlobalLeftInput),
                        f,
                        chunks_middle_column2[1],
                    );
                    self.app.view(
                        &Id::KeyEditor(IdKeyEditor::GlobalDown),
                        f,
                        chunks_middle_column1[2],
                    );
                    self.app.view(
                        &Id::KeyEditor(IdKeyEditor::GlobalDownInput),
                        f,
                        chunks_middle_column2[2],
                    );
                    self.app.view(
                        &Id::KeyEditor(IdKeyEditor::GlobalUp),
                        f,
                        chunks_middle_column1[3],
                    );
                    self.app.view(
                        &Id::KeyEditor(IdKeyEditor::GlobalUpInput),
                        f,
                        chunks_middle_column2[3],
                    );
                    self.app.view(
                        &Id::KeyEditor(IdKeyEditor::GlobalRight),
                        f,
                        chunks_middle_column1[4],
                    );
                    self.app.view(
                        &Id::KeyEditor(IdKeyEditor::GlobalRightInput),
                        f,
                        chunks_middle_column2[4],
                    );
                    self.app.view(
                        &Id::KeyEditor(IdKeyEditor::GlobalGotoTop),
                        f,
                        chunks_middle_column1[5],
                    );
                    self.app.view(
                        &Id::KeyEditor(IdKeyEditor::GlobalGotoTopInput),
                        f,
                        chunks_middle_column2[5],
                    );
                    self.app.view(
                        &Id::KeyEditor(IdKeyEditor::GlobalGotoBottom),
                        f,
                        chunks_middle_column1[6],
                    );
                    self.app.view(
                        &Id::KeyEditor(IdKeyEditor::GlobalGotoBottomInput),
                        f,
                        chunks_middle_column2[6],
                    );

                    self.app.view(
                        &Id::KeyEditor(IdKeyEditor::GlobalPlayerTogglePause),
                        f,
                        chunks_middle_column1[7],
                    );
                    self.app.view(
                        &Id::KeyEditor(IdKeyEditor::GlobalPlayerTogglePauseInput),
                        f,
                        chunks_middle_column2[7],
                    );

                    self.app.view(
                        &Id::KeyEditor(IdKeyEditor::GlobalPlayerNext),
                        f,
                        chunks_middle_column1[8],
                    );
                    self.app.view(
                        &Id::KeyEditor(IdKeyEditor::GlobalPlayerNextInput),
                        f,
                        chunks_middle_column2[8],
                    );

                    self.app.view(
                        &Id::KeyEditor(IdKeyEditor::GlobalPlayerPrevious),
                        f,
                        chunks_middle_column3[0],
                    );
                    self.app.view(
                        &Id::KeyEditor(IdKeyEditor::GlobalPlayerPreviousInput),
                        f,
                        chunks_middle_column4[0],
                    );
                    self.app.view(
                        &Id::KeyEditor(IdKeyEditor::GlobalHelp),
                        f,
                        chunks_middle_column3[1],
                    );
                    self.app.view(
                        &Id::KeyEditor(IdKeyEditor::GlobalHelpInput),
                        f,
                        chunks_middle_column4[1],
                    );
                    self.app.view(
                        &Id::KeyEditor(IdKeyEditor::GlobalVolumeUp),
                        f,
                        chunks_middle_column3[2],
                    );
                    self.app.view(
                        &Id::KeyEditor(IdKeyEditor::GlobalVolumeUpInput),
                        f,
                        chunks_middle_column4[2],
                    );

                    self.app.view(
                        &Id::KeyEditor(IdKeyEditor::GlobalVolumeDown),
                        f,
                        chunks_middle_column3[3],
                    );
                    self.app.view(
                        &Id::KeyEditor(IdKeyEditor::GlobalVolumeDownInput),
                        f,
                        chunks_middle_column4[3],
                    );

                    self.app.view(
                        &Id::KeyEditor(IdKeyEditor::GlobalPlayerSeekForward),
                        f,
                        chunks_middle_column3[4],
                    );
                    self.app.view(
                        &Id::KeyEditor(IdKeyEditor::GlobalPlayerSeekForwardInput),
                        f,
                        chunks_middle_column4[4],
                    );

                    self.app.view(
                        &Id::KeyEditor(IdKeyEditor::GlobalPlayerSeekBackward),
                        f,
                        chunks_middle_column3[5],
                    );
                    self.app.view(
                        &Id::KeyEditor(IdKeyEditor::GlobalPlayerSeekBackwardInput),
                        f,
                        chunks_middle_column4[5],
                    );

                    self.app.view(
                        &Id::KeyEditor(IdKeyEditor::GlobalPlayerSpeedUp),
                        f,
                        chunks_middle_column3[6],
                    );
                    self.app.view(
                        &Id::KeyEditor(IdKeyEditor::GlobalPlayerSpeedUpInput),
                        f,
                        chunks_middle_column4[6],
                    );

                    self.app.view(
                        &Id::KeyEditor(IdKeyEditor::GlobalPlayerSpeedDown),
                        f,
                        chunks_middle_column3[7],
                    );
                    self.app.view(
                        &Id::KeyEditor(IdKeyEditor::GlobalPlayerSpeedDownInput),
                        f,
                        chunks_middle_column4[7],
                    );

                    self.app.view(
                        &Id::KeyEditor(IdKeyEditor::GlobalLyricAdjustForward),
                        f,
                        chunks_middle_column3[8],
                    );
                    self.app.view(
                        &Id::KeyEditor(IdKeyEditor::GlobalLyricAdjustForwardInput),
                        f,
                        chunks_middle_column4[8],
                    );

                    self.app.view(
                        &Id::KeyEditor(IdKeyEditor::GlobalLyricAdjustBackward),
                        f,
                        chunks_middle_column5[0],
                    );
                    self.app.view(
                        &Id::KeyEditor(IdKeyEditor::GlobalLyricAdjustBackwardInput),
                        f,
                        chunks_middle_column6[0],
                    );

                    self.app.view(
                        &Id::KeyEditor(IdKeyEditor::GlobalLyricCycle),
                        f,
                        chunks_middle_column5[1],
                    );
                    self.app.view(
                        &Id::KeyEditor(IdKeyEditor::GlobalLyricCycleInput),
                        f,
                        chunks_middle_column6[1],
                    );
                    self.app.view(
                        &Id::KeyEditor(IdKeyEditor::GlobalColorEditor),
                        f,
                        chunks_middle_column5[2],
                    );
                    self.app.view(
                        &Id::KeyEditor(IdKeyEditor::GlobalColorEditorInput),
                        f,
                        chunks_middle_column6[2],
                    );
                    self.app.view(
                        &Id::KeyEditor(IdKeyEditor::GlobalKeyEditor),
                        f,
                        chunks_middle_column5[3],
                    );
                    self.app.view(
                        &Id::KeyEditor(IdKeyEditor::GlobalKeyEditorInput),
                        f,
                        chunks_middle_column6[3],
                    );
                    self.app.view(
                        &Id::KeyEditor(IdKeyEditor::LibraryTagEditor),
                        f,
                        chunks_middle_column5[4],
                    );
                    self.app.view(
                        &Id::KeyEditor(IdKeyEditor::LibraryTagEditorInput),
                        f,
                        chunks_middle_column6[4],
                    );

                    self.app.view(
                        &Id::KeyEditor(IdKeyEditor::LibraryDelete),
                        f,
                        chunks_middle_column5[5],
                    );
                    self.app.view(
                        &Id::KeyEditor(IdKeyEditor::LibraryDeleteInput),
                        f,
                        chunks_middle_column6[5],
                    );

                    self.app.view(
                        &Id::KeyEditor(IdKeyEditor::LibraryLoadDir),
                        f,
                        chunks_middle_column5[6],
                    );
                    self.app.view(
                        &Id::KeyEditor(IdKeyEditor::LibraryLoadDirInput),
                        f,
                        chunks_middle_column6[6],
                    );

                    self.app.view(
                        &Id::KeyEditor(IdKeyEditor::LibraryYank),
                        f,
                        chunks_middle_column5[7],
                    );
                    self.app.view(
                        &Id::KeyEditor(IdKeyEditor::LibraryYankInput),
                        f,
                        chunks_middle_column6[7],
                    );

                    self.app.view(
                        &Id::KeyEditor(IdKeyEditor::LibraryPaste),
                        f,
                        chunks_middle_column5[8],
                    );
                    self.app.view(
                        &Id::KeyEditor(IdKeyEditor::LibraryPasteInput),
                        f,
                        chunks_middle_column6[8],
                    );

                    self.app.view(
                        &Id::KeyEditor(IdKeyEditor::LibrarySearch),
                        f,
                        chunks_middle_column7[0],
                    );
                    self.app.view(
                        &Id::KeyEditor(IdKeyEditor::LibrarySearchInput),
                        f,
                        chunks_middle_column8[0],
                    );

                    self.app.view(
                        &Id::KeyEditor(IdKeyEditor::LibrarySearchYoutube),
                        f,
                        chunks_middle_column7[1],
                    );
                    self.app.view(
                        &Id::KeyEditor(IdKeyEditor::LibrarySearchYoutubeInput),
                        f,
                        chunks_middle_column8[1],
                    );

                    self.app.view(
                        &Id::KeyEditor(IdKeyEditor::PlaylistDelete),
                        f,
                        chunks_middle_column7[2],
                    );
                    self.app.view(
                        &Id::KeyEditor(IdKeyEditor::PlaylistDeleteInput),
                        f,
                        chunks_middle_column8[2],
                    );
                    self.app.view(
                        &Id::KeyEditor(IdKeyEditor::PlaylistDeleteAll),
                        f,
                        chunks_middle_column7[3],
                    );
                    self.app.view(
                        &Id::KeyEditor(IdKeyEditor::PlaylistDeleteAllInput),
                        f,
                        chunks_middle_column8[3],
                    );
                    self.app.view(
                        &Id::KeyEditor(IdKeyEditor::PlaylistSearch),
                        f,
                        chunks_middle_column7[4],
                    );
                    self.app.view(
                        &Id::KeyEditor(IdKeyEditor::PlaylistSearchInput),
                        f,
                        chunks_middle_column8[4],
                    );

                    self.app.view(
                        &Id::KeyEditor(IdKeyEditor::PlaylistShuffle),
                        f,
                        chunks_middle_column7[5],
                    );
                    self.app.view(
                        &Id::KeyEditor(IdKeyEditor::PlaylistShuffleInput),
                        f,
                        chunks_middle_column8[5],
                    );

                    self.app.view(
                        &Id::KeyEditor(IdKeyEditor::PlaylistAddFront),
                        f,
                        chunks_middle_column7[6],
                    );
                    self.app.view(
                        &Id::KeyEditor(IdKeyEditor::PlaylistAddFrontInput),
                        f,
                        chunks_middle_column8[6],
                    );

                    self.app.view(
                        &Id::KeyEditor(IdKeyEditor::PlaylistModeCycle),
                        f,
                        chunks_middle_column7[7],
                    );
                    self.app.view(
                        &Id::KeyEditor(IdKeyEditor::PlaylistModeCycleInput),
                        f,
                        chunks_middle_column8[7],
                    );

                    self.app.view(
                        &Id::KeyEditor(IdKeyEditor::PlaylistPlaySelected),
                        f,
                        chunks_middle_column7[8],
                    );
                    self.app.view(
                        &Id::KeyEditor(IdKeyEditor::PlaylistPlaySelectedInput),
                        f,
                        chunks_middle_column8[8],
                    );

                    self.app.view(
                        &Id::KeyEditor(IdKeyEditor::PlaylistSwapDown),
                        f,
                        chunks_middle_column9[0],
                    );
                    self.app.view(
                        &Id::KeyEditor(IdKeyEditor::PlaylistSwapDownInput),
                        f,
                        chunks_middle_column10[0],
                    );

                    self.app.view(
                        &Id::KeyEditor(IdKeyEditor::PlaylistSwapUp),
                        f,
                        chunks_middle_column9[1],
                    );
                    self.app.view(
                        &Id::KeyEditor(IdKeyEditor::PlaylistSwapUpInput),
                        f,
                        chunks_middle_column10[1],
                    );

                    self.app.view(
                        &Id::KeyEditor(IdKeyEditor::GlobalLayoutTreeview),
                        f,
                        chunks_middle_column9[2],
                    );
                    self.app.view(
                        &Id::KeyEditor(IdKeyEditor::GlobalLayoutTreeviewInput),
                        f,
                        chunks_middle_column10[2],
                    );

                    self.app.view(
                        &Id::KeyEditor(IdKeyEditor::GlobalLayoutDatabase),
                        f,
                        chunks_middle_column9[3],
                    );
                    self.app.view(
                        &Id::KeyEditor(IdKeyEditor::GlobalLayoutDatabaseInput),
                        f,
                        chunks_middle_column10[3],
                    );

                    self.app.view(
                        &Id::KeyEditor(IdKeyEditor::DatabaseAddAll),
                        f,
                        chunks_middle_column9[4],
                    );
                    self.app.view(
                        &Id::KeyEditor(IdKeyEditor::DatabaseAddAllInput),
                        f,
                        chunks_middle_column10[4],
                    );
                    if self.app.mounted(&Id::KeyEditor(IdKeyEditor::HelpPopup)) {
                        let popup = draw_area_in_relative(f.size(), 50, 70);
                        f.render_widget(Clear, popup);
                        self.app
                            .view(&Id::KeyEditor(IdKeyEditor::HelpPopup), f, popup);
                    }
                }
            })
            .is_ok());
    }
}
