use crate::config::{load_alacritty, BindingForEvent, ColorTermusic};
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
use crate::ui::{ConfigEditorMsg, Id, IdConfigEditor, IdKey, KFMsg, Model, Msg};
use std::path::PathBuf;

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
            // Handle focus of general page
            ConfigEditorMsg::AlbumPhotoAlignBlurDown | ConfigEditorMsg::ExitConfirmationBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::MusicDir))
                    .ok();
            }
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
            ConfigEditorMsg::PlaylistDisplaySymbolBlurDown
            | ConfigEditorMsg::PlaylistRandomAlbumBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::PlaylistRandomTrack))
                    .ok();
            }
            ConfigEditorMsg::PlaylistRandomTrackBlurDown | ConfigEditorMsg::AlbumPhotoXBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::PlaylistRandomAlbum))
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
            ConfigEditorMsg::ConfigSaveOk => {
                self.app
                    .umount(&Id::ConfigEditor(IdConfigEditor::ConfigSavePopup))
                    .ok();
                match self.collect_config_data() {
                    Ok(()) => self.umount_config_editor(),
                    Err(e) => {
                        self.mount_error_popup(format!("save config error: {}", e).as_str());
                        self.config_changed = true;
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

            ConfigEditorMsg::KeyChange(id, binding) => self.update_key(id, binding),
            ConfigEditorMsg::KeyFocus(msg) => self.update_key_focus(msg),
        }
        None
    }

    #[allow(clippy::too_many_lines)]
    fn update_key_focus(&mut self, msg: &KFMsg) {
        match msg {
            // Focus of key global page
            KFMsg::GlobalConfigBlurDown | KFMsg::GlobalLeftBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::Key(IdKey::GlobalQuit)))
                    .ok();
            }
            KFMsg::GlobalQuitBlurDown | KFMsg::GlobalDownBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::Key(IdKey::GlobalLeft)))
                    .ok();
            }

            KFMsg::GlobalLeftBlurDown | KFMsg::GlobalUpBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::Key(IdKey::GlobalDown)))
                    .ok();
            }

            KFMsg::GlobalDownBlurDown | KFMsg::GlobalRightBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::Key(IdKey::GlobalUp)))
                    .ok();
            }

            KFMsg::GlobalUpBlurDown | KFMsg::GlobalGotoTopBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::Key(IdKey::GlobalRight)))
                    .ok();
            }
            KFMsg::GlobalRightBlurDown | KFMsg::GlobalGotoBottomBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::Key(IdKey::GlobalGotoTop)))
                    .ok();
            }
            KFMsg::GlobalGotoTopBlurDown | KFMsg::GlobalPlayerTogglePauseBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::Key(
                        IdKey::GlobalGotoBottom,
                    )))
                    .ok();
            }
            KFMsg::GlobalGotoBottomBlurDown | KFMsg::GlobalPlayerNextBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::Key(
                        IdKey::GlobalPlayerTogglePause,
                    )))
                    .ok();
            }
            KFMsg::GlobalPlayerTogglePauseBlurDown | KFMsg::GlobalPlayerPreviousBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::Key(
                        IdKey::GlobalPlayerNext,
                    )))
                    .ok();
            }
            KFMsg::GlobalPlayerNextBlurDown | KFMsg::GlobalHelpBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::Key(
                        IdKey::GlobalPlayerPrevious,
                    )))
                    .ok();
            }
            KFMsg::GlobalPlayerPreviousBlurDown | KFMsg::GlobalVolumeUpBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::Key(IdKey::GlobalHelp)))
                    .ok();
            }
            KFMsg::GlobalHelpBlurDown | KFMsg::GlobalVolumeDownBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::Key(
                        IdKey::GlobalVolumeUp,
                    )))
                    .ok();
            }
            KFMsg::GlobalVolumeUpBlurDown | KFMsg::GlobalPlayerSeekForwardBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::Key(
                        IdKey::GlobalVolumeDown,
                    )))
                    .ok();
            }
            KFMsg::GlobalVolumeDownBlurDown | KFMsg::GlobalPlayerSeekBackwardBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::Key(
                        IdKey::GlobalPlayerSeekForward,
                    )))
                    .ok();
            }

            KFMsg::GlobalPlayerSeekForwardBlurDown | KFMsg::GlobalPlayerSpeedUpBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::Key(
                        IdKey::GlobalPlayerSeekBackward,
                    )))
                    .ok();
            }

            KFMsg::GlobalPlayerSeekBackwardBlurDown | KFMsg::GlobalPlayerSpeedDownBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::Key(
                        IdKey::GlobalPlayerSpeedUp,
                    )))
                    .ok();
            }

            KFMsg::GlobalPlayerSpeedUpBlurDown | KFMsg::GlobalLyricAdjustForwardBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::Key(
                        IdKey::GlobalPlayerSpeedDown,
                    )))
                    .ok();
            }

            KFMsg::GlobalPlayerSpeedDownBlurDown | KFMsg::GlobalLyricAdjustBackwardBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::Key(
                        IdKey::GlobalLyricAdjustForward,
                    )))
                    .ok();
            }

            KFMsg::GlobalLyricAdjustForwardBlurDown | KFMsg::GlobalLyricCycleBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::Key(
                        IdKey::GlobalLyricAdjustBackward,
                    )))
                    .ok();
            }

            KFMsg::GlobalLyricAdjustBackwardBlurDown | KFMsg::GlobalLayoutTreeviewBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::Key(
                        IdKey::GlobalLyricCycle,
                    )))
                    .ok();
            }

            KFMsg::GlobalLyricCycleBlurDown | KFMsg::GlobalLayoutDatabaseBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::Key(
                        IdKey::GlobalLayoutTreeview,
                    )))
                    .ok();
            }

            KFMsg::GlobalLayoutTreeviewBlurDown | KFMsg::GlobalPlayerToggleGaplessBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::Key(
                        IdKey::GlobalLayoutDatabase,
                    )))
                    .ok();
            }

            KFMsg::GlobalLayoutDatabaseBlurDown | KFMsg::GlobalConfigBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::Key(
                        IdKey::GlobalPlayerToggleGapless,
                    )))
                    .ok();
            }

            KFMsg::GlobalPlayerToggleGaplessBlurDown | KFMsg::GlobalQuitBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::Key(IdKey::GlobalConfig)))
                    .ok();
            }

            // Focus of key 2 page
            KFMsg::LibraryRemoveRootBlurDown | KFMsg::LibraryDeleteBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::Key(
                        IdKey::LibraryTagEditor,
                    )))
                    .ok();
            }

            KFMsg::LibraryTagEditorBlurDown | KFMsg::LibraryLoadDirBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::Key(IdKey::LibraryDelete)))
                    .ok();
            }

            KFMsg::LibraryDeleteBlurDown | KFMsg::LibraryYankBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::Key(
                        IdKey::LibraryLoadDir,
                    )))
                    .ok();
            }

            KFMsg::LibraryLoadDirBlurDown | KFMsg::LibraryPasteBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::Key(IdKey::LibraryYank)))
                    .ok();
            }

            KFMsg::LibraryYankBlurDown | KFMsg::LibrarySearchBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::Key(IdKey::LibraryPaste)))
                    .ok();
            }

            KFMsg::LibraryPasteBlurDown | KFMsg::LibrarySearchYoutubeBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::Key(IdKey::LibrarySearch)))
                    .ok();
            }

            KFMsg::LibrarySearchBlurDown | KFMsg::PlaylistDeleteBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::Key(
                        IdKey::LibrarySearchYoutube,
                    )))
                    .ok();
            }

            KFMsg::LibrarySearchYoutubeBlurDown | KFMsg::PlaylistDeleteAllBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::Key(
                        IdKey::PlaylistDelete,
                    )))
                    .ok();
            }

            KFMsg::PlaylistDeleteBlurDown | KFMsg::PlaylistSearchBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::Key(
                        IdKey::PlaylistDeleteAll,
                    )))
                    .ok();
            }

            KFMsg::PlaylistDeleteAllBlurDown | KFMsg::PlaylistShuffleBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::Key(
                        IdKey::PlaylistSearch,
                    )))
                    .ok();
            }

            KFMsg::PlaylistSearchBlurDown | KFMsg::PlaylistAddFrontBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::Key(
                        IdKey::PlaylistShuffle,
                    )))
                    .ok();
            }

            KFMsg::PlaylistShuffleBlurDown | KFMsg::PlaylistModeCycleBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::Key(
                        IdKey::PlaylistAddFront,
                    )))
                    .ok();
            }

            KFMsg::PlaylistAddFrontBlurDown | KFMsg::PlaylistPlaySelectedBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::Key(
                        IdKey::PlaylistModeCycle,
                    )))
                    .ok();
            }

            KFMsg::PlaylistModeCycleBlurDown | KFMsg::PlaylistSwapDownBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::Key(
                        IdKey::PlaylistPlaySelected,
                    )))
                    .ok();
            }

            KFMsg::PlaylistPlaySelectedBlurDown | KFMsg::PlaylistSwapUpBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::Key(
                        IdKey::PlaylistSwapDown,
                    )))
                    .ok();
            }

            KFMsg::PlaylistSwapDownBlurDown | KFMsg::DatabaseAddAllBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::Key(
                        IdKey::PlaylistSwapUp,
                    )))
                    .ok();
            }

            KFMsg::PlaylistSwapUpBlurDown | KFMsg::PlaylistLqueueBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::Key(
                        IdKey::DatabaseAddAll,
                    )))
                    .ok();
            }

            KFMsg::DatabaseAddAllBlurDown | KFMsg::PlaylistTqueueBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::Key(
                        IdKey::PlaylistLqueue,
                    )))
                    .ok();
            }

            KFMsg::PlaylistLqueueBlurDown | KFMsg::LibrarySwitchRootBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::Key(
                        IdKey::PlaylistTqueue,
                    )))
                    .ok();
            }

            KFMsg::PlaylistTqueueBlurDown | KFMsg::LibraryAddRootBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::Key(
                        IdKey::LibrarySwitchRoot,
                    )))
                    .ok();
            }

            KFMsg::LibrarySwitchRootBlurDown | KFMsg::LibraryRemoveRootBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::Key(
                        IdKey::LibraryAddRoot,
                    )))
                    .ok();
            }
            KFMsg::LibraryAddRootBlurDown | KFMsg::LibraryTagEditorBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::Key(
                        IdKey::LibraryRemoveRoot,
                    )))
                    .ok();
            }
        }
    }

    fn update_key(&mut self, id: &IdKey, binding: &BindingForEvent) {
        self.config_changed = true;
        match id {
            IdKey::DatabaseAddAll => self.ke_key_config.database_add_all = *binding,
            IdKey::GlobalConfig => self.ke_key_config.global_config_open = *binding,
            IdKey::GlobalDown => self.ke_key_config.global_down = *binding,
            IdKey::GlobalGotoBottom => self.ke_key_config.global_goto_bottom = *binding,
            IdKey::GlobalGotoTop => self.ke_key_config.global_goto_top = *binding,
            IdKey::GlobalHelp => self.ke_key_config.global_help = *binding,
            IdKey::GlobalLayoutTreeview => self.ke_key_config.global_layout_treeview = *binding,
            IdKey::GlobalLayoutDatabase => self.ke_key_config.global_layout_database = *binding,
            IdKey::GlobalLeft => self.ke_key_config.global_left = *binding,
            IdKey::GlobalLyricAdjustForward => {
                self.ke_key_config.global_lyric_adjust_forward = *binding;
            }
            IdKey::GlobalLyricAdjustBackward => {
                self.ke_key_config.global_lyric_adjust_backward = *binding;
            }
            IdKey::GlobalLyricCycle => self.ke_key_config.global_lyric_cycle = *binding,
            IdKey::GlobalPlayerToggleGapless => {
                self.ke_key_config.global_player_toggle_gapless = *binding;
            }
            IdKey::GlobalPlayerTogglePause => {
                self.ke_key_config.global_player_toggle_pause = *binding;
            }
            IdKey::GlobalPlayerNext => self.ke_key_config.global_player_next = *binding,
            IdKey::GlobalPlayerPrevious => self.ke_key_config.global_player_previous = *binding,
            IdKey::GlobalPlayerSeekForward => {
                self.ke_key_config.global_player_seek_forward = *binding;
            }
            IdKey::GlobalPlayerSeekBackward => {
                self.ke_key_config.global_player_seek_backward = *binding;
            }
            IdKey::GlobalPlayerSpeedUp => self.ke_key_config.global_player_speed_up = *binding,
            IdKey::GlobalPlayerSpeedDown => self.ke_key_config.global_player_speed_down = *binding,
            IdKey::GlobalQuit => self.ke_key_config.global_quit = *binding,
            IdKey::GlobalRight => self.ke_key_config.global_right = *binding,
            IdKey::GlobalUp => self.ke_key_config.global_up = *binding,
            IdKey::GlobalVolumeDown => self.ke_key_config.global_player_volume_minus_2 = *binding,
            IdKey::GlobalVolumeUp => self.ke_key_config.global_player_volume_plus_2 = *binding,
            IdKey::LibraryDelete => self.ke_key_config.library_delete = *binding,
            IdKey::LibraryLoadDir => self.ke_key_config.library_load_dir = *binding,
            IdKey::LibraryPaste => self.ke_key_config.library_paste = *binding,
            IdKey::LibrarySearch => self.ke_key_config.library_search = *binding,
            IdKey::LibrarySearchYoutube => self.ke_key_config.library_search_youtube = *binding,
            IdKey::LibraryTagEditor => self.ke_key_config.library_tag_editor_open = *binding,
            IdKey::LibraryYank => self.ke_key_config.library_yank = *binding,
            IdKey::PlaylistDelete => self.ke_key_config.playlist_delete = *binding,
            IdKey::PlaylistDeleteAll => self.ke_key_config.playlist_delete_all = *binding,
            IdKey::PlaylistShuffle => self.ke_key_config.playlist_shuffle = *binding,
            IdKey::PlaylistModeCycle => self.ke_key_config.playlist_mode_cycle = *binding,
            IdKey::PlaylistPlaySelected => self.ke_key_config.playlist_play_selected = *binding,
            IdKey::PlaylistAddFront => self.ke_key_config.playlist_add_front = *binding,
            IdKey::PlaylistSearch => self.ke_key_config.playlist_search = *binding,
            IdKey::PlaylistSwapDown => self.ke_key_config.playlist_swap_down = *binding,
            IdKey::PlaylistSwapUp => self.ke_key_config.playlist_swap_up = *binding,
            IdKey::PlaylistLqueue => self.ke_key_config.playlist_cmus_lqueue = *binding,
            IdKey::PlaylistTqueue => self.ke_key_config.playlist_cmus_tqueue = *binding,
            IdKey::LibrarySwitchRoot => self.ke_key_config.library_switch_root = *binding,
            IdKey::LibraryAddRoot => self.ke_key_config.library_add_root = *binding,
            IdKey::LibraryRemoveRoot => self.ke_key_config.library_remove_root = *binding,
        }
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
