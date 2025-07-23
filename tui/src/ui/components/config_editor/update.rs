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
use crate::ui::tui_cmd::TuiCmd;
use anyhow::Context;
use termusiclib::config::new_shared_tui_settings;
use termusiclib::config::v2::server::config_extra::ServerConfigVersionedDefaulted;
use termusiclib::config::v2::tui::config_extra::TuiConfigVersionedDefaulted;
use termusiclib::config::v2::tui::keys::KeyBinding;
use termusiclib::config::v2::tui::theme::ThemeColors;
use termusiclib::config::v2::tui::theme::styles::ColorTermusic;
use termusiclib::ids::{Id, IdConfigEditor, IdKeyGlobal, IdKeyOther};
use termusiclib::types::{ConfigEditorMsg, IdKey, KFMsg, Msg};
use termusiclib::utils::get_app_config_path;

/// How many Themes there are without actual files and always exist
pub const THEMES_WITHOUT_FILES: usize = 2;

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
            ConfigEditorMsg::ExtraYtdlpArgsBlurDown | ConfigEditorMsg::ExitConfirmationBlurUp => {
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

            ConfigEditorMsg::PlayerUseDiscordBlurDown | ConfigEditorMsg::ExtraYtdlpArgsBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::PlayerPort))
                    .ok();
            }

            ConfigEditorMsg::PlayerPortBlurDown | ConfigEditorMsg::MusicDirBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::ExtraYtdlpArgs))
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
                        self.config_editor.theme.style.library.highlight_symbol = symbol;
                    }
                    IdConfigEditor::PlaylistHighlightSymbol => {
                        self.config_editor.theme.style.playlist.highlight_symbol = symbol;
                    }
                    IdConfigEditor::CurrentlyPlayingTrackSymbol => {
                        self.config_editor.theme.style.playlist.current_track_symbol = symbol;
                    }
                    _ => {}
                }
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
        if index == 1 {
            self.preview_theme_apply(ThemeColors::full_native(), 1);

            return;
        }

        // idx - THEMES_WITHOUT_FILES as 0 until THEMES_WITHOUT_FILES table-entries are termusic themes without files, which always exists
        if let Some(theme_filename) = self.config_editor.themes.get(index - THEMES_WITHOUT_FILES) {
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
                            error!("Failed to load theme colors: {e:?}");
                        }
                    }
                }
                Err(e) => {
                    error!("Error getting config path: {e:?}");
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
        self.remount_config_color(&config, Some(index)).unwrap();
    }

    #[allow(clippy::too_many_lines)]
    fn update_key_focus(&mut self, msg: KFMsg) {
        match msg {
            // Focus of key global page
            KFMsg::GlobalXywhHideBlurDown | KFMsg::GlobalLeftBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::KeyGlobal(
                        IdKeyGlobal::GlobalQuit,
                    )))
                    .ok();
            }
            KFMsg::GlobalQuitBlurDown | KFMsg::GlobalDownBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::KeyGlobal(
                        IdKeyGlobal::GlobalLeft,
                    )))
                    .ok();
            }

            KFMsg::GlobalLeftBlurDown | KFMsg::GlobalUpBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::KeyGlobal(
                        IdKeyGlobal::GlobalDown,
                    )))
                    .ok();
            }

            KFMsg::GlobalDownBlurDown | KFMsg::GlobalRightBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::KeyGlobal(
                        IdKeyGlobal::GlobalUp,
                    )))
                    .ok();
            }

            KFMsg::GlobalUpBlurDown | KFMsg::GlobalGotoTopBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::KeyGlobal(
                        IdKeyGlobal::GlobalRight,
                    )))
                    .ok();
            }
            KFMsg::GlobalRightBlurDown | KFMsg::GlobalGotoBottomBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::KeyGlobal(
                        IdKeyGlobal::GlobalGotoTop,
                    )))
                    .ok();
            }
            KFMsg::GlobalGotoTopBlurDown | KFMsg::GlobalPlayerTogglePauseBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::KeyGlobal(
                        IdKeyGlobal::GlobalGotoBottom,
                    )))
                    .ok();
            }
            KFMsg::GlobalGotoBottomBlurDown | KFMsg::GlobalPlayerNextBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::KeyGlobal(
                        IdKeyGlobal::GlobalPlayerTogglePause,
                    )))
                    .ok();
            }
            KFMsg::GlobalPlayerTogglePauseBlurDown | KFMsg::GlobalPlayerPreviousBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::KeyGlobal(
                        IdKeyGlobal::GlobalPlayerNext,
                    )))
                    .ok();
            }
            KFMsg::GlobalPlayerNextBlurDown | KFMsg::GlobalHelpBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::KeyGlobal(
                        IdKeyGlobal::GlobalPlayerPrevious,
                    )))
                    .ok();
            }
            KFMsg::GlobalPlayerPreviousBlurDown | KFMsg::GlobalVolumeUpBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::KeyGlobal(
                        IdKeyGlobal::GlobalHelp,
                    )))
                    .ok();
            }
            KFMsg::GlobalHelpBlurDown | KFMsg::GlobalVolumeDownBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::KeyGlobal(
                        IdKeyGlobal::GlobalPlayerVolumeUp,
                    )))
                    .ok();
            }
            KFMsg::GlobalVolumeUpBlurDown | KFMsg::GlobalPlayerSeekForwardBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::KeyGlobal(
                        IdKeyGlobal::GlobalPlayerVolumeDown,
                    )))
                    .ok();
            }
            KFMsg::GlobalVolumeDownBlurDown | KFMsg::GlobalPlayerSeekBackwardBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::KeyGlobal(
                        IdKeyGlobal::GlobalPlayerSeekForward,
                    )))
                    .ok();
            }

            KFMsg::GlobalPlayerSeekForwardBlurDown | KFMsg::GlobalPlayerSpeedUpBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::KeyGlobal(
                        IdKeyGlobal::GlobalPlayerSeekBackward,
                    )))
                    .ok();
            }

            KFMsg::GlobalPlayerSeekBackwardBlurDown | KFMsg::GlobalPlayerSpeedDownBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::KeyGlobal(
                        IdKeyGlobal::GlobalPlayerSpeedUp,
                    )))
                    .ok();
            }

            KFMsg::GlobalPlayerSpeedUpBlurDown | KFMsg::GlobalLyricAdjustForwardBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::KeyGlobal(
                        IdKeyGlobal::GlobalPlayerSpeedDown,
                    )))
                    .ok();
            }

            KFMsg::GlobalPlayerSpeedDownBlurDown | KFMsg::GlobalLyricAdjustBackwardBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::KeyGlobal(
                        IdKeyGlobal::GlobalLyricAdjustForward,
                    )))
                    .ok();
            }

            KFMsg::GlobalLyricAdjustForwardBlurDown | KFMsg::GlobalLyricCycleBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::KeyGlobal(
                        IdKeyGlobal::GlobalLyricAdjustBackward,
                    )))
                    .ok();
            }

            KFMsg::GlobalLyricAdjustBackwardBlurDown | KFMsg::GlobalLayoutTreeviewBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::KeyGlobal(
                        IdKeyGlobal::GlobalLyricCycle,
                    )))
                    .ok();
            }

            KFMsg::GlobalLyricCycleBlurDown | KFMsg::GlobalLayoutDatabaseBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::KeyGlobal(
                        IdKeyGlobal::GlobalLayoutTreeview,
                    )))
                    .ok();
            }

            KFMsg::GlobalLayoutTreeviewBlurDown | KFMsg::GlobalPlayerToggleGaplessBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::KeyGlobal(
                        IdKeyGlobal::GlobalLayoutDatabase,
                    )))
                    .ok();
            }

            KFMsg::GlobalLayoutDatabaseBlurDown | KFMsg::GlobalConfigBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::KeyGlobal(
                        IdKeyGlobal::GlobalPlayerToggleGapless,
                    )))
                    .ok();
            }

            KFMsg::GlobalPlayerToggleGaplessBlurDown | KFMsg::GlobalSavePlaylistBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::KeyGlobal(
                        IdKeyGlobal::GlobalConfig,
                    )))
                    .ok();
            }

            KFMsg::GlobalConfigBlurDown | KFMsg::GlobalLayoutPodcastBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::KeyGlobal(
                        IdKeyGlobal::GlobalSavePlaylist,
                    )))
                    .ok();
            }

            KFMsg::GlobalSavePlaylistBlurDown | KFMsg::GlobalXywhMoveLeftBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::KeyGlobal(
                        IdKeyGlobal::GlobalLayoutPodcast,
                    )))
                    .ok();
            }
            KFMsg::GlobalLayoutPodcastBlurDown | KFMsg::GlobalXywhMoveRightBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::KeyGlobal(
                        IdKeyGlobal::GlobalXywhMoveLeft,
                    )))
                    .ok();
            }
            KFMsg::GlobalXywhMoveLeftBlurDown | KFMsg::GlobalXywhMoveUpBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::KeyGlobal(
                        IdKeyGlobal::GlobalXywhMoveRight,
                    )))
                    .ok();
            }
            KFMsg::GlobalXywhMoveRightBlurDown | KFMsg::GlobalXywhMoveDownBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::KeyGlobal(
                        IdKeyGlobal::GlobalXywhMoveUp,
                    )))
                    .ok();
            }
            KFMsg::GlobalXywhMoveUpBlurDown | KFMsg::GlobalXywhZoomInBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::KeyGlobal(
                        IdKeyGlobal::GlobalXywhMoveDown,
                    )))
                    .ok();
            }
            KFMsg::GlobalXywhMoveDownBlurDown | KFMsg::GlobalXywhZoomOutBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::KeyGlobal(
                        IdKeyGlobal::GlobalXywhZoomIn,
                    )))
                    .ok();
            }
            KFMsg::GlobalXywhZoomInBlurDown | KFMsg::GlobalXywhHideBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::KeyGlobal(
                        IdKeyGlobal::GlobalXywhZoomOut,
                    )))
                    .ok();
            }
            KFMsg::GlobalXywhZoomOutBlurDown | KFMsg::GlobalQuitBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::KeyGlobal(
                        IdKeyGlobal::GlobalXywhHide,
                    )))
                    .ok();
            }

            // Focus of key 2 page
            KFMsg::PodcastSearchAddFeedBlurDown | KFMsg::LibraryDeleteBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::KeyOther(
                        IdKeyOther::LibraryTagEditor,
                    )))
                    .ok();
            }

            KFMsg::LibraryTagEditorBlurDown | KFMsg::LibraryLoadDirBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::KeyOther(
                        IdKeyOther::LibraryDelete,
                    )))
                    .ok();
            }

            KFMsg::LibraryDeleteBlurDown | KFMsg::LibraryYankBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::KeyOther(
                        IdKeyOther::LibraryLoadDir,
                    )))
                    .ok();
            }

            KFMsg::LibraryLoadDirBlurDown | KFMsg::LibraryPasteBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::KeyOther(
                        IdKeyOther::LibraryYank,
                    )))
                    .ok();
            }

            KFMsg::LibraryYankBlurDown | KFMsg::LibrarySearchBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::KeyOther(
                        IdKeyOther::LibraryPaste,
                    )))
                    .ok();
            }

            KFMsg::LibraryPasteBlurDown | KFMsg::LibrarySearchYoutubeBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::KeyOther(
                        IdKeyOther::LibrarySearch,
                    )))
                    .ok();
            }

            KFMsg::LibrarySearchBlurDown | KFMsg::PlaylistDeleteBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::KeyOther(
                        IdKeyOther::LibrarySearchYoutube,
                    )))
                    .ok();
            }

            KFMsg::LibrarySearchYoutubeBlurDown | KFMsg::PlaylistDeleteAllBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::KeyOther(
                        IdKeyOther::PlaylistDelete,
                    )))
                    .ok();
            }

            KFMsg::PlaylistDeleteBlurDown | KFMsg::PlaylistSearchBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::KeyOther(
                        IdKeyOther::PlaylistDeleteAll,
                    )))
                    .ok();
            }

            KFMsg::PlaylistDeleteAllBlurDown | KFMsg::PlaylistShuffleBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::KeyOther(
                        IdKeyOther::PlaylistSearch,
                    )))
                    .ok();
            }

            KFMsg::PlaylistSearchBlurDown | KFMsg::PlaylistModeCycleBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::KeyOther(
                        IdKeyOther::PlaylistShuffle,
                    )))
                    .ok();
            }

            KFMsg::PlaylistShuffleBlurDown | KFMsg::PlaylistPlaySelectedBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::KeyOther(
                        IdKeyOther::PlaylistModeCycle,
                    )))
                    .ok();
            }

            KFMsg::PlaylistModeCycleBlurDown | KFMsg::PlaylistSwapDownBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::KeyOther(
                        IdKeyOther::PlaylistPlaySelected,
                    )))
                    .ok();
            }

            KFMsg::PlaylistPlaySelectedBlurDown | KFMsg::PlaylistSwapUpBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::KeyOther(
                        IdKeyOther::PlaylistSwapDown,
                    )))
                    .ok();
            }

            KFMsg::PlaylistSwapDownBlurDown | KFMsg::DatabaseAddAllBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::KeyOther(
                        IdKeyOther::PlaylistSwapUp,
                    )))
                    .ok();
            }

            KFMsg::PlaylistSwapUpBlurDown | KFMsg::DatabaseAddSelectedBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::KeyOther(
                        IdKeyOther::DatabaseAddAll,
                    )))
                    .ok();
            }

            KFMsg::DatabaseAddAllBlurDown | KFMsg::PlaylistAddRandomAlbumBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::KeyOther(
                        IdKeyOther::DatabaseAddSelected,
                    )))
                    .ok();
            }

            KFMsg::DatabaseAddSelectedBlurDown | KFMsg::PlaylistAddRandomTracksBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::KeyOther(
                        IdKeyOther::PlaylistAddRandomAlbum,
                    )))
                    .ok();
            }

            KFMsg::PlaylistAddRandomAlbumBlurDown | KFMsg::LibrarySwitchRootBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::KeyOther(
                        IdKeyOther::PlaylistAddRandomTracks,
                    )))
                    .ok();
            }

            KFMsg::PlaylistAddRandomTracksBlurDown | KFMsg::LibraryAddRootBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::KeyOther(
                        IdKeyOther::LibrarySwitchRoot,
                    )))
                    .ok();
            }

            KFMsg::LibrarySwitchRootBlurDown | KFMsg::LibraryRemoveRootBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::KeyOther(
                        IdKeyOther::LibraryAddRoot,
                    )))
                    .ok();
            }
            KFMsg::LibraryAddRootBlurDown | KFMsg::PodcastMarkPlayedBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::KeyOther(
                        IdKeyOther::LibraryRemoveRoot,
                    )))
                    .ok();
            }

            KFMsg::LibraryRemoveRootBlurDown | KFMsg::PodcastMarkAllPlayedBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::KeyOther(
                        IdKeyOther::PodcastMarkPlayed,
                    )))
                    .ok();
            }
            KFMsg::PodcastMarkPlayedBlurDown | KFMsg::PodcastEpDownloadBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::KeyOther(
                        IdKeyOther::PodcastMarkAllPlayed,
                    )))
                    .ok();
            }
            KFMsg::PodcastMarkAllPlayedBlurDown | KFMsg::PodcastEpDeleteFileBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::KeyOther(
                        IdKeyOther::PodcastEpDownload,
                    )))
                    .ok();
            }
            KFMsg::PodcastEpDownloadBlurDown | KFMsg::PodcastDeleteFeedBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::KeyOther(
                        IdKeyOther::PodcastEpDeleteFile,
                    )))
                    .ok();
            }
            KFMsg::PodcastEpDeleteFileBlurDown | KFMsg::PodcastDeleteAllFeedsBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::KeyOther(
                        IdKeyOther::PodcastDeleteFeed,
                    )))
                    .ok();
            }
            KFMsg::PodcastDeleteFeedBlurDown | KFMsg::PodcastRefreshFeedBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::KeyOther(
                        IdKeyOther::PodcastDeleteAllFeeds,
                    )))
                    .ok();
            }
            KFMsg::PodcastDeleteAllFeedsBlurDown | KFMsg::PodcastRefreshAllFeedsBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::KeyOther(
                        IdKeyOther::PodcastRefreshFeed,
                    )))
                    .ok();
            }
            KFMsg::PodcastRefreshFeedBlurDown | KFMsg::PodcastSearchAddFeedBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::KeyOther(
                        IdKeyOther::PodcastRefreshAllFeeds,
                    )))
                    .ok();
            }
            KFMsg::PodcastRefreshAllFeedsBlurDown | KFMsg::LibraryTagEditorBlurUp => {
                self.app
                    .active(&Id::ConfigEditor(IdConfigEditor::KeyOther(
                        IdKeyOther::PodcastSearchAddFeed,
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
            IdKey::Other(IdKeyOther::DatabaseAddAll) => keys.database_keys.add_all = binding,
            IdKey::Other(IdKeyOther::DatabaseAddSelected) => {
                keys.database_keys.add_selected = binding;
            }
            IdKey::Global(IdKeyGlobal::GlobalConfig) => keys.select_view_keys.open_config = binding,
            IdKey::Global(IdKeyGlobal::GlobalDown) => keys.navigation_keys.down = binding,
            IdKey::Global(IdKeyGlobal::GlobalGotoBottom) => {
                keys.navigation_keys.goto_bottom = binding;
            }
            IdKey::Global(IdKeyGlobal::GlobalGotoTop) => keys.navigation_keys.goto_top = binding,
            IdKey::Global(IdKeyGlobal::GlobalHelp) => keys.select_view_keys.open_help = binding,
            IdKey::Global(IdKeyGlobal::GlobalLayoutTreeview) => {
                keys.select_view_keys.view_library = binding;
            }
            IdKey::Global(IdKeyGlobal::GlobalLayoutDatabase) => {
                keys.select_view_keys.view_database = binding;
            }
            IdKey::Global(IdKeyGlobal::GlobalLeft) => keys.navigation_keys.left = binding,
            IdKey::Global(IdKeyGlobal::GlobalLyricAdjustForward) => {
                keys.lyric_keys.adjust_offset_forwards = binding;
            }
            IdKey::Global(IdKeyGlobal::GlobalLyricAdjustBackward) => {
                keys.lyric_keys.adjust_offset_backwards = binding;
            }
            IdKey::Global(IdKeyGlobal::GlobalLyricCycle) => keys.lyric_keys.cycle_frames = binding,
            IdKey::Global(IdKeyGlobal::GlobalPlayerToggleGapless) => {
                keys.player_keys.toggle_prefetch = binding;
            }
            IdKey::Global(IdKeyGlobal::GlobalPlayerTogglePause) => {
                keys.player_keys.toggle_pause = binding;
            }
            IdKey::Global(IdKeyGlobal::GlobalPlayerNext) => keys.player_keys.next_track = binding,
            IdKey::Global(IdKeyGlobal::GlobalPlayerPrevious) => {
                keys.player_keys.previous_track = binding;
            }
            IdKey::Global(IdKeyGlobal::GlobalPlayerSeekForward) => {
                keys.player_keys.seek_forward = binding;
            }
            IdKey::Global(IdKeyGlobal::GlobalPlayerSeekBackward) => {
                keys.player_keys.seek_backward = binding;
            }
            IdKey::Global(IdKeyGlobal::GlobalPlayerSpeedUp) => keys.player_keys.speed_up = binding,
            IdKey::Global(IdKeyGlobal::GlobalPlayerSpeedDown) => {
                keys.player_keys.speed_down = binding;
            }
            IdKey::Global(IdKeyGlobal::GlobalQuit) => keys.quit = binding,
            IdKey::Global(IdKeyGlobal::GlobalRight) => keys.navigation_keys.right = binding,
            IdKey::Global(IdKeyGlobal::GlobalUp) => keys.navigation_keys.up = binding,
            IdKey::Global(IdKeyGlobal::GlobalPlayerVolumeDown) => {
                keys.player_keys.volume_down = binding;
            }
            IdKey::Global(IdKeyGlobal::GlobalPlayerVolumeUp) => {
                keys.player_keys.volume_up = binding;
            }
            IdKey::Global(IdKeyGlobal::GlobalSavePlaylist) => {
                keys.player_keys.save_playlist = binding;
            }
            IdKey::Other(IdKeyOther::LibraryDelete) => keys.library_keys.delete = binding,
            IdKey::Other(IdKeyOther::LibraryLoadDir) => keys.library_keys.load_dir = binding,
            IdKey::Other(IdKeyOther::LibraryPaste) => keys.library_keys.paste = binding,
            IdKey::Other(IdKeyOther::LibrarySearch) => keys.library_keys.search = binding,
            IdKey::Other(IdKeyOther::LibrarySearchYoutube) => {
                keys.library_keys.youtube_search = binding;
            }
            IdKey::Other(IdKeyOther::LibraryTagEditor) => {
                keys.library_keys.open_tag_editor = binding;
            }
            IdKey::Other(IdKeyOther::LibraryYank) => keys.library_keys.yank = binding,
            IdKey::Other(IdKeyOther::PlaylistDelete) => keys.playlist_keys.delete = binding,
            IdKey::Other(IdKeyOther::PlaylistDeleteAll) => keys.playlist_keys.delete_all = binding,
            IdKey::Other(IdKeyOther::PlaylistShuffle) => keys.playlist_keys.shuffle = binding,
            IdKey::Other(IdKeyOther::PlaylistModeCycle) => {
                keys.playlist_keys.cycle_loop_mode = binding;
            }
            IdKey::Other(IdKeyOther::PlaylistPlaySelected) => {
                keys.playlist_keys.play_selected = binding;
            }
            IdKey::Other(IdKeyOther::PlaylistSearch) => keys.playlist_keys.search = binding,
            IdKey::Other(IdKeyOther::PlaylistSwapDown) => keys.playlist_keys.swap_down = binding,
            IdKey::Other(IdKeyOther::PlaylistSwapUp) => keys.playlist_keys.swap_up = binding,
            IdKey::Other(IdKeyOther::PlaylistAddRandomAlbum) => {
                keys.playlist_keys.add_random_album = binding;
            }
            IdKey::Other(IdKeyOther::PlaylistAddRandomTracks) => {
                keys.playlist_keys.add_random_songs = binding;
            }
            IdKey::Other(IdKeyOther::LibrarySwitchRoot) => keys.library_keys.cycle_root = binding,
            IdKey::Other(IdKeyOther::LibraryAddRoot) => keys.library_keys.add_root = binding,
            IdKey::Other(IdKeyOther::LibraryRemoveRoot) => keys.library_keys.remove_root = binding,
            IdKey::Global(IdKeyGlobal::GlobalLayoutPodcast) => {
                keys.select_view_keys.view_podcasts = binding;
            }
            IdKey::Global(IdKeyGlobal::GlobalXywhMoveLeft) => {
                keys.move_cover_art_keys.move_left = binding;
            }
            IdKey::Global(IdKeyGlobal::GlobalXywhMoveRight) => {
                keys.move_cover_art_keys.move_right = binding;
            }
            IdKey::Global(IdKeyGlobal::GlobalXywhMoveUp) => {
                keys.move_cover_art_keys.move_up = binding;
            }
            IdKey::Global(IdKeyGlobal::GlobalXywhMoveDown) => {
                keys.move_cover_art_keys.move_down = binding;
            }
            IdKey::Global(IdKeyGlobal::GlobalXywhZoomIn) => {
                keys.move_cover_art_keys.increase_size = binding;
            }
            IdKey::Global(IdKeyGlobal::GlobalXywhZoomOut) => {
                keys.move_cover_art_keys.decrease_size = binding;
            }
            IdKey::Global(IdKeyGlobal::GlobalXywhHide) => {
                keys.move_cover_art_keys.toggle_hide = binding;
            }
            IdKey::Other(IdKeyOther::PodcastMarkPlayed) => keys.podcast_keys.mark_played = binding,
            IdKey::Other(IdKeyOther::PodcastMarkAllPlayed) => {
                keys.podcast_keys.mark_all_played = binding;
            }
            IdKey::Other(IdKeyOther::PodcastEpDownload) => {
                keys.podcast_keys.download_episode = binding;
            }
            IdKey::Other(IdKeyOther::PodcastEpDeleteFile) => {
                keys.podcast_keys.delete_local_episode = binding;
            }
            IdKey::Other(IdKeyOther::PodcastDeleteFeed) => keys.podcast_keys.delete_feed = binding,
            IdKey::Other(IdKeyOther::PodcastDeleteAllFeeds) => {
                keys.podcast_keys.delete_all_feeds = binding;
            }
            IdKey::Other(IdKeyOther::PodcastSearchAddFeed) => keys.podcast_keys.search = binding,
            IdKey::Other(IdKeyOther::PodcastRefreshFeed) => {
                keys.podcast_keys.refresh_feed = binding;
            }
            IdKey::Other(IdKeyOther::PodcastRefreshAllFeeds) => {
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
