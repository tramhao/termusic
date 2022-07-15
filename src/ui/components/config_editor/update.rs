use crate::config::{load_alacritty, BindingForEvent, ColorTermusic};
use crate::ui::components::config_editor::MODIFIER_LIST;
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
use crate::ui::{ConfigEditorMsg, Id, IdConfigEditor, Model, Msg};
use std::path::PathBuf;

use tuirealm::props::{AttrValue, Attribute};
use tuirealm::{
    event::{Key, KeyModifiers},
    State, StateValue,
};

impl Model {
    #[allow(clippy::too_many_lines)]
    pub fn update_config_editor(&mut self, msg: &ConfigEditorMsg) -> Option<Msg> {
        match msg {
            ConfigEditorMsg::Open => {
                self.ce_style_color_symbol = self.config.style_color_symbol.clone();
                self.ke_key_config = self.config.keys.clone();
                self.mount_config_editor();
            }
            ConfigEditorMsg::CloseCancel => {
                self.config_changed = false;
                self.umount_config_editor();
            }
            ConfigEditorMsg::CloseOk => {
                if self.config_changed {
                    self.config_changed = false;
                    self.mount_config_save_popup();
                } else {
                    self.umount_config_editor();
                }
            }
            ConfigEditorMsg::ChangeLayout => self.action_change_layout(),
            ConfigEditorMsg::ConfigChanged => self.config_changed = true,
            ConfigEditorMsg::MusicDirBlurDown | ConfigEditorMsg::PlaylistDisplaySymbolBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::ExitConfirmation))
                    .ok();
            }
            ConfigEditorMsg::ExitConfirmationBlurDown
            | ConfigEditorMsg::PlaylistRandomTrackBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::PlaylistDisplaySymbol))
                    .ok();
            }
            ConfigEditorMsg::AlbumPhotoXBlurUp | ConfigEditorMsg::PlaylistRandomTrackBlurDown => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::PlaylistRandomAlbum))
                    .ok();
            }
            ConfigEditorMsg::PlaylistDisplaySymbolBlurDown
            | ConfigEditorMsg::PlaylistRandomAlbumBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::PlaylistRandomTrack))
                    .ok();
            }
            ConfigEditorMsg::PlaylistRandomAlbumBlurDown | ConfigEditorMsg::AlbumPhotoYBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::AlbumPhotoX))
                    .ok();
            }
            ConfigEditorMsg::AlbumPhotoXBlurDown | ConfigEditorMsg::AlbumPhotoWidthBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::AlbumPhotoY))
                    .ok();
            }
            ConfigEditorMsg::AlbumPhotoYBlurDown | ConfigEditorMsg::AlbumPhotoAlignBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::AlbumPhotoWidth))
                    .ok();
            }
            ConfigEditorMsg::AlbumPhotoWidthBlurDown | ConfigEditorMsg::MusicDirBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::AlbumPhotoAlign))
                    .ok();
            }
            ConfigEditorMsg::AlbumPhotoAlignBlurDown | ConfigEditorMsg::ExitConfirmationBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::MusicDir))
                    .ok();
            }
            ConfigEditorMsg::ConfigSaveOk => {
                self.app
                    .umount(&Id::ConfigEditor(IdConfigEditor::ConfigSavePopup))
                    .ok();
                match self.collect_config_data() {
                    Ok(()) => self.umount_config_editor(),
                    Err(e) => {
                        self.mount_error_popup(format!("save config error: {}", e).as_str());
                    }
                }
            }
            ConfigEditorMsg::ConfigSaveCancel => {
                self.app
                    .umount(&Id::ConfigEditor(IdConfigEditor::ConfigSavePopup))
                    .ok();
                self.umount_config_editor();
            }

            // Focus of color page
            ConfigEditorMsg::ThemeSelectBlurDown | ConfigEditorMsg::LibraryBackgroundBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::LibraryForeground))
                    .ok();
            }
            ConfigEditorMsg::LibraryForegroundBlurUp | ConfigEditorMsg::LyricBorderBlurDown => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::CEThemeSelect))
                    .ok();
            }
            ConfigEditorMsg::LibraryForegroundBlurDown | ConfigEditorMsg::LibraryBorderBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::LibraryBackground))
                    .ok();
            }
            ConfigEditorMsg::LibraryBackgroundBlurDown
            | ConfigEditorMsg::LibraryHighlightBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::LibraryBorder))
                    .ok();
            }
            ConfigEditorMsg::LibraryBorderBlurDown
            | ConfigEditorMsg::LibraryHighlightSymbolBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::LibraryHighlight))
                    .ok();
            }
            ConfigEditorMsg::LibraryHighlightBlurDown
            | ConfigEditorMsg::PlaylistForegroundBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::LibraryHighlightSymbol))
                    .ok();
            }
            ConfigEditorMsg::LibraryHighlightSymbolBlurDown
            | ConfigEditorMsg::PlaylistBackgroundBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::PlaylistForeground))
                    .ok();
            }
            ConfigEditorMsg::PlaylistForegroundBlurDown | ConfigEditorMsg::PlaylistBorderBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::PlaylistBackground))
                    .ok();
            }
            ConfigEditorMsg::PlaylistBackgroundBlurDown
            | ConfigEditorMsg::PlaylistHighlightBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::PlaylistBorder))
                    .ok();
            }
            ConfigEditorMsg::PlaylistBorderBlurDown
            | ConfigEditorMsg::PlaylistHighlightSymbolBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::PlaylistHighlight))
                    .ok();
            }
            ConfigEditorMsg::PlaylistHighlightBlurDown
            | ConfigEditorMsg::ProgressForegroundBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::PlaylistHighlightSymbol))
                    .ok();
            }
            ConfigEditorMsg::PlaylistHighlightSymbolBlurDown
            | ConfigEditorMsg::ProgressBackgroundBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::ProgressForeground))
                    .ok();
            }
            ConfigEditorMsg::ProgressForegroundBlurDown | ConfigEditorMsg::ProgressBorderBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::ProgressBackground))
                    .ok();
            }
            ConfigEditorMsg::ProgressBackgroundBlurDown
            | ConfigEditorMsg::LyricForegroundBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::ProgressBorder))
                    .ok();
            }
            ConfigEditorMsg::ProgressBorderBlurDown | ConfigEditorMsg::LyricBackgroundBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::LyricForeground))
                    .ok();
            }
            ConfigEditorMsg::LyricForegroundBlurDown | ConfigEditorMsg::LyricBorderBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::LyricBackground))
                    .ok();
            }
            ConfigEditorMsg::LyricBackgroundBlurDown | ConfigEditorMsg::ThemeSelectBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::LyricBorder))
                    .ok();
            }
            ConfigEditorMsg::ThemeSelectLoad(index) => {
                if let Some(t) = self.ce_themes.get(*index) {
                    let path = PathBuf::from(t);
                    if let Some(n) = path.file_stem() {
                        self.config.theme_selected = n.to_string_lossy().to_string();
                        if let Ok(theme) = load_alacritty(t) {
                            self.ce_style_color_symbol.alacritty_theme = theme;
                        }
                    }
                }
                self.config_changed = true;
                let mut config = self.config.clone();
                // This is for preview the theme colors
                config.style_color_symbol = self.ce_style_color_symbol.clone();
                self.remount_config_color(&config);
            }
            ConfigEditorMsg::ColorChanged(id, color_config) => {
                self.config_changed = true;
                self.update_config_editor_color_changed(id, *color_config);
            }
            ConfigEditorMsg::SymbolChanged(id, symbol) => {
                self.config_changed = true;

                match id {
                    IdConfigEditor::LibraryHighlightSymbol => {
                        self.ce_style_color_symbol.library_highlight_symbol = symbol.to_string();
                    }
                    IdConfigEditor::PlaylistHighlightSymbol => {
                        self.ce_style_color_symbol.playlist_highlight_symbol = symbol.to_string();
                    }
                    _ => {}
                };
            }

            ConfigEditorMsg::KeyChanged(id) => {
                self.config_changed = true;
                self.update_config_editor_key_changed(id);
            }
            // Focus of key global page
            ConfigEditorMsg::GlobalConfigInputBlurDown | ConfigEditorMsg::GlobalQuitInputBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::GlobalQuit))
                    .ok();
            }
            ConfigEditorMsg::GlobalQuitBlurDown | ConfigEditorMsg::GlobalLeftBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::GlobalQuitInput))
                    .ok();
            }
            ConfigEditorMsg::GlobalQuitInputBlurDown | ConfigEditorMsg::GlobalLeftInputBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::GlobalLeft))
                    .ok();
            }

            ConfigEditorMsg::GlobalLeftBlurDown | ConfigEditorMsg::GlobalDownBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::GlobalLeftInput))
                    .ok();
            }
            ConfigEditorMsg::GlobalLeftInputBlurDown | ConfigEditorMsg::GlobalDownInputBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::GlobalDown))
                    .ok();
            }

            ConfigEditorMsg::GlobalDownBlurDown | ConfigEditorMsg::GlobalUpBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::GlobalDownInput))
                    .ok();
            }

            ConfigEditorMsg::GlobalDownInputBlurDown | ConfigEditorMsg::GlobalUpInputBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::GlobalUp))
                    .ok();
            }

            ConfigEditorMsg::GlobalUpBlurDown | ConfigEditorMsg::GlobalRightBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::GlobalUpInput))
                    .ok();
            }

            ConfigEditorMsg::GlobalUpInputBlurDown | ConfigEditorMsg::GlobalRightInputBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::GlobalRight))
                    .ok();
            }

            ConfigEditorMsg::GlobalRightBlurDown | ConfigEditorMsg::GlobalGotoTopBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::GlobalRightInput))
                    .ok();
            }
            ConfigEditorMsg::GlobalRightInputBlurDown
            | ConfigEditorMsg::GlobalGotoTopInputBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::GlobalGotoTop))
                    .ok();
            }
            ConfigEditorMsg::GlobalGotoTopBlurDown | ConfigEditorMsg::GlobalGotoBottomBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::GlobalGotoTopInput))
                    .ok();
            }
            ConfigEditorMsg::GlobalGotoTopInputBlurDown
            | ConfigEditorMsg::GlobalGotoBottomInputBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::GlobalGotoBottom))
                    .ok();
            }
            ConfigEditorMsg::GlobalGotoBottomBlurDown
            | ConfigEditorMsg::GlobalPlayerTogglePauseBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::GlobalGotoBottomInput))
                    .ok();
            }
            ConfigEditorMsg::GlobalGotoBottomInputBlurDown
            | ConfigEditorMsg::GlobalPlayerTogglePauseInputBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::GlobalPlayerTogglePause))
                    .ok();
            }
            ConfigEditorMsg::GlobalPlayerTogglePauseBlurDown
            | ConfigEditorMsg::GlobalPlayerNextBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(
                        IdConfigEditor::GlobalPlayerTogglePauseInput,
                    ))
                    .ok();
            }
            ConfigEditorMsg::GlobalPlayerTogglePauseInputBlurDown
            | ConfigEditorMsg::GlobalPlayerNextInputBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::GlobalPlayerNext))
                    .ok();
            }
            ConfigEditorMsg::GlobalPlayerNextBlurDown
            | ConfigEditorMsg::GlobalPlayerPreviousBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::GlobalPlayerNextInput))
                    .ok();
            }
            ConfigEditorMsg::GlobalPlayerNextInputBlurDown
            | ConfigEditorMsg::GlobalPlayerPreviousInputBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::GlobalPlayerPrevious))
                    .ok();
            }
            ConfigEditorMsg::GlobalPlayerPreviousBlurDown | ConfigEditorMsg::GlobalHelpBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::GlobalPlayerPreviousInput))
                    .ok();
            }

            ConfigEditorMsg::GlobalPlayerPreviousInputBlurDown
            | ConfigEditorMsg::GlobalHelpInputBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::GlobalHelp))
                    .ok();
            }
            ConfigEditorMsg::GlobalHelpBlurDown | ConfigEditorMsg::GlobalVolumeUpBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::GlobalHelpInput))
                    .ok();
            }

            ConfigEditorMsg::GlobalHelpInputBlurDown
            | ConfigEditorMsg::GlobalVolumeUpInputBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::GlobalVolumeUp))
                    .ok();
            }
            ConfigEditorMsg::GlobalVolumeUpBlurDown | ConfigEditorMsg::GlobalVolumeDownBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::GlobalVolumeUpInput))
                    .ok();
            }

            ConfigEditorMsg::GlobalVolumeUpInputBlurDown
            | ConfigEditorMsg::GlobalVolumeDownInputBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::GlobalVolumeDown))
                    .ok();
            }
            ConfigEditorMsg::GlobalVolumeDownBlurDown
            | ConfigEditorMsg::GlobalPlayerSeekForwardBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::GlobalVolumeDownInput))
                    .ok();
            }

            ConfigEditorMsg::GlobalVolumeDownInputBlurDown
            | ConfigEditorMsg::GlobalPlayerSeekForwardInputBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::GlobalPlayerSeekForward))
                    .ok();
            }

            ConfigEditorMsg::GlobalPlayerSeekForwardBlurDown
            | ConfigEditorMsg::GlobalPlayerSeekBackwardBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(
                        IdConfigEditor::GlobalPlayerSeekForwardInput,
                    ))
                    .ok();
            }

            ConfigEditorMsg::GlobalPlayerSeekForwardInputBlurDown
            | ConfigEditorMsg::GlobalPlayerSeekBackwardInputBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::GlobalPlayerSeekBackward))
                    .ok();
            }

            ConfigEditorMsg::GlobalPlayerSeekBackwardBlurDown
            | ConfigEditorMsg::GlobalPlayerSpeedUpBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(
                        IdConfigEditor::GlobalPlayerSeekBackwardInput,
                    ))
                    .ok();
            }

            ConfigEditorMsg::GlobalPlayerSeekBackwardInputBlurDown
            | ConfigEditorMsg::GlobalPlayerSpeedUpInputBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::GlobalPlayerSpeedUp))
                    .ok();
            }

            ConfigEditorMsg::GlobalPlayerSpeedUpBlurDown
            | ConfigEditorMsg::GlobalPlayerSpeedDownBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::GlobalPlayerSpeedUpInput))
                    .ok();
            }
            ConfigEditorMsg::GlobalPlayerSpeedUpInputBlurDown
            | ConfigEditorMsg::GlobalPlayerSpeedDownInputBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::GlobalPlayerSpeedDown))
                    .ok();
            }
            ConfigEditorMsg::GlobalPlayerSpeedDownBlurDown
            | ConfigEditorMsg::GlobalLyricAdjustForwardBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(
                        IdConfigEditor::GlobalPlayerSpeedDownInput,
                    ))
                    .ok();
            }

            ConfigEditorMsg::GlobalPlayerSpeedDownInputBlurDown
            | ConfigEditorMsg::GlobalLyricAdjustForwardInputBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::GlobalLyricAdjustForward))
                    .ok();
            }

            ConfigEditorMsg::GlobalLyricAdjustForwardBlurDown
            | ConfigEditorMsg::GlobalLyricAdjustBackwardBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(
                        IdConfigEditor::GlobalLyricAdjustForwardInput,
                    ))
                    .ok();
            }

            ConfigEditorMsg::GlobalLyricAdjustForwardInputBlurDown
            | ConfigEditorMsg::GlobalLyricAdjustBackwardInputBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::GlobalLyricAdjustBackward))
                    .ok();
            }

            ConfigEditorMsg::GlobalLyricAdjustBackwardBlurDown
            | ConfigEditorMsg::GlobalLyricCyleBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(
                        IdConfigEditor::GlobalLyricAdjustBackwardInput,
                    ))
                    .ok();
            }

            ConfigEditorMsg::GlobalLyricAdjustBackwardInputBlurDown
            | ConfigEditorMsg::GlobalLyricCyleInputBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::GlobalLyricCycle))
                    .ok();
            }

            ConfigEditorMsg::GlobalLyricCyleBlurDown
            | ConfigEditorMsg::GlobalLayoutTreeviewBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::GlobalLyricCycleInput))
                    .ok();
            }

            ConfigEditorMsg::GlobalLyricCyleInputBlurDown
            | ConfigEditorMsg::GlobalLayoutTreeviewInputBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::GlobalLayoutTreeview))
                    .ok();
            }

            ConfigEditorMsg::GlobalLayoutTreeviewBlurDown
            | ConfigEditorMsg::GlobalLayoutDatabaseBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::GlobalLayoutTreeviewInput))
                    .ok();
            }

            ConfigEditorMsg::GlobalLayoutTreeviewInputBlurDown
            | ConfigEditorMsg::GlobalLayoutDatabaseInputBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::GlobalLayoutDatabase))
                    .ok();
            }

            ConfigEditorMsg::GlobalLayoutDatabaseBlurDown
            | ConfigEditorMsg::GlobalPlayerToggleGaplessBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::GlobalLayoutDatabaseInput))
                    .ok();
            }

            ConfigEditorMsg::GlobalLayoutDatabaseInputBlurDown
            | ConfigEditorMsg::GlobalPlayerToggleGaplessInputBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::GlobalPlayerToggleGapless))
                    .ok();
            }

            ConfigEditorMsg::GlobalPlayerToggleGaplessBlurDown
            | ConfigEditorMsg::GlobalConfigBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(
                        IdConfigEditor::GlobalPlayerToggleGaplessInput,
                    ))
                    .ok();
            }

            ConfigEditorMsg::GlobalPlayerToggleGaplessInputBlurDown
            | ConfigEditorMsg::GlobalConfigInputBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::GlobalConfig))
                    .ok();
            }

            ConfigEditorMsg::GlobalConfigBlurDown | ConfigEditorMsg::GlobalQuitBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::GlobalConfigInput))
                    .ok();
            }

            // Focus of key 2 page
            ConfigEditorMsg::DatabaseAddAllInputBlurDown
            | ConfigEditorMsg::LibraryTagEditorInputBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::LibraryTagEditor))
                    .ok();
            }

            ConfigEditorMsg::LibraryTagEditorBlurDown | ConfigEditorMsg::LibraryDeleteBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::LibraryTagEditorInput))
                    .ok();
            }

            ConfigEditorMsg::LibraryTagEditorInputBlurDown
            | ConfigEditorMsg::LibraryDeleteInputBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::LibraryDelete))
                    .ok();
            }

            ConfigEditorMsg::LibraryDeleteBlurDown | ConfigEditorMsg::LibraryLoadDirBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::LibraryDeleteInput))
                    .ok();
            }
            ConfigEditorMsg::LibraryDeleteInputBlurDown
            | ConfigEditorMsg::LibraryLoadDirInputBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::LibraryLoadDir))
                    .ok();
            }

            ConfigEditorMsg::LibraryLoadDirBlurDown | ConfigEditorMsg::LibraryYankBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::LibraryLoadDirInput))
                    .ok();
            }
            ConfigEditorMsg::LibraryLoadDirInputBlurDown
            | ConfigEditorMsg::LibraryYankInputBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::LibraryYank))
                    .ok();
            }

            ConfigEditorMsg::LibraryYankBlurDown | ConfigEditorMsg::LibraryPasteBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::LibraryYankInput))
                    .ok();
            }

            ConfigEditorMsg::LibraryYankInputBlurDown
            | ConfigEditorMsg::LibraryPasteInputBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::LibraryPaste))
                    .ok();
            }

            ConfigEditorMsg::LibraryPasteBlurDown | ConfigEditorMsg::LibrarySearchBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::LibraryPasteInput))
                    .ok();
            }
            ConfigEditorMsg::LibraryPasteInputBlurDown
            | ConfigEditorMsg::LibrarySearchInputBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::LibrarySearch))
                    .ok();
            }

            ConfigEditorMsg::LibrarySearchBlurDown
            | ConfigEditorMsg::LibrarySearchYoutubeBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::LibrarySearchInput))
                    .ok();
            }
            ConfigEditorMsg::LibrarySearchInputBlurDown
            | ConfigEditorMsg::LibrarySearchYoutubeInputBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::LibrarySearchYoutube))
                    .ok();
            }

            ConfigEditorMsg::LibrarySearchYoutubeBlurDown
            | ConfigEditorMsg::PlaylistDeleteBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::LibrarySearchYoutubeInput))
                    .ok();
            }

            ConfigEditorMsg::LibrarySearchYoutubeInputBlurDown
            | ConfigEditorMsg::PlaylistDeleteInputBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::PlaylistDelete))
                    .ok();
            }

            ConfigEditorMsg::PlaylistDeleteBlurDown | ConfigEditorMsg::PlaylistDeleteAllBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::PlaylistDeleteInput))
                    .ok();
            }
            ConfigEditorMsg::PlaylistDeleteInputBlurDown
            | ConfigEditorMsg::PlaylistDeleteAllInputBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::PlaylistDeleteAll))
                    .ok();
            }

            ConfigEditorMsg::PlaylistDeleteAllBlurDown | ConfigEditorMsg::PlaylistSearchBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::PlaylistDeleteAllInput))
                    .ok();
            }

            ConfigEditorMsg::PlaylistDeleteAllInputBlurDown
            | ConfigEditorMsg::PlaylistSearchInputBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::PlaylistSearch))
                    .ok();
            }

            ConfigEditorMsg::PlaylistSearchBlurDown | ConfigEditorMsg::PlaylistShuffleBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::PlaylistSearchInput))
                    .ok();
            }
            ConfigEditorMsg::PlaylistSearchInputBlurDown
            | ConfigEditorMsg::PlaylistShuffleInputBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::PlaylistShuffle))
                    .ok();
            }

            ConfigEditorMsg::PlaylistShuffleBlurDown | ConfigEditorMsg::PlaylistAddFrontBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::PlaylistShuffleInput))
                    .ok();
            }

            ConfigEditorMsg::PlaylistShuffleInputBlurDown
            | ConfigEditorMsg::PlaylistAddFrontInputBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::PlaylistAddFront))
                    .ok();
            }

            ConfigEditorMsg::PlaylistAddFrontBlurDown
            | ConfigEditorMsg::PlaylistModeCycleBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::PlaylistAddFrontInput))
                    .ok();
            }
            ConfigEditorMsg::PlaylistAddFrontInputBlurDown
            | ConfigEditorMsg::PlaylistModeCycleInputBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::PlaylistModeCycle))
                    .ok();
            }

            ConfigEditorMsg::PlaylistModeCycleBlurDown
            | ConfigEditorMsg::PlaylistPlaySelectedBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::PlaylistModeCycleInput))
                    .ok();
            }

            ConfigEditorMsg::PlaylistModeCycleInputBlurDown
            | ConfigEditorMsg::PlaylistPlaySelectedInputBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::PlaylistPlaySelected))
                    .ok();
            }

            ConfigEditorMsg::PlaylistPlaySelectedBlurDown
            | ConfigEditorMsg::PlaylistSwapDownBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::PlaylistPlaySelectedInput))
                    .ok();
            }

            ConfigEditorMsg::PlaylistPlaySelectedInputBlurDown
            | ConfigEditorMsg::PlaylistSwapDownInputBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::PlaylistSwapDown))
                    .ok();
            }

            ConfigEditorMsg::PlaylistSwapDownBlurDown | ConfigEditorMsg::PlaylistSwapUpBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::PlaylistSwapDownInput))
                    .ok();
            }

            ConfigEditorMsg::PlaylistSwapDownInputBlurDown
            | ConfigEditorMsg::PlaylistSwapUpInputBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::PlaylistSwapUp))
                    .ok();
            }

            ConfigEditorMsg::PlaylistSwapUpBlurDown | ConfigEditorMsg::DatabaseAddAllBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::PlaylistSwapUpInput))
                    .ok();
            }

            ConfigEditorMsg::PlaylistSwapUpInputBlurDown
            | ConfigEditorMsg::DatabaseAddAllInputBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::DatabaseAddAll))
                    .ok();
            }

            ConfigEditorMsg::DatabaseAddAllBlurDown | ConfigEditorMsg::LibraryTagEditorBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::DatabaseAddAllInput))
                    .ok();
            }
        }
        None
    }

    fn update_config_editor_color_changed(
        &mut self,
        id: &IdConfigEditor,
        color_config: ColorTermusic,
    ) {
        match id {
            IdConfigEditor::LibraryForeground => {
                self.ce_style_color_symbol.library_foreground = color_config;
            }
            IdConfigEditor::LibraryBackground => {
                self.ce_style_color_symbol.library_background = color_config;
            }
            IdConfigEditor::LibraryBorder => {
                self.ce_style_color_symbol.library_border = color_config;
            }
            IdConfigEditor::LibraryHighlight => {
                self.ce_style_color_symbol.library_highlight = color_config;
            }
            IdConfigEditor::PlaylistForeground => {
                self.ce_style_color_symbol.playlist_foreground = color_config;
            }
            IdConfigEditor::PlaylistBackground => {
                self.ce_style_color_symbol.playlist_background = color_config;
            }
            IdConfigEditor::PlaylistBorder => {
                self.ce_style_color_symbol.playlist_border = color_config;
            }
            IdConfigEditor::PlaylistHighlight => {
                self.ce_style_color_symbol.playlist_highlight = color_config;
            }
            IdConfigEditor::ProgressForeground => {
                self.ce_style_color_symbol.progress_foreground = color_config;
            }
            IdConfigEditor::ProgressBackground => {
                self.ce_style_color_symbol.progress_background = color_config;
            }
            IdConfigEditor::ProgressBorder => {
                self.ce_style_color_symbol.progress_border = color_config;
            }
            IdConfigEditor::LyricForeground => {
                self.ce_style_color_symbol.lyric_foreground = color_config;
            }
            IdConfigEditor::LyricBackground => {
                self.ce_style_color_symbol.lyric_background = color_config;
            }
            IdConfigEditor::LyricBorder => {
                self.ce_style_color_symbol.lyric_border = color_config;
            }

            _ => {}
        }
    }

    #[allow(clippy::too_many_lines)]
    pub fn update_config_editor_key_changed(&mut self, id: &IdConfigEditor) {
        match id {
            IdConfigEditor::GlobalQuit | IdConfigEditor::GlobalQuitInput => {
                self.ke_key_config.global_quit = self.extract_key_mod_and_code(
                    IdConfigEditor::GlobalQuit,
                    IdConfigEditor::GlobalQuitInput,
                );
            }
            IdConfigEditor::GlobalLeft | IdConfigEditor::GlobalLeftInput => {
                self.ke_key_config.global_left = self.extract_key_mod_and_code(
                    IdConfigEditor::GlobalLeft,
                    IdConfigEditor::GlobalLeftInput,
                );
            }
            IdConfigEditor::GlobalRight | IdConfigEditor::GlobalRightInput => {
                self.ke_key_config.global_right = self.extract_key_mod_and_code(
                    IdConfigEditor::GlobalRight,
                    IdConfigEditor::GlobalRightInput,
                );
            }
            IdConfigEditor::GlobalUp | IdConfigEditor::GlobalUpInput => {
                self.ke_key_config.global_up = self.extract_key_mod_and_code(
                    IdConfigEditor::GlobalUp,
                    IdConfigEditor::GlobalUpInput,
                );
            }

            IdConfigEditor::GlobalDown | IdConfigEditor::GlobalDownInput => {
                self.ke_key_config.global_down = self.extract_key_mod_and_code(
                    IdConfigEditor::GlobalDown,
                    IdConfigEditor::GlobalDownInput,
                );
            }
            IdConfigEditor::GlobalGotoTop | IdConfigEditor::GlobalGotoTopInput => {
                self.ke_key_config.global_goto_top = self.extract_key_mod_and_code(
                    IdConfigEditor::GlobalGotoTop,
                    IdConfigEditor::GlobalGotoTopInput,
                );
            }
            IdConfigEditor::GlobalGotoBottom | IdConfigEditor::GlobalGotoBottomInput => {
                self.ke_key_config.global_goto_bottom = self.extract_key_mod_and_code(
                    IdConfigEditor::GlobalGotoBottom,
                    IdConfigEditor::GlobalGotoBottomInput,
                );
            }
            IdConfigEditor::GlobalPlayerTogglePause
            | IdConfigEditor::GlobalPlayerTogglePauseInput => {
                self.ke_key_config.global_player_toggle_pause = self.extract_key_mod_and_code(
                    IdConfigEditor::GlobalPlayerTogglePause,
                    IdConfigEditor::GlobalPlayerTogglePauseInput,
                );
            }
            IdConfigEditor::GlobalPlayerNext | IdConfigEditor::GlobalPlayerNextInput => {
                self.ke_key_config.global_player_next = self.extract_key_mod_and_code(
                    IdConfigEditor::GlobalPlayerNext,
                    IdConfigEditor::GlobalPlayerNextInput,
                );
            }
            IdConfigEditor::GlobalPlayerPrevious | IdConfigEditor::GlobalPlayerPreviousInput => {
                self.ke_key_config.global_player_previous = self.extract_key_mod_and_code(
                    IdConfigEditor::GlobalPlayerPrevious,
                    IdConfigEditor::GlobalPlayerPreviousInput,
                );
            }

            IdConfigEditor::GlobalHelp | IdConfigEditor::GlobalHelpInput => {
                self.ke_key_config.global_help = self.extract_key_mod_and_code(
                    IdConfigEditor::GlobalHelp,
                    IdConfigEditor::GlobalHelpInput,
                );
            }
            IdConfigEditor::GlobalVolumeUp | IdConfigEditor::GlobalVolumeUpInput => {
                self.ke_key_config.global_player_volume_plus_2 = self.extract_key_mod_and_code(
                    IdConfigEditor::GlobalVolumeUp,
                    IdConfigEditor::GlobalVolumeUpInput,
                );
            }
            IdConfigEditor::GlobalVolumeDown | IdConfigEditor::GlobalVolumeDownInput => {
                self.ke_key_config.global_player_volume_minus_2 = self.extract_key_mod_and_code(
                    IdConfigEditor::GlobalVolumeDown,
                    IdConfigEditor::GlobalVolumeDownInput,
                );
            }

            IdConfigEditor::GlobalPlayerSeekForward
            | IdConfigEditor::GlobalPlayerSeekForwardInput => {
                self.ke_key_config.global_player_seek_forward = self.extract_key_mod_and_code(
                    IdConfigEditor::GlobalPlayerSeekForward,
                    IdConfigEditor::GlobalPlayerSeekForwardInput,
                );
            }

            IdConfigEditor::GlobalPlayerSeekBackward
            | IdConfigEditor::GlobalPlayerSeekBackwardInput => {
                self.ke_key_config.global_player_seek_backward = self.extract_key_mod_and_code(
                    IdConfigEditor::GlobalPlayerSeekBackward,
                    IdConfigEditor::GlobalPlayerSeekBackwardInput,
                );
            }

            IdConfigEditor::GlobalPlayerSpeedUp | IdConfigEditor::GlobalPlayerSpeedUpInput => {
                self.ke_key_config.global_player_speed_up = self.extract_key_mod_and_code(
                    IdConfigEditor::GlobalPlayerSpeedUp,
                    IdConfigEditor::GlobalPlayerSpeedUpInput,
                );
            }

            IdConfigEditor::GlobalPlayerSpeedDown | IdConfigEditor::GlobalPlayerSpeedDownInput => {
                self.ke_key_config.global_player_speed_down = self.extract_key_mod_and_code(
                    IdConfigEditor::GlobalPlayerSpeedDown,
                    IdConfigEditor::GlobalPlayerSpeedDownInput,
                );
            }

            IdConfigEditor::GlobalLyricAdjustForward
            | IdConfigEditor::GlobalLyricAdjustForwardInput => {
                self.ke_key_config.global_lyric_adjust_forward = self.extract_key_mod_and_code(
                    IdConfigEditor::GlobalLyricAdjustForward,
                    IdConfigEditor::GlobalLyricAdjustForwardInput,
                );
            }

            IdConfigEditor::GlobalLyricAdjustBackward
            | IdConfigEditor::GlobalLyricAdjustBackwardInput => {
                self.ke_key_config.global_lyric_adjust_backward = self.extract_key_mod_and_code(
                    IdConfigEditor::GlobalLyricAdjustBackward,
                    IdConfigEditor::GlobalLyricAdjustBackwardInput,
                );
            }

            IdConfigEditor::GlobalLyricCycle | IdConfigEditor::GlobalLyricCycleInput => {
                self.ke_key_config.global_lyric_cycle = self.extract_key_mod_and_code(
                    IdConfigEditor::GlobalLyricCycle,
                    IdConfigEditor::GlobalLyricCycleInput,
                );
            }
            IdConfigEditor::LibraryDelete | IdConfigEditor::LibraryDeleteInput => {
                self.ke_key_config.library_delete = self.extract_key_mod_and_code(
                    IdConfigEditor::LibraryDelete,
                    IdConfigEditor::LibraryDeleteInput,
                );
            }
            IdConfigEditor::LibraryLoadDir | IdConfigEditor::LibraryLoadDirInput => {
                self.ke_key_config.library_load_dir = self.extract_key_mod_and_code(
                    IdConfigEditor::LibraryLoadDir,
                    IdConfigEditor::LibraryLoadDirInput,
                );
            }
            IdConfigEditor::LibraryYank | IdConfigEditor::LibraryYankInput => {
                self.ke_key_config.library_yank = self.extract_key_mod_and_code(
                    IdConfigEditor::LibraryYank,
                    IdConfigEditor::LibraryYankInput,
                );
            }

            IdConfigEditor::LibraryPaste | IdConfigEditor::LibraryPasteInput => {
                self.ke_key_config.library_paste = self.extract_key_mod_and_code(
                    IdConfigEditor::LibraryPaste,
                    IdConfigEditor::LibraryPasteInput,
                );
            }

            IdConfigEditor::LibrarySearch | IdConfigEditor::LibrarySearchInput => {
                self.ke_key_config.library_search = self.extract_key_mod_and_code(
                    IdConfigEditor::LibrarySearch,
                    IdConfigEditor::LibrarySearchInput,
                );
            }
            IdConfigEditor::LibrarySearchYoutube | IdConfigEditor::LibrarySearchYoutubeInput => {
                self.ke_key_config.library_search_youtube = self.extract_key_mod_and_code(
                    IdConfigEditor::LibrarySearchYoutube,
                    IdConfigEditor::LibrarySearchYoutubeInput,
                );
            }

            IdConfigEditor::LibraryTagEditor | IdConfigEditor::LibraryTagEditorInput => {
                self.ke_key_config.library_tag_editor_open = self.extract_key_mod_and_code(
                    IdConfigEditor::LibraryTagEditor,
                    IdConfigEditor::LibraryTagEditorInput,
                );
            }
            IdConfigEditor::PlaylistDelete | IdConfigEditor::PlaylistDeleteInput => {
                self.ke_key_config.playlist_delete = self.extract_key_mod_and_code(
                    IdConfigEditor::PlaylistDelete,
                    IdConfigEditor::PlaylistDeleteInput,
                );
            }
            IdConfigEditor::PlaylistDeleteAll | IdConfigEditor::PlaylistDeleteAllInput => {
                self.ke_key_config.playlist_delete_all = self.extract_key_mod_and_code(
                    IdConfigEditor::PlaylistDeleteAll,
                    IdConfigEditor::PlaylistDeleteAllInput,
                );
            }
            IdConfigEditor::PlaylistShuffle | IdConfigEditor::PlaylistShuffleInput => {
                self.ke_key_config.playlist_shuffle = self.extract_key_mod_and_code(
                    IdConfigEditor::PlaylistShuffle,
                    IdConfigEditor::PlaylistShuffleInput,
                );
            }
            IdConfigEditor::PlaylistModeCycle | IdConfigEditor::PlaylistModeCycleInput => {
                self.ke_key_config.playlist_mode_cycle = self.extract_key_mod_and_code(
                    IdConfigEditor::PlaylistModeCycle,
                    IdConfigEditor::PlaylistModeCycleInput,
                );
            }
            IdConfigEditor::PlaylistPlaySelected | IdConfigEditor::PlaylistPlaySelectedInput => {
                self.ke_key_config.playlist_play_selected = self.extract_key_mod_and_code(
                    IdConfigEditor::PlaylistPlaySelected,
                    IdConfigEditor::PlaylistPlaySelectedInput,
                );
            }

            IdConfigEditor::PlaylistAddFront | IdConfigEditor::PlaylistAddFrontInput => {
                self.ke_key_config.playlist_add_front = self.extract_key_mod_and_code(
                    IdConfigEditor::PlaylistAddFront,
                    IdConfigEditor::PlaylistAddFrontInput,
                );
            }

            IdConfigEditor::PlaylistSearch | IdConfigEditor::PlaylistSearchInput => {
                self.ke_key_config.playlist_search = self.extract_key_mod_and_code(
                    IdConfigEditor::PlaylistSearch,
                    IdConfigEditor::PlaylistSearchInput,
                );
            }

            IdConfigEditor::PlaylistSwapDown | IdConfigEditor::PlaylistSwapDownInput => {
                self.ke_key_config.playlist_swap_down = self.extract_key_mod_and_code(
                    IdConfigEditor::PlaylistSwapDown,
                    IdConfigEditor::PlaylistSwapDownInput,
                );
            }

            IdConfigEditor::PlaylistSwapUp | IdConfigEditor::PlaylistSwapUpInput => {
                self.ke_key_config.playlist_swap_up = self.extract_key_mod_and_code(
                    IdConfigEditor::PlaylistSwapUp,
                    IdConfigEditor::PlaylistSwapUpInput,
                );
            }

            IdConfigEditor::GlobalLayoutTreeview | IdConfigEditor::GlobalLayoutTreeviewInput => {
                self.ke_key_config.global_layout_treeview = self.extract_key_mod_and_code(
                    IdConfigEditor::GlobalLayoutTreeview,
                    IdConfigEditor::GlobalLayoutTreeviewInput,
                );
            }

            IdConfigEditor::GlobalLayoutDatabase | IdConfigEditor::GlobalLayoutDatabaseInput => {
                self.ke_key_config.global_layout_database = self.extract_key_mod_and_code(
                    IdConfigEditor::GlobalLayoutDatabase,
                    IdConfigEditor::GlobalLayoutDatabaseInput,
                );
            }

            IdConfigEditor::DatabaseAddAll | IdConfigEditor::DatabaseAddAllInput => {
                self.ke_key_config.database_add_all = self.extract_key_mod_and_code(
                    IdConfigEditor::DatabaseAddAll,
                    IdConfigEditor::DatabaseAddAllInput,
                );
            }

            IdConfigEditor::GlobalPlayerToggleGapless
            | IdConfigEditor::GlobalPlayerToggleGaplessInput => {
                self.ke_key_config.global_player_toggle_gapless = self.extract_key_mod_and_code(
                    IdConfigEditor::GlobalPlayerToggleGapless,
                    IdConfigEditor::GlobalPlayerToggleGaplessInput,
                );
            }
            IdConfigEditor::GlobalConfig | IdConfigEditor::GlobalConfigInput => {
                self.ke_key_config.global_config_open = self.extract_key_mod_and_code(
                    IdConfigEditor::GlobalConfig,
                    IdConfigEditor::GlobalConfigInput,
                );
            }
            _ => {}
        }
    }

    fn extract_key_mod_and_code(
        &mut self,
        id_select: IdConfigEditor,
        id_input: IdConfigEditor,
    ) -> BindingForEvent {
        let mut code = Key::Null;
        let mut modifier = KeyModifiers::CONTROL;
        self.update_key_input_by_modifier(id_select.clone(), id_input.clone());
        if let Ok(State::One(StateValue::Usize(index))) =
            self.app.state(&Id::ConfigEditor(id_select))
        {
            modifier = MODIFIER_LIST[index].modifier();
            if let Ok(State::One(StateValue::String(codes))) =
                self.app.state(&Id::ConfigEditor(id_input))
            {
                if let Ok(c) = BindingForEvent::key_from_str(&codes) {
                    code = c;
                }
            }
        }

        BindingForEvent { code, modifier }
    }
    fn update_key_input_by_modifier(
        &mut self,
        id_select: IdConfigEditor,
        id_input: IdConfigEditor,
    ) {
        if let Ok(State::One(StateValue::Usize(index))) =
            self.app.state(&Id::ConfigEditor(id_select))
        {
            let modifier = MODIFIER_LIST[index].modifier();
            if let Ok(State::One(StateValue::String(codes))) =
                self.app.state(&Id::ConfigEditor(id_input.clone()))
            {
                // For Function keys, no need to change case
                if codes.starts_with('F') {
                    return;
                }

                // For other keys, if shift is in modifier, change case accordingly
                if modifier.bits() % 2 == 1 {
                    self.app
                        .attr(
                            &Id::ConfigEditor(id_input),
                            Attribute::Value,
                            AttrValue::String(codes.to_uppercase()),
                        )
                        .ok();
                } else {
                    self.app
                        .attr(
                            &Id::ConfigEditor(id_input),
                            Attribute::Value,
                            AttrValue::String(codes.to_lowercase()),
                        )
                        .ok();
                }
            }
        }
    }
}
