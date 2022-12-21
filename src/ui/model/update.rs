/**
 * MIT License
 *
 * termusic - Copyright (C) 2021 Larry Hao
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy
 * of this software and associated documentation files (the "Software"), to deal
 * in the Software without restriction, including without limitation the rights
 * to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 * copies of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in all
 * copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 * AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
 * SOFTWARE.
 */
use crate::player::{PlayerMsg, PlayerTrait};
use crate::sqlite::SearchCriteria;
use crate::ui::{
    model::{TermusicLayout, UpdateComponents},
    DBMsg, GSMsg, Id, IdTagEditor, LIMsg, Model, Msg, PLMsg, YSMsg,
};
use std::thread::{self, sleep};
use std::time::Duration;
use tuirealm::props::{AttrValue, Attribute, Color};
use tuirealm::Update;

impl Update<Msg> for Model {
    fn update(&mut self, msg: Option<Msg>) -> Option<Msg> {
        if let Some(msg) = msg {
            // Set redraw
            self.redraw = true;
            // Match message
            match msg {
                Msg::ConfigEditor(m) => self.update_config_editor(&m),
                Msg::DataBase(m) => self.update_database_list(&m),

                Msg::DeleteConfirmShow
                | Msg::DeleteConfirmCloseCancel
                | Msg::DeleteConfirmCloseOk => self.update_delete_confirmation(&msg),

                Msg::ErrorPopupClose => {
                    if self.app.mounted(&Id::ErrorPopup) {
                        self.umount_error_popup();
                    }
                    None
                }
                Msg::QuitPopupShow => {
                    if self.config.enable_exit_confirmation {
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
                    self.update_library(&m);
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

                Msg::PlayerSeek(offset) => {
                    self.player_seek(offset as i64);
                    None
                }

                Msg::PlayerTogglePause
                | Msg::PlayerToggleGapless
                | Msg::PlayerSpeedUp
                | Msg::PlayerSpeedDown
                | Msg::PlayerVolumeUp
                | Msg::PlayerVolumeDown => self.update_player(&msg),

                Msg::HelpPopupShow => {
                    self.mount_help_popup();
                    None
                }
                Msg::HelpPopupClose => {
                    self.app.umount(&Id::HelpPopup).ok();
                    None
                }
                Msg::YoutubeSearch(m) => {
                    self.update_youtube_search(&m);
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
                    self.update_tageditor(&m);
                    None
                }
                Msg::UpdatePhoto => {
                    if let Err(e) = self.update_photo() {
                        self.mount_error_popup(format!("update photo error: {e}"));
                    }
                    None
                }
                Msg::LayoutDataBase | Msg::LayoutTreeView => self.update_layout(&msg),

                Msg::None => None,
            }
        } else {
            None
        }
    }
}

impl Model {
    fn update_player(&mut self, msg: &Msg) -> Option<Msg> {
        match msg {
            Msg::PlayerTogglePause => {
                self.player_toggle_pause();
            }
            Msg::PlayerSeek(offset) => {
                self.player_seek(*offset as i64);
            }
            Msg::PlayerSpeedUp => {
                self.player.speed_up();
                self.config.speed = self.player.speed();
                self.progress_update_title();
            }
            Msg::PlayerSpeedDown => {
                self.player.speed_down();
                self.config.speed = self.player.speed();
                self.progress_update_title();
            }
            Msg::PlayerVolumeUp => {
                self.player.volume_up();
                self.config.volume = self.player.volume();
                self.progress_update_title();
            }
            Msg::PlayerVolumeDown => {
                self.player.volume_down();
                self.config.volume = self.player.volume();
                self.progress_update_title();
            }
            Msg::PlayerToggleGapless => {
                self.config.gapless = !self.config.gapless;
                self.player.toggle_gapless();
                self.progress_update_title();
            }
            _ => {}
        }
        None
    }
    fn update_layout(&mut self, msg: &Msg) -> Option<Msg> {
        match msg {
            Msg::LayoutDataBase => {
                if let Ok(f) = self.app.query(&Id::Library, Attribute::Focus) {
                    if Some(AttrValue::Flag(true)) == f {
                        self.app.active(&Id::DBListCriteria).ok();
                    }
                }

                self.layout = TermusicLayout::DataBase;
                None
            }
            Msg::LayoutTreeView => {
                if let Ok(f) = self.app.query(&Id::DBListCriteria, Attribute::Focus) {
                    if Some(AttrValue::Flag(true)) == f {
                        self.app.active(&Id::Library).ok();
                    }
                }

                if let Ok(f) = self.app.query(&Id::DBListSearchResult, Attribute::Focus) {
                    if Some(AttrValue::Flag(true)) == f {
                        self.app.active(&Id::Library).ok();
                    }
                }

                if let Ok(f) = self.app.query(&Id::DBListSearchTracks, Attribute::Focus) {
                    if Some(AttrValue::Flag(true)) == f {
                        self.app.active(&Id::Library).ok();
                    }
                }

                self.layout = TermusicLayout::TreeView;
                None
            }
            _ => None,
        }
    }
    fn update_database_list(&mut self, msg: &DBMsg) -> Option<Msg> {
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
            DBMsg::SearchResult(index) => {
                self.db_criteria = SearchCriteria::from(*index);
                self.database_update_search_results();
            }
            DBMsg::SearchTrack(index) => {
                self.database_update_search_tracks(*index);
            }
            DBMsg::AddPlaylist(index) => {
                if !self.db_search_tracks.is_empty() {
                    if let Some(track) = self.db_search_tracks.get(*index) {
                        let file = track.file.clone();
                        self.playlist_add(&file);
                    }
                }
            }
            DBMsg::AddAllToPlaylist => {
                let db_search_tracks = self.db_search_tracks.clone();
                self.playlist_add_all_from_db(&db_search_tracks);
            }
        }
        None
    }

    fn update_library(&mut self, msg: &LIMsg) {
        match msg {
            LIMsg::TreeBlur => {
                assert!(self.app.active(&Id::Playlist).is_ok());
            }
            LIMsg::TreeExtendDir(path) => {
                self.library_stepinto(path);
            }
            LIMsg::TreeGoToUpperDir => {
                self.library_stepout();
            }
            LIMsg::Yank => {
                self.library_yank();
            }
            LIMsg::Paste => {
                if let Err(e) = self.library_paste() {
                    self.mount_error_popup(format!("Paste error: {e}"));
                }
            }
            LIMsg::SwitchRoot => self.library_switch_root(),
            LIMsg::AddRoot => self.library_add_root(),
            LIMsg::RemoveRoot => {
                if let Err(e) = self.library_remove_root() {
                    self.mount_error_popup(format!("Remove root error: {e}"));
                }
            }
        }
    }

    fn update_youtube_search(&mut self, msg: &YSMsg) {
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
                    match self.youtube_dl(url) {
                        Ok(_) => {}
                        Err(e) => {
                            self.mount_error_popup(format!("download error: {e}"));
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
                if let Err(e) = self.youtube_options_download(*index) {
                    let tx = self.sender.clone();
                    std::thread::spawn(move || {
                        tx.send(UpdateComponents::DownloadErrDownload(e.to_string()))
                            .ok();
                        sleep(Duration::from_secs(5));
                        tx.send(UpdateComponents::DownloadCompleted(None)).ok();
                    });
                }
            }
        }
    }
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
                    self.mount_error_popup(format!("update photo error: {e}"));
                }
            }

            GSMsg::PopupCloseLibraryAddPlaylist => {
                self.general_search_after_library_add_playlist();
            }
            GSMsg::PopupCloseOkLibraryLocate => {
                self.general_search_after_library_select();
                self.app.umount(&Id::GeneralSearchInput).ok();
                self.app.umount(&Id::GeneralSearchTable).ok();

                if let Err(e) = self.update_photo() {
                    self.mount_error_popup(format!("update photo error: {e}"));
                }
            }
            GSMsg::PopupClosePlaylistPlaySelected => {
                self.general_search_after_playlist_play_selected();
                self.app.umount(&Id::GeneralSearchInput).ok();
                self.app.umount(&Id::GeneralSearchTable).ok();

                if let Err(e) = self.update_photo() {
                    self.mount_error_popup(format!("update photo error: {e}"));
                }
            }
            GSMsg::PopupCloseOkPlaylistLocate => {
                self.general_search_after_playlist_select();
                self.app.umount(&Id::GeneralSearchInput).ok();
                self.app.umount(&Id::GeneralSearchTable).ok();

                if let Err(e) = self.update_photo() {
                    self.mount_error_popup(format!("update photo error: {e}"));
                }
            }
            GSMsg::PopupCloseDatabaseAddPlaylist => {
                if let Err(e) = self.general_search_after_database_add_playlist() {
                    self.mount_error_popup(format!("db add playlist error: {e}"));
                };
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
                    let _ = self.app.umount(&Id::DeleteConfirmRadioPopup);
                }
                if self.app.mounted(&Id::DeleteConfirmInputPopup) {
                    let _ = self.app.umount(&Id::DeleteConfirmInputPopup);
                }
            }
            Msg::DeleteConfirmCloseOk => {
                if self.app.mounted(&Id::DeleteConfirmRadioPopup) {
                    let _ = self.app.umount(&Id::DeleteConfirmRadioPopup);
                }
                if self.app.mounted(&Id::DeleteConfirmInputPopup) {
                    let _ = self.app.umount(&Id::DeleteConfirmInputPopup);
                }
                if let Err(e) = self.library_delete_song() {
                    self.mount_error_popup(format!("Delete error: {e}"));
                };
            }
            _ => {}
        }
        None
    }
    fn update_playlist(&mut self, msg: &PLMsg) {
        match msg {
            PLMsg::Add(current_node) => {
                self.playlist_add(current_node);
            }
            PLMsg::Delete(index) => {
                self.playlist_delete_item(*index);
            }
            PLMsg::DeleteAll => {
                self.playlist_empty();
            }
            PLMsg::Shuffle => {
                self.playlist_shuffle();
            }
            PLMsg::PlaySelected(index) => {
                // if let Some(song) = self.playlist_items.get(index) {}
                self.playlist_play_selected(*index);
            }
            PLMsg::LoopModeCycle => {
                self.playlist_cycle_loop_mode();
            }
            PLMsg::AddFront => {
                self.config.add_playlist_front = !self.config.add_playlist_front;
                self.playlist_update_title();
            }
            PLMsg::TableBlur => match self.layout {
                TermusicLayout::TreeView => assert!(self.app.active(&Id::Library).is_ok()),
                TermusicLayout::DataBase => assert!(self.app.active(&Id::DBListCriteria).is_ok()),
            },
            PLMsg::NextSong => {
                self.player_save_last_position();
                self.player.skip();
            }

            PLMsg::PrevSong => {
                self.player_previous();
            }
            PLMsg::SwapDown(index) => {
                self.player.playlist.swap_down(*index);
                self.playlist_sync();
            }
            PLMsg::SwapUp(index) => {
                self.player.playlist.swap_up(*index);
                self.playlist_sync();
            }
            PLMsg::CmusLQueue => {
                self.playlist_add_cmus_lqueue();
            }
            PLMsg::CmusTQueue => {
                self.playlist_add_cmus_tqueue();
            }
        }
    }

    // change status bar text to indicate the downloading state
    #[allow(clippy::too_many_lines)]
    pub fn update_components(&mut self) {
        if let Ok(update_components_state) = self.receiver.try_recv() {
            self.redraw = true;
            match update_components_state {
                UpdateComponents::DownloadRunning => {
                    self.downloading_item_quantity += 1;
                    let label_str = if self.downloading_item_quantity > 1 {
                        format!(" {} items downloading... ", self.downloading_item_quantity)
                    } else {
                        format!(" {} item downloading... ", self.downloading_item_quantity)
                    };
                    self.remount_label_help(
                        Some(&label_str),
                        Some(
                            self.config
                                .style_color_symbol
                                .library_highlight()
                                .unwrap_or(Color::Black),
                        ),
                        Some(
                            self.config
                                .style_color_symbol
                                .library_background()
                                .unwrap_or(Color::Yellow),
                        ),
                    );
                }
                UpdateComponents::DownloadSuccess => {
                    self.downloading_item_quantity -= 1;
                    if self.downloading_item_quantity > 0 {
                        self.app
                            .attr(
                                &Id::LabelCounter,
                                Attribute::Text,
                                AttrValue::String(self.downloading_item_quantity.to_string()),
                            )
                            .ok();
                        self.remount_label_help(
                            Some(
                                format!(
                                    " 1 of {} Download Success! {} is still running.",
                                    self.downloading_item_quantity + 1,
                                    self.downloading_item_quantity
                                )
                                .as_str(),
                            ),
                            Some(
                                self.config
                                    .style_color_symbol
                                    .library_highlight()
                                    .unwrap_or(Color::Black),
                            ),
                            Some(
                                self.config
                                    .style_color_symbol
                                    .library_background()
                                    .unwrap_or(Color::Yellow),
                            ),
                        );
                    } else {
                        self.remount_label_help(
                            Some(" All Downloads Success! "),
                            Some(
                                self.config
                                    .style_color_symbol
                                    .library_highlight()
                                    .unwrap_or(Color::Black),
                            ),
                            Some(
                                self.config
                                    .style_color_symbol
                                    .library_background()
                                    .unwrap_or(Color::Yellow),
                            ),
                        );
                    }

                    if self.app.mounted(&Id::TagEditor(IdTagEditor::LabelHint)) {
                        self.umount_tageditor();
                    }
                }
                UpdateComponents::DownloadCompleted(Some(file)) => {
                    if self.downloading_item_quantity > 0 {
                        return;
                    }
                    self.library_reload_with_node_focus(Some(file.as_str()));
                    self.remount_label_help(None, None, None);
                }
                UpdateComponents::DownloadCompleted(None) => {
                    if self.downloading_item_quantity > 0 {
                        return;
                    }
                    self.library_reload_tree();
                    self.remount_label_help(None, None, None);
                }
                UpdateComponents::DownloadErrDownload(error_message) => {
                    self.downloading_item_quantity -= 1;
                    self.app
                        .attr(
                            &Id::LabelCounter,
                            Attribute::Text,
                            AttrValue::String(self.downloading_item_quantity.to_string()),
                        )
                        .ok();
                    self.mount_error_popup(format!("download failed: {error_message}"));
                    if self.downloading_item_quantity > 0 {
                        self.remount_label_help(
                            Some(
                                format!(
                                    " 1 item download error! {} is still running. ",
                                    self.downloading_item_quantity
                                )
                                .as_str(),
                            ),
                            Some(
                                self.config
                                    .style_color_symbol
                                    .library_highlight()
                                    .unwrap_or(Color::Black),
                            ),
                            Some(
                                self.config
                                    .style_color_symbol
                                    .library_background()
                                    .unwrap_or(Color::Yellow),
                            ),
                        );
                        return;
                    }

                    self.remount_label_help(
                        Some(" Download error "),
                        Some(
                            self.config
                                .style_color_symbol
                                .library_highlight()
                                .unwrap_or(Color::Black),
                        ),
                        Some(
                            self.config
                                .style_color_symbol
                                .library_background()
                                .unwrap_or(Color::Yellow),
                        ),
                    );
                }
                UpdateComponents::DownloadErrEmbedData => {
                    self.mount_error_popup("download ok but tag info is not complete.");
                    self.remount_label_help(
                        Some(" Download Error! "),
                        Some(
                            self.config
                                .style_color_symbol
                                .library_highlight()
                                .unwrap_or(Color::Black),
                        ),
                        Some(
                            self.config
                                .style_color_symbol
                                .library_background()
                                .unwrap_or(Color::Yellow),
                        ),
                    );
                }
                UpdateComponents::YoutubeSearchSuccess(y) => {
                    self.youtube_options = y;
                    self.sync_youtube_options();
                    self.redraw = true;
                }
                UpdateComponents::YoutubeSearchFail(e) => {
                    self.mount_error_popup(format!("Youtube search fail: {e}"));
                }
                UpdateComponents::MessageShow((title, text)) => {
                    self.mount_message(&title, &text);
                }
                UpdateComponents::MessageHide((title, text)) => {
                    self.umount_message(&title, &text);
                } //_ => {}
            }
        };
    }

    // update playlist items when loading
    // pub fn update_playlist_items(&mut self) {
    //     if let Ok(playlist_items) = self.receiver_playlist_items.try_recv() {
    //         self.player.playlist_items = playlist_items;
    //         self.playlist_sync();
    //         // self.redraw = true;
    //     }
    // }

    // show a popup for playing song
    pub fn update_playing_song(&self) {
        if let Some(song) = &self.player.playlist.current_track {
            let name = song.name().unwrap_or("Unknown Song");
            self.show_message_timeout("Current Playing", name, None);
        }
    }

    pub fn show_message_timeout(&self, title: &str, text: &str, time_out: Option<u64>) {
        let tx = self.sender.clone();
        let title_string = title.to_string();
        let text_string = text.to_string();
        let time_out = time_out.unwrap_or(5);

        thread::spawn(move || {
            tx.send(UpdateComponents::MessageShow((
                title_string.clone(),
                text_string.clone(),
            )))
            .ok();
            sleep(Duration::from_secs(time_out));
            tx.send(UpdateComponents::MessageHide((title_string, text_string)))
                .ok();
        });
    }

    // update player messages
    pub fn update_player_msg(&mut self) {
        if let Ok(msg) = self.player.message_rx.try_recv() {
            match msg {
                PlayerMsg::Eos => {
                    // eprintln!("Eos received");
                    // self.player_clear_last_position();
                    if self.player.playlist.is_empty() {
                        self.player_stop();
                        return;
                    }
                    self.player.handle_current_track();
                    self.player.start_play();
                    self.player_restore_last_position();
                }
                PlayerMsg::AboutToFinish => {
                    if self.config.gapless {
                        // eprintln!("about to finish received");
                        self.player.enqueue_next();
                    }
                }
                PlayerMsg::CurrentTrackUpdated => {
                    // eprintln!("current track update received");
                    self.player_update_current_track_after();
                    if (self.config.speed - 10).abs() >= 1 {
                        self.player.set_speed(self.config.speed);
                    }
                }
                PlayerMsg::Progress(time_pos, duration) => {
                    self.progress_update(time_pos, duration);
                }
            }
        }
    }
}
