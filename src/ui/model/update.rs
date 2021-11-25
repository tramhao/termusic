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
use crate::ui::{model::MAX_DEPTH, Id, Model, Msg};

use crate::ui::Status;
use std::path::PathBuf;
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
                Msg::DeleteConfirmShow => {
                    self.update_library_delete();
                    None
                }
                Msg::DeleteConfirmCloseCancel => {
                    if self.app.mounted(&Id::DeleteConfirmRadioPopup) {
                        let _ = self.app.umount(&Id::DeleteConfirmRadioPopup);
                    }
                    if self.app.mounted(&Id::DeleteConfirmInputPopup) {
                        let _ = self.app.umount(&Id::DeleteConfirmInputPopup);
                    }
                    None
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
                    None
                }
                Msg::QuitPopupShow => {
                    if self.app.mounted(&Id::HelpPopup) {
                        println!("help mounted");
                        return None;
                    }
                    println!("help not mounted");
                    self.mount_quit_popup();
                    None
                }
                Msg::QuitPopupClose => {
                    let _ = self.app.umount(&Id::QuitPopup);
                    None
                }
                Msg::QuitPopupCloseQuit => {
                    self.quit = true;
                    None
                }
                Msg::LibraryTreeBlur => {
                    // Give focus to letter counter
                    assert!(self.app.active(&Id::Playlist).is_ok());
                    None
                }
                Msg::PlaylistTableBlur => {
                    assert!(self.app.active(&Id::Library).is_ok());
                    None
                }
                Msg::LibraryTreeExtendDir(path) => {
                    self.extend_dir(&path, PathBuf::from(path.as_str()).as_path(), MAX_DEPTH);
                    self.reload_tree();
                    None
                }
                Msg::LibraryTreeGoToUpperDir => {
                    if let Some(parent) = self.upper_dir() {
                        self.scan_dir(parent.as_path());
                        self.reload_tree();
                    }
                    None
                }
                Msg::PlaylistAdd(current_node) => {
                    if let Err(e) = self.add_playlist(&current_node) {
                        self.mount_error_popup(format!("Application error: {}", e).as_str());
                    }
                    None
                } // _ => None,
                Msg::PlaylistDelete(index) => {
                    self.delete_item_playlist(index);
                    None
                }
                Msg::PlaylistDeleteAll => {
                    self.empty_playlist();
                    None
                }
                Msg::PlaylistShuffle => {
                    self.shuffle();
                    None
                }
                Msg::ErrorPopupClose => {
                    let _ = self.app.umount(&Id::ErrorPopup);
                    None
                }
                Msg::PlaylistLoopModeCycle => {
                    self.cycle_loop_mode();
                    None
                }
                Msg::PlayerTogglePause => {
                    self.player.toggle_pause();
                    match self.status {
                        Some(Status::Running) => self.status = Some(Status::Paused),
                        Some(Status::Paused) => self.status = Some(Status::Running),
                        _ => {}
                    }
                    None
                }
                Msg::PlayerVolumeUp => {
                    self.player.volume_up();
                    self.config.volume = self.player.volume();
                    self.update_progress_title();
                    None
                }
                Msg::PlayerVolumeDown => {
                    self.player.volume_down();
                    self.config.volume = self.player.volume();
                    self.update_progress_title();
                    None
                }
                Msg::PlaylistNextSong => {
                    self.next_song();
                    None
                }
                Msg::HelpPopupShow => {
                    self.mount_help_popup();
                    None
                }
                Msg::HelpPopupClose => {
                    let _ = self.app.umount(&Id::HelpPopup);
                    None
                }

                Msg::None => None,
            }
        } else {
            None
        }
    }
}
