use crate::{
    song::Song,
    ui::{Application, Id, Msg},
    VERSION,
};

use crate::ui::components::{
    draw_area_in, draw_area_top_right, DeleteConfirmInputPopup, DeleteConfirmRadioPopup,
    ErrorPopup, GSInputPopup, GSTablePopup, GlobalListener, HelpPopup, Label, Lyric, MessagePopup,
    MusicLibrary, Playlist, Progress, QuitPopup, Source, TECounterDelete, TEHelpPopup,
    TEInputArtist, TEInputTitle, TERadioTag, TESelectLyric, TETableLyricOptions, TETextareaLyric,
    YSInputPopup, YSTablePopup,
};
use crate::ui::model::Model;
use std::path::Path;
use std::str::FromStr;
use std::time::{Duration, Instant};
use tui_realm_treeview::Tree;
use tuirealm::props::{
    Alignment, AttrValue, Attribute, Color, PropPayload, PropValue, TextModifiers, TextSpan,
};
use tuirealm::tui::layout::{Constraint, Direction, Layout};
use tuirealm::tui::widgets::Clear;
use tuirealm::{EventListenerCfg, NoUserEvent};

impl Model {
    pub fn init_app(tree: &Tree) -> Application<Id, Msg, NoUserEvent> {
        // Setup application
        // NOTE: NoUserEvent is a shorthand to tell tui-realm we're not going to use any custom user event
        // NOTE: the event listener is configured to use the default crossterm input listener and to raise a Tick event each second
        // which we will use to update the clock

        let mut app: Application<Id, Msg, NoUserEvent> = Application::init(
            EventListenerCfg::default()
                .default_input_listener(Duration::from_millis(25))
                .poll_timeout(Duration::from_millis(25))
                .tick_interval(Duration::from_secs(1)),
        );
        assert!(app
            .mount(Id::Library, Box::new(MusicLibrary::new(tree, None)), vec![])
            .is_ok());
        assert!(app
            .mount(Id::Playlist, Box::new(Playlist::default()), vec![])
            .is_ok());
        assert!(app
            .mount(Id::Progress, Box::new(Progress::default()), vec![])
            .is_ok());
        assert!(app
            .mount(Id::Lyric, Box::new(Lyric::default()), vec![])
            .is_ok());
        assert!(app
            .mount(
                Id::Label,
                Box::new(
                    Label::default()
                        .text(format!("Press <CTRL+H> for help. Version: {}", VERSION,))
                        .alignment(Alignment::Left)
                        .background(Color::Reset)
                        .foreground(Color::Cyan)
                        .modifiers(TextModifiers::BOLD),
                ),
                Vec::default(),
            )
            .is_ok());
        // Mount counters
        assert!(app
            .mount(
                Id::GlobalListener,
                Box::new(GlobalListener::default()),
                Self::subscribe(),
            )
            .is_ok());
        // Active letter counter
        assert!(app.active(&Id::Library).is_ok());
        app
    }

    #[allow(clippy::too_many_lines)]
    pub fn view(&mut self) {
        if self.redraw {
            self.redraw = false;
            self.last_redraw = Instant::now();
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

                    // app.view(&Id::Progress, f, chunks_right[1]);

                    self.app.view(&Id::Library, f, chunks_left[0]);
                    self.app.view(&Id::Playlist, f, chunks_right[0]);
                    self.app.view(&Id::Progress, f, chunks_right[1]);
                    self.app.view(&Id::Lyric, f, chunks_right[2]);
                    self.app.view(&Id::Label, f, chunks_main[1]);
                    // -- popups
                    if self.app.mounted(&Id::QuitPopup) {
                        let popup = draw_area_in(f.size(), 30, 10);
                        f.render_widget(Clear, popup);
                        self.app.view(&Id::QuitPopup, f, popup);
                    } else if self.app.mounted(&Id::HelpPopup) {
                        let popup = draw_area_in(f.size(), 60, 90);
                        f.render_widget(Clear, popup);
                        self.app.view(&Id::HelpPopup, f, popup);
                    } else if self.app.mounted(&Id::DeleteConfirmRadioPopup) {
                        let popup = draw_area_in(f.size(), 30, 10);
                        f.render_widget(Clear, popup);
                        self.app.view(&Id::DeleteConfirmRadioPopup, f, popup);
                    } else if self.app.mounted(&Id::DeleteConfirmInputPopup) {
                        let popup = draw_area_in(f.size(), 30, 10);
                        f.render_widget(Clear, popup);
                        self.app.view(&Id::DeleteConfirmInputPopup, f, popup);
                    } else if self.app.mounted(&Id::GeneralSearchInput) {
                        let popup = draw_area_in(f.size(), 65, 68);
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
                        self.app.view(&Id::GeneralSearchInput, f, popup_chunks[0]);
                        self.app.view(&Id::GeneralSearchTable, f, popup_chunks[1]);
                    } else if self.app.mounted(&Id::YoutubeSearchInputPopup) {
                        let popup = draw_area_in(f.size(), 30, 10);
                        f.render_widget(Clear, popup);
                        self.app.view(&Id::YoutubeSearchInputPopup, f, popup);
                    } else if self.app.mounted(&Id::YoutubeSearchTablePopup) {
                        let popup = draw_area_in(f.size(), 65, 68);
                        f.render_widget(Clear, popup);
                        self.app.view(&Id::YoutubeSearchTablePopup, f, popup);
                    } else if self.app.mounted(&Id::TELabelHint) {
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
                            .constraints(
                                [Constraint::Ratio(3, 5), Constraint::Ratio(2, 5)].as_ref(),
                            )
                            .split(chunks_main[2]);

                        let chunks_middle2_right = Layout::default()
                            .direction(Direction::Vertical)
                            .margin(0)
                            .constraints([Constraint::Length(6), Constraint::Min(2)].as_ref())
                            .split(chunks_middle2[1]);

                        let chunks_middle2_right_top = Layout::default()
                            .direction(Direction::Horizontal)
                            .margin(0)
                            .constraints(
                                [Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)].as_ref(),
                            )
                            .split(chunks_middle2_right[0]);

                        self.app.view(&Id::TELabelHint, f, chunks_main[0]);
                        self.app.view(&Id::Label, f, chunks_main[3]);
                        self.app.view(&Id::TEInputArtist, f, chunks_middle1[0]);
                        self.app.view(&Id::TEInputTitle, f, chunks_middle1[1]);
                        self.app.view(&Id::TERadioTag, f, chunks_middle1[2]);
                        self.app
                            .view(&Id::TETableLyricOptions, f, chunks_middle2[0]);
                        self.app
                            .view(&Id::TESelectLyric, f, chunks_middle2_right_top[0]);
                        self.app
                            .view(&Id::TECounterDelete, f, chunks_middle2_right_top[1]);
                        self.app
                            .view(&Id::TETextareaLyric, f, chunks_middle2_right[1]);

                        if self.app.mounted(&Id::TEHelpPopup) {
                            let popup = draw_area_in(f.size(), 50, 70);
                            f.render_widget(Clear, popup);
                            self.app.view(&Id::TEHelpPopup, f, popup);
                        }
                    }
                    if self.app.mounted(&Id::MessagePopup) {
                        let popup = draw_area_top_right(f.size(), 32, 15);
                        f.render_widget(Clear, popup);
                        self.app.view(&Id::MessagePopup, f, popup);
                    }
                    if self.app.mounted(&Id::ErrorPopup) {
                        let popup = draw_area_in(f.size(), 50, 10);
                        f.render_widget(Clear, popup);
                        self.app.view(&Id::ErrorPopup, f, popup);
                    }
                })
                .is_ok());
        }
    }

    // Mount error and give focus to it
    pub fn mount_error_popup(&mut self, err: &str) {
        // pub fn mount_error_popup(&mut self, err: impl ToString) {
        assert!(self
            .app
            .remount(
                Id::ErrorPopup,
                Box::new(ErrorPopup::new(err.to_string())),
                vec![]
            )
            .is_ok());
        assert!(self.app.active(&Id::ErrorPopup).is_ok());
        // self.app.lock_subs();
    }
    /// Mount quit popup
    pub fn mount_quit_popup(&mut self) {
        assert!(self
            .app
            .remount(Id::QuitPopup, Box::new(QuitPopup::default()), vec![])
            .is_ok());
        assert!(self.app.active(&Id::QuitPopup).is_ok());
        self.app.lock_subs();
    }
    /// Mount help popup
    pub fn mount_help_popup(&mut self) {
        assert!(self
            .app
            .remount(Id::HelpPopup, Box::new(HelpPopup::default()), vec![])
            .is_ok());
        assert!(self.app.active(&Id::HelpPopup).is_ok());
        self.app.lock_subs();
    }

    pub fn mount_confirm_radio(&mut self) {
        assert!(self
            .app
            .remount(
                Id::DeleteConfirmRadioPopup,
                Box::new(DeleteConfirmRadioPopup::default()),
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
                Box::new(DeleteConfirmInputPopup::default()),
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
                Box::new(GSInputPopup::new(Source::Library)),
                vec![]
            )
            .is_ok());
        assert!(self
            .app
            .remount(
                Id::GeneralSearchTable,
                Box::new(GSTablePopup::new(Source::Library)),
                vec![]
            )
            .is_ok());

        assert!(self.app.active(&Id::GeneralSearchInput).is_ok());
        self.app.lock_subs();
    }

    pub fn mount_search_playlist(&mut self) {
        assert!(self
            .app
            .remount(
                Id::GeneralSearchInput,
                Box::new(GSInputPopup::new(Source::Playlist)),
                vec![]
            )
            .is_ok());
        assert!(self
            .app
            .remount(
                Id::GeneralSearchTable,
                Box::new(GSTablePopup::new(Source::Playlist)),
                vec![]
            )
            .is_ok());
        assert!(self
            .app
            .attr(
                &Id::GeneralSearchTable,
                Attribute::Title,
                AttrValue::Title((
                    "Results:(Enter: locate/l: play selected)".to_string(),
                    Alignment::Left
                ))
            )
            .is_ok());

        assert!(self.app.active(&Id::GeneralSearchInput).is_ok());
        self.app.lock_subs();
    }

    pub fn mount_youtube_search_input(&mut self) {
        assert!(self
            .app
            .remount(
                Id::YoutubeSearchInputPopup,
                Box::new(YSInputPopup::default()),
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
                Box::new(YSTablePopup::default()),
                vec![]
            )
            .is_ok());
        assert!(self.app.active(&Id::YoutubeSearchTablePopup).is_ok());
        self.app.lock_subs();
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

        if let Err(e) = self.clear_photo() {
            self.mount_error_popup(format!("clear photo error: {}", e).as_str());
        }
        let p = p.to_string_lossy();
        match Song::from_str(&p) {
            Ok(s) => {
                assert!(self
                    .app
                    .remount(
                        Id::TELabelHint,
                        Box::new(
                            Label::default()
                                .text("Press <ENTER> to search:".to_string())
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
                        Id::TEInputArtist,
                        Box::new(TEInputArtist::default()),
                        vec![]
                    )
                    .is_ok());
                assert!(self
                    .app
                    .remount(Id::TEInputTitle, Box::new(TEInputTitle::default()), vec![])
                    .is_ok());
                assert!(self
                    .app
                    .remount(Id::TERadioTag, Box::new(TERadioTag::default()), vec![])
                    .is_ok());
                assert!(self
                    .app
                    .remount(
                        Id::TETableLyricOptions,
                        Box::new(TETableLyricOptions::default()),
                        vec![]
                    )
                    .is_ok());
                assert!(self
                    .app
                    .remount(
                        Id::TESelectLyric,
                        Box::new(TESelectLyric::default()),
                        vec![]
                    )
                    .is_ok());
                assert!(self
                    .app
                    .remount(
                        Id::TECounterDelete,
                        Box::new(TECounterDelete::new(5)),
                        vec![]
                    )
                    .is_ok());
                assert!(self
                    .app
                    .remount(
                        Id::TETextareaLyric,
                        Box::new(TETextareaLyric::default()),
                        vec![]
                    )
                    .is_ok());

                self.app.active(&Id::TEInputArtist).ok();
                self.app.lock_subs();
                self.init_by_song(&s);
            }
            Err(e) => {
                self.mount_error_popup(format!("song load error: {}", e).as_ref());
            }
        };
    }
    pub fn umount_tageditor(&mut self) {
        self.app.umount(&Id::TELabelHint).ok();
        // self.app.umount(&Id::TELabelHelp).ok();
        self.app.umount(&Id::TEInputArtist).ok();
        self.app.umount(&Id::TEInputTitle).ok();
        self.app.umount(&Id::TERadioTag).ok();
        self.app.umount(&Id::TETableLyricOptions).ok();
        self.app.umount(&Id::TESelectLyric).ok();
        self.app.umount(&Id::TECounterDelete).ok();
        self.app.umount(&Id::TETextareaLyric).ok();
        if let Err(e) = self.update_photo() {
            self.mount_error_popup(format!("update photo error: {}", e).as_ref());
        }
        self.app.unlock_subs();
    }
    // initialize the value in tageditor based on info from Song
    #[allow(clippy::cast_possible_wrap)]
    pub fn init_by_song(&mut self, s: &Song) {
        self.tageditor_song = Some(s.clone());
        if let Some(artist) = s.artist() {
            assert!(self
                .app
                .attr(
                    &Id::TEInputArtist,
                    Attribute::Value,
                    AttrValue::String(artist.to_string()),
                )
                .is_ok());
        }

        if let Some(title) = s.title() {
            assert!(self
                .app
                .attr(
                    &Id::TEInputTitle,
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
                &Id::TESelectLyric,
                Attribute::Content,
                AttrValue::Payload(PropPayload::Vec(
                    vec_lang
                        .iter()
                        .map(|x| PropValue::Str((*x).to_string()))
                        .collect(),
                )),
            )
            .is_ok());
        assert!(self
            .app
            .attr(
                &Id::TECounterDelete,
                Attribute::Value,
                AttrValue::Number(vec_lang.len() as isize),
            )
            .is_ok());

        let mut vec_lyric: Vec<TextSpan> = vec![];
        if let Some(f) = s.lyric_selected() {
            for line in f.text.split('\n') {
                vec_lyric.push(TextSpan::from(line));
            }
        }
        assert!(self
            .app
            .attr(
                &Id::TETextareaLyric,
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
                &Id::TETextareaLyric,
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
                &Id::TESelectLyric,
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
            .attr(&Id::TECounterDelete, Attribute::Value, AttrValue::Number(0),)
            .is_ok());

        assert!(self
            .app
            .attr(
                &Id::TETextareaLyric,
                Attribute::Title,
                AttrValue::Title(("Empty Lyric".to_string(), Alignment::Left))
            )
            .is_ok());
        assert!(self
            .app
            .attr(
                &Id::TETextareaLyric,
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
            .remount(Id::TEHelpPopup, Box::new(TEHelpPopup::default()), vec![])
            .is_ok());
        // Active help
        assert!(self.app.active(&Id::TEHelpPopup).is_ok());
    }
}
