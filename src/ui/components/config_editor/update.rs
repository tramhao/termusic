use crate::config::load_alacritty;
use crate::config::ColorTermusic;
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

impl Model {
    #[allow(clippy::too_many_lines)]
    pub fn update_config_editor(&mut self, msg: &ConfigEditorMsg) -> Option<Msg> {
        match msg {
            ConfigEditorMsg::Open => {
                self.ce_style_color_symbol = self.config.style_color_symbol.clone();
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
                self.collect_config_data();
                self.app
                    .umount(&Id::ConfigEditor(IdConfigEditor::ConfigSavePopup))
                    .ok();
                self.umount_config_editor();
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

            ConfigEditorMsg::KeyChanged(_) => todo!(),
            // Focus of key 1 page
            ConfigEditorMsg::GlobalQuitBlurDown => todo!(),
            ConfigEditorMsg::GlobalQuitBlurUp => todo!(),
            ConfigEditorMsg::GlobalQuitInputBlurDown => todo!(),
            ConfigEditorMsg::GlobalQuitInputBlurUp => todo!(),
            ConfigEditorMsg::GlobalDownBlurDown => todo!(),
            ConfigEditorMsg::GlobalDownBlurUp => todo!(),
            ConfigEditorMsg::GlobalDownInputBlurDown => todo!(),
            ConfigEditorMsg::GlobalDownInputBlurUp => todo!(),
            ConfigEditorMsg::GlobalGotoBottomBlurDown => todo!(),
            ConfigEditorMsg::GlobalGotoBottomBlurUp => todo!(),
            ConfigEditorMsg::GlobalGotoBottomInputBlurDown => todo!(),
            ConfigEditorMsg::GlobalGotoBottomInputBlurUp => todo!(),
            ConfigEditorMsg::GlobalGotoTopBlurDown => todo!(),
            ConfigEditorMsg::GlobalGotoTopBlurUp => todo!(),
            ConfigEditorMsg::GlobalGotoTopInputBlurDown => todo!(),
            ConfigEditorMsg::GlobalGotoTopInputBlurUp => todo!(),
            ConfigEditorMsg::GlobalHelpBlurDown => todo!(),
            ConfigEditorMsg::GlobalHelpBlurUp => todo!(),
            ConfigEditorMsg::GlobalHelpInputBlurDown => todo!(),
            ConfigEditorMsg::GlobalHelpInputBlurUp => todo!(),
            ConfigEditorMsg::GlobalLayoutTreeviewBlurDown => todo!(),
            ConfigEditorMsg::GlobalLayoutTreeviewBlurUp => todo!(),
            ConfigEditorMsg::GlobalLayoutTreeviewInputBlurDown => todo!(),
            ConfigEditorMsg::GlobalLayoutTreeviewInputBlurUp => todo!(),
            ConfigEditorMsg::GlobalLayoutDatabaseBlurDown => todo!(),
            ConfigEditorMsg::GlobalLayoutDatabaseBlurUp => todo!(),
            ConfigEditorMsg::GlobalLayoutDatabaseInputBlurDown => todo!(),
            ConfigEditorMsg::GlobalLayoutDatabaseInputBlurUp => todo!(),
            ConfigEditorMsg::GlobalLeftBlurDown => todo!(),
            ConfigEditorMsg::GlobalLeftBlurUp => todo!(),
            ConfigEditorMsg::GlobalLeftInputBlurDown => todo!(),
            ConfigEditorMsg::GlobalLeftInputBlurUp => todo!(),
            ConfigEditorMsg::GlobalLyricAdjustForwardBlurDown => todo!(),
            ConfigEditorMsg::GlobalLyricAdjustForwardBlurUp => todo!(),
            ConfigEditorMsg::GlobalLyricAdjustBackwardBlurDown => todo!(),
            ConfigEditorMsg::GlobalLyricAdjustBackwardBlurUp => todo!(),
            ConfigEditorMsg::GlobalLyricAdjustForwardInputBlurDown => todo!(),
            ConfigEditorMsg::GlobalLyricAdjustForwardInputBlurUp => todo!(),
            ConfigEditorMsg::GlobalLyricAdjustBackwardInputBlurDown => todo!(),
            ConfigEditorMsg::GlobalLyricAdjustBackwardInputBlurUp => todo!(),
            ConfigEditorMsg::GlobalLyricCyleBlurDown => todo!(),
            ConfigEditorMsg::GlobalLyricCyleBlurUp => todo!(),
            ConfigEditorMsg::GlobalLyricCyleInputBlurDown => todo!(),
            ConfigEditorMsg::GlobalLyricCyleInputBlurUp => todo!(),
            ConfigEditorMsg::GlobalPlayerNextBlurDown => todo!(),
            ConfigEditorMsg::GlobalPlayerNextBlurUp => todo!(),
            ConfigEditorMsg::GlobalPlayerNextInputBlurDown => todo!(),
            ConfigEditorMsg::GlobalPlayerNextInputBlurUp => todo!(),
            ConfigEditorMsg::GlobalPlayerPreviousBlurDown => todo!(),
            ConfigEditorMsg::GlobalPlayerPreviousBlurUp => todo!(),
            ConfigEditorMsg::GlobalPlayerPreviousInputBlurDown => todo!(),
            ConfigEditorMsg::GlobalPlayerPreviousInputBlurUp => todo!(),
            ConfigEditorMsg::GlobalPlayerSeekForwardBlurDown => todo!(),
            ConfigEditorMsg::GlobalPlayerSeekForwardBlurUp => todo!(),
            ConfigEditorMsg::GlobalPlayerSeekForwardInputBlurDown => todo!(),
            ConfigEditorMsg::GlobalPlayerSeekForwardInputBlurUp => todo!(),
            ConfigEditorMsg::GlobalPlayerSeekBackwardBlurDown => todo!(),
            ConfigEditorMsg::GlobalPlayerSeekBackwardBlurUp => todo!(),
            ConfigEditorMsg::GlobalPlayerSeekBackwardInputBlurDown => todo!(),
            ConfigEditorMsg::GlobalPlayerSeekBackwardInputBlurUp => todo!(),
            ConfigEditorMsg::GlobalPlayerSpeedUpBlurDown => todo!(),
            ConfigEditorMsg::GlobalPlayerSpeedUpBlurUp => todo!(),
            ConfigEditorMsg::GlobalPlayerSpeedUpInputBlurDown => todo!(),
            ConfigEditorMsg::GlobalPlayerSpeedUpInputBlurUp => todo!(),
            ConfigEditorMsg::GlobalPlayerSpeedDownBlurDown => todo!(),
            ConfigEditorMsg::GlobalPlayerSpeedDownBlurUp => todo!(),
            ConfigEditorMsg::GlobalPlayerSpeedDownInputBlurDown => todo!(),
            ConfigEditorMsg::GlobalPlayerSpeedDownInputBlurUp => todo!(),
            ConfigEditorMsg::GlobalPlayerToggleGaplessBlurDown => todo!(),
            ConfigEditorMsg::GlobalPlayerToggleGaplessBlurUp => todo!(),
            ConfigEditorMsg::GlobalPlayerToggleGaplessInputBlurDown => todo!(),
            ConfigEditorMsg::GlobalPlayerToggleGaplessInputBlurUp => todo!(),
            ConfigEditorMsg::GlobalPlayerTogglePauseBlurDown => todo!(),
            ConfigEditorMsg::GlobalPlayerTogglePauseBlurUp => todo!(),
            ConfigEditorMsg::GlobalPlayerTogglePauseInputBlurDown => todo!(),
            ConfigEditorMsg::GlobalPlayerTogglePauseInputBlurUp => todo!(),
            ConfigEditorMsg::GlobalRightBlurDown => todo!(),
            ConfigEditorMsg::GlobalRightBlurUp => todo!(),
            ConfigEditorMsg::GlobalRightInputBlurDown => todo!(),
            ConfigEditorMsg::GlobalRightInputBlurUp => todo!(),
            ConfigEditorMsg::GlobalUpBlurDown => todo!(),
            ConfigEditorMsg::GlobalUpBlurUp => todo!(),
            ConfigEditorMsg::GlobalUpInputBlurDown => todo!(),
            ConfigEditorMsg::GlobalUpInputBlurUp => todo!(),
            ConfigEditorMsg::GlobalVolumeDownBlurDown => todo!(),
            ConfigEditorMsg::GlobalVolumeDownBlurUp => todo!(),
            ConfigEditorMsg::GlobalVolumeDownInputBlurDown => todo!(),
            ConfigEditorMsg::GlobalVolumeDownInputBlurUp => todo!(),
            ConfigEditorMsg::GlobalVolumeUpBlurDown => todo!(),
            ConfigEditorMsg::GlobalVolumeUpBlurUp => todo!(),
            ConfigEditorMsg::GlobalVolumeUpInputBlurDown => todo!(),
            ConfigEditorMsg::GlobalVolumeUpInputBlurUp => todo!(),

            // Focus of key 2 page
            ConfigEditorMsg::DatabaseAddAllBlurDown => todo!(),
            ConfigEditorMsg::DatabaseAddAllBlurUp => todo!(),
            ConfigEditorMsg::DatabaseAddAllInputBlurDown => todo!(),
            ConfigEditorMsg::DatabaseAddAllInputBlurUp => todo!(),
            ConfigEditorMsg::LibraryDeleteBlurDown => todo!(),
            ConfigEditorMsg::LibraryDeleteBlurUp => todo!(),
            ConfigEditorMsg::LibraryDeleteInputBlurDown => todo!(),
            ConfigEditorMsg::LibraryDeleteInputBlurUp => todo!(),
            ConfigEditorMsg::LibraryLoadDirBlurDown => todo!(),
            ConfigEditorMsg::LibraryLoadDirBlurUp => todo!(),
            ConfigEditorMsg::LibraryLoadDirInputBlurDown => todo!(),
            ConfigEditorMsg::LibraryLoadDirInputBlurUp => todo!(),
            ConfigEditorMsg::LibraryPasteBlurDown => todo!(),
            ConfigEditorMsg::LibraryPasteBlurUp => todo!(),
            ConfigEditorMsg::LibraryPasteInputBlurDown => todo!(),
            ConfigEditorMsg::LibraryPasteInputBlurUp => todo!(),
            ConfigEditorMsg::LibrarySearchBlurDown => todo!(),
            ConfigEditorMsg::LibrarySearchBlurUp => todo!(),
            ConfigEditorMsg::LibrarySearchInputBlurDown => todo!(),
            ConfigEditorMsg::LibrarySearchInputBlurUp => todo!(),
            ConfigEditorMsg::LibrarySearchYoutubeBlurDown => todo!(),
            ConfigEditorMsg::LibrarySearchYoutubeBlurUp => todo!(),
            ConfigEditorMsg::LibrarySearchYoutubeInputBlurDown => todo!(),
            ConfigEditorMsg::LibrarySearchYoutubeInputBlurUp => todo!(),
            ConfigEditorMsg::LibraryTagEditorBlurDown => todo!(),
            ConfigEditorMsg::LibraryTagEditorBlurUp => todo!(),
            ConfigEditorMsg::LibraryTagEditorInputBlurDown => todo!(),
            ConfigEditorMsg::LibraryTagEditorInputBlurUp => todo!(),
            ConfigEditorMsg::LibraryYankBlurDown => todo!(),
            ConfigEditorMsg::LibraryYankBlurUp => todo!(),
            ConfigEditorMsg::LibraryYankInputBlurDown => todo!(),
            ConfigEditorMsg::LibraryYankInputBlurUp => todo!(),
            ConfigEditorMsg::PlaylistDeleteBlurDown => todo!(),
            ConfigEditorMsg::PlaylistDeleteBlurUp => todo!(),
            ConfigEditorMsg::PlaylistDeleteInputBlurDown => todo!(),
            ConfigEditorMsg::PlaylistDeleteInputBlurUp => todo!(),
            ConfigEditorMsg::PlaylistDeleteAllBlurDown => todo!(),
            ConfigEditorMsg::PlaylistDeleteAllBlurUp => todo!(),
            ConfigEditorMsg::PlaylistDeleteAllInputBlurDown => todo!(),
            ConfigEditorMsg::PlaylistDeleteAllInputBlurUp => todo!(),
            ConfigEditorMsg::PlaylistShuffleBlurDown => todo!(),
            ConfigEditorMsg::PlaylistShuffleBlurUp => todo!(),
            ConfigEditorMsg::PlaylistShuffleInputBlurDown => todo!(),
            ConfigEditorMsg::PlaylistShuffleInputBlurUp => todo!(),
            ConfigEditorMsg::PlaylistModeCycleBlurDown => todo!(),
            ConfigEditorMsg::PlaylistModeCycleBlurUp => todo!(),
            ConfigEditorMsg::PlaylistModeCycleInputBlurDown => todo!(),
            ConfigEditorMsg::PlaylistModeCycleInputBlurUp => todo!(),
            ConfigEditorMsg::PlaylistPlaySelectedBlurDown => todo!(),
            ConfigEditorMsg::PlaylistPlaySelectedBlurUp => todo!(),
            ConfigEditorMsg::PlaylistPlaySelectedInputBlurDown => todo!(),
            ConfigEditorMsg::PlaylistPlaySelectedInputBlurUp => todo!(),
            ConfigEditorMsg::PlaylistAddFrontBlurDown => todo!(),
            ConfigEditorMsg::PlaylistAddFrontBlurUp => todo!(),
            ConfigEditorMsg::PlaylistAddFrontInputBlurDown => todo!(),
            ConfigEditorMsg::PlaylistAddFrontInputBlurUp => todo!(),
            ConfigEditorMsg::PlaylistSearchBlurDown => todo!(),
            ConfigEditorMsg::PlaylistSearchBlurUp => todo!(),
            ConfigEditorMsg::PlaylistSearchInputBlurDown => todo!(),
            ConfigEditorMsg::PlaylistSearchInputBlurUp => todo!(),
            ConfigEditorMsg::PlaylistSwapDownBlurDown => todo!(),
            ConfigEditorMsg::PlaylistSwapDownBlurUp => todo!(),
            ConfigEditorMsg::PlaylistSwapDownInputBlurDown => todo!(),
            ConfigEditorMsg::PlaylistSwapDownInputBlurUp => todo!(),
            ConfigEditorMsg::PlaylistSwapUpBlurDown => todo!(),
            ConfigEditorMsg::PlaylistSwapUpBlurUp => todo!(),
            ConfigEditorMsg::PlaylistSwapUpInputBlurDown => todo!(),
            ConfigEditorMsg::PlaylistSwapUpInputBlurUp => todo!(),
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
}
