use crate::ui::tui_cmd::TuiCmd;
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
use crate::ui::Model;
use anyhow::Context;
use termusiclib::config::new_shared_tui_settings;
use termusiclib::config::v2::server::config_extra::ServerConfigVersionedDefaulted;
use termusiclib::config::v2::tui::config_extra::TuiConfigVersionedDefaulted;
use termusiclib::config::v2::tui::keys::KeyBinding;
use termusiclib::config::v2::tui::theme::styles::ColorTermusic;
use termusiclib::config::v2::tui::theme::ThemeColors;
use termusiclib::types::{ConfigEditorMsg, Id, IdConfigEditor, IdKey, KFMsg, Msg};
use termusiclib::utils::get_app_config_path;

impl Model {
    #[allow(clippy::too_many_lines)]
    pub fn update_config_editor(&mut self, msg: ConfigEditorMsg) -> Option<Msg> {
        match msg {
            ConfigEditorMsg::Open => {
                self.config_editor.theme = self.config_tui.read().settings.theme.clone();
                self.config_editor.key_config = self.config_tui.read().settings.keys.clone();
                self.mount_config_editor();
            }
            ConfigEditorMsg::CloseCancel => {
                self.config_editor.config_changed = false;
                self.umount_config_editor();
            }
            ConfigEditorMsg::CloseOk => {
                if self.config_editor.config_changed {
                    self.config_editor.config_changed = false;
                    self.mount_config_save_popup();
                } else {
                    self.umount_config_editor();
                }
            }
            ConfigEditorMsg::ChangeLayout => self.action_change_layout(),
            ConfigEditorMsg::ConfigChanged => self.config_editor.config_changed = true,
            // Handle focus of general page
            ConfigEditorMsg::PlayerPortBlurDown | ConfigEditorMsg::ExitConfirmationBlurUp => {
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
            ConfigEditorMsg::PlaylistRandomTrackBlurDown | ConfigEditorMsg::PodcastDirBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::PlaylistRandomAlbum))
                    .ok();
            }
            ConfigEditorMsg::PlaylistRandomAlbumBlurDown
            | ConfigEditorMsg::PodcastSimulDownloadBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::PodcastDir))
                    .ok();
            }
            ConfigEditorMsg::PodcastDirBlurDown | ConfigEditorMsg::PodcastMaxRetriesBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::PodcastSimulDownload))
                    .ok();
            }
            ConfigEditorMsg::PodcastSimulDownloadBlurDown
            | ConfigEditorMsg::AlbumPhotoAlignBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::PodcastMaxRetries))
                    .ok();
            }

            ConfigEditorMsg::PodcastMaxRetriesBlurDown
            | ConfigEditorMsg::SaveLastPosotionBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::AlbumPhotoAlign))
                    .ok();
            }

            ConfigEditorMsg::AlbumPhotoAlignBlurDown | ConfigEditorMsg::SeekStepBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::SaveLastPosition))
                    .ok();
            }

            ConfigEditorMsg::SaveLastPositionBlurDown | ConfigEditorMsg::KillDaemonBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::SeekStep))
                    .ok();
            }

            ConfigEditorMsg::SeekStepBlurDown | ConfigEditorMsg::PlayerUseMprisBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::KillDamon))
                    .ok();
            }

            ConfigEditorMsg::KillDaemonBlurDown | ConfigEditorMsg::PlayerUseDiscordBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::PlayerUseMpris))
                    .ok();
            }

            ConfigEditorMsg::PlayerUseMprisBlurDown | ConfigEditorMsg::PlayerPortBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::PlayerUseDiscord))
                    .ok();
            }

            ConfigEditorMsg::PlayerUseDiscordBlurDown | ConfigEditorMsg::MusicDirBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::PlayerPort))
                    .ok();
            }
            ConfigEditorMsg::ConfigSaveOk => {
                self.app
                    .umount(&Id::ConfigEditor(IdConfigEditor::ConfigSavePopup))
                    .ok();
                match self.collect_config_data() {
                    Ok(()) => {
                        let res_server = ServerConfigVersionedDefaulted::save_config_path(
                            &self.config_server.read().settings,
                        )
                        .context("config editor save server settings");
                        let res_tui = TuiConfigVersionedDefaulted::save_config_path(
                            &self.config_tui.read().settings,
                        )
                        .context("config editor save tui settings");

                        let both_ok = res_server.is_ok() && res_tui.is_ok();

                        if let Err(err) = res_server {
                            self.mount_error_popup(err);
                        }

                        if let Err(err) = res_tui {
                            self.mount_error_popup(err);
                        }

                        if both_ok {
                            self.command(TuiCmd::ReloadConfig);

                            // only exit config editor if saving was successful
                            self.umount_config_editor();
                        }
                    }
                    Err(e) => {
                        self.mount_error_popup(e.context("collect config data"));
                        self.config_editor.config_changed = true;
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
            ConfigEditorMsg::LibraryForegroundBlurUp
            | ConfigEditorMsg::FallbackHighlightBlurDown => {
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
            | ConfigEditorMsg::CurrentlyPlayingTrackSymbolBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::PlaylistHighlightSymbol))
                    .ok();
            }
            ConfigEditorMsg::PlaylistHighlightSymbolBlurDown
            | ConfigEditorMsg::ProgressForegroundBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(
                        IdConfigEditor::CurrentlyPlayingTrackSymbol,
                    ))
                    .ok();
            }
            ConfigEditorMsg::CurrentlyPlayingTrackSymbolBlurDown
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
            ConfigEditorMsg::LyricBackgroundBlurDown
            | ConfigEditorMsg::ImportantPopupForegroundBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::LyricBorder))
                    .ok();
            }

            ConfigEditorMsg::LyricBorderBlurDown
            | ConfigEditorMsg::ImportantPopupBackgroundBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::ImportantPopupForeground))
                    .ok();
            }
            ConfigEditorMsg::ImportantPopupForegroundBlurDown
            | ConfigEditorMsg::ImportantPopupBorderBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::ImportantPopupBackground))
                    .ok();
            }
            ConfigEditorMsg::ImportantPopupBackgroundBlurDown
            | ConfigEditorMsg::FallbackForegroundBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::ImportantPopupBorder))
                    .ok();
            }

            ConfigEditorMsg::ImportantPopupBorderBlurDown
            | ConfigEditorMsg::FallbackBackgroundBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::FallbackForeground))
                    .ok();
            }
            ConfigEditorMsg::FallbackForegroundBlurDown | ConfigEditorMsg::FallbackBorderBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::FallbackBackground))
                    .ok();
            }
            ConfigEditorMsg::FallbackBackgroundBlurDown
            | ConfigEditorMsg::FallbackHighlightBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::FallbackBorder))
                    .ok();
            }
            ConfigEditorMsg::FallbackBorderBlurDown | ConfigEditorMsg::ThemeSelectBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::FallbackHighlight))
                    .ok();
            }

            ConfigEditorMsg::ThemeSelectLoad(index) => {
                self.preview_theme(index);
            }
            ConfigEditorMsg::ColorChanged(id, color_config) => {
                self.config_editor.config_changed = true;
                self.update_config_editor_color_changed(id, color_config);
            }
            ConfigEditorMsg::SymbolChanged(id, symbol) => {
                self.config_editor.config_changed = true;

                match id {
                    IdConfigEditor::LibraryHighlightSymbol => {
                        self.config_editor.theme.style.library.highlight_symbol =
                            symbol.to_string();
                    }
                    IdConfigEditor::PlaylistHighlightSymbol => {
                        self.config_editor.theme.style.playlist.highlight_symbol =
                            symbol.to_string();
                    }
                    IdConfigEditor::CurrentlyPlayingTrackSymbol => {
                        self.config_editor.theme.style.playlist.current_track_symbol =
                            symbol.to_string();
                    }
                    _ => {}
                };
            }

            ConfigEditorMsg::KeyChange(id, binding) => self.update_key(id, binding),
            ConfigEditorMsg::KeyFocus(msg) => self.update_key_focus(msg),
        }
        None
    }

    /// Preview theme at Table index
    fn preview_theme(&mut self, index: usize) {
        // table entry 0 is termusic default
        if index == 0 {
            self.preview_theme_apply(ThemeColors::full_default(), 0);

            return;
        }

        // idx - 1 as 0 table-entry is termusic default, which always exists
        if let Some(theme_filename) = self.config_editor.themes.get(index - 1) {
            match get_app_config_path() {
                Ok(mut theme_path) => {
                    theme_path.push("themes");
                    theme_path.push(format!("{theme_filename}.yml"));
                    match ThemeColors::from_yaml_file(&theme_path) {
                        Ok(mut theme) => {
                            theme.file_name = Some(theme_filename.to_string());

                            self.preview_theme_apply(theme, index);
                        }
                        Err(e) => {
                            error!("Failed to load theme colors: {:?}", e);
                        }
                    }
                }
                Err(e) => {
                    error!("Error getting config path: {:?}", e);
                }
            }
        }
    }

    /// Apply the given theme as a preview
    fn preview_theme_apply(&mut self, theme: ThemeColors, index: usize) {
        self.config_editor.theme.theme = theme;
        self.config_editor.config_changed = true;

        // This is for preview the theme colors
        let mut config = self.config_tui.read().clone();
        config.settings.theme = self.config_editor.theme.clone();
        let config = new_shared_tui_settings(config);
        self.remount_config_color(&config, Some(index));
    }

    #[allow(clippy::too_many_lines)]
    fn update_key_focus(&mut self, msg: KFMsg) {
        match msg {
            // Focus of key global page
            KFMsg::GlobalXywhHideBlurDown | KFMsg::GlobalLeftBlurUp => {
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

            KFMsg::GlobalPlayerToggleGaplessBlurDown | KFMsg::GlobalSavePlaylistBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::Key(IdKey::GlobalConfig)))
                    .ok();
            }

            KFMsg::GlobalConfigBlurDown | KFMsg::GlobalLayoutPodcastBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::Key(
                        IdKey::GlobalSavePlaylist,
                    )))
                    .ok();
            }

            KFMsg::GlobalSavePlaylistBlurDown | KFMsg::GlobalXywhMoveLeftBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::Key(
                        IdKey::GlobalLayoutPodcast,
                    )))
                    .ok();
            }
            KFMsg::GlobalLayoutPodcastBlurDown | KFMsg::GlobalXywhMoveRightBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::Key(
                        IdKey::GlobalXywhMoveLeft,
                    )))
                    .ok();
            }
            KFMsg::GlobalXywhMoveLeftBlurDown | KFMsg::GlobalXywhMoveUpBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::Key(
                        IdKey::GlobalXywhMoveRight,
                    )))
                    .ok();
            }
            KFMsg::GlobalXywhMoveRightBlurDown | KFMsg::GlobalXywhMoveDownBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::Key(
                        IdKey::GlobalXywhMoveUp,
                    )))
                    .ok();
            }
            KFMsg::GlobalXywhMoveUpBlurDown | KFMsg::GlobalXywhZoomInBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::Key(
                        IdKey::GlobalXywhMoveDown,
                    )))
                    .ok();
            }
            KFMsg::GlobalXywhMoveDownBlurDown | KFMsg::GlobalXywhZoomOutBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::Key(
                        IdKey::GlobalXywhZoomIn,
                    )))
                    .ok();
            }
            KFMsg::GlobalXywhZoomInBlurDown | KFMsg::GlobalXywhHideBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::Key(
                        IdKey::GlobalXywhZoomOut,
                    )))
                    .ok();
            }
            KFMsg::GlobalXywhZoomOutBlurDown | KFMsg::GlobalQuitBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::Key(
                        IdKey::GlobalXywhHide,
                    )))
                    .ok();
            }

            // Focus of key 2 page
            KFMsg::PodcastSearchAddFeedBlurDown | KFMsg::LibraryDeleteBlurUp => {
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

            KFMsg::PlaylistSearchBlurDown | KFMsg::PlaylistModeCycleBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::Key(
                        IdKey::PlaylistShuffle,
                    )))
                    .ok();
            }

            KFMsg::PlaylistShuffleBlurDown | KFMsg::PlaylistPlaySelectedBlurUp => {
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

            KFMsg::PlaylistSwapUpBlurDown | KFMsg::DatabaseAddSelectedBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::Key(
                        IdKey::DatabaseAddAll,
                    )))
                    .ok();
            }

            KFMsg::DatabaseAddAllBlurDown | KFMsg::PlaylistAddRandomAlbumBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::Key(
                        IdKey::DatabaseAddSelected,
                    )))
                    .ok();
            }

            KFMsg::DatabaseAddSelectedBlurDown | KFMsg::PlaylistAddRandomTracksBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::Key(
                        IdKey::PlaylistAddRandomAlbum,
                    )))
                    .ok();
            }

            KFMsg::PlaylistAddRandomAlbumBlurDown | KFMsg::LibrarySwitchRootBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::Key(
                        IdKey::PlaylistAddRandomTracks,
                    )))
                    .ok();
            }

            KFMsg::PlaylistAddRandomTracksBlurDown | KFMsg::LibraryAddRootBlurUp => {
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
            KFMsg::LibraryAddRootBlurDown | KFMsg::PodcastMarkPlayedBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::Key(
                        IdKey::LibraryRemoveRoot,
                    )))
                    .ok();
            }

            KFMsg::LibraryRemoveRootBlurDown | KFMsg::PodcastMarkAllPlayedBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::Key(
                        IdKey::PodcastMarkPlayed,
                    )))
                    .ok();
            }
            KFMsg::PodcastMarkPlayedBlurDown | KFMsg::PodcastEpDownloadBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::Key(
                        IdKey::PodcastMarkAllPlayed,
                    )))
                    .ok();
            }
            KFMsg::PodcastMarkAllPlayedBlurDown | KFMsg::PodcastEpDeleteFileBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::Key(
                        IdKey::PodcastEpDownload,
                    )))
                    .ok();
            }
            KFMsg::PodcastEpDownloadBlurDown | KFMsg::PodcastDeleteFeedBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::Key(
                        IdKey::PodcastEpDeleteFile,
                    )))
                    .ok();
            }
            KFMsg::PodcastEpDeleteFileBlurDown | KFMsg::PodcastDeleteAllFeedsBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::Key(
                        IdKey::PodcastDeleteFeed,
                    )))
                    .ok();
            }
            KFMsg::PodcastDeleteFeedBlurDown | KFMsg::PodcastRefreshFeedBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::Key(
                        IdKey::PodcastDeleteAllFeeds,
                    )))
                    .ok();
            }
            KFMsg::PodcastDeleteAllFeedsBlurDown | KFMsg::PodcastRefreshAllFeedsBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::Key(
                        IdKey::PodcastRefreshFeed,
                    )))
                    .ok();
            }
            KFMsg::PodcastRefreshFeedBlurDown | KFMsg::PodcastSearchAddFeedBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::Key(
                        IdKey::PodcastRefreshAllFeeds,
                    )))
                    .ok();
            }
            KFMsg::PodcastRefreshAllFeedsBlurDown | KFMsg::LibraryTagEditorBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::Key(
                        IdKey::PodcastSearchAddFeed,
                    )))
                    .ok();
            }
        }
    }

    // cannot reduce a match statement
    #[allow(clippy::too_many_lines)]
    fn update_key(&mut self, id: IdKey, binding: KeyBinding) {
        self.config_editor.config_changed = true;

        // alias to reduce line length
        let keys = &mut self.config_editor.key_config;

        match id {
            IdKey::DatabaseAddAll => keys.database_keys.add_all = binding,
            IdKey::DatabaseAddSelected => keys.database_keys.add_selected = binding,
            IdKey::GlobalConfig => keys.select_view_keys.open_config = binding,
            IdKey::GlobalDown => keys.navigation_keys.down = binding,
            IdKey::GlobalGotoBottom => keys.navigation_keys.goto_bottom = binding,
            IdKey::GlobalGotoTop => keys.navigation_keys.goto_top = binding,
            IdKey::GlobalHelp => keys.select_view_keys.open_help = binding,
            IdKey::GlobalLayoutTreeview => {
                keys.select_view_keys.view_library = binding;
            }
            IdKey::GlobalLayoutDatabase => {
                keys.select_view_keys.view_database = binding;
            }
            IdKey::GlobalLeft => keys.navigation_keys.left = binding,
            IdKey::GlobalLyricAdjustForward => {
                keys.lyric_keys.adjust_offset_forwards = binding;
            }
            IdKey::GlobalLyricAdjustBackward => {
                keys.lyric_keys.adjust_offset_backwards = binding;
            }
            IdKey::GlobalLyricCycle => keys.lyric_keys.cycle_frames = binding,
            IdKey::GlobalPlayerToggleGapless => {
                keys.player_keys.toggle_prefetch = binding;
            }
            IdKey::GlobalPlayerTogglePause => {
                keys.player_keys.toggle_pause = binding;
            }
            IdKey::GlobalPlayerNext => keys.player_keys.next_track = binding,
            IdKey::GlobalPlayerPrevious => keys.player_keys.previous_track = binding,
            IdKey::GlobalPlayerSeekForward => {
                keys.player_keys.seek_forward = binding;
            }
            IdKey::GlobalPlayerSeekBackward => {
                keys.player_keys.seek_backward = binding;
            }
            IdKey::GlobalPlayerSpeedUp => keys.player_keys.speed_up = binding,
            IdKey::GlobalPlayerSpeedDown => keys.player_keys.speed_down = binding,
            IdKey::GlobalQuit => keys.quit = binding,
            IdKey::GlobalRight => keys.navigation_keys.right = binding,
            IdKey::GlobalUp => keys.navigation_keys.up = binding,
            IdKey::GlobalVolumeDown => keys.player_keys.volume_down = binding,
            IdKey::GlobalVolumeUp => keys.player_keys.volume_up = binding,
            IdKey::GlobalSavePlaylist => keys.player_keys.save_playlist = binding,
            IdKey::LibraryDelete => keys.library_keys.delete = binding,
            IdKey::LibraryLoadDir => keys.library_keys.load_dir = binding,
            IdKey::LibraryPaste => keys.library_keys.paste = binding,
            IdKey::LibrarySearch => keys.library_keys.search = binding,
            IdKey::LibrarySearchYoutube => keys.library_keys.youtube_search = binding,
            IdKey::LibraryTagEditor => keys.library_keys.open_tag_editor = binding,
            IdKey::LibraryYank => keys.library_keys.yank = binding,
            IdKey::PlaylistDelete => keys.playlist_keys.delete = binding,
            IdKey::PlaylistDeleteAll => keys.playlist_keys.delete_all = binding,
            IdKey::PlaylistShuffle => keys.playlist_keys.shuffle = binding,
            IdKey::PlaylistModeCycle => keys.playlist_keys.cycle_loop_mode = binding,
            IdKey::PlaylistPlaySelected => keys.playlist_keys.play_selected = binding,
            IdKey::PlaylistSearch => keys.playlist_keys.search = binding,
            IdKey::PlaylistSwapDown => keys.playlist_keys.swap_down = binding,
            IdKey::PlaylistSwapUp => keys.playlist_keys.swap_up = binding,
            IdKey::PlaylistAddRandomAlbum => {
                keys.playlist_keys.add_random_album = binding;
            }
            IdKey::PlaylistAddRandomTracks => {
                keys.playlist_keys.add_random_songs = binding;
            }
            IdKey::LibrarySwitchRoot => keys.library_keys.cycle_root = binding,
            IdKey::LibraryAddRoot => keys.library_keys.add_root = binding,
            IdKey::LibraryRemoveRoot => keys.library_keys.remove_root = binding,
            IdKey::GlobalLayoutPodcast => {
                keys.select_view_keys.view_podcasts = binding;
            }
            IdKey::GlobalXywhMoveLeft => keys.move_cover_art_keys.move_left = binding,
            IdKey::GlobalXywhMoveRight => {
                keys.move_cover_art_keys.move_right = binding;
            }
            IdKey::GlobalXywhMoveUp => keys.move_cover_art_keys.move_up = binding,
            IdKey::GlobalXywhMoveDown => keys.move_cover_art_keys.move_down = binding,
            IdKey::GlobalXywhZoomIn => {
                keys.move_cover_art_keys.increase_size = binding;
            }
            IdKey::GlobalXywhZoomOut => {
                keys.move_cover_art_keys.decrease_size = binding;
            }
            IdKey::GlobalXywhHide => keys.move_cover_art_keys.toggle_hide = binding,
            IdKey::PodcastMarkPlayed => keys.podcast_keys.mark_played = binding,
            IdKey::PodcastMarkAllPlayed => {
                keys.podcast_keys.mark_all_played = binding;
            }
            IdKey::PodcastEpDownload => keys.podcast_keys.download_episode = binding,
            IdKey::PodcastEpDeleteFile => {
                keys.podcast_keys.delete_local_episode = binding;
            }
            IdKey::PodcastDeleteFeed => keys.podcast_keys.delete_feed = binding,
            IdKey::PodcastDeleteAllFeeds => {
                keys.podcast_keys.delete_all_feeds = binding;
            }
            IdKey::PodcastSearchAddFeed => keys.podcast_keys.search = binding,
            IdKey::PodcastRefreshFeed => keys.podcast_keys.refresh_feed = binding,
            IdKey::PodcastRefreshAllFeeds => {
                keys.podcast_keys.refresh_all_feeds = binding;
            }
        }
    }

    fn update_config_editor_color_changed(
        &mut self,
        id: IdConfigEditor,
        color_config: ColorTermusic,
    ) {
        // alias to reduce line length
        let style = &mut self.config_editor.theme.style;

        match id {
            IdConfigEditor::LibraryForeground => {
                style.library.foreground_color = color_config;
            }
            IdConfigEditor::LibraryBackground => {
                style.library.background_color = color_config;
            }
            IdConfigEditor::LibraryBorder => {
                style.library.border_color = color_config;
            }
            IdConfigEditor::LibraryHighlight => {
                style.library.highlight_color = color_config;
            }
            IdConfigEditor::PlaylistForeground => {
                style.playlist.foreground_color = color_config;
            }
            IdConfigEditor::PlaylistBackground => {
                style.playlist.background_color = color_config;
            }
            IdConfigEditor::PlaylistBorder => {
                style.playlist.border_color = color_config;
            }
            IdConfigEditor::PlaylistHighlight => {
                style.playlist.highlight_color = color_config;
            }
            IdConfigEditor::ProgressForeground => {
                style.progress.foreground_color = color_config;
            }
            IdConfigEditor::ProgressBackground => {
                style.progress.background_color = color_config;
            }
            IdConfigEditor::ProgressBorder => {
                style.progress.border_color = color_config;
            }
            IdConfigEditor::LyricForeground => {
                style.lyric.foreground_color = color_config;
            }
            IdConfigEditor::LyricBackground => {
                style.lyric.background_color = color_config;
            }
            IdConfigEditor::LyricBorder => {
                style.lyric.border_color = color_config;
            }

            _ => {}
        }
    }
}
