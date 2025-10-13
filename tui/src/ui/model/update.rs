use std::path::Path;
use std::time::Duration;

use anyhow::{Result, anyhow};
use termusiclib::player::{PlayerProgress, RunningStatus, UpdateEvents, UpdatePlaylistEvents};
use termusiclib::podcast::{PodcastDLResult, PodcastSyncResult};
use termusiclib::track::MediaTypesSimple;
use tokio::runtime::Handle;
use tokio::time::sleep;
use tuirealm::Update;
use tuirealm::props::{AttrValue, Attribute};

use crate::ui::ids::Id;
use crate::ui::model::youtube_options::YTDLMsg;
use crate::ui::msg::{
    CoverDLResult, DBMsg, DeleteConfirmMsg, ErrorPopupMsg, GSMsg, HelpPopupMsg, LIMsg, LyricMsg,
    MainLayoutMsg, Msg, NotificationMsg, PCMsg, PLMsg, PlayerMsg, QuitPopupMsg, SavePlaylistMsg,
    ServerReqResponse, XYWHMsg, YSMsg,
};
use crate::ui::tui_cmd::TuiCmd;
use crate::ui::{Model, model::TermusicLayout};

impl Update<Msg> for Model {
    #[allow(clippy::too_many_lines)]
    fn update(&mut self, msg: Option<Msg>) -> Option<Msg> {
        let msg = msg?;
        // Set redraw
        self.redraw = true;
        // Match message
        match msg {
            Msg::ConfigEditor(msg) => self.update_config_editor(msg),
            Msg::DataBase(msg) => self.update_database_list(msg),

            Msg::DeleteConfirm(msg) => self.update_delete_confirmation(&msg),

            Msg::ErrorPopup(msg) => self.update_error_popup_msg(&msg),
            Msg::QuitPopup(msg) => self.update_quit_popup_msg(&msg),

            Msg::Library(msg) => {
                self.update_library(msg);
                None
            }
            Msg::GeneralSearch(msg) => {
                self.update_general_search(&msg);
                None
            }
            Msg::Playlist(msg) => {
                self.update_playlist(&msg);
                None
            }

            Msg::Player(msg) => self.update_player(msg),

            Msg::HelpPopup(msg) => self.update_help_popup_msg(&msg),
            Msg::YoutubeSearch(msg) => {
                self.update_youtube_search(msg);
                None
            }
            Msg::TagEditor(msg) => {
                self.update_tageditor(msg);
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

            Msg::Podcast(msg) => self.update_podcast(msg),
            Msg::LyricMessage(msg) => self.update_lyric_msg(msg),
            Msg::Notification(msg) => self.update_notification_msg(msg),
            Msg::Xywh(msg) => self.update_xywh_msg(msg),
            Msg::ServerReqResponse(msg) => self.update_server_resp_msg(msg),
            Msg::StreamUpdate(msg) => self.update_events_msg(msg),

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

    /// Handle all [`ErrorPopupMsg`] messages. Sub-function for [`update`](Self::update).
    fn update_error_popup_msg(&mut self, msg: &ErrorPopupMsg) -> Option<Msg> {
        match msg {
            ErrorPopupMsg::Close => {
                if self.app.mounted(&Id::ErrorPopup) {
                    self.umount_error_popup();
                }
            }
        }

        None
    }

    /// Handle all [`HelpPopupMsg`] messages. Sub-function for [`update`](Self::update).
    fn update_help_popup_msg(&mut self, msg: &HelpPopupMsg) -> Option<Msg> {
        match msg {
            HelpPopupMsg::Show => {
                self.mount_help_popup();
            }
            HelpPopupMsg::Close => {
                if self.app.mounted(&Id::HelpPopup) {
                    self.app.umount(&Id::HelpPopup).ok();
                }
                self.update_photo().ok();
            }
        }

        None
    }

    /// Handle all [`QuitPopupMsg`] messages. Sub-function for [`update`](Self::update).
    fn update_quit_popup_msg(&mut self, msg: &QuitPopupMsg) -> Option<Msg> {
        match msg {
            QuitPopupMsg::Show => {
                if self.config_tui.read().settings.behavior.confirm_quit {
                    self.mount_quit_popup();
                } else {
                    self.quit = true;
                }
            }
            QuitPopupMsg::CloseCancel => {
                self.app.umount(&Id::QuitPopup).ok();
            }
            QuitPopupMsg::CloseOk => {
                self.quit = true;
            }
        }

        None
    }

    /// Handle all [`XYWHMsg`] messages. Sub-function for [`update`](Self::update).
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
            XYWHMsg::CoverDLResult(msg) => match msg {
                CoverDLResult::FetchPhotoSuccess(image_wrapper) => {
                    self.show_image(&image_wrapper.data).ok();
                }
                CoverDLResult::FetchPhotoErr(err_text) => {
                    self.show_message_timeout_label_help(err_text, None, None, None);
                }
            },
        }
        None
    }

    /// Handle all [`LyricMsg`] messages. Sub-function for [`update`](Self::update).
    fn update_lyric_msg(&mut self, msg: LyricMsg) -> Option<Msg> {
        match msg {
            LyricMsg::Cycle => {
                self.lyric_cycle();
                None
            }
            LyricMsg::AdjustDelay(offset) => {
                self.lyric_adjust_delay(offset);
                None
            }
            LyricMsg::TextAreaBlurUp => self.app.active(&Id::Playlist).ok(),
            LyricMsg::TextAreaBlurDown => match self.layout {
                TermusicLayout::TreeView => self.app.active(&Id::Library).ok(),
                TermusicLayout::DataBase => self.app.active(&Id::DBListCriteria).ok(),
                TermusicLayout::Podcast => self.app.active(&Id::Podcast).ok(),
            },
        };
        None
    }

    /// Handle all [`NotificationMsg`] messages. Sub-function for [`update`](Self::update).
    fn update_notification_msg(&mut self, msg: NotificationMsg) -> Option<Msg> {
        match msg {
            NotificationMsg::MessageShow((title, text)) => {
                self.mount_message(&title, &text);
            }
            NotificationMsg::MessageHide((title, text)) => {
                self.umount_message(&title, &text);
            }
        }

        None
    }

    /// Handle all [`PCMsg`] messages. Sub-function for [`update`](Self::update).
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

            PCMsg::SyncResult(msg) => self.podcast_handle_sync_result(msg),
            PCMsg::DLResult(msg) => self.podcast_handle_dl_result(msg),

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

            PCMsg::EpisodeDownload(index) => {
                if let Err(e) = self.episode_download(Some(index)) {
                    self.mount_error_popup(e.context("podcast episode download"));
                }
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

    /// Handle all cases for [`PodcastSyncResult`].
    fn podcast_handle_sync_result(&mut self, msg: PodcastSyncResult) {
        match msg {
            PodcastSyncResult::FetchPodcastStart(url) => {
                self.download_tracker.increase_one(url);
                self.show_message_timeout_label_help(
                    self.download_tracker.message_sync_start(),
                    None,
                    None,
                    None,
                );
            }
            PodcastSyncResult::SyncData((id, pod)) => {
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
            PodcastSyncResult::NewData(pod) => {
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
            PodcastSyncResult::Error(feed) => {
                self.download_tracker.decrease_one(&feed.url);
                self.mount_error_popup(anyhow!("Error happened with feed: {:?}", feed.title));
                self.show_message_timeout_label_help(
                    self.download_tracker.message_feed_sync_failed(),
                    None,
                    None,
                    None,
                );
            }
        }
    }

    /// Handle all cases for [`PodcastDLResult`].
    fn podcast_handle_dl_result(&mut self, msg: PodcastDLResult) {
        match msg {
            PodcastDLResult::DLStart(ep_data) => {
                self.download_tracker.increase_one(&ep_data.url);
                self.show_message_timeout_label_help(
                    self.download_tracker.message_download_start(&ep_data.title),
                    None,
                    None,
                    None,
                );
            }
            PodcastDLResult::DLComplete(ep_data) => {
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
            PodcastDLResult::DLResponseError(ep_data) => {
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
            PodcastDLResult::DLFileCreateError(ep_data) => {
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
            PodcastDLResult::DLFileWriteError(ep_data) => {
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
        }
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

    /// Switch the main view / layout.
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
                self.lyric_update_title();
                self.lyric_update();
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
                self.lyric_update_title();
                self.lyric_update();
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
                self.lyric_update_title();
                self.lyric_update();
            }
        }

        None
    }

    /// Handle all [`DBMsg`] messages. Sub-function for [`update`](Self::update).
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
                        let file = track.as_pathbuf();
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

    /// Handle all [`LIMsg`] messages. Sub-function for [`update`](Self::update).
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

    /// Handle all [`YSMsg`] messages. Sub-function for [`update`](Self::update).
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
            YSMsg::ReqNextPage => {
                self.youtube_options_next_page();
            }
            YSMsg::ReqPreviousPage => {
                self.youtube_options_prev_page();
            }
            YSMsg::PageLoaded(data) => {
                self.youtube_options.data = data;
                self.sync_youtube_options();
            }
            YSMsg::PageLoadError(err) => {
                self.mount_error_popup(anyhow!(err));
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
            YSMsg::Download(msg) => self.update_ys_download_msg(msg),
        }
    }

    /// Handle all [`YSMsg`] messages. Sub-function for [`update_youtube_search`](Self::update_youtube_search).
    fn update_ys_download_msg(&mut self, msg: YTDLMsg) {
        match msg {
            YTDLMsg::Start(url, title) => {
                self.download_tracker.increase_one(&*url);
                self.show_message_timeout_label_help(
                    self.download_tracker.message_download_start(&title),
                    None,
                    None,
                    None,
                );
            }
            YTDLMsg::Success(url) => {
                self.download_tracker.decrease_one(&url);
                self.show_message_timeout_label_help(
                    self.download_tracker.message_download_complete(),
                    None,
                    None,
                    None,
                );
            }
            YTDLMsg::Completed(_url, file) => {
                if self.download_tracker.visible() {
                    return;
                }
                self.library_reload_with_node_focus(file);
            }
            YTDLMsg::Err(url, title, error_message) => {
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
        }
    }

    /// Handle all [`GSMsg`] messages. Sub-function for [`update`](Self::update).
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

    /// Handle all [`DeleteConfirmMsg`] messages. Sub-function for [`update`](Self::update).
    fn update_delete_confirmation(&mut self, msg: &DeleteConfirmMsg) -> Option<Msg> {
        match msg {
            DeleteConfirmMsg::Show => {
                self.library_before_delete();
            }
            DeleteConfirmMsg::CloseCancel => {
                if self.app.mounted(&Id::DeleteConfirmRadioPopup) {
                    let _drop = self.app.umount(&Id::DeleteConfirmRadioPopup);
                }
                if self.app.mounted(&Id::DeleteConfirmInputPopup) {
                    let _drop = self.app.umount(&Id::DeleteConfirmInputPopup);
                }
            }
            DeleteConfirmMsg::CloseOk => {
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
        }
        None
    }

    /// Handle all [`PLMsg`] messages. Sub-function for [`update`](Self::update).
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
            let _ = tx.send(Msg::Notification(NotificationMsg::MessageShow((
                title_string.clone(),
                text_string.clone(),
            ))));

            sleep(Duration::from_secs(delay)).await;

            let _ = tx.send(Msg::Notification(NotificationMsg::MessageHide((
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
                },
            }
        }
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

    /// Handle all [`ServerReqResponse`].
    fn update_server_resp_msg(&mut self, msg: ServerReqResponse) -> Option<Msg> {
        match msg {
            ServerReqResponse::GetProgress(response) => {
                let pprogress: PlayerProgress = response.progress.unwrap_or_default().into();
                self.progress_update(
                    pprogress.position,
                    pprogress.total_duration.unwrap_or_default(),
                );

                self.lyric_update_for_radio(response.radio_title);

                self.playback
                    .set_status(RunningStatus::from_u32(response.status));
                // "GetProgress" is, as of ~termusic 0.11.0~0.12.0, only called initially or having missed events, so everything should be reloaded.
                self.player_update_current_track_after();
            }
            ServerReqResponse::FullPlaylist(playlist_tracks) => {
                info!("Processing Playlist from server");
                let current_track_index = playlist_tracks.current_track_index;
                if let Err(err) = self
                    .playback
                    .load_from_grpc(playlist_tracks, &self.podcast.db_podcast)
                {
                    self.mount_error_popup(err);
                }

                self.playlist_sync();

                self.handle_current_track_index(
                    usize::try_from(current_track_index).unwrap(),
                    true,
                );
            }
        }

        None
    }

    /// Handle Stream updates [`UpdateEvents`].
    ///
    /// In case of lag, sends a [`TuiCmd::GetProgress`].
    fn update_events_msg(&mut self, msg: UpdateEvents) -> Option<Msg> {
        match msg {
            UpdateEvents::MissedEvents { amount } => {
                warn!("Stream Lagged, missed events: {amount}");
                // we know that we missed events, force to get full information from GetProgress endpoint
                self.command(TuiCmd::GetProgress);
            }
            UpdateEvents::VolumeChanged { volume } => {
                self.config_server.write().settings.player.volume = volume;
                self.progress_update_title();
            }
            UpdateEvents::SpeedChanged { speed } => {
                self.config_server.write().settings.player.speed = speed;
                self.progress_update_title();
            }
            UpdateEvents::PlayStateChanged { playing } => {
                self.playback.set_status(RunningStatus::from_u32(playing));

                // there is no special event for "no more tracks" or "track EOF", so we have to
                // handle "no more tracks / stopped" in this
                if self.playback.is_stopped() {
                    self.playback.clear_current_track();
                    self.lyric_update_title();
                    self.lyric_update();
                    self.progress_update(Some(Duration::ZERO), Duration::ZERO);
                }

                self.progress_update_title();
            }
            UpdateEvents::TrackChanged(track_changed_info) => {
                if let Some(progress) = track_changed_info.progress {
                    self.progress_update(
                        progress.position,
                        progress.total_duration.unwrap_or_default(),
                    );
                }

                if track_changed_info.current_track_updated {
                    self.handle_current_track_index(
                        usize::try_from(track_changed_info.current_track_index).unwrap(),
                        false,
                    );
                }

                if let Some(title) = track_changed_info.title {
                    self.lyric_update_for_radio(title);
                } else {
                    // fallback in case no title is immediately available on radio start.
                    // matching that the current track is actually radio, is in the function itself.
                    self.lyric_update_for_radio("");
                }
            }
            UpdateEvents::GaplessChanged { gapless } => {
                self.config_server.write().settings.player.gapless = gapless;
                self.progress_update_title();
            }
            UpdateEvents::Progress(progress) => {
                self.progress_update(
                    progress.position,
                    progress.total_duration.unwrap_or_default(),
                );
            }
            UpdateEvents::PlaylistChanged(ev) => {
                if let Err(err) = self.update_update_events_playlist_msg(ev) {
                    self.mount_error_popup(err);
                }
            }
        }

        None
    }

    /// Handle Playlist Update Events [`UpdatePlaylistEvents`].
    fn update_update_events_playlist_msg(&mut self, msg: UpdatePlaylistEvents) -> Result<()> {
        match msg {
            UpdatePlaylistEvents::PlaylistAddTrack(playlist_add_track) => {
                self.handle_playlist_add(playlist_add_track)?;
            }
            UpdatePlaylistEvents::PlaylistRemoveTrack(playlist_remove_track) => {
                self.handle_playlist_remove(&playlist_remove_track)?;
            }
            UpdatePlaylistEvents::PlaylistCleared => {
                self.handle_playlist_clear();
            }
            UpdatePlaylistEvents::PlaylistLoopMode(loop_mode) => {
                self.handle_playlist_loopmode(&loop_mode)?;
            }
            UpdatePlaylistEvents::PlaylistSwapTracks(swapped_tracks) => {
                self.handle_playlist_swap_tracks(&swapped_tracks)?;
            }
            UpdatePlaylistEvents::PlaylistShuffled(shuffled) => {
                self.handle_playlist_shuffled(shuffled)?;
            }
        }

        Ok(())
    }
}
