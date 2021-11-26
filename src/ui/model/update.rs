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
use crate::ui::{Id, Model, Msg};

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
                        self.app.unlock_subs();
                    }
                    if self.app.mounted(&Id::DeleteConfirmInputPopup) {
                        let _ = self.app.umount(&Id::DeleteConfirmInputPopup);
                        self.app.unlock_subs();
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
                    self.app.unlock_subs();
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
                Msg::LibrarySearchPopupShow => {
                    self.mount_search_library();
                    self.update_search_library("*");
                    None
                }
                Msg::LibrarySearchPopupUpdate(input) => {
                    self.update_search_library(&input);
                    None
                }
                Msg::LibrarySearchPopupCloseCancel => {
                    self.app.umount(&Id::LibrarySearchInput).ok();
                    self.app.umount(&Id::LibrarySearchTable).ok();
                    self.app.unlock_subs();
                    None
                }
                Msg::LibrarySearchInputBlur => {
                    if self.app.mounted(&Id::LibrarySearchTable) {
                        self.app.active(&Id::LibrarySearchTable).ok();
                    }
                    None
                }
                Msg::LibrarySearchTableBlur => {
                    if self.app.mounted(&Id::LibrarySearchInput) {
                        self.app.active(&Id::LibrarySearchInput).ok();
                    }
                    None
                }
                Msg::LibrarySearchPopupCloseAddPlaylist => {
                    self.add_playlist_after_search_library();
                    self.app.umount(&Id::LibrarySearchInput).ok();
                    self.app.umount(&Id::LibrarySearchTable).ok();
                    self.app.unlock_subs();

                    None
                }
                Msg::LibrarySearchPopupCloseOkLocate => {
                    self.select_after_search_library();
                    self.app.umount(&Id::LibrarySearchInput).ok();
                    self.app.umount(&Id::LibrarySearchTable).ok();
                    self.app.unlock_subs();
                    None
                }
                Msg::PlaylistAdd(current_node) => {
                    if let Err(e) = self.add_playlist(&current_node) {
                        self.mount_error_popup(format!("Add Playlist error: {}", e).as_str());
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
                Msg::PlaylistPlaySelected => {
                    // if let Some(song) = self.playlist_items.get(index) {}
                    self.playlist_play_selected();
                    None
                }
                Msg::ErrorPopupClose => {
                    let _ = self.app.umount(&Id::ErrorPopup);
                    self.app.unlock_subs();
                    None
                }
                Msg::PlaylistLoopModeCycle => {
                    self.cycle_loop_mode();
                    None
                }
                Msg::PlayerTogglePause => {
                    self.play_pause();
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
                Msg::PlaylistPrevSong => {
                    self.previous_song();
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

                Msg::None => None,
            }
        } else {
            None
        }
    }
}
