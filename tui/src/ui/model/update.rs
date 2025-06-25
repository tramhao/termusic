//! SPDX-License-Identifier: MIT

use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::anyhow;
use termusiclib::ids::{Id, IdTagEditor};
use termusiclib::track::MediaTypesSimple;
use termusiclib::types::{
    DBMsg, DLMsg, GSMsg, LIMsg, LyricMsg, MainLayoutMsg, Msg, PCMsg, PLMsg, PlayerMsg,
    SavePlaylistMsg, XYWHMsg, YSMsg,
};
use tokio::runtime::Handle;
use tokio::time::sleep;
use tuirealm::props::{AttrValue, Attribute};
use tuirealm::Update;

use crate::ui::tui_cmd::TuiCmd;
use crate::ui::{model::TermusicLayout, Model};

impl Update<Msg> for Model {
    #[allow(clippy::too_many_lines)]
    fn update(&mut self, msg: Option<Msg>) -> Option<Msg> {
        let msg = msg?;
        // Set redraw
        self.redraw = true;
        // Match message
        match msg {
            Msg::ConfigEditor(m) => self.update_config_editor(m),
            Msg::DataBase(m) => self.update_database_list(m),

            Msg::DeleteConfirmShow | Msg::DeleteConfirmCloseCancel | Msg::DeleteConfirmCloseOk => {
                self.update_delete_confirmation(&msg)
            }

            Msg::ErrorPopupClose => {
                if self.app.mounted(&Id::ErrorPopup) {
                    self.umount_error_popup();
                }
                None
            }
            Msg::QuitPopupShow => {
                if self.config_tui.read().settings.behavior.confirm_quit {
                    self.mount_quit_popup();
                } else {
                    self.quit = true;
                }
                None
            }
            Msg::QuitPopupCloseCancel => {
                self.app.umount(&Id::QuitPopup).ok();
                None
            }
            Msg::QuitPopupCloseOk => {
                self.quit = true;
                None
            }
            Msg::Library(m) => {
                self.update_library(m);
                None
            }
            Msg::GeneralSearch(m) => {
                self.update_general_search(&m);
                None
            }
            Msg::Playlist(m) => {
                self.update_playlist(&m);
                None
            }

            Msg::Player(msg) => self.update_player(msg),

            Msg::HelpPopupShow => {
                self.mount_help_popup();
                None
            }
            Msg::HelpPopupClose => {
                if self.app.mounted(&Id::HelpPopup) {
                    self.app.umount(&Id::HelpPopup).ok();
                }
                self.update_photo().ok();
                None
            }
            Msg::YoutubeSearch(m) => {
                self.update_youtube_search(m);
                None
            }
            Msg::LyricCycle => {
                self.lyric_cycle();
                None
            }
            Msg::LyricAdjustDelay(offset) => {
                self.lyric_adjust_delay(offset);
                None
            }
            Msg::TagEditor(m) => {
                self.update_tageditor(m);
                None
            }
            Msg::UpdatePhoto => {
                if let Err(e) = self.update_photo() {
                    self.mount_error_popup(e.context("update_photo"));
                }
                None
            }
            Msg::Layout(msg) => self.update_layout(msg),

            Msg::SavePlaylist(msg) => self.update_save_playlist(msg),

            Msg::Podcast(m) => self.update_podcast(m),
            Msg::LyricMessage(m) => self.update_lyric_textarea(m),
            Msg::Download(m) => self.update_download_msg(m),
            Msg::Xywh(m) => self.update_xywh_msg(m),

            Msg::ForceRedraw => None,
        }
    }
}

impl Model {
    /// Ensure the `QuitPopup` always has the focus (top-most)
    pub fn ensure_quit_popup_top_most_focus(&mut self) {
        if self.app.mounted(&Id::QuitPopup)
            && !self.app.focus().is_some_and(|v| *v == Id::QuitPopup)
        {
            self.app.active(&Id::QuitPopup).ok();
        }
    }

    fn update_xywh_msg(&mut self, msg: XYWHMsg) -> Option<Msg> {
        match msg {
            XYWHMsg::MoveLeft => self.xywh_move_left(),
            XYWHMsg::MoveRight => self.xywh_move_right(),
            XYWHMsg::MoveUp => self.xywh_move_up(),
            XYWHMsg::MoveDown => self.xywh_move_down(),
            XYWHMsg::ZoomIn => self.xywh_zoom_in(),
            XYWHMsg::ZoomOut => self.xywh_zoom_out(),
            XYWHMsg::ToggleHidden => {
                self.xywh_toggle_hide();
            }
        }
        None
    }

    fn update_lyric_textarea(&mut self, msg: LyricMsg) -> Option<Msg> {
        match msg {
            LyricMsg::LyricTextAreaBlurUp => self.app.active(&Id::Playlist).ok(),
            LyricMsg::LyricTextAreaBlurDown => match self.layout {
                TermusicLayout::TreeView => self.app.active(&Id::Library).ok(),
                TermusicLayout::DataBase => self.app.active(&Id::DBListCriteria).ok(),
                TermusicLayout::Podcast => self.app.active(&Id::Podcast).ok(),
                TermusicLayout::DlnaServer => self.app.active(&Id::DlnaServer).ok(),
            },
        };
        None
    }

    #[allow(clippy::too_many_lines)]
    fn update_podcast(&mut self, msg: PCMsg) -> Option<Msg> {
        match msg {
            PCMsg::PodcastBlurDown => {
                self.app.active(&Id::Episode).ok();
            }
            PCMsg::PodcastBlurUp => {
                self.app.active(&Id::Lyric).ok();
            }
            PCMsg::EpisodeBlurDown => {
                self.app.active(&Id::Playlist).ok();
            }
            PCMsg::EpisodeBlurUp => {
                self.app.active(&Id::Podcast).ok();
            }
            PCMsg::PodcastAddPopupShow => self.mount_podcast_add_popup(),
            PCMsg::PodcastAddPopupCloseOk(url) => {
                self.umount_podcast_add_popup();

                if url.starts_with("http") {
                    self.podcast_add(url);
                } else {
                    self.podcast_search_itunes(&url);
                    self.mount_podcast_search_table();
                }
            }
            PCMsg::PodcastAddPopupCloseCancel => self.umount_podcast_add_popup(),
            PCMsg::SyncData((id, pod)) => {
                self.download_tracker.decrease_one(&pod.url);
                self.show_message_timeout_label_help(
                    self.download_tracker.message_sync_success(),
                    None,
                    None,
                    None,
                );
                if let Err(e) = self.add_or_sync_data(&pod, Some(id)) {
                    self.mount_error_popup(e.context("add or sync data"));
                }
            }
            PCMsg::NewData(pod) => {
                self.download_tracker.decrease_one(&pod.url);
                self.show_message_timeout_label_help(
                    self.download_tracker.message_feeds_added(),
                    None,
                    None,
                    None,
                );
                if let Err(e) = self.add_or_sync_data(&pod, None) {
                    self.mount_error_popup(e.context("add or sync data"));
                }
            }
            PCMsg::Error(feed) => {
                self.download_tracker.decrease_one(&feed.url);
                self.mount_error_popup(anyhow!("Error happened with feed: {:?}", feed.title));
                self.show_message_timeout_label_help(
                    self.download_tracker.message_feed_sync_failed(),
                    None,
                    None,
                    None,
                );
            }
            PCMsg::PodcastSelected(index) => {
                self.podcast.podcasts_index = index;
                if let Err(e) = self.podcast_sync_episodes() {
                    self.mount_error_popup(e.context("podcast sync episodes"));
                }
            }
            PCMsg::DescriptionUpdate => self.lyric_update(),
            PCMsg::EpisodeAdd(index) => {
                if let Err(e) = self.playlist_add_episode(index) {
                    self.mount_error_popup(e.context("podcast playlist add episode"));
                }
            }
            PCMsg::EpisodeMarkPlayed(index) => {
                if let Err(e) = self.episode_mark_played(index) {
                    self.mount_error_popup(e.context("podcast episode mark played"));
                }
            }
            PCMsg::EpisodeMarkAllPlayed => {
                if let Err(e) = self.episode_mark_all_played() {
                    self.mount_error_popup(e.context("podcast episode mark all played"));
                }
            }
            PCMsg::PodcastRefreshOne(index) => {
                if let Err(e) = self.podcast_refresh_feeds(Some(index)) {
                    self.mount_error_popup(e.context("podcast refresh feeds one"));
                }
            }
            PCMsg::PodcastRefreshAll => {
                if let Err(e) = self.podcast_refresh_feeds(None) {
                    self.mount_error_popup(e.context("podcast refresh feeds all"));
                }
            }
            PCMsg::FetchPodcastStart(url) => {
                self.download_tracker.increase_one(url);
                self.show_message_timeout_label_help(
                    self.download_tracker.message_sync_start(),
                    None,
                    None,
                    None,
                );
            }
            PCMsg::EpisodeDownload(index) => {
                if let Err(e) = self.episode_download(Some(index)) {
                    self.mount_error_popup(e.context("podcast episode download"));
                }
            }
            PCMsg::DLStart(ep_data) => {
                self.download_tracker.increase_one(&ep_data.url);
                self.show_message_timeout_label_help(
                    self.download_tracker.message_download_start(&ep_data.title),
                    None,
                    None,
                    None,
                );
            }
            PCMsg::DLComplete(ep_data) => {
                if let Err(e) = self.episode_download_complete(ep_data.clone()) {
                    self.mount_error_popup(e.context("podcast episode download complete"));
                }
                self.download_tracker.decrease_one(&ep_data.url);
                self.show_message_timeout_label_help(
                    self.download_tracker.message_download_complete(),
                    None,
                    None,
                    None,
                );
            }
            PCMsg::DLResponseError(ep_data) => {
                self.download_tracker.decrease_one(&ep_data.url);
                self.mount_error_popup(anyhow!("download failed for episode: {}", ep_data.title));
                self.show_message_timeout_label_help(
                    self.download_tracker
                        .message_download_error_response(&ep_data.title),
                    None,
                    None,
                    None,
                );
            }
            PCMsg::DLFileCreateError(ep_data) => {
                self.download_tracker.decrease_one(&ep_data.url);
                self.mount_error_popup(anyhow!("download failed for episode: {}", ep_data.title));
                self.show_message_timeout_label_help(
                    self.download_tracker
                        .message_download_error_file_create(&ep_data.title),
                    None,
                    None,
                    None,
                );
            }
            PCMsg::DLFileWriteError(ep_data) => {
                self.download_tracker.decrease_one(&ep_data.url);
                self.mount_error_popup(anyhow!("download failed for episode: {}", ep_data.title));
                self.show_message_timeout_label_help(
                    self.download_tracker
                        .message_download_error_file_write(&ep_data.title),
                    None,
                    None,
                    None,
                );
            }
            PCMsg::EpisodeDeleteFile(index) => {
                if let Err(e) = self.episode_delete_file(index) {
                    self.mount_error_popup(e.context("podcast episode delete"));
                }
            }
            PCMsg::FeedDeleteShow => self.mount_feed_delete_confirm_radio(),
            PCMsg::FeedDeleteCloseOk => {
                self.umount_feed_delete_confirm_radio();
                if let Err(e) = self.podcast_remove_feed() {
                    self.mount_error_popup(e.context("podcast remove feed"));
                }
            }
            PCMsg::FeedDeleteCloseCancel => self.umount_feed_delete_confirm_radio(),
            PCMsg::FeedsDeleteShow => self.mount_feed_delete_confirm_input(),
            PCMsg::FeedsDeleteCloseOk => {
                self.umount_feed_delete_confirm_input();
                if let Err(e) = self.podcast_remove_all_feeds() {
                    self.mount_error_popup(e.context("podcast remove all feeds"));
                }
            }
            PCMsg::FeedsDeleteCloseCancel => self.umount_feed_delete_confirm_input(),
            PCMsg::SearchItunesCloseCancel => self.umount_podcast_search_table(),
            PCMsg::SearchItunesCloseOk(index) => {
                if let Some(vec) = &self.podcast.search_results {
                    if let Some(pod) = vec.get(index) {
                        self.podcast_add(pod.url.clone());
                    }
                }
            }
            PCMsg::SearchSuccess(vec) => {
                self.podcast.search_results = Some(vec.clone());
                self.update_podcast_search_table();
            }
            PCMsg::SearchError(e) => self.mount_error_popup(anyhow!(e)),
        }
        None
    }

    /// Handle Player related messages & events
    fn update_player(&mut self, msg: PlayerMsg) -> Option<Msg> {
        match msg {
            PlayerMsg::TogglePause => {
                self.player_toggle_pause();
            }
            PlayerMsg::SeekForward => {
                if self.is_radio() {
                    self.show_message_timeout_label_help(
                        "seek is not available for live radio",
                        None,
                        None,
                        None,
                    );
                    return None;
                }
                self.command(TuiCmd::SeekForward);
            }
            PlayerMsg::SeekBackward => {
                if self.is_radio() {
                    self.show_message_timeout_label_help(
                        "seek is not available for live radio",
                        None,
                        None,
                        None,
                    );
                    return None;
                }
                self.command(TuiCmd::SeekBackward);
            }
            PlayerMsg::SpeedUp => {
                self.command(TuiCmd::SpeedUp);
            }
            PlayerMsg::SpeedDown => {
                self.command(TuiCmd::SpeedDown);
            }
            PlayerMsg::VolumeUp => {
                self.command(TuiCmd::VolumeUp);
            }
            PlayerMsg::VolumeDown => {
                self.command(TuiCmd::VolumeDown);
            }
            PlayerMsg::ToggleGapless => {
                self.command(TuiCmd::ToggleGapless);
            }
        }

        None
    }

    /// Switch the main view / layout
    fn update_layout(&mut self, msg: MainLayoutMsg) -> Option<Msg> {
        match msg {
            MainLayoutMsg::DataBase => {
                let mut need_to_set_focus = true;
                if let Ok(Some(AttrValue::Flag(true))) =
                    self.app.query(&Id::DBListCriteria, Attribute::Focus)
                {
                    need_to_set_focus = false;
                }

                if let Ok(Some(AttrValue::Flag(true))) =
                    self.app.query(&Id::DBListSearchResult, Attribute::Focus)
                {
                    need_to_set_focus = false;
                }
                if let Ok(Some(AttrValue::Flag(true))) =
                    self.app.query(&Id::DBListSearchTracks, Attribute::Focus)
                {
                    need_to_set_focus = false;
                }

                if let Ok(Some(AttrValue::Flag(true))) =
                    self.app.query(&Id::Playlist, Attribute::Focus)
                {
                    need_to_set_focus = false;
                }
                if need_to_set_focus {
                    self.app.active(&Id::DBListCriteria).ok();
                }

                self.layout = TermusicLayout::DataBase;
                self.playlist_switch_layout();
            }
            MainLayoutMsg::TreeView => {
                let mut need_to_set_focus = true;
                if let Ok(Some(AttrValue::Flag(true))) =
                    self.app.query(&Id::Playlist, Attribute::Focus)
                {
                    need_to_set_focus = false;
                }

                if let Ok(Some(AttrValue::Flag(true))) =
                    self.app.query(&Id::Library, Attribute::Focus)
                {
                    need_to_set_focus = false;
                }

                if need_to_set_focus {
                    self.app.active(&Id::Library).ok();
                }

                self.layout = TermusicLayout::TreeView;
                self.playlist_switch_layout();
            }
            MainLayoutMsg::Podcast => {
                let mut need_to_set_focus = true;
                if let Ok(Some(AttrValue::Flag(true))) =
                    self.app.query(&Id::Podcast, Attribute::Focus)
                {
                    need_to_set_focus = false;
                }

                if let Ok(Some(AttrValue::Flag(true))) =
                    self.app.query(&Id::Episode, Attribute::Focus)
                {
                    need_to_set_focus = false;
                }
                if let Ok(Some(AttrValue::Flag(true))) =
                    self.app.query(&Id::Playlist, Attribute::Focus)
                {
                    need_to_set_focus = false;
                }

                if let Ok(Some(AttrValue::Flag(true))) =
                    self.app.query(&Id::Lyric, Attribute::Focus)
                {
                    need_to_set_focus = false;
                }

                if need_to_set_focus {
                    self.app.active(&Id::Podcast).ok();
                }

                self.layout = TermusicLayout::Podcast;
                self.podcast_sync_feeds_and_episodes();
                self.playlist_switch_layout();
            }
            MainLayoutMsg::DlnaServer => {
                let mut need_to_set_focus = true;
                /*if let Ok(Some(AttrValue::Flag(true))) =
                    self.app.query(&Id::Podcast, Attribute::Focus)
                {
                    need_to_set_focus = false;
                }

                if let Ok(Some(AttrValue::Flag(true))) =
                    self.app.query(&Id::Episode, Attribute::Focus)
                {
                    need_to_set_focus = false;
                }
                if let Ok(Some(AttrValue::Flag(true))) =
                    self.app.query(&Id::Playlist, Attribute::Focus)
                {
                    need_to_set_focus = false;
                }

                if let Ok(Some(AttrValue::Flag(true))) =
                    self.app.query(&Id::Lyric, Attribute::Focus)
                {
                    need_to_set_focus = false;
                }*/

                if need_to_set_focus {
                    self.app.active(&Id::DlnaServer).ok();
                }

                self.layout = TermusicLayout::DlnaServer;
                // self.podcast_sync_feeds_and_episodes();
                // self.playlist_switch_layout();
            }
        }

        None
    }
    fn update_database_list(&mut self, msg: DBMsg) -> Option<Msg> {
        match msg {
            DBMsg::CriteriaBlurDown | DBMsg::SearchTracksBlurUp => {
                self.app.active(&Id::DBListSearchResult).ok();
            }
            DBMsg::SearchResultBlurDown => {
                self.app.active(&Id::DBListSearchTracks).ok();
            }
            DBMsg::SearchTracksBlurDown | DBMsg::CriteriaBlurUp => {
                self.app.active(&Id::Playlist).ok();
            }
            DBMsg::SearchResultBlurUp => {
                self.app.active(&Id::DBListCriteria).ok();
            }
            DBMsg::SearchResult(criteria) => {
                self.dw.criteria = criteria;
                self.database_update_search_results();
            }
            DBMsg::SearchTrack(index) => {
                self.database_update_search_tracks(index);
            }
            DBMsg::AddPlaylist(index) => {
                if !self.dw.search_tracks.is_empty() {
                    if let Some(track) = self.dw.search_tracks.get(index) {
                        let file = PathBuf::from(&track.file);
                        if let Err(e) = self.playlist_add(&file) {
                            self.mount_error_popup(e.context("playlist add"));
                        }
                    }
                }
            }
            DBMsg::AddAllToPlaylist => {
                let db_search_tracks = self.dw.search_tracks.clone();
                self.playlist_add_all_from_db(&db_search_tracks);
            }

            DBMsg::AddResultToPlaylist(index) => {
                if let Some(result) = self.dw.search_results.get(index).cloned() {
                    if let Some(result) =
                        self.database_get_tracks_by_criteria(self.dw.criteria, &result)
                    {
                        self.playlist_add_all_from_db(&result);
                    }
                }
            }
            DBMsg::AddAllResultsToPlaylist => {
                self.database_add_all_results();
            }

            DBMsg::AddAllResultsConfirmShow => {
                // dont try showing the popup if there is nothing to add
                if !self.dw.search_results.is_empty() {
                    self.mount_results_add_confirm_database(self.dw.criteria);
                }
            }
            DBMsg::AddAllResultsConfirmCancel => {
                self.umount_results_add_confirm_database();
            }
        }
        None
    }

    fn update_library(&mut self, msg: LIMsg) {
        match msg {
            LIMsg::TreeBlur => {
                assert!(self.app.active(&Id::Playlist).is_ok());
            }
            LIMsg::TreeStepInto(path) => {
                self.library_stepinto(&path);
            }
            LIMsg::TreeStepOut => {
                self.library_stepout();
            }
            LIMsg::Yank => {
                self.library_yank();
            }
            LIMsg::Paste => {
                if let Err(e) = self.library_paste() {
                    self.mount_error_popup(e.context("library paste"));
                }
            }
            LIMsg::SwitchRoot => self.library_switch_root(),
            LIMsg::AddRoot => {
                if let Err(e) = self.library_add_root() {
                    self.mount_error_popup(e.context("library add root"));
                }
            }
            LIMsg::RemoveRoot => {
                if let Err(e) = self.library_remove_root() {
                    self.mount_error_popup(e.context("library remove root"));
                }
            }
            LIMsg::TreeNodeReady(vec, focus_node) => {
                self.library_apply_as_tree(vec, focus_node);
            }
        }
    }

    fn update_youtube_search(&mut self, msg: YSMsg) {
        match msg {
            YSMsg::InputPopupShow => {
                self.mount_youtube_search_input();
            }
            YSMsg::InputPopupCloseCancel => {
                if self.app.mounted(&Id::YoutubeSearchInputPopup) {
                    assert!(self.app.umount(&Id::YoutubeSearchInputPopup).is_ok());
                }
            }
            YSMsg::InputPopupCloseOk(url) => {
                if self.app.mounted(&Id::YoutubeSearchInputPopup) {
                    assert!(self.app.umount(&Id::YoutubeSearchInputPopup).is_ok());
                }
                if url.starts_with("http") {
                    match self.youtube_dl(&url) {
                        Ok(()) => {}
                        Err(e) => {
                            self.mount_error_popup(e.context("youtube-dl download"));
                        }
                    }
                } else {
                    self.mount_youtube_search_table();
                    self.youtube_options_search(url);
                }
            }
            YSMsg::TablePopupCloseCancel => {
                self.umount_youtube_search_table_popup();
            }
            YSMsg::TablePopupNext => {
                self.youtube_options_next_page();
            }
            YSMsg::TablePopupPrevious => {
                self.youtube_options_prev_page();
            }
            YSMsg::TablePopupCloseOk(index) => {
                if let Err(e) = self.youtube_options_download(index) {
                    self.library_reload_with_node_focus(None);
                    self.mount_error_popup(e.context("youtube-dl options download"));
                }
            }
            YSMsg::YoutubeSearchSuccess(youtube_options) => {
                self.youtube_options = youtube_options;
                self.sync_youtube_options();
                self.redraw = true;
            }
            YSMsg::YoutubeSearchFail(e) => {
                self.redraw = true;
                self.mount_error_popup(anyhow!("Youtube search fail: {e}"));
            }
        }
    }

    #[allow(clippy::too_many_lines)]
    fn update_general_search(&mut self, msg: &GSMsg) {
        match msg {
            GSMsg::PopupShowDatabase => {
                self.mount_search_database();
                self.database_update_search("*");
            }
            GSMsg::PopupShowLibrary => {
                self.mount_search_library();
                self.library_update_search("*");
            }
            GSMsg::PopupShowPlaylist => {
                self.mount_search_playlist();
                self.playlist_update_search("*");
            }
            GSMsg::PopupShowEpisode => {
                self.mount_search_episode();
                self.podcast_update_search_episode("*");
            }

            GSMsg::PopupShowPodcast => {
                self.mount_search_podcast();
                self.podcast_update_search_podcast("*");
            }
            GSMsg::PopupUpdateLibrary(input) => self.library_update_search(input),

            GSMsg::PopupUpdatePlaylist(input) => self.playlist_update_search(input),

            GSMsg::PopupUpdateDatabase(input) => self.database_update_search(input),

            GSMsg::InputBlur => {
                if self.app.mounted(&Id::GeneralSearchTable) {
                    self.app.active(&Id::GeneralSearchTable).ok();
                }
            }
            GSMsg::TableBlur => {
                if self.app.mounted(&Id::GeneralSearchInput) {
                    self.app.active(&Id::GeneralSearchInput).ok();
                }
            }
            GSMsg::PopupCloseCancel => {
                self.app.umount(&Id::GeneralSearchInput).ok();
                self.app.umount(&Id::GeneralSearchTable).ok();
                if let Err(e) = self.update_photo() {
                    self.mount_error_popup(e.context("update_photo"));
                }
            }

            GSMsg::PopupCloseLibraryAddPlaylist => {
                if let Err(e) = self.general_search_after_library_add_playlist() {
                    self.mount_error_popup(e.context("general search"));
                }
            }
            GSMsg::PopupCloseOkLibraryLocate => {
                self.general_search_after_library_select();
                self.app.umount(&Id::GeneralSearchInput).ok();
                self.app.umount(&Id::GeneralSearchTable).ok();

                if let Err(e) = self.update_photo() {
                    self.mount_error_popup(e.context("update_photo"));
                }
            }
            GSMsg::PopupClosePlaylistPlaySelected => {
                self.general_search_after_playlist_play_selected();
                self.app.umount(&Id::GeneralSearchInput).ok();
                self.app.umount(&Id::GeneralSearchTable).ok();

                if let Err(e) = self.update_photo() {
                    self.mount_error_popup(e.context("update_photo"));
                }
            }
            GSMsg::PopupCloseOkPlaylistLocate => {
                self.general_search_after_playlist_select();
                self.app.umount(&Id::GeneralSearchInput).ok();
                self.app.umount(&Id::GeneralSearchTable).ok();

                if let Err(e) = self.update_photo() {
                    self.mount_error_popup(e.context("update_photo"));
                }
            }
            GSMsg::PopupCloseDatabaseAddPlaylist => {
                if let Err(e) = self.general_search_after_database_add_playlist() {
                    self.mount_error_popup(e.context("add to playlist from database search"));
                }
            }
            GSMsg::PopupCloseEpisodeAddPlaylist => {
                if let Err(e) = self.general_search_after_episode_add_playlist() {
                    self.mount_error_popup(e.context("add to playlist from episode search"));
                }
            }
            GSMsg::PopupCloseOkEpisodeLocate => {
                if let Err(e) = self.general_search_after_episode_select() {
                    self.mount_error_popup(e.context("general search after episode select"));
                }
                self.app.umount(&Id::GeneralSearchInput).ok();
                self.app.umount(&Id::GeneralSearchTable).ok();
                self.podcast_focus_episode_list();
                if let Err(e) = self.update_photo() {
                    self.mount_error_popup(e.context("update_photo"));
                }
            }

            GSMsg::PopupUpdateEpisode(input) => self.podcast_update_search_episode(input),
            GSMsg::PopupUpdatePodcast(input) => self.podcast_update_search_podcast(input),
            GSMsg::PopupCloseOkPodcastLocate => {
                if let Err(e) = self.general_search_after_podcast_select() {
                    self.mount_error_popup(e.context("general search after podcast select"));
                }
                self.app.umount(&Id::GeneralSearchInput).ok();
                self.app.umount(&Id::GeneralSearchTable).ok();
                self.podcast_focus_podcast_list();
                if let Err(e) = self.update_photo() {
                    self.mount_error_popup(e.context("update_photo"));
                }
            }
        }
    }
    fn update_delete_confirmation(&mut self, msg: &Msg) -> Option<Msg> {
        match msg {
            Msg::DeleteConfirmShow => {
                self.library_before_delete();
            }
            Msg::DeleteConfirmCloseCancel => {
                if self.app.mounted(&Id::DeleteConfirmRadioPopup) {
                    let _drop = self.app.umount(&Id::DeleteConfirmRadioPopup);
                }
                if self.app.mounted(&Id::DeleteConfirmInputPopup) {
                    let _drop = self.app.umount(&Id::DeleteConfirmInputPopup);
                }
            }
            Msg::DeleteConfirmCloseOk => {
                if self.app.mounted(&Id::DeleteConfirmRadioPopup) {
                    let _drop = self.app.umount(&Id::DeleteConfirmRadioPopup);
                }
                if self.app.mounted(&Id::DeleteConfirmInputPopup) {
                    let _drop = self.app.umount(&Id::DeleteConfirmInputPopup);
                }
                if let Err(e) = self.library_delete_node() {
                    self.mount_error_popup(e.context("library delete song"));
                }
            }
            _ => {}
        }
        None
    }
    fn update_playlist(&mut self, msg: &PLMsg) {
        match msg {
            PLMsg::Add(current_node) => {
                if let Err(e) = self.playlist_add(current_node) {
                    self.mount_error_popup(e.context("playlist add"));
                }
            }
            PLMsg::Delete(index) => {
                self.playlist_delete_item(*index);
            }
            PLMsg::DeleteAll => {
                self.playlist_clear();
            }
            PLMsg::Shuffle => {
                self.playlist_shuffle();
            }
            PLMsg::PlaySelected(index) => {
                self.playlist_play_selected(*index);
            }
            PLMsg::LoopModeCycle => {
                self.command(TuiCmd::CycleLoop);
                self.config_server.write().settings.player.loop_mode =
                    self.playback.playlist.cycle_loop_mode();
                self.playlist_update_title();
            }
            PLMsg::PlaylistTableBlurDown => match self.layout {
                TermusicLayout::TreeView => assert!(self.app.active(&Id::Library).is_ok()),
                TermusicLayout::DataBase => assert!(self.app.active(&Id::DBListCriteria).is_ok()),
                TermusicLayout::Podcast => assert!(self.app.active(&Id::Lyric).is_ok()),
                TermusicLayout::DlnaServer => assert!(self.app.active(&Id::DlnaServer).is_ok()),
            },
            PLMsg::NextSong => {
                self.command(TuiCmd::SkipNext);
            }

            PLMsg::PrevSong => {
                self.player_previous();
            }
            PLMsg::SwapDown(index) => {
                self.playlist_swap_down(*index);
            }
            PLMsg::SwapUp(index) => {
                self.playlist_swap_up(*index);
            }
            PLMsg::AddRandomAlbum => {
                self.playlist_add_random_album();
            }
            PLMsg::AddRandomTracks => {
                self.playlist_add_random_tracks();
            }
            PLMsg::PlaylistTableBlurUp => match self.layout {
                TermusicLayout::TreeView => assert!(self.app.active(&Id::Library).is_ok()),
                TermusicLayout::DataBase => {
                    assert!(self.app.active(&Id::DBListSearchTracks).is_ok());
                }
                TermusicLayout::Podcast => assert!(self.app.active(&Id::Episode).is_ok()),
                TermusicLayout::DlnaServer => assert!(self.app.active(&Id::DlnaServer).is_ok()),
            },
        }
    }

    // show a popup for playing song
    pub fn update_playing_song(&mut self) {
        if let Some(track) = self.playback.current_track() {
            if self.layout == TermusicLayout::Podcast {
                let title = track.title().unwrap_or("Unknown Episode");
                self.update_show_message_timeout("Currently Playing", title, None);
                return;
            }
            let name = track.title().map_or_else(|| track.id_str(), Into::into);
            self.update_show_message_timeout("Currently Playing", &name, None);

            // TODO: is there a better way to update only a single / 2 columns (prev/next) instead of re-doing the whole playist; OR a way to decide at draw-time?
            // sync playlist to update any dynamic parts added to the columns (like current playing symbol)
            self.playlist_sync();
        }
    }

    /// Show a message with a `title` and `text`, and hide it again after `time_out` or 10 seconds.
    ///
    /// This function requires to run in a tokio context.
    pub fn update_show_message_timeout(&self, title: &str, text: &str, time_out: Option<u64>) {
        let title_string = title.to_string();
        let text_string = text.to_string();
        let tx = self.tx_to_main.clone();
        let delay = time_out.unwrap_or(10);

        Handle::current().spawn(async move {
            let _ = tx.send(Msg::Download(DLMsg::MessageShow((
                title_string.clone(),
                text_string.clone(),
            ))));

            sleep(Duration::from_secs(delay)).await;

            let _ = tx.send(Msg::Download(DLMsg::MessageHide((
                title_string,
                text_string,
            ))));
        });
    }

    pub fn update_layout_for_current_track(&mut self) {
        if let Some(track) = self.playback.current_track() {
            match track.media_type() {
                MediaTypesSimple::Podcast => {
                    if self.layout == TermusicLayout::Podcast {
                        return;
                    }
                    self.update_layout(MainLayoutMsg::Podcast);
                }
                MediaTypesSimple::Music | MediaTypesSimple::LiveRadio => match self.layout {
                    TermusicLayout::TreeView | TermusicLayout::DataBase => {}
                    TermusicLayout::Podcast => {
                        self.update_layout(MainLayoutMsg::TreeView);
                    }
                    TermusicLayout::DlnaServer => {
                        self.update_layout(MainLayoutMsg::TreeView);
                    }
                },
            }
        }
    }

    // update other messages
    pub fn update_outside_msg(&mut self) {
        if let Ok(msg) = self.rx_to_main.try_recv() {
            self.update(Some(msg));
        }
    }

    // change status bar text to indicate the downloading state
    fn update_download_msg(&mut self, msg: DLMsg) -> Option<Msg> {
        self.redraw = true;
        match msg {
            DLMsg::DownloadRunning(url, title) => {
                self.download_tracker.increase_one(&*url);
                self.show_message_timeout_label_help(
                    self.download_tracker.message_download_start(&title),
                    None,
                    None,
                    None,
                );
            }
            DLMsg::DownloadSuccess(url) => {
                self.download_tracker.decrease_one(&url);
                self.show_message_timeout_label_help(
                    self.download_tracker.message_download_complete(),
                    None,
                    None,
                    None,
                );

                if self.app.mounted(&Id::TagEditor(IdTagEditor::LabelHint)) {
                    self.umount_tageditor();
                }
            }
            DLMsg::DownloadCompleted(_url, file) => {
                if self.download_tracker.visible() {
                    return None;
                }
                self.library_reload_with_node_focus(file);
            }
            DLMsg::DownloadErrDownload(url, title, error_message) => {
                self.download_tracker.decrease_one(&url);
                self.mount_error_popup(anyhow!("download failed: {error_message}"));
                self.show_message_timeout_label_help(
                    self.download_tracker
                        .message_download_error_response(&title),
                    None,
                    None,
                    None,
                );
            }
            DLMsg::DownloadErrEmbedData(_url, title) => {
                self.mount_error_popup(anyhow!("download ok but tag info is not complete."));
                self.show_message_timeout_label_help(
                    self.download_tracker
                        .message_download_error_embed_data(&title),
                    None,
                    None,
                    None,
                );
            }
            DLMsg::MessageShow((title, text)) => {
                self.mount_message(&title, &text);
            }
            DLMsg::MessageHide((title, text)) => {
                self.umount_message(&title, &text);
            }
            DLMsg::FetchPhotoSuccess(image_wrapper) => {
                self.show_image(&image_wrapper.data).ok();
            }
            DLMsg::FetchPhotoErr(err_text) => {
                self.show_message_timeout_label_help(err_text, None, None, None);
            }
        }
        None
    }

    /// Handle & update [`SavePlaylistMsg`] related components.
    fn update_save_playlist(&mut self, msg: SavePlaylistMsg) -> Option<Msg> {
        match msg {
            SavePlaylistMsg::PopupShow => {
                if let Err(e) = self.mount_save_playlist() {
                    self.mount_error_popup(e.context("mount save playlist"));
                }
            }
            SavePlaylistMsg::PopupCloseCancel => {
                self.umount_save_playlist();
            }
            SavePlaylistMsg::PopupCloseOk(filename) => {
                self.umount_save_playlist();
                if let Err(e) = self.playlist_save_m3u_before(&filename) {
                    self.mount_error_popup(e.context("save m3u playlist before"));
                }
            }
            SavePlaylistMsg::PopupUpdate(filename) => {
                if let Err(e) = self.remount_save_playlist_label(&filename) {
                    self.mount_error_popup(e.context("remount save playlist label"));
                }
            }
            SavePlaylistMsg::ConfirmCloseCancel => {
                self.umount_save_playlist_confirm();
            }
            SavePlaylistMsg::ConfirmCloseOk(filename) => {
                if let Err(e) = self.playlist_save_m3u(Path::new(&filename)) {
                    self.mount_error_popup(e.context("save m3u playlist"));
                }
                self.umount_save_playlist_confirm();
            }
        }

        None
    }
}
