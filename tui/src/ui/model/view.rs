use std::path::Path;
use std::time::Duration;

use anyhow::{Result, bail};
use termusiclib::utils::get_parent_folder;
use tokio::runtime::Handle;
use tokio::sync::mpsc::UnboundedReceiver;
use tuirealm::EventListenerCfg;
use tuirealm::props::{AttrValue, Attribute, Color, PropPayload, PropValue, TextSpan};
use tuirealm::ratatui::layout::{Constraint, Layout};
use tuirealm::ratatui::widgets::Clear;
use tuirealm::{Frame, State, StateValue};

use crate::ui::Application;
use crate::ui::components::{
    DBListCriteria, DBListSearchResult, DBListSearchTracks, DownloadSpinner, EpisodeList,
    FeedsList, Footer, GSInputPopup, GSTablePopup, LabelSpan, Lyric, MusicLibrary, Playlist,
    Progress, Source,
};
use crate::ui::ids::{Id, IdConfigEditor, IdTagEditor};
use crate::ui::model::ports::rx_main::PortRxMain;
use crate::ui::model::ports::stream_events::PortStreamEvents;
use crate::ui::model::{Model, TermusicLayout, UserEvent};
use crate::ui::msg::{DBMsg, Msg, PCMsg};
use crate::ui::utils::{
    draw_area_in_absolute, draw_area_in_relative, draw_area_top_right_absolute,
};

impl Model {
    pub fn init_app(
        tx_to_main: UnboundedReceiver<Msg>,
        stream_event_port: PortStreamEvents,
    ) -> Application<Id, Msg, UserEvent> {
        // Setup application
        Application::init(
            EventListenerCfg::default()
                .with_handle(Handle::current())
                .async_crossterm_input_listener(Duration::ZERO, 10)
                .poll_timeout(Duration::from_secs(10))
                .async_tick(true)
                .tick_interval(Duration::from_secs(1))
                .add_async_port(Box::new(PortRxMain::new(tx_to_main)), Duration::ZERO, 10)
                .add_async_port(Box::new(stream_event_port), Duration::ZERO, 1),
        )
    }

    /// Mount the Main components for the TUI.
    pub fn mount_main(&mut self) -> Result<()> {
        self.remount_global_listener()?;

        self.app.mount(
            Id::Library,
            Box::new(MusicLibrary::new(
                &self.library.tree,
                None,
                self.config_tui.clone(),
            )),
            Vec::new(),
        )?;
        self.app.mount(
            Id::DBListCriteria,
            Box::new(DBListCriteria::new(
                self.config_tui.clone(),
                Msg::DataBase(DBMsg::CriteriaBlurDown),
                Msg::DataBase(DBMsg::CriteriaBlurUp),
            )),
            Vec::new(),
        )?;

        self.app.mount(
            Id::DBListSearchResult,
            Box::new(DBListSearchResult::new(
                self.config_tui.clone(),
                Msg::DataBase(DBMsg::SearchResultBlurDown),
                Msg::DataBase(DBMsg::SearchResultBlurUp),
            )),
            Vec::new(),
        )?;
        self.app.mount(
            Id::DBListSearchTracks,
            Box::new(DBListSearchTracks::new(
                self.config_tui.clone(),
                Msg::DataBase(DBMsg::SearchTracksBlurDown),
                Msg::DataBase(DBMsg::SearchTracksBlurUp),
            )),
            Vec::new(),
        )?;
        self.app.mount(
            Id::Playlist,
            Box::new(Playlist::new(self.config_tui.clone())),
            Vec::new(),
        )?;
        self.app.mount(
            Id::Progress,
            Box::new(Progress::new(&self.config_tui.read())),
            Vec::new(),
        )?;
        self.app.mount(
            Id::Lyric,
            Box::new(Lyric::new(self.config_tui.clone())),
            Vec::new(),
        )?;

        self.app.mount(
            Id::Podcast,
            Box::new(FeedsList::new(
                self.config_tui.clone(),
                Msg::Podcast(PCMsg::PodcastBlurDown),
                Msg::Podcast(PCMsg::PodcastBlurUp),
            )),
            Vec::new(),
        )?;

        self.app.mount(
            Id::Episode,
            Box::new(EpisodeList::new(
                self.config_tui.clone(),
                Msg::Podcast(PCMsg::EpisodeBlurDown),
                Msg::Podcast(PCMsg::EpisodeBlurUp),
            )),
            Vec::new(),
        )?;
        self.app.mount(
            Id::DownloadSpinner,
            Box::new(DownloadSpinner::new(&self.config_tui.read())),
            Vec::new(),
        )?;

        // Set the Library component as the initally focused one
        self.app.active(&Id::Library)?;

        Ok(())
    }

    /// The entrypoint to start drawing the full TUI, if a redraw is requested.
    pub fn view(&mut self) {
        if self.redraw {
            self.redraw = false;

            if self
                .app
                .mounted(&Id::TagEditor(IdTagEditor::TableLyricOptions))
            {
                self.view_tag_editor();
                return;
            } else if self.app.mounted(&Id::ConfigEditor(IdConfigEditor::Header)) {
                self.view_config_editor();
                return;
            }

            match self.layout {
                TermusicLayout::TreeView => self.view_layout_treeview(),
                TermusicLayout::DataBase => self.view_layout_database(),
                TermusicLayout::Podcast => self.view_layout_podcast(),
            }
        }
    }

    fn view_layout_podcast(&mut self) {
        self.terminal
            .raw_mut()
            .draw(|f| {
                let [chunks_main, progress, _bottom_help] = Layout::vertical([
                    Constraint::Min(2),
                    Constraint::Length(3),
                    Constraint::Length(1),
                ])
                .areas(f.area());
                let [center_left, center_right] =
                    Layout::horizontal([Constraint::Ratio(1, 3), Constraint::Ratio(2, 3)])
                        .areas(chunks_main);

                let [left_podcasts, left_episodes] =
                    Layout::vertical([Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)])
                        .areas(center_left);
                let [right_playlist, right_lyric] =
                    Layout::vertical([Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)])
                        .areas(center_right);

                self.app.view(&Id::Podcast, f, left_podcasts);
                self.app.view(&Id::Episode, f, left_episodes);

                self.app.view(&Id::Playlist, f, right_playlist);
                self.app.view(&Id::Lyric, f, right_lyric);
                self.app.view(&Id::Progress, f, progress);

                Self::view_layout_commons(f, &mut self.app, self.download_tracker.visible());
            })
            .expect("Expected to draw without error");
    }

    fn view_layout_database(&mut self) {
        self.terminal
            .raw_mut()
            .draw(|f| {
                let [chunks_main, _bottom_help] =
                    Layout::vertical([Constraint::Min(2), Constraint::Length(1)]).areas(f.area());
                let [chunks_main_left, chunks_main_right] =
                    Layout::horizontal([Constraint::Ratio(1, 3), Constraint::Ratio(2, 3)])
                        .areas(chunks_main);

                let [left_criteria, left_search_result, left_search_tracks] = Layout::vertical([
                    Constraint::Length(DBListCriteria::num_options() + 2), // + 2 as this area still includes the borders
                    // maybe resize based on which one is focused?
                    Constraint::Fill(1),
                    Constraint::Fill(2),
                ])
                .areas(chunks_main_left);
                let [right_playlist, right_progress, right_lyric] = Layout::vertical([
                    Constraint::Min(2),
                    Constraint::Length(3),
                    Constraint::Length(4),
                ])
                .areas(chunks_main_right);

                self.app.view(&Id::DBListCriteria, f, left_criteria);
                self.app
                    .view(&Id::DBListSearchResult, f, left_search_result);
                self.app
                    .view(&Id::DBListSearchTracks, f, left_search_tracks);

                self.app.view(&Id::Playlist, f, right_playlist);
                self.app.view(&Id::Progress, f, right_progress);
                self.app.view(&Id::Lyric, f, right_lyric);

                Self::view_layout_commons(f, &mut self.app, self.download_tracker.visible());
            })
            .expect("Expected to draw without error");
    }

    fn view_layout_treeview(&mut self) {
        self.terminal
            .raw_mut()
            .draw(|f| {
                let [chunks_main, _bottom_help] =
                    Layout::vertical([Constraint::Min(2), Constraint::Length(1)]).areas(f.area());
                let [left_library, right] =
                    Layout::horizontal([Constraint::Ratio(1, 3), Constraint::Ratio(2, 3)])
                        .areas(chunks_main);
                let [right_playlist, right_progress, right_lyric] = Layout::vertical([
                    Constraint::Min(2),
                    Constraint::Length(3),
                    Constraint::Length(4),
                ])
                .areas(right);

                self.app.view(&Id::Library, f, left_library);

                self.app.view(&Id::Playlist, f, right_playlist);
                self.app.view(&Id::Progress, f, right_progress);
                self.app.view(&Id::Lyric, f, right_lyric);

                Self::view_layout_commons(f, &mut self.app, self.download_tracker.visible());
            })
            .expect("Expected to draw without error");
    }

    /// Draw the footer in the last line.
    fn view_common_footer(
        f: &mut Frame<'_>,
        app: &mut Application<Id, Msg, UserEvent>,
        downloading_visible: bool,
    ) {
        let [_content, bottom_label] =
            Layout::vertical([Constraint::Min(2), Constraint::Length(1)]).areas(f.area());

        if downloading_visible {
            let [_spacer, spinner, remainder] = Layout::horizontal([
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Min(10),
            ])
            .areas(bottom_label);

            app.view(&Id::DownloadSpinner, f, spinner);
            app.view(&Id::Label, f, remainder);
        } else {
            app.view(&Id::Label, f, bottom_label);
        }
    }

    /// Draw any popup.
    fn view_popups(f: &mut Frame<'_>, app: &mut Application<Id, Msg, UserEvent>) {
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
            let popup_chunks = Layout::vertical([
                Constraint::Length(3), // Input form
                Constraint::Min(2),    // Yes/No
            ])
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
            let popup_chunks =
                Layout::vertical([Constraint::Length(3), Constraint::Length(3)]).split(popup);
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

    /// Draw common things, like the bottom label and popups.
    fn view_layout_commons(
        f: &mut Frame<'_>,
        app: &mut Application<Id, Msg, UserEvent>,
        downloading_visible: bool,
    ) {
        Self::view_common_footer(f, app, downloading_visible);

        Self::view_popups(f, app);
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

    pub fn show_message_timeout_label_help<S: Into<String>>(
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
