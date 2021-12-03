//! ## Model
//!
//! app model
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
use crate::ui::{model::UpdateComponents, Id, Model, Msg, StatusLine};
use std::thread::{self, sleep};
use std::time::Duration;
use tuirealm::props::{AttrValue, Attribute, Color};
use tuirealm::Update;

// Let's implement Update for model

impl Update<Msg> for Model {
    #[allow(clippy::too_many_lines)]
    fn update(&mut self, msg: Option<Msg>) -> Option<Msg> {
        if let Some(msg) = msg {
            // Set redraw
            self.redraw = true;
            // Match message
            match msg {
                Msg::DeleteConfirmShow
                | Msg::DeleteConfirmCloseCancel
                | Msg::DeleteConfirmCloseOk => {
                    self.update_delete_confirmation(&msg);
                    None
                }
                Msg::ErrorPopupClose => {
                    if self.app.mounted(&Id::ErrorPopup) {
                        self.app.umount(&Id::ErrorPopup).ok();
                    }
                    None
                }
                Msg::QuitPopupShow => {
                    self.mount_quit_popup();
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
                Msg::LibraryTreeBlur => {
                    assert!(self.app.active(&Id::Playlist).is_ok());
                    None
                }
                Msg::PlaylistTableBlur => {
                    assert!(self.app.active(&Id::Library).is_ok());
                    None
                }
                Msg::LibraryTreeExtendDir(path) => {
                    self.library_stepinto(&path);
                    // self.extend_dir(&path, PathBuf::from(path.as_str()).as_path(), MAX_DEPTH);
                    // self.reload_tree();
                    None
                }
                Msg::LibraryTreeGoToUpperDir => {
                    self.library_stepout();
                    None
                }
                Msg::LibraryYank => {
                    self.library_yank();
                    None
                }
                Msg::LibraryPaste => {
                    if let Err(e) = self.library_paste() {
                        self.mount_error_popup(format!("Paste error: {}", e).as_str());
                    }
                    None
                }
                Msg::LibrarySearchPopupShow
                | Msg::LibrarySearchPopupUpdate(_)
                | Msg::LibrarySearchPopupCloseCancel
                | Msg::LibrarySearchInputBlur
                | Msg::LibrarySearchTableBlur
                | Msg::LibrarySearchPopupCloseAddPlaylist
                | Msg::LibrarySearchPopupCloseOkLocate => {
                    self.update_library_search(&msg);
                    None
                }
                Msg::PlaylistAdd(_)
                | Msg::PlaylistDelete(_)
                | Msg::PlaylistDeleteAll
                | Msg::PlaylistShuffle
                | Msg::PlaylistPlaySelected(_)
                | Msg::PlaylistAddFront
                | Msg::PlaylistLoopModeCycle => {
                    self.update_playlist(msg);
                    None
                }
                Msg::PlayerTogglePause => {
                    self.player_toggle_pause();
                    None
                }
                Msg::PlayerSeek(offset) => {
                    self.player_seek(offset as i64);
                    self.progress_update();
                    None
                }
                Msg::PlayerVolumeUp => {
                    self.player.volume_up();
                    self.config.volume = self.player.volume();
                    self.progress_update_title();
                    None
                }
                Msg::PlayerVolumeDown => {
                    self.player.volume_down();
                    self.config.volume = self.player.volume();
                    self.progress_update_title();
                    None
                }
                Msg::PlaylistNextSong => {
                    self.player_next();
                    None
                }
                Msg::PlaylistPrevSong => {
                    self.player_previous();
                    None
                }

                Msg::HelpPopupShow => {
                    self.mount_help_popup();
                    None
                }
                Msg::HelpPopupClose => {
                    let _ = self.app.umount(&Id::HelpPopup);
                    self.app.unlock_subs();
                    None
                }
                Msg::YoutubeSearchInputPopupShow
                | Msg::YoutubeSearchInputPopupCloseCancel
                | Msg::YoutubeSearchInputPopupCloseOk(_)
                | Msg::YoutubeSearchTablePopupCloseCancel
                | Msg::YoutubeSearchTablePopupNext
                | Msg::YoutubeSearchTablePopupPrevious
                | Msg::YoutubeSearchTablePopupCloseOk(_) => {
                    self.update_youtube_search(&msg);
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
                Msg::TagEditorBlur(_)
                | Msg::TagEditorRun(_)
                | Msg::TERadioTagBlur
                | Msg::TEInputTitleBlur
                | Msg::TEInputArtistBlur
                | Msg::TESelectLyricBlur
                | Msg::TESelectLyricOk(_)
                | Msg::TECounterDeleteBlur
                | Msg::TECounterDeleteOk
                | Msg::TEDownload(_)
                | Msg::TEEmbed(_)
                | Msg::TEHelpPopupClose
                | Msg::TEHelpPopupShow
                | Msg::TERadioTagOk
                | Msg::TESearch
                | Msg::TETextareaLyricBlur
                | Msg::TETableLyricOptionsBlur => {
                    self.update_tageditor(msg);
                    None
                }
                // Msg::None | _ => None,
                Msg::None => None,
            }
        } else {
            None
        }
    }
}

impl Model {
    pub fn update_youtube_search(&mut self, msg: &Msg) {
        match msg {
            Msg::YoutubeSearchInputPopupShow => {
                self.mount_youtube_search_input();
            }
            Msg::YoutubeSearchInputPopupCloseCancel => {
                if self.app.mounted(&Id::YoutubeSearchInputPopup) {
                    assert!(self.app.umount(&Id::YoutubeSearchInputPopup).is_ok());
                }
                self.app.unlock_subs();
            }
            Msg::YoutubeSearchInputPopupCloseOk(url) => {
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
            Msg::YoutubeSearchTablePopupCloseCancel => {
                if self.app.mounted(&Id::YoutubeSearchTablePopup) {
                    assert!(self.app.umount(&Id::YoutubeSearchTablePopup).is_ok());
                }
                self.app.unlock_subs();
            }
            Msg::YoutubeSearchTablePopupNext => {
                self.youtube_options_next_page();
            }
            Msg::YoutubeSearchTablePopupPrevious => {
                self.youtube_options_prev_page();
            }
            Msg::YoutubeSearchTablePopupCloseOk(index) => {
                if let Err(e) = self.youtube_options_download(*index) {
                    self.mount_error_popup(format!("download song error: {}", e).as_str());
                }

                if self.app.mounted(&Id::YoutubeSearchTablePopup) {
                    assert!(self.app.umount(&Id::YoutubeSearchTablePopup).is_ok());
                }
                self.app.unlock_subs();
            }
            _ => {}
        }
    }
    pub fn update_library_search(&mut self, msg: &Msg) {
        match msg {
            Msg::LibrarySearchPopupShow => {
                self.mount_search_library();
                self.library_update_search("*");
            }
            Msg::LibrarySearchPopupUpdate(input) => {
                self.library_update_search(input);
            }
            Msg::LibrarySearchPopupCloseCancel => {
                self.app.umount(&Id::LibrarySearchInput).ok();
                self.app.umount(&Id::LibrarySearchTable).ok();
                self.app.unlock_subs();
            }
            Msg::LibrarySearchInputBlur => {
                if self.app.mounted(&Id::LibrarySearchTable) {
                    self.app.active(&Id::LibrarySearchTable).ok();
                }
            }
            Msg::LibrarySearchTableBlur => {
                if self.app.mounted(&Id::LibrarySearchInput) {
                    self.app.active(&Id::LibrarySearchInput).ok();
                }
            }
            Msg::LibrarySearchPopupCloseAddPlaylist => {
                self.library_add_playlist_after_search();
                self.app.umount(&Id::LibrarySearchInput).ok();
                self.app.umount(&Id::LibrarySearchTable).ok();
                self.app.unlock_subs();
            }
            Msg::LibrarySearchPopupCloseOkLocate => {
                self.library_select_after_search();
                self.app.umount(&Id::LibrarySearchInput).ok();
                self.app.umount(&Id::LibrarySearchTable).ok();
                self.app.unlock_subs();
            }
            _ => {}
        }
    }
    pub fn update_delete_confirmation(&mut self, msg: &Msg) {
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
    }
    pub fn update_playlist(&mut self, msg: Msg) {
        match msg {
            Msg::PlaylistAdd(current_node) => {
                self.playlist_add(&current_node);
            }
            Msg::PlaylistDelete(index) => {
                self.playlist_delete_item(index);
            }
            Msg::PlaylistDeleteAll => {
                self.playlist_empty();
            }
            Msg::PlaylistShuffle => {
                self.playlist_shuffle();
            }
            Msg::PlaylistPlaySelected(index) => {
                // if let Some(song) = self.playlist_items.get(index) {}
                self.playlist_play_selected(index);
            }
            Msg::PlaylistLoopModeCycle => {
                self.playlist_cycle_loop_mode();
            }
            Msg::PlaylistAddFront => {
                self.config.add_playlist_front = !self.config.add_playlist_front;
                self.playlist_update_title();
            }
            _ => {}
        }
    }
    pub fn update_tageditor(&mut self, msg: Msg) {
        match msg {
            Msg::TagEditorRun(node_id) => {
                self.mount_tageditor(&node_id);
            }
            Msg::TagEditorBlur(song) => {
                if let Some(_s) = song {}
                self.umount_tageditor();
            }
            Msg::TEInputArtistBlur => {
                self.app.active(&Id::TEInputTitle).ok();
            }
            Msg::TEInputTitleBlur => {
                self.app.active(&Id::TERadioTag).ok();
            }
            Msg::TERadioTagBlur => {
                self.app.active(&Id::TETableLyricOptions).ok();
            }
            Msg::TETableLyricOptionsBlur => {
                self.app.active(&Id::TESelectLyric).ok();
            }
            Msg::TESelectLyricBlur => {
                self.app.active(&Id::TECounterDelete).ok();
            }
            Msg::TECounterDeleteBlur => {
                self.app.active(&Id::TETextareaLyric).ok();
            }
            Msg::TETextareaLyricBlur => {
                self.app.active(&Id::TEInputArtist).ok();
            }
            Msg::TECounterDeleteOk => {
                self.te_delete_lyric();
            }
            Msg::TESelectLyricOk(index) => {
                if let Some(mut song) = self.tageditor_song.clone() {
                    song.set_lyric_selected_index(index);
                    self.init_by_song(&song);
                }
            }
            Msg::TEHelpPopupClose => {
                if self.app.mounted(&Id::TEHelpPopup) {
                    self.app.umount(&Id::TEHelpPopup).ok();
                }
            }
            Msg::TEHelpPopupShow => {
                self.mount_tageditor_help();
            }
            Msg::TESearch => {
                self.te_songtag_search();
            }
            Msg::TEDownload(index) => {
                if let Err(e) = self.te_songtag_download(index) {
                    self.mount_error_popup(format!("download song by tag error: {}", e).as_str());
                }
            }
            Msg::TEEmbed(index) => {
                if let Err(e) = self.te_load_lyric_and_photo(index) {
                    self.mount_error_popup(format!("embed error: {}", e).as_str());
                }
            }
            Msg::TERadioTagOk => {
                if let Err(e) = self.te_rename_song_by_tag() {
                    self.mount_error_popup(format!("rename song by tag error: {}", e).as_str());
                }
            }
            _ => {}
        }
    }

    // change status bar text to indicate the downloading state
    pub fn update_components(&mut self) {
        if let Ok(update_components_state) = self.receiver.try_recv() {
            match update_components_state {
                UpdateComponents::DownloadRunning => {
                    self.update_status_line(StatusLine::Running);
                }
                UpdateComponents::DownloadSuccess => {
                    self.update_status_line(StatusLine::Success);
                    if self.app.mounted(&Id::TELabelHint) {
                        self.umount_tageditor();
                    }
                }
                UpdateComponents::DownloadCompleted(Some(file)) => {
                    self.library_sync(Some(file.as_str()));
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
    pub fn update_status_line(&mut self, s: StatusLine) {
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
    pub fn update_playlist_items(&mut self) {
        if let Ok(playlist_items) = self.receiver_playlist_items.try_recv() {
            self.playlist_items = playlist_items;
            self.playlist_sync();
            // self.redraw = true;
        }
    }

    // show a popup for playing song
    pub fn update_playing_song(&self) {
        if let Some(song) = &self.current_song {
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

    // fn update_duration(&mut self) {
    //     let (_new_prog, _time_pos, duration) = self.player.get_progress();
    //     if let Some(song) = &mut self.current_song {
    //         let diff = song.duration().as_secs().checked_sub(duration as u64);
    //         if let Some(d) = diff {
    //             if d > 1 {
    //                 let _drop = song.update_duration();
    //             }
    //         } else {
    //             let _drop = song.update_duration();
    //         }
    //     }
    // }
}
