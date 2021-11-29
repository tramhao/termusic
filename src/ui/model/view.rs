use crate::{
    ui::{Application, Id, Msg},
    VERSION,
};

use crate::ui::components::{
    draw_area_in, draw_area_top_right, DeleteConfirmInputPopup, DeleteConfirmRadioPopup,
    ErrorPopup, GlobalListener, HelpPopup, Label, LibrarySearchInputPopup, LibrarySearchTablePopup,
    Lyric, MessagePopup, MusicLibrary, Playlist, Progress, QuitPopup, YoutubeSearchInputPopup,
    YoutubeSearchTablePopup,
};
use crate::ui::model::Model;
use std::time::{Duration, Instant};
use tui_realm_treeview::Tree;
use tuirealm::props::{Alignment, AttrValue, Attribute, Color, PropPayload, TextModifiers};
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
                .default_input_listener(Duration::from_millis(30))
                .poll_timeout(Duration::from_millis(1000))
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
                    } else if self.app.mounted(&Id::ErrorPopup) {
                        let popup = draw_area_in(f.size(), 50, 15);
                        f.render_widget(Clear, popup);
                        self.app.view(&Id::ErrorPopup, f, popup);
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
                    } else if self.app.mounted(&Id::LibrarySearchInput) {
                        let popup = draw_area_in(f.size(), 76, 60);
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
                        self.app.view(&Id::LibrarySearchInput, f, popup_chunks[0]);
                        self.app.view(&Id::LibrarySearchTable, f, popup_chunks[1]);
                    } else if self.app.mounted(&Id::YoutubeSearchInputPopup) {
                        let popup = draw_area_in(f.size(), 30, 10);
                        f.render_widget(Clear, popup);
                        self.app.view(&Id::YoutubeSearchInputPopup, f, popup);
                    } else if self.app.mounted(&Id::YoutubeSearchTablePopup) {
                        let popup = draw_area_in(f.size(), 60, 70);
                        f.render_widget(Clear, popup);
                        self.app.view(&Id::YoutubeSearchTablePopup, f, popup);
                    }

                    if self.app.mounted(&Id::MessagePopup) {
                        let popup = draw_area_top_right(f.size(), 32, 15);
                        f.render_widget(Clear, popup);
                        self.app.view(&Id::MessagePopup, f, popup);
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
        self.app.lock_subs();
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
                Id::LibrarySearchInput,
                Box::new(LibrarySearchInputPopup::default()),
                vec![]
            )
            .is_ok());
        assert!(self
            .app
            .remount(
                Id::LibrarySearchTable,
                Box::new(LibrarySearchTablePopup::default()),
                vec![]
            )
            .is_ok());

        assert!(self.app.active(&Id::LibrarySearchInput).is_ok());
        self.app.lock_subs();
    }

    pub fn mount_youtube_search_input(&mut self) {
        assert!(self
            .app
            .remount(
                Id::YoutubeSearchInputPopup,
                Box::new(YoutubeSearchInputPopup::default()),
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
                Box::new(YoutubeSearchTablePopup::default()),
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
}
