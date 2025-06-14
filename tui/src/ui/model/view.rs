//! SPDX-License-Identifier: MIT

use std::path::Path;
use std::time::{Duration, Instant};

use anyhow::{bail, Result};
use termusiclib::config::SharedTuiSettings;
use termusiclib::ids::{Id, IdConfigEditor, IdTagEditor};
use termusiclib::types::{DBMsg, Msg, PCMsg};
use termusiclib::utils::get_parent_folder;
use tui_realm_treeview::Tree;
use tuirealm::props::{AttrValue, Attribute, Color, PropPayload, PropValue, TextSpan};
use tuirealm::ratatui::layout::{Constraint, Direction, Layout};
use tuirealm::ratatui::widgets::Clear;
use tuirealm::EventListenerCfg;
use tuirealm::{Frame, State, StateValue};

use crate::ui::components::{
    DBListCriteria, DBListSearchResult, DBListSearchTracks, DownloadSpinner, EpisodeList,
    FeedsList, Footer, GSInputPopup, GSTablePopup, GlobalListener, LabelSpan, Lyric, MusicLibrary,
    Playlist, Progress, Source,
};
use crate::ui::model::{ConfigEditorLayout, Model, TermusicLayout, UserEvent};
use crate::ui::utils::{
    draw_area_in_absolute, draw_area_in_relative, draw_area_top_right_absolute,
};
use crate::ui::Application;

impl Model {
    #[allow(clippy::too_many_lines)]
    pub fn init_app(
        tree: &Tree<String>,
        config: &SharedTuiSettings,
    ) -> Application<Id, Msg, UserEvent> {
        // Setup application
        // NOTE: NoUserEvent is a shorthand to tell tui-realm we're not going to use any custom user event
        // NOTE: the event listener is configured to use the default crossterm input listener and to raise a Tick event each second
        // which we will use to update the clock

        let mut app: Application<Id, Msg, UserEvent> = Application::init(
            EventListenerCfg::default()
                .crossterm_input_listener(Duration::from_millis(20), 20)
                .poll_timeout(Duration::from_millis(10))
                .tick_interval(Duration::from_secs(1)),
        );

        Self::mount_main(&mut app, config, tree).unwrap();

        // Mount global hotkey listener
        assert!(app
            .mount(
                Id::GlobalListener,
                Box::new(GlobalListener::new(config.clone())),
                Self::subscribe(&config.read().settings.keys),
            )
            .is_ok());
        // Active library
        assert!(app.active(&Id::Library).is_ok());
        app
    }

    /// Mount the Main components for the TUI
    fn mount_main(
        app: &mut Application<Id, Msg, UserEvent>,
        config: &SharedTuiSettings,
        tree: &Tree<String>,
    ) -> Result<()> {
        app.mount(
            Id::Library,
            Box::new(MusicLibrary::new(tree, None, config.clone())),
            Vec::new(),
        )?;
        app.mount(
            Id::DBListCriteria,
            Box::new(DBListCriteria::new(
                config.clone(),
                Msg::DataBase(DBMsg::CriteriaBlurDown),
                Msg::DataBase(DBMsg::CriteriaBlurUp),
            )),
            Vec::new(),
        )?;

        app.mount(
            Id::DBListSearchResult,
            Box::new(DBListSearchResult::new(
                config.clone(),
                Msg::DataBase(DBMsg::SearchResultBlurDown),
                Msg::DataBase(DBMsg::SearchResultBlurUp),
            )),
            Vec::new(),
        )?;
        app.mount(
            Id::DBListSearchTracks,
            Box::new(DBListSearchTracks::new(
                config.clone(),
                Msg::DataBase(DBMsg::SearchTracksBlurDown),
                Msg::DataBase(DBMsg::SearchTracksBlurUp),
            )),
            Vec::new(),
        )?;
        app.mount(
            Id::Playlist,
            Box::new(Playlist::new(config.clone())),
            Vec::new(),
        )?;
        app.mount(
            Id::Progress,
            Box::new(Progress::new(&config.read())),
            Vec::new(),
        )?;
        app.mount(Id::Lyric, Box::new(Lyric::new(config.clone())), Vec::new())?;

        app.mount(
            Id::Podcast,
            Box::new(FeedsList::new(
                config.clone(),
                Msg::Podcast(PCMsg::PodcastBlurDown),
                Msg::Podcast(PCMsg::PodcastBlurUp),
            )),
            Vec::new(),
        )?;

        app.mount(
            Id::Episode,
            Box::new(EpisodeList::new(
                config.clone(),
                Msg::Podcast(PCMsg::EpisodeBlurDown),
                Msg::Podcast(PCMsg::EpisodeBlurUp),
            )),
            Vec::new(),
        )?;
        app.mount(
            Id::DownloadSpinner,
            Box::new(DownloadSpinner::new(&config.read())),
            Vec::new(),
        )?;

        Ok(())
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
                match self.config_editor.layout {
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
                TermusicLayout::Podcast => self.view_layout_podcast(),
            }
        }
    }

    pub fn view_layout_podcast(&mut self) {
        self.terminal
            .raw_mut()
            .draw(|f| {
                let chunks_main = Layout::default()
                    .direction(Direction::Vertical)
                    .margin(0)
                    .constraints(
                        [
                            Constraint::Min(2),
                            Constraint::Length(3),
                            Constraint::Length(1),
                        ]
                        .as_ref(),
                    )
                    .split(f.area());
                let chunks_center = Layout::default()
                    .direction(Direction::Horizontal)
                    .margin(0)
                    .constraints([Constraint::Ratio(1, 3), Constraint::Ratio(2, 3)].as_ref())
                    .split(chunks_main[0]);

                let chunks_left = Layout::default()
                    .direction(Direction::Vertical)
                    .margin(0)
                    .constraints([Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)].as_ref())
                    .split(chunks_center[0]);
                let chunks_right = Layout::default()
                    .direction(Direction::Vertical)
                    .margin(0)
                    .constraints([Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)].as_ref())
                    .split(chunks_center[1]);

                self.app.view(&Id::Podcast, f, chunks_left[0]);
                self.app.view(&Id::Episode, f, chunks_left[1]);
                self.app.view(&Id::Playlist, f, chunks_right[0]);
                self.app.view(&Id::Lyric, f, chunks_right[1]);
                self.app.view(&Id::Progress, f, chunks_main[1]);
                self.app.view(&Id::Label, f, chunks_main[2]);

                Self::view_layout_commons(f, &mut self.app, self.download_tracker.visible());
            })
            .expect("Expected to draw without error");
    }
    pub fn view_layout_database(&mut self) {
        self.terminal
            .raw_mut()
            .draw(|f| {
                let chunks_main = Layout::default()
                    .direction(Direction::Vertical)
                    .margin(0)
                    .constraints([Constraint::Min(2), Constraint::Length(1)].as_ref())
                    .split(f.area());
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
                Self::view_layout_commons(f, &mut self.app, self.download_tracker.visible());
            })
            .expect("Expected to draw without error");
    }

    pub fn view_layout_treeview(&mut self) {
        self.terminal
            .raw_mut()
            .draw(|f| {
                let chunks_main = Layout::default()
                    .direction(Direction::Vertical)
                    .margin(0)
                    .constraints([Constraint::Min(2), Constraint::Length(1)].as_ref())
                    .split(f.area());
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

                Self::view_layout_commons(f, &mut self.app, self.download_tracker.visible());
            })
            .expect("Expected to draw without error");
    }

    #[allow(clippy::too_many_lines)]
    fn view_layout_commons(
        f: &mut Frame<'_>,
        app: &mut Application<Id, Msg, UserEvent>,
        downloading_visible: bool,
    ) {
        // -- footer
        if downloading_visible {
            let chunks_main = Layout::default()
                .direction(Direction::Vertical)
                .margin(0)
                .constraints([Constraint::Min(2), Constraint::Length(1)].as_ref())
                .split(f.area());
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
            app.view(&Id::Label, f, chunks_footer[2]);
        } else {
            let chunks_main = Layout::default()
                .direction(Direction::Vertical)
                .margin(0)
                .constraints([Constraint::Min(2), Constraint::Length(1)].as_ref())
                .split(f.area());
            app.view(&Id::Label, f, chunks_main[1]);
        }

        // -- popups
        if app.mounted(&Id::QuitPopup) {
            let popup = draw_area_in_absolute(f.area(), 30, 3);
            f.render_widget(Clear, popup);
            app.view(&Id::QuitPopup, f, popup);
        } else if app.mounted(&Id::HelpPopup) {
            let popup = draw_area_in_relative(f.area(), 88, 91);
            f.render_widget(Clear, popup);
            app.view(&Id::HelpPopup, f, popup);
        } else if app.mounted(&Id::DeleteConfirmRadioPopup) {
            let popup = draw_area_in_absolute(f.area(), 30, 3);
            f.render_widget(Clear, popup);
            app.view(&Id::DeleteConfirmRadioPopup, f, popup);
        } else if app.mounted(&Id::DeleteConfirmInputPopup) {
            let popup = draw_area_in_absolute(f.area(), 72, 3);
            f.render_widget(Clear, popup);
            app.view(&Id::DeleteConfirmInputPopup, f, popup);
        } else if app.mounted(&Id::FeedDeleteConfirmRadioPopup) {
            let popup = draw_area_in_absolute(f.area(), 60, 3);
            f.render_widget(Clear, popup);
            app.view(&Id::FeedDeleteConfirmRadioPopup, f, popup);
        } else if app.mounted(&Id::FeedDeleteConfirmInputPopup) {
            let popup = draw_area_in_absolute(f.area(), 60, 3);
            f.render_widget(Clear, popup);
            app.view(&Id::FeedDeleteConfirmInputPopup, f, popup);
        } else if app.mounted(&Id::GeneralSearchInput) {
            let popup = draw_area_in_relative(f.area(), 65, 68);
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
            let popup = draw_area_in_absolute(f.area(), 50, 3);
            f.render_widget(Clear, popup);
            app.view(&Id::YoutubeSearchInputPopup, f, popup);
        } else if app.mounted(&Id::YoutubeSearchTablePopup) {
            let popup = draw_area_in_relative(f.area(), 65, 68);
            f.render_widget(Clear, popup);
            app.view(&Id::YoutubeSearchTablePopup, f, popup);
        } else if app.mounted(&Id::PodcastSearchTablePopup) {
            let popup = draw_area_in_relative(f.area(), 65, 68);
            f.render_widget(Clear, popup);
            app.view(&Id::PodcastSearchTablePopup, f, popup);
        } else if app.mounted(&Id::SavePlaylistPopup) {
            let popup = draw_area_in_absolute(f.area(), 76, 6);
            f.render_widget(Clear, popup);
            let popup_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(3), Constraint::Length(3)].as_ref())
                .split(popup);
            app.view(&Id::SavePlaylistPopup, f, popup_chunks[0]);
            app.view(&Id::SavePlaylistLabel, f, popup_chunks[1]);
        } else if app.mounted(&Id::SavePlaylistConfirm) {
            let popup = draw_area_in_absolute(f.area(), 40, 3);
            f.render_widget(Clear, popup);
            app.view(&Id::SavePlaylistConfirm, f, popup);
        } else if app.mounted(&Id::PodcastAddPopup) {
            let popup = draw_area_in_absolute(f.area(), 65, 3);
            f.render_widget(Clear, popup);
            app.view(&Id::PodcastAddPopup, f, popup);
        } else if app.mounted(&Id::DatabaseAddConfirmPopup) {
            let popup = draw_area_in_absolute(f.area(), 60, 3);
            f.render_widget(Clear, popup);
            app.view(&Id::DatabaseAddConfirmPopup, f, popup);
        }
        if app.mounted(&Id::MessagePopup) {
            let popup = draw_area_top_right_absolute(f.area(), 25, 4);
            f.render_widget(Clear, popup);
            app.view(&Id::MessagePopup, f, popup);
        }
        if app.mounted(&Id::ErrorPopup) {
            let popup = draw_area_in_absolute(f.area(), 50, 4);
            f.render_widget(Clear, popup);
            app.view(&Id::ErrorPopup, f, popup);
        }
    }

    /// Mount / Remount a search popup for the provided source
    fn mount_search(&mut self, source: Source) {
        self.app
            .remount(
                Id::GeneralSearchInput,
                Box::new(GSInputPopup::new(source, &self.config_tui.read())),
                Vec::new(),
            )
            .unwrap();
        self.app
            .remount(
                Id::GeneralSearchTable,
                Box::new(GSTablePopup::new(source, self.config_tui.clone())),
                Vec::new(),
            )
            .unwrap();

        self.app.active(&Id::GeneralSearchInput).unwrap();
        if let Err(e) = self.update_photo() {
            self.mount_error_popup(e.context("update_photo"));
        }
    }

    #[inline]
    pub fn mount_search_library(&mut self) {
        self.mount_search(Source::Library);
    }

    #[inline]
    pub fn mount_search_playlist(&mut self) {
        self.mount_search(Source::Playlist);
    }

    #[inline]
    pub fn mount_search_database(&mut self) {
        self.mount_search(Source::Database);
    }

    #[inline]
    pub fn mount_search_episode(&mut self) {
        self.mount_search(Source::Episode);
    }

    #[inline]
    pub fn mount_search_podcast(&mut self) {
        self.mount_search(Source::Podcast);
    }

    pub fn mount_label_help(&mut self) {
        let config = self.config_tui.read();
        self.app
            .remount(Id::Label, Box::new(Footer::new(&config)), Vec::new())
            .expect("Expected to remount without error");
    }

    pub fn remount_save_playlist_label(&mut self, filename: &str) -> Result<()> {
        let current_node: String = match self.app.state(&Id::Library).ok().unwrap() {
            State::One(StateValue::String(id)) => id,
            _ => bail!("Invalid node selected in library"),
        };

        let mut path_string = get_parent_folder(Path::new(&current_node))
            .to_string_lossy()
            .to_string();
        // push extra "/" as "Path::to_string()" does not end with a "/"
        path_string.push('/');

        let config = self.config_tui.read();

        self.app
            .remount(
                Id::SavePlaylistLabel,
                Box::new(LabelSpan::new(
                    &config,
                    &[
                        TextSpan::new("Full name: ")
                            .fg(config.settings.theme.fallback_highlight())
                            .bold(),
                        TextSpan::new(path_string)
                            .fg(config.settings.theme.fallback_foreground())
                            .bold(),
                        TextSpan::new(filename).fg(Color::Cyan).bold(),
                        TextSpan::new(".m3u")
                            .fg(config.settings.theme.fallback_foreground())
                            .bold(),
                    ],
                )),
                Vec::new(),
            )
            .expect("Expected to remount without error");
        Ok(())
    }

    pub fn show_message_timeout_label_help<S: AsRef<str>>(
        &mut self,
        active_msg: S,
        foreground: Option<Color>,
        background: Option<Color>,
        timeout: Option<isize>,
    ) {
        let config = self.config_tui.read();
        let textspan = &[TextSpan::new(active_msg)
            .fg(foreground.unwrap_or_else(|| config.settings.theme.library_highlight()))
            .bold()
            .bg(background.unwrap_or_else(|| config.settings.theme.library_background()))];
        self.app
            .attr(
                &Id::Label,
                Attribute::Text,
                AttrValue::Payload(PropPayload::Vec(
                    textspan.iter().cloned().map(PropValue::TextSpan).collect(),
                )),
            )
            .ok();
        self.app
            .attr(
                &Id::Label,
                Attribute::Value,
                AttrValue::Number(timeout.unwrap_or(10)),
            )
            .ok();
    }
}
