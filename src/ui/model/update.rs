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
use crate::player::GeneralP;
use crate::ui::components::{load_alacritty_theme, ColorConfig};
use crate::ui::{
    model::UpdateComponents, CEMsg, GSMsg, Id, IdColorEditor, IdKeyEditor, IdTagEditor, KEMsg,
    LIMsg, Model, Msg, PLMsg, StatusLine, TEMsg, YSMsg,
};
use std::path::PathBuf;
use std::thread::{self, sleep};
use std::time::Duration;
use tuirealm::props::{AttrValue, Attribute, Color};
use tuirealm::Update;

// Let's implement Update for model

#[allow(clippy::too_many_lines)]
impl Update<Msg> for Model {
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
                    if self.config.disable_exit_confirmation {
                        self.quit = true;
                    } else {
                        self.mount_quit_popup();
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
                Msg::ColorEditor(m) => {
                    self.update_color_editor(&m);
                    None
                }
                Msg::KeyEditor(m) => {
                    self.update_key_editor(&m);
                    None
                }

                Msg::UpdatePhoto => {
                    if let Err(e) = self.update_photo() {
                        self.mount_error_popup(&e.to_string());
                    }
                    None
                }
                Msg::None => None,
            }
        } else {
            None
        }
    }
}

impl Model {
    #[allow(clippy::too_many_lines)]
    fn update_key_editor(&mut self, msg: &KEMsg) {
        match msg {
            KEMsg::KeyEditorShow => {
                self.ke_key_config = self.config.keys.clone();
                self.mount_key_editor();
            }
            KEMsg::KeyEditorCloseOk => {
                self.config.keys = self.ke_key_config.clone();
                if self.app.mounted(&Id::KeyEditor(IdKeyEditor::GlobalQuit)) {
                    self.umount_key_editor();
                }
            }
            KEMsg::KeyEditorCloseCancel => {
                if self.app.mounted(&Id::KeyEditor(IdKeyEditor::GlobalQuit)) {
                    self.umount_key_editor();
                }
            }

            KEMsg::HelpPopupShow => {
                self.mount_key_editor_help();
            }
            KEMsg::HelpPopupClose => {
                if self.app.mounted(&Id::KeyEditor(IdKeyEditor::HelpPopup)) {
                    self.app.umount(&Id::KeyEditor(IdKeyEditor::HelpPopup)).ok();
                }
            }
            KEMsg::KeyChanged(id) => {
                self.update_key_editor_key_changed(id);
            }

            KEMsg::RadioOkBlurUp
            | KEMsg::RadioOkBlurDown
            | KEMsg::PlaylistDeleteBlurDown
            | KEMsg::PlaylistDeleteBlurUp
            | KEMsg::PlaylistDeleteInputBlurDown
            | KEMsg::PlaylistDeleteInputBlurUp
            | KEMsg::PlaylistDeleteAllBlurDown
            | KEMsg::PlaylistDeleteAllBlurUp
            | KEMsg::PlaylistDeleteAllInputBlurDown
            | KEMsg::PlaylistDeleteAllInputBlurUp
            | KEMsg::PlaylistShuffleBlurDown
            | KEMsg::PlaylistShuffleBlurUp
            | KEMsg::PlaylistShuffleInputBlurDown
            | KEMsg::PlaylistShuffleInputBlurUp
            | KEMsg::PlaylistModeCycleBlurDown
            | KEMsg::PlaylistModeCycleBlurUp
            | KEMsg::PlaylistModeCycleInputBlurDown
            | KEMsg::PlaylistModeCycleInputBlurUp
            | KEMsg::PlaylistPlaySelectedBlurDown
            | KEMsg::PlaylistPlaySelectedBlurUp
            | KEMsg::PlaylistPlaySelectedInputBlurDown
            | KEMsg::PlaylistPlaySelectedInputBlurUp
            | KEMsg::PlaylistAddFrontBlurDown
            | KEMsg::PlaylistAddFrontBlurUp
            | KEMsg::PlaylistAddFrontInputBlurDown
            | KEMsg::PlaylistAddFrontInputBlurUp
            | KEMsg::PlaylistSearchBlurDown
            | KEMsg::PlaylistSearchBlurUp
            | KEMsg::PlaylistSearchInputBlurDown
            | KEMsg::PlaylistSearchInputBlurUp
            | KEMsg::GlobalColorEditorBlurDown
            | KEMsg::GlobalColorEditorBlurUp
            | KEMsg::GlobalColorEditorInputBlurDown
            | KEMsg::GlobalColorEditorInputBlurUp
            | KEMsg::GlobalKeyEditorBlurDown
            | KEMsg::GlobalKeyEditorBlurUp
            | KEMsg::GlobalKeyEditorInputBlurDown
            | KEMsg::GlobalKeyEditorInputBlurUp
            | KEMsg::LibraryDeleteBlurDown
            | KEMsg::LibraryDeleteBlurUp
            | KEMsg::LibraryDeleteInputBlurDown
            | KEMsg::LibraryDeleteInputBlurUp
            | KEMsg::LibraryLoadDirBlurDown
            | KEMsg::LibraryLoadDirBlurUp
            | KEMsg::LibraryLoadDirInputBlurDown
            | KEMsg::LibraryLoadDirInputBlurUp
            | KEMsg::LibraryPasteBlurDown
            | KEMsg::LibraryPasteBlurUp
            | KEMsg::LibraryPasteInputBlurDown
            | KEMsg::LibraryPasteInputBlurUp
            | KEMsg::LibrarySearchBlurDown
            | KEMsg::LibrarySearchBlurUp
            | KEMsg::LibrarySearchInputBlurDown
            | KEMsg::LibrarySearchInputBlurUp
            | KEMsg::LibrarySearchYoutubeBlurDown
            | KEMsg::LibrarySearchYoutubeBlurUp
            | KEMsg::LibrarySearchYoutubeInputBlurDown
            | KEMsg::LibrarySearchYoutubeInputBlurUp
            | KEMsg::LibraryTagEditorBlurDown
            | KEMsg::LibraryTagEditorBlurUp
            | KEMsg::LibraryTagEditorInputBlurDown
            | KEMsg::LibraryTagEditorInputBlurUp
            | KEMsg::LibraryYankBlurDown
            | KEMsg::LibraryYankBlurUp
            | KEMsg::LibraryYankInputBlurDown
            | KEMsg::LibraryYankInputBlurUp
            | KEMsg::GlobalPlayerSeekForwardBlurDown
            | KEMsg::GlobalPlayerSeekForwardBlurUp
            | KEMsg::GlobalPlayerSeekForwardInputBlurDown
            | KEMsg::GlobalPlayerSeekForwardInputBlurUp
            | KEMsg::GlobalPlayerSeekBackwardBlurDown
            | KEMsg::GlobalPlayerSeekBackwardBlurUp
            | KEMsg::GlobalPlayerSeekBackwardInputBlurDown
            | KEMsg::GlobalPlayerSeekBackwardInputBlurUp
            | KEMsg::GlobalLyricAdjustForwardBlurDown
            | KEMsg::GlobalLyricAdjustForwardBlurUp
            | KEMsg::GlobalLyricAdjustBackwardBlurDown
            | KEMsg::GlobalLyricAdjustBackwardBlurUp
            | KEMsg::GlobalLyricAdjustForwardInputBlurDown
            | KEMsg::GlobalLyricAdjustForwardInputBlurUp
            | KEMsg::GlobalLyricAdjustBackwardInputBlurDown
            | KEMsg::GlobalLyricAdjustBackwardInputBlurUp
            | KEMsg::GlobalLyricCyleBlurDown
            | KEMsg::GlobalLyricCyleBlurUp
            | KEMsg::GlobalLyricCyleInputBlurDown
            | KEMsg::GlobalLyricCyleInputBlurUp
            | KEMsg::GlobalHelpBlurDown
            | KEMsg::GlobalHelpBlurUp
            | KEMsg::GlobalHelpInputBlurDown
            | KEMsg::GlobalHelpInputBlurUp
            | KEMsg::GlobalVolumeDownBlurDown
            | KEMsg::GlobalVolumeDownBlurUp
            | KEMsg::GlobalVolumeDownInputBlurDown
            | KEMsg::GlobalVolumeDownInputBlurUp
            | KEMsg::GlobalVolumeUpBlurDown
            | KEMsg::GlobalVolumeUpBlurUp
            | KEMsg::GlobalVolumeUpInputBlurDown
            | KEMsg::GlobalVolumeUpInputBlurUp
            | KEMsg::GlobalGotoTopBlurUp
            | KEMsg::GlobalGotoTopBlurDown
            | KEMsg::GlobalGotoTopInputBlurDown
            | KEMsg::GlobalGotoTopInputBlurUp
            | KEMsg::GlobalGotoBottomBlurUp
            | KEMsg::GlobalGotoBottomBlurDown
            | KEMsg::GlobalGotoBottomInputBlurUp
            | KEMsg::GlobalGotoBottomInputBlurDown
            | KEMsg::GlobalPlayerTogglePauseBlurDown
            | KEMsg::GlobalPlayerTogglePauseBlurUp
            | KEMsg::GlobalPlayerTogglePauseInputBlurDown
            | KEMsg::GlobalPlayerTogglePauseInputBlurUp
            | KEMsg::GlobalPlayerNextBlurUp
            | KEMsg::GlobalPlayerNextBlurDown
            | KEMsg::GlobalPlayerNextInputBlurUp
            | KEMsg::GlobalPlayerNextInputBlurDown
            | KEMsg::GlobalPlayerPreviousBlurUp
            | KEMsg::GlobalPlayerPreviousBlurDown
            | KEMsg::GlobalPlayerPreviousInputBlurUp
            | KEMsg::GlobalPlayerPreviousInputBlurDown
            | KEMsg::GlobalLeftBlurUp
            | KEMsg::GlobalLeftBlurDown
            | KEMsg::GlobalLeftInputBlurUp
            | KEMsg::GlobalLeftInputBlurDown
            | KEMsg::GlobalRightBlurUp
            | KEMsg::GlobalRightBlurDown
            | KEMsg::GlobalRightInputBlurUp
            | KEMsg::GlobalRightInputBlurDown
            | KEMsg::GlobalUpBlurUp
            | KEMsg::GlobalUpBlurDown
            | KEMsg::GlobalUpInputBlurUp
            | KEMsg::GlobalUpInputBlurDown
            | KEMsg::GlobalDownBlurUp
            | KEMsg::GlobalDownBlurDown
            | KEMsg::GlobalDownInputBlurUp
            | KEMsg::GlobalDownInputBlurDown
            | KEMsg::GlobalQuitBlurUp
            | KEMsg::GlobalQuitInputBlurUp
            | KEMsg::GlobalQuitInputBlurDown
            | KEMsg::GlobalQuitBlurDown => {
                self.update_key_editor_focus(msg);
            }
        }
    }

    #[allow(clippy::too_many_lines)]
    fn update_key_editor_focus(&mut self, msg: &KEMsg) {
        match msg {
            KEMsg::RadioOkBlurDown | KEMsg::GlobalQuitInputBlurUp => {
                self.app
                    .active(&Id::KeyEditor(IdKeyEditor::GlobalQuit))
                    .ok();
            }
            KEMsg::GlobalQuitBlurDown | KEMsg::GlobalLeftBlurUp => {
                self.app
                    .active(&Id::KeyEditor(IdKeyEditor::GlobalQuitInput))
                    .ok();
            }
            KEMsg::GlobalQuitInputBlurDown | KEMsg::GlobalLeftInputBlurUp => {
                self.app
                    .active(&Id::KeyEditor(IdKeyEditor::GlobalLeft))
                    .ok();
            }

            KEMsg::GlobalLeftBlurDown | KEMsg::GlobalDownBlurUp => {
                self.app
                    .active(&Id::KeyEditor(IdKeyEditor::GlobalLeftInput))
                    .ok();
            }
            KEMsg::GlobalLeftInputBlurDown | KEMsg::GlobalDownInputBlurUp => {
                self.app
                    .active(&Id::KeyEditor(IdKeyEditor::GlobalDown))
                    .ok();
            }

            KEMsg::GlobalDownBlurDown | KEMsg::GlobalUpBlurUp => {
                self.app
                    .active(&Id::KeyEditor(IdKeyEditor::GlobalDownInput))
                    .ok();
            }

            KEMsg::GlobalDownInputBlurDown | KEMsg::GlobalUpInputBlurUp => {
                self.app.active(&Id::KeyEditor(IdKeyEditor::GlobalUp)).ok();
            }

            KEMsg::GlobalUpBlurDown | KEMsg::GlobalRightBlurUp => {
                self.app
                    .active(&Id::KeyEditor(IdKeyEditor::GlobalUpInput))
                    .ok();
            }

            KEMsg::GlobalUpInputBlurDown | KEMsg::GlobalRightInputBlurUp => {
                self.app
                    .active(&Id::KeyEditor(IdKeyEditor::GlobalRight))
                    .ok();
            }

            KEMsg::GlobalRightBlurDown | KEMsg::GlobalGotoTopBlurUp => {
                self.app
                    .active(&Id::KeyEditor(IdKeyEditor::GlobalRightInput))
                    .ok();
            }
            KEMsg::GlobalRightInputBlurDown | KEMsg::GlobalGotoTopInputBlurUp => {
                self.app
                    .active(&Id::KeyEditor(IdKeyEditor::GlobalGotoTop))
                    .ok();
            }
            KEMsg::GlobalGotoTopBlurDown | KEMsg::GlobalGotoBottomBlurUp => {
                self.app
                    .active(&Id::KeyEditor(IdKeyEditor::GlobalGotoTopInput))
                    .ok();
            }
            KEMsg::GlobalGotoTopInputBlurDown | KEMsg::GlobalGotoBottomInputBlurUp => {
                self.app
                    .active(&Id::KeyEditor(IdKeyEditor::GlobalGotoBottom))
                    .ok();
            }
            KEMsg::GlobalGotoBottomBlurDown | KEMsg::GlobalPlayerTogglePauseBlurUp => {
                self.app
                    .active(&Id::KeyEditor(IdKeyEditor::GlobalGotoBottomInput))
                    .ok();
            }
            KEMsg::GlobalGotoBottomInputBlurDown | KEMsg::GlobalPlayerTogglePauseInputBlurUp => {
                self.app
                    .active(&Id::KeyEditor(IdKeyEditor::GlobalPlayerTogglePause))
                    .ok();
            }
            KEMsg::GlobalPlayerTogglePauseBlurDown | KEMsg::GlobalPlayerNextBlurUp => {
                self.app
                    .active(&Id::KeyEditor(IdKeyEditor::GlobalPlayerTogglePauseInput))
                    .ok();
            }
            KEMsg::GlobalPlayerTogglePauseInputBlurDown | KEMsg::GlobalPlayerNextInputBlurUp => {
                self.app
                    .active(&Id::KeyEditor(IdKeyEditor::GlobalPlayerNext))
                    .ok();
            }
            KEMsg::GlobalPlayerNextBlurDown | KEMsg::GlobalPlayerPreviousBlurUp => {
                self.app
                    .active(&Id::KeyEditor(IdKeyEditor::GlobalPlayerNextInput))
                    .ok();
            }
            KEMsg::GlobalPlayerNextInputBlurDown | KEMsg::GlobalPlayerPreviousInputBlurUp => {
                self.app
                    .active(&Id::KeyEditor(IdKeyEditor::GlobalPlayerPrevious))
                    .ok();
            }
            KEMsg::GlobalPlayerPreviousBlurDown | KEMsg::GlobalHelpBlurUp => {
                self.app
                    .active(&Id::KeyEditor(IdKeyEditor::GlobalPlayerPreviousInput))
                    .ok();
            }

            KEMsg::GlobalPlayerPreviousInputBlurDown | KEMsg::GlobalHelpInputBlurUp => {
                self.app
                    .active(&Id::KeyEditor(IdKeyEditor::GlobalHelp))
                    .ok();
            }
            KEMsg::GlobalHelpBlurDown | KEMsg::GlobalVolumeUpBlurUp => {
                self.app
                    .active(&Id::KeyEditor(IdKeyEditor::GlobalHelpInput))
                    .ok();
            }

            KEMsg::GlobalHelpInputBlurDown | KEMsg::GlobalVolumeUpInputBlurUp => {
                self.app
                    .active(&Id::KeyEditor(IdKeyEditor::GlobalVolumeUp))
                    .ok();
            }
            KEMsg::GlobalVolumeUpBlurDown | KEMsg::GlobalVolumeDownBlurUp => {
                self.app
                    .active(&Id::KeyEditor(IdKeyEditor::GlobalVolumeUpInput))
                    .ok();
            }

            KEMsg::GlobalVolumeUpInputBlurDown | KEMsg::GlobalVolumeDownInputBlurUp => {
                self.app
                    .active(&Id::KeyEditor(IdKeyEditor::GlobalVolumeDown))
                    .ok();
            }
            KEMsg::GlobalVolumeDownBlurDown | KEMsg::GlobalPlayerSeekForwardBlurUp => {
                self.app
                    .active(&Id::KeyEditor(IdKeyEditor::GlobalVolumeDownInput))
                    .ok();
            }

            KEMsg::GlobalVolumeDownInputBlurDown | KEMsg::GlobalPlayerSeekForwardInputBlurUp => {
                self.app
                    .active(&Id::KeyEditor(IdKeyEditor::GlobalPlayerSeekForward))
                    .ok();
            }

            KEMsg::GlobalPlayerSeekForwardBlurDown | KEMsg::GlobalPlayerSeekBackwardBlurUp => {
                self.app
                    .active(&Id::KeyEditor(IdKeyEditor::GlobalPlayerSeekForwardInput))
                    .ok();
            }

            KEMsg::GlobalPlayerSeekForwardInputBlurDown
            | KEMsg::GlobalPlayerSeekBackwardInputBlurUp => {
                self.app
                    .active(&Id::KeyEditor(IdKeyEditor::GlobalPlayerSeekBackward))
                    .ok();
            }

            KEMsg::GlobalPlayerSeekBackwardBlurDown | KEMsg::GlobalLyricAdjustForwardBlurUp => {
                self.app
                    .active(&Id::KeyEditor(IdKeyEditor::GlobalPlayerSeekBackwardInput))
                    .ok();
            }

            KEMsg::GlobalPlayerSeekBackwardInputBlurDown
            | KEMsg::GlobalLyricAdjustForwardInputBlurUp => {
                self.app
                    .active(&Id::KeyEditor(IdKeyEditor::GlobalLyricAdjustForward))
                    .ok();
            }

            KEMsg::GlobalLyricAdjustForwardBlurDown | KEMsg::GlobalLyricAdjustBackwardBlurUp => {
                self.app
                    .active(&Id::KeyEditor(IdKeyEditor::GlobalLyricAdjustForwardInput))
                    .ok();
            }

            KEMsg::GlobalLyricAdjustForwardInputBlurDown
            | KEMsg::GlobalLyricAdjustBackwardInputBlurUp => {
                self.app
                    .active(&Id::KeyEditor(IdKeyEditor::GlobalLyricAdjustBackward))
                    .ok();
            }

            KEMsg::GlobalLyricAdjustBackwardBlurDown | KEMsg::GlobalLyricCyleBlurUp => {
                self.app
                    .active(&Id::KeyEditor(IdKeyEditor::GlobalLyricAdjustBackwardInput))
                    .ok();
            }

            KEMsg::GlobalLyricAdjustBackwardInputBlurDown | KEMsg::GlobalLyricCyleInputBlurUp => {
                self.app
                    .active(&Id::KeyEditor(IdKeyEditor::GlobalLyricCycle))
                    .ok();
            }

            KEMsg::GlobalLyricCyleBlurDown | KEMsg::GlobalColorEditorBlurUp => {
                self.app
                    .active(&Id::KeyEditor(IdKeyEditor::GlobalLyricCycleInput))
                    .ok();
            }

            KEMsg::GlobalLyricCyleInputBlurDown | KEMsg::GlobalColorEditorInputBlurUp => {
                self.app
                    .active(&Id::KeyEditor(IdKeyEditor::GlobalColorEditor))
                    .ok();
            }

            KEMsg::GlobalColorEditorBlurDown | KEMsg::GlobalKeyEditorBlurUp => {
                self.app
                    .active(&Id::KeyEditor(IdKeyEditor::GlobalColorEditorInput))
                    .ok();
            }

            KEMsg::GlobalColorEditorInputBlurDown | KEMsg::GlobalKeyEditorInputBlurUp => {
                self.app
                    .active(&Id::KeyEditor(IdKeyEditor::GlobalKeyEditor))
                    .ok();
            }

            KEMsg::GlobalKeyEditorBlurDown | KEMsg::LibraryTagEditorBlurUp => {
                self.app
                    .active(&Id::KeyEditor(IdKeyEditor::GlobalKeyEditorInput))
                    .ok();
            }

            KEMsg::GlobalKeyEditorInputBlurDown | KEMsg::LibraryTagEditorInputBlurUp => {
                self.app
                    .active(&Id::KeyEditor(IdKeyEditor::LibraryTagEditor))
                    .ok();
            }

            KEMsg::LibraryTagEditorBlurDown | KEMsg::LibraryDeleteBlurUp => {
                self.app
                    .active(&Id::KeyEditor(IdKeyEditor::LibraryTagEditorInput))
                    .ok();
            }

            KEMsg::LibraryTagEditorInputBlurDown | KEMsg::LibraryDeleteInputBlurUp => {
                self.app
                    .active(&Id::KeyEditor(IdKeyEditor::LibraryDelete))
                    .ok();
            }

            KEMsg::LibraryDeleteBlurDown | KEMsg::LibraryLoadDirBlurUp => {
                self.app
                    .active(&Id::KeyEditor(IdKeyEditor::LibraryDeleteInput))
                    .ok();
            }
            KEMsg::LibraryDeleteInputBlurDown | KEMsg::LibraryLoadDirInputBlurUp => {
                self.app
                    .active(&Id::KeyEditor(IdKeyEditor::LibraryLoadDir))
                    .ok();
            }

            KEMsg::LibraryLoadDirBlurDown | KEMsg::LibraryYankBlurUp => {
                self.app
                    .active(&Id::KeyEditor(IdKeyEditor::LibraryLoadDirInput))
                    .ok();
            }
            KEMsg::LibraryLoadDirInputBlurDown | KEMsg::LibraryYankInputBlurUp => {
                self.app
                    .active(&Id::KeyEditor(IdKeyEditor::LibraryYank))
                    .ok();
            }

            KEMsg::LibraryYankBlurDown | KEMsg::LibraryPasteBlurUp => {
                self.app
                    .active(&Id::KeyEditor(IdKeyEditor::LibraryYankInput))
                    .ok();
            }

            KEMsg::LibraryYankInputBlurDown | KEMsg::LibraryPasteInputBlurUp => {
                self.app
                    .active(&Id::KeyEditor(IdKeyEditor::LibraryPaste))
                    .ok();
            }

            KEMsg::LibraryPasteBlurDown | KEMsg::LibrarySearchBlurUp => {
                self.app
                    .active(&Id::KeyEditor(IdKeyEditor::LibraryPasteInput))
                    .ok();
            }
            KEMsg::LibraryPasteInputBlurDown | KEMsg::LibrarySearchInputBlurUp => {
                self.app
                    .active(&Id::KeyEditor(IdKeyEditor::LibrarySearch))
                    .ok();
            }

            KEMsg::LibrarySearchBlurDown | KEMsg::LibrarySearchYoutubeBlurUp => {
                self.app
                    .active(&Id::KeyEditor(IdKeyEditor::LibrarySearchInput))
                    .ok();
            }
            KEMsg::LibrarySearchInputBlurDown | KEMsg::LibrarySearchYoutubeInputBlurUp => {
                self.app
                    .active(&Id::KeyEditor(IdKeyEditor::LibrarySearchYoutube))
                    .ok();
            }

            KEMsg::LibrarySearchYoutubeBlurDown | KEMsg::PlaylistDeleteBlurUp => {
                self.app
                    .active(&Id::KeyEditor(IdKeyEditor::LibrarySearchYoutubeInput))
                    .ok();
            }

            KEMsg::LibrarySearchYoutubeInputBlurDown | KEMsg::PlaylistDeleteInputBlurUp => {
                self.app
                    .active(&Id::KeyEditor(IdKeyEditor::PlaylistDelete))
                    .ok();
            }

            KEMsg::PlaylistDeleteBlurDown | KEMsg::PlaylistDeleteAllBlurUp => {
                self.app
                    .active(&Id::KeyEditor(IdKeyEditor::PlaylistDeleteInput))
                    .ok();
            }
            KEMsg::PlaylistDeleteInputBlurDown | KEMsg::PlaylistDeleteAllInputBlurUp => {
                self.app
                    .active(&Id::KeyEditor(IdKeyEditor::PlaylistDeleteAll))
                    .ok();
            }

            KEMsg::PlaylistDeleteAllBlurDown | KEMsg::PlaylistSearchBlurUp => {
                self.app
                    .active(&Id::KeyEditor(IdKeyEditor::PlaylistDeleteAllInput))
                    .ok();
            }

            KEMsg::PlaylistDeleteAllInputBlurDown | KEMsg::PlaylistSearchInputBlurUp => {
                self.app
                    .active(&Id::KeyEditor(IdKeyEditor::PlaylistSearch))
                    .ok();
            }

            KEMsg::PlaylistSearchBlurDown | KEMsg::PlaylistShuffleBlurUp => {
                self.app
                    .active(&Id::KeyEditor(IdKeyEditor::PlaylistSearchInput))
                    .ok();
            }
            KEMsg::PlaylistSearchInputBlurDown | KEMsg::PlaylistShuffleInputBlurUp => {
                self.app
                    .active(&Id::KeyEditor(IdKeyEditor::PlaylistShuffle))
                    .ok();
            }

            KEMsg::PlaylistShuffleBlurDown | KEMsg::PlaylistAddFrontBlurUp => {
                self.app
                    .active(&Id::KeyEditor(IdKeyEditor::PlaylistShuffleInput))
                    .ok();
            }

            KEMsg::PlaylistShuffleInputBlurDown | KEMsg::PlaylistAddFrontInputBlurUp => {
                self.app
                    .active(&Id::KeyEditor(IdKeyEditor::PlaylistAddFront))
                    .ok();
            }

            KEMsg::PlaylistAddFrontBlurDown | KEMsg::PlaylistModeCycleBlurUp => {
                self.app
                    .active(&Id::KeyEditor(IdKeyEditor::PlaylistAddFrontInput))
                    .ok();
            }
            KEMsg::PlaylistAddFrontInputBlurDown | KEMsg::PlaylistModeCycleInputBlurUp => {
                self.app
                    .active(&Id::KeyEditor(IdKeyEditor::PlaylistModeCycle))
                    .ok();
            }

            KEMsg::PlaylistModeCycleBlurDown | KEMsg::PlaylistPlaySelectedBlurUp => {
                self.app
                    .active(&Id::KeyEditor(IdKeyEditor::PlaylistModeCycleInput))
                    .ok();
            }

            KEMsg::PlaylistModeCycleInputBlurDown | KEMsg::PlaylistPlaySelectedInputBlurUp => {
                self.app
                    .active(&Id::KeyEditor(IdKeyEditor::PlaylistPlaySelected))
                    .ok();
            }

            KEMsg::PlaylistPlaySelectedBlurDown | KEMsg::RadioOkBlurUp => {
                self.app
                    .active(&Id::KeyEditor(IdKeyEditor::PlaylistPlaySelectedInput))
                    .ok();
            }

            KEMsg::PlaylistPlaySelectedInputBlurDown | KEMsg::GlobalQuitBlurUp => {
                self.app.active(&Id::KeyEditor(IdKeyEditor::RadioOk)).ok();
            }
            _ => {}
        }
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
    fn update_color_editor(&mut self, msg: &CEMsg) {
        match msg {
            CEMsg::ThemeSelectBlurDown
            | CEMsg::ThemeSelectBlurUp
            | CEMsg::ColorEditorOkBlurDown
            | CEMsg::ColorEditorOkBlurUp
            | CEMsg::LibraryForegroundBlurDown
            | CEMsg::LibraryForegroundBlurUp
            | CEMsg::LibraryBackgroundBlurDown
            | CEMsg::LibraryBackgroundBlurUp
            | CEMsg::LibraryBorderBlurDown
            | CEMsg::LibraryBorderBlurUp
            | CEMsg::LibraryHighlightBlurDown
            | CEMsg::LibraryHighlightBlurUp
            | CEMsg::LibraryHighlightSymbolBlurDown
            | CEMsg::LibraryHighlightSymbolBlurUp
            | CEMsg::PlaylistForegroundBlurDown
            | CEMsg::PlaylistForegroundBlurUp
            | CEMsg::PlaylistBackgroundBlurDown
            | CEMsg::PlaylistBackgroundBlurUp
            | CEMsg::PlaylistBorderBlurDown
            | CEMsg::PlaylistBorderBlurUp
            | CEMsg::PlaylistHighlightBlurDown
            | CEMsg::PlaylistHighlightBlurUp
            | CEMsg::PlaylistHighlightSymbolBlurDown
            | CEMsg::PlaylistHighlightSymbolBlurUp
            | CEMsg::ProgressForegroundBlurDown
            | CEMsg::ProgressForegroundBlurUp
            | CEMsg::ProgressBackgroundBlurDown
            | CEMsg::ProgressBackgroundBlurUp
            | CEMsg::ProgressBorderBlurDown
            | CEMsg::ProgressBorderBlurUp
            | CEMsg::LyricForegroundBlurDown
            | CEMsg::LyricForegroundBlurUp
            | CEMsg::LyricBackgroundBlurDown
            | CEMsg::LyricBackgroundBlurUp
            | CEMsg::LyricBorderBlurDown
            | CEMsg::LyricBorderBlurUp => {
                self.update_color_editor_focus(msg);
            }

            CEMsg::ColorEditorShow => {
                self.ce_style_color_symbol = self.config.style_color_symbol.clone();
                self.mount_color_editor();
            }
            CEMsg::ColorEditorCloseCancel => {
                if self
                    .app
                    .mounted(&Id::ColorEditor(IdColorEditor::ThemeSelect))
                {
                    self.umount_color_editor();
                }
            }
            CEMsg::ColorEditorCloseOk => {
                self.config.style_color_symbol = self.ce_style_color_symbol.clone();
                if self
                    .app
                    .mounted(&Id::ColorEditor(IdColorEditor::ThemeSelect))
                {
                    self.umount_color_editor();
                }
            }
            CEMsg::ThemeSelectLoad(index) => {
                if let Some(t) = self.ce_themes.get(*index) {
                    let path = PathBuf::from(t);
                    if let Some(n) = path.file_stem() {
                        self.config.theme_selected = n.to_string_lossy().to_string();
                        if let Ok(theme) = load_alacritty_theme(t) {
                            self.ce_style_color_symbol.alacritty_theme = theme;
                        }
                    }
                }
                self.umount_color_editor();
                self.mount_color_editor();
            }
            CEMsg::ColorChanged(id, color_config) => {
                self.update_color_editor_color_changed(id, color_config);
            }
            CEMsg::SymbolChanged(id, symbol) => match id {
                IdColorEditor::LibraryHighlightSymbol => {
                    self.ce_style_color_symbol.library_highlight_symbol = symbol.to_string();
                }
                IdColorEditor::PlaylistHighlightSymbol => {
                    self.ce_style_color_symbol.playlist_highlight_symbol = symbol.to_string();
                }
                _ => {}
            },
            CEMsg::HelpPopupShow => self.mount_color_editor_help(),
            CEMsg::HelpPopupClose => {
                if self.app.mounted(&Id::ColorEditor(IdColorEditor::HelpPopup)) {
                    self.app
                        .umount(&Id::ColorEditor(IdColorEditor::HelpPopup))
                        .ok();
                }
            }
        }
    }
    fn update_color_editor_focus(&mut self, msg: &CEMsg) {
        match msg {
            CEMsg::ThemeSelectBlurDown | CEMsg::LibraryForegroundBlurUp => {
                self.app
                    .active(&Id::ColorEditor(IdColorEditor::RadioOk))
                    .ok();
            }
            CEMsg::ColorEditorOkBlurDown | CEMsg::LibraryBackgroundBlurUp => {
                self.app
                    .active(&Id::ColorEditor(IdColorEditor::LibraryForeground))
                    .ok();
            }

            CEMsg::LibraryForegroundBlurDown | CEMsg::LibraryBorderBlurUp => {
                self.app
                    .active(&Id::ColorEditor(IdColorEditor::LibraryBackground))
                    .ok();
            }
            CEMsg::LibraryBackgroundBlurDown | CEMsg::LibraryHighlightBlurUp => {
                self.app
                    .active(&Id::ColorEditor(IdColorEditor::LibraryBorder))
                    .ok();
            }
            CEMsg::LibraryBorderBlurDown | CEMsg::LibraryHighlightSymbolBlurUp => {
                self.app
                    .active(&Id::ColorEditor(IdColorEditor::LibraryHighlight))
                    .ok();
            }
            CEMsg::LibraryHighlightBlurDown | CEMsg::PlaylistForegroundBlurUp => {
                self.app
                    .active(&Id::ColorEditor(IdColorEditor::LibraryHighlightSymbol))
                    .ok();
            }
            CEMsg::LibraryHighlightSymbolBlurDown | CEMsg::PlaylistBackgroundBlurUp => {
                self.app
                    .active(&Id::ColorEditor(IdColorEditor::PlaylistForeground))
                    .ok();
            }
            CEMsg::PlaylistForegroundBlurDown | CEMsg::PlaylistBorderBlurUp => {
                self.app
                    .active(&Id::ColorEditor(IdColorEditor::PlaylistBackground))
                    .ok();
            }
            CEMsg::PlaylistBackgroundBlurDown | CEMsg::PlaylistHighlightBlurUp => {
                self.app
                    .active(&Id::ColorEditor(IdColorEditor::PlaylistBorder))
                    .ok();
            }
            CEMsg::PlaylistBorderBlurDown | CEMsg::PlaylistHighlightSymbolBlurUp => {
                self.app
                    .active(&Id::ColorEditor(IdColorEditor::PlaylistHighlight))
                    .ok();
            }
            CEMsg::PlaylistHighlightBlurDown | CEMsg::ProgressForegroundBlurUp => {
                self.app
                    .active(&Id::ColorEditor(IdColorEditor::PlaylistHighlightSymbol))
                    .ok();
            }
            CEMsg::PlaylistHighlightSymbolBlurDown | CEMsg::ProgressBackgroundBlurUp => {
                self.app
                    .active(&Id::ColorEditor(IdColorEditor::ProgressForeground))
                    .ok();
            }
            CEMsg::ProgressForegroundBlurDown | CEMsg::ProgressBorderBlurUp => {
                self.app
                    .active(&Id::ColorEditor(IdColorEditor::ProgressBackground))
                    .ok();
            }
            CEMsg::ProgressBackgroundBlurDown | CEMsg::LyricForegroundBlurUp => {
                self.app
                    .active(&Id::ColorEditor(IdColorEditor::ProgressBorder))
                    .ok();
            }
            CEMsg::ProgressBorderBlurDown | CEMsg::LyricBackgroundBlurUp => {
                self.app
                    .active(&Id::ColorEditor(IdColorEditor::LyricForeground))
                    .ok();
            }
            CEMsg::LyricForegroundBlurDown | CEMsg::LyricBorderBlurUp => {
                self.app
                    .active(&Id::ColorEditor(IdColorEditor::LyricBackground))
                    .ok();
            }
            CEMsg::LyricBackgroundBlurDown | CEMsg::ThemeSelectBlurUp => {
                self.app
                    .active(&Id::ColorEditor(IdColorEditor::LyricBorder))
                    .ok();
            }
            CEMsg::LyricBorderBlurDown | CEMsg::ColorEditorOkBlurUp => {
                self.app
                    .active(&Id::ColorEditor(IdColorEditor::ThemeSelect))
                    .ok();
            }
            _ => {}
        }
    }
    fn update_color_editor_color_changed(
        &mut self,
        id: &IdColorEditor,
        color_config: &ColorConfig,
    ) {
        match id {
            IdColorEditor::LibraryForeground => {
                self.ce_style_color_symbol.library_foreground = color_config.clone();
            }
            IdColorEditor::LibraryBackground => {
                self.ce_style_color_symbol.library_background = color_config.clone();
            }
            IdColorEditor::LibraryBorder => {
                self.ce_style_color_symbol.library_border = color_config.clone();
            }
            IdColorEditor::LibraryHighlight => {
                self.ce_style_color_symbol.library_highlight = color_config.clone();
            }
            IdColorEditor::PlaylistForeground => {
                self.ce_style_color_symbol.playlist_foreground = color_config.clone();
            }
            IdColorEditor::PlaylistBackground => {
                self.ce_style_color_symbol.playlist_background = color_config.clone();
            }
            IdColorEditor::PlaylistBorder => {
                self.ce_style_color_symbol.playlist_border = color_config.clone();
            }
            IdColorEditor::PlaylistHighlight => {
                self.ce_style_color_symbol.playlist_highlight = color_config.clone();
            }
            IdColorEditor::ProgressForeground => {
                self.ce_style_color_symbol.progress_foreground = color_config.clone();
            }
            IdColorEditor::ProgressBackground => {
                self.ce_style_color_symbol.progress_background = color_config.clone();
            }
            IdColorEditor::ProgressBorder => {
                self.ce_style_color_symbol.progress_border = color_config.clone();
            }
            IdColorEditor::LyricForeground => {
                self.ce_style_color_symbol.lyric_foreground = color_config.clone();
            }
            IdColorEditor::LyricBackground => {
                self.ce_style_color_symbol.lyric_background = color_config.clone();
            }
            IdColorEditor::LyricBorder => {
                self.ce_style_color_symbol.lyric_border = color_config.clone();
            }

            _ => {}
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
                    self.mount_error_popup(format!("download song error: {}", e).as_str());
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
            GSMsg::PopupShowLibrary => {
                self.mount_search_library();
                self.library_update_search("*");
            }
            GSMsg::PopupShowPlaylist => {
                self.mount_search_playlist();
                self.playlist_update_search("*");
            }

            GSMsg::PopupUpdateLibrary(input) => {
                self.library_update_search(input);
            }
            GSMsg::PopupUpdatePlaylist(input) => {
                self.playlist_update_search(input);
            }

            GSMsg::PopupCloseCancel => {
                self.app.umount(&Id::GeneralSearchInput).ok();
                self.app.umount(&Id::GeneralSearchTable).ok();
                self.app.unlock_subs();
            }
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
            GSMsg::PopupCloseLibraryAddPlaylist => {
                self.general_search_after_library_add_playlist();
                // self.app.umount(&Id::GeneralSearchInput).ok();
                // self.app.umount(&Id::GeneralSearchTable).ok();
                // self.app.unlock_subs();
            }
            GSMsg::PopupCloseOkLibraryLocate => {
                self.general_search_after_library_select();
                self.app.umount(&Id::GeneralSearchInput).ok();
                self.app.umount(&Id::GeneralSearchTable).ok();
                self.app.unlock_subs();
            }
            GSMsg::PopupClosePlaylistPlaySelected => {
                self.general_search_after_playlist_play_selected();
                self.app.umount(&Id::GeneralSearchInput).ok();
                self.app.umount(&Id::GeneralSearchTable).ok();
                self.app.unlock_subs();
            }
            GSMsg::PopupCloseOkPlaylistLocate => {
                self.general_search_after_playlist_select();
                self.app.umount(&Id::GeneralSearchInput).ok();
                self.app.umount(&Id::GeneralSearchTable).ok();
                self.app.unlock_subs();
            }
        }
    }
    fn update_delete_confirmation(&mut self, msg: &Msg) {
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
            PLMsg::TableBlur => {
                assert!(self.app.active(&Id::Library).is_ok());
            }
            PLMsg::NextSong => {
                self.player_next();
            }
            PLMsg::PrevSong => {
                self.player_previous();
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
                    self.library_sync(s.file());
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
