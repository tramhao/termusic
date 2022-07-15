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
    DBMsg, GSMsg, Id, IdTagEditor, LIMsg, Model, Msg, PLMsg, StatusLine, TEMsg, YSMsg,
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
                        self.app.umount(&Id::ErrorPopup).ok();
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
                    let _ = self.app.umount(&Id::QuitPopup);
                    self.app.unlock_subs();
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
                    let _ = self.app.umount(&Id::HelpPopup);
                    self.app.unlock_subs();
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
                        self.mount_error_popup(&e.to_string());
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
                    self.mount_error_popup(format!("Paste error: {}", e).as_str());
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
                self.app.unlock_subs();
            }
            YSMsg::InputPopupCloseOk(url) => {
                if self.app.mounted(&Id::YoutubeSearchInputPopup) {
                    assert!(self.app.umount(&Id::YoutubeSearchInputPopup).is_ok());
                }
                self.app.unlock_subs();
                if url.starts_with("http") {
                    match self.youtube_dl(url) {
                        Ok(_) => {}
                        Err(e) => {
                            self.mount_error_popup(format!("download error: {}", e).as_str());
                        }
                    }
                } else {
                    self.mount_youtube_search_table();
                    self.youtube_options_search(url);
                }
            }
            YSMsg::TablePopupCloseCancel => {
                if self.app.mounted(&Id::YoutubeSearchTablePopup) {
                    assert!(self.app.umount(&Id::YoutubeSearchTablePopup).is_ok());
                }
                if let Err(e) = self.update_photo() {
                    self.mount_error_popup(format!("update photo error: {}", e).as_ref());
                }
                self.app.unlock_subs();
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

                if self.app.mounted(&Id::YoutubeSearchTablePopup) {
                    assert!(self.app.umount(&Id::YoutubeSearchTablePopup).is_ok());
                }
                if let Err(e) = self.update_photo() {
                    self.mount_error_popup(format!("update photo error: {}", e).as_ref());
                }

                self.app.unlock_subs();
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
                self.app.unlock_subs();
                if let Err(e) = self.update_photo() {
                    self.mount_error_popup(format!("update photo error: {}", e).as_ref());
                }
            }

            GSMsg::PopupCloseLibraryAddPlaylist => {
                self.general_search_after_library_add_playlist();
            }
            GSMsg::PopupCloseOkLibraryLocate => {
                self.general_search_after_library_select();
                self.app.umount(&Id::GeneralSearchInput).ok();
                self.app.umount(&Id::GeneralSearchTable).ok();
                self.app.unlock_subs();

                if let Err(e) = self.update_photo() {
                    self.mount_error_popup(format!("update photo error: {}", e).as_ref());
                }
            }
            GSMsg::PopupClosePlaylistPlaySelected => {
                self.general_search_after_playlist_play_selected();
                self.app.umount(&Id::GeneralSearchInput).ok();
                self.app.umount(&Id::GeneralSearchTable).ok();
                self.app.unlock_subs();

                if let Err(e) = self.update_photo() {
                    self.mount_error_popup(format!("update photo error: {}", e).as_ref());
                }
            }
            GSMsg::PopupCloseOkPlaylistLocate => {
                self.general_search_after_playlist_select();
                self.app.umount(&Id::GeneralSearchInput).ok();
                self.app.umount(&Id::GeneralSearchTable).ok();
                self.app.unlock_subs();

                if let Err(e) = self.update_photo() {
                    self.mount_error_popup(format!("update photo error: {}", e).as_ref());
                }
            }
            GSMsg::PopupCloseDatabaseAddPlaylist => {
                if let Err(e) = self.general_search_after_database_add_playlist() {
                    self.mount_error_popup(format!("db add playlist error: {}", e).as_str());
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
                    self.app.unlock_subs();
                }
                if self.app.mounted(&Id::DeleteConfirmInputPopup) {
                    let _ = self.app.umount(&Id::DeleteConfirmInputPopup);
                    self.app.unlock_subs();
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
                    self.mount_error_popup(format!("Delete error: {}", e).as_str());
                };
                self.app.unlock_subs();
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
    fn update_tageditor(&mut self, msg: &TEMsg) {
        match msg {
            TEMsg::TagEditorRun(node_id) => {
                self.mount_tageditor(node_id);
            }
            TEMsg::TagEditorClose(_song) => {
                self.umount_tageditor();
                if let Some(s) = self.tageditor_song.clone() {
                    self.library_reload_with_node_focus(s.file());
                }
            }
            TEMsg::TEInputArtistBlurDown | TEMsg::TERadioTagBlurUp => {
                self.app
                    .active(&Id::TagEditor(IdTagEditor::InputTitle))
                    .ok();
            }
            TEMsg::TEInputTitleBlurDown | TEMsg::TETableLyricOptionsBlurUp => {
                self.app.active(&Id::TagEditor(IdTagEditor::RadioTag)).ok();
            }
            TEMsg::TERadioTagBlurDown | TEMsg::TESelectLyricBlurUp => {
                self.app
                    .active(&Id::TagEditor(IdTagEditor::TableLyricOptions))
                    .ok();
            }
            TEMsg::TETableLyricOptionsBlurDown | TEMsg::TECounterDeleteBlurUp => {
                self.app
                    .active(&Id::TagEditor(IdTagEditor::SelectLyric))
                    .ok();
            }
            TEMsg::TESelectLyricBlurDown | TEMsg::TETextareaLyricBlurUp => {
                self.app
                    .active(&Id::TagEditor(IdTagEditor::CounterDelete))
                    .ok();
            }
            TEMsg::TECounterDeleteBlurDown | TEMsg::TEInputArtistBlurUp => {
                self.app
                    .active(&Id::TagEditor(IdTagEditor::TextareaLyric))
                    .ok();
            }
            TEMsg::TETextareaLyricBlurDown | TEMsg::TEInputTitleBlurUp => {
                self.app
                    .active(&Id::TagEditor(IdTagEditor::InputArtist))
                    .ok();
            }
            TEMsg::TECounterDeleteOk => {
                self.te_delete_lyric();
            }
            TEMsg::TESelectLyricOk(index) => {
                if let Some(mut song) = self.tageditor_song.clone() {
                    song.set_lyric_selected_index(*index);
                    self.init_by_song(&song);
                }
            }
            TEMsg::TEHelpPopupClose => {
                if self.app.mounted(&Id::TagEditor(IdTagEditor::HelpPopup)) {
                    self.app.umount(&Id::TagEditor(IdTagEditor::HelpPopup)).ok();
                }
            }
            TEMsg::TEHelpPopupShow => {
                self.mount_tageditor_help();
            }
            TEMsg::TESearch => {
                self.te_songtag_search();
            }
            TEMsg::TEDownload(index) => {
                if let Err(e) = self.te_songtag_download(*index) {
                    self.mount_error_popup(format!("download song by tag error: {}", e).as_str());
                }
            }
            TEMsg::TEEmbed(index) => {
                if let Err(e) = self.te_load_lyric_and_photo(*index) {
                    self.mount_error_popup(format!("embed error: {}", e).as_str());
                }
            }
            TEMsg::TERadioTagOk => {
                if let Err(e) = self.te_rename_song_by_tag() {
                    self.mount_error_popup(format!("rename song by tag error: {}", e).as_str());
                }
            } // _ => {}
        }
    }

    // change status bar text to indicate the downloading state
    pub fn update_components(&mut self) {
        if let Ok(update_components_state) = self.receiver.try_recv() {
            self.redraw = true;
            match update_components_state {
                UpdateComponents::DownloadRunning => {
                    self.update_status_line(StatusLine::Running);
                }
                UpdateComponents::DownloadSuccess => {
                    self.update_status_line(StatusLine::Success);
                    if self.app.mounted(&Id::TagEditor(IdTagEditor::LabelHint)) {
                        self.umount_tageditor();
                    }
                }
                UpdateComponents::DownloadCompleted(Some(file)) => {
                    self.library_reload_with_node_focus(Some(file.as_str()));
                    self.update_status_line(StatusLine::Default);
                }
                UpdateComponents::DownloadCompleted(None) => {
                    // self.library_sync(None);
                    self.update_status_line(StatusLine::Default);
                }
                UpdateComponents::DownloadErrDownload(error_message) => {
                    self.mount_error_popup(format!("download failed: {}", error_message).as_str());
                    self.update_status_line(StatusLine::Error);
                }
                UpdateComponents::DownloadErrEmbedData => {
                    self.mount_error_popup("download ok but tag info is not complete.");
                    self.update_status_line(StatusLine::Error);
                }
                UpdateComponents::YoutubeSearchSuccess(y) => {
                    self.youtube_options = y;
                    self.sync_youtube_options();
                    self.redraw = true;
                }
                UpdateComponents::YoutubeSearchFail(e) => {
                    self.mount_error_popup(format!("Youtube search fail: {}", e).as_str());
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

    // change status bar text to indicate the downloading state
    fn update_status_line(&mut self, s: StatusLine) {
        match s {
            StatusLine::Default => {
                let text = format!("Press <CTRL+H> for help. Version: {}", crate::VERSION);
                assert!(self
                    .app
                    .attr(&Id::Label, Attribute::Text, AttrValue::String(text))
                    .is_ok());
                assert!(self
                    .app
                    .attr(&Id::Label, Attribute::Color, AttrValue::Color(Color::Cyan))
                    .is_ok());
                assert!(self
                    .app
                    .attr(
                        &Id::Label,
                        Attribute::Background,
                        AttrValue::Color(Color::Reset)
                    )
                    .is_ok());
            }
            StatusLine::Running => {
                let text = " Downloading...".to_string();
                assert!(self
                    .app
                    .attr(&Id::Label, Attribute::Text, AttrValue::String(text))
                    .is_ok());
                assert!(self
                    .app
                    .attr(&Id::Label, Attribute::Color, AttrValue::Color(Color::Black))
                    .is_ok());
                assert!(self
                    .app
                    .attr(
                        &Id::Label,
                        Attribute::Background,
                        AttrValue::Color(Color::Yellow)
                    )
                    .is_ok());
            }
            StatusLine::Success => {
                let text = " Download Success!".to_string();

                assert!(self
                    .app
                    .attr(&Id::Label, Attribute::Text, AttrValue::String(text))
                    .is_ok());
                assert!(self
                    .app
                    .attr(&Id::Label, Attribute::Color, AttrValue::Color(Color::White))
                    .is_ok());
                assert!(self
                    .app
                    .attr(
                        &Id::Label,
                        Attribute::Background,
                        AttrValue::Color(Color::Green)
                    )
                    .is_ok());
            }
            StatusLine::Error => {
                let text = " Download Error!".to_string();

                assert!(self
                    .app
                    .attr(&Id::Label, Attribute::Text, AttrValue::String(text))
                    .is_ok());
                assert!(self
                    .app
                    .attr(&Id::Label, Attribute::Color, AttrValue::Color(Color::White))
                    .is_ok());
                assert!(self
                    .app
                    .attr(
                        &Id::Label,
                        Attribute::Background,
                        AttrValue::Color(Color::Red)
                    )
                    .is_ok());
            }
        }
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
                    if self.player.playlist.is_empty() {
                        self.player_stop();
                        return;
                    }
                    self.player.start_play();
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
