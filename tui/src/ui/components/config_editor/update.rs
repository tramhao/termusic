use anyhow::Context;
use termusiclib::config::new_shared_tui_settings;
use termusiclib::config::v2::server::config_extra::ServerConfigVersionedDefaulted;
use termusiclib::config::v2::tui::config_extra::TuiConfigVersionedDefaulted;
use termusiclib::config::v2::tui::keys::KeyBinding;
use termusiclib::config::v2::tui::theme::ThemeColors;
use termusiclib::config::v2::tui::theme::styles::ColorTermusic;
use termusiclib::utils::get_app_config_path;

use crate::ui::Model;
use crate::ui::ids::{Id, IdCETheme, IdConfigEditor, IdKey, IdKeyGlobal, IdKeyOther};
use crate::ui::msg::{
    ConfigEditorMsg, GENERAL_FOCUS_ORDER, KFGLOBAL_FOCUS_ORDER, KFMsg, KFOTHER_FOCUS_ORDER, Msg,
    THEME_FOCUS_ORDER,
};
use crate::ui::tui_cmd::TuiCmd;

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
            ConfigEditorMsg::General(msg) => self.update_general(msg),
            ConfigEditorMsg::Theme(msg) => self.update_theme(msg),

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
                    IdConfigEditor::Theme(IdCETheme::LibraryHighlightSymbol) => {
                        self.config_editor.theme.style.library.highlight_symbol = symbol;
                    }
                    IdConfigEditor::Theme(IdCETheme::PlaylistHighlightSymbol) => {
                        self.config_editor.theme.style.playlist.highlight_symbol = symbol;
                    }
                    IdConfigEditor::Theme(IdCETheme::CurrentlyPlayingTrackSymbol) => {
                        self.config_editor.theme.style.playlist.current_track_symbol = symbol;
                    }
                    _ => {}
                }
            }

            ConfigEditorMsg::KeyChange(id, binding) => self.update_key(id, binding),
            ConfigEditorMsg::KeyFocusGlobal(msg) => self.update_key_focus_global(msg),
            ConfigEditorMsg::KeyFocusOther(msg) => self.update_key_focus_other(msg),
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
                            theme.file_name = Some(theme_filename.clone());

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

    /// Handle focus of the "General" tab
    fn update_general(&mut self, msg: KFMsg) {
        let focus_elem = self.app.focus().and_then(|v| {
            if let Id::ConfigEditor(IdConfigEditor::General(id)) = *v {
                Some(id)
            } else {
                None
            }
        });

        // fallback in case somehow the focus gets lost or is on a weird element, reset it to the first element
        let Some(focus_elem) = focus_elem else {
            let _ = self
                .app
                .active(&Id::ConfigEditor(GENERAL_FOCUS_ORDER[0].into()));
            return;
        };

        let focus = match msg {
            KFMsg::Next => GENERAL_FOCUS_ORDER
                .iter()
                .skip_while(|v| **v != focus_elem)
                .nth(1)
                .unwrap_or(&GENERAL_FOCUS_ORDER[0]),
            KFMsg::Previous => GENERAL_FOCUS_ORDER
                .iter()
                .rev()
                .skip_while(|v| **v != focus_elem)
                .nth(1)
                .unwrap_or(GENERAL_FOCUS_ORDER.last().unwrap()),
        };

        let _ = self.app.active(&Id::ConfigEditor(focus.into()));
    }

    /// Handle focus of the "Theme" tab
    fn update_theme(&mut self, msg: KFMsg) {
        let focus_elem = self.app.focus().and_then(|v| {
            if let Id::ConfigEditor(IdConfigEditor::Theme(id)) = *v {
                Some(id)
            } else {
                None
            }
        });

        // fallback in case somehow the focus gets lost or is on a weird element, reset it to the first element
        let Some(focus_elem) = focus_elem else {
            let _ = self
                .app
                .active(&Id::ConfigEditor(THEME_FOCUS_ORDER[0].into()));
            return;
        };

        let focus = match msg {
            KFMsg::Next => THEME_FOCUS_ORDER
                .iter()
                .skip_while(|v| **v != focus_elem)
                .nth(1)
                .unwrap_or(&THEME_FOCUS_ORDER[0]),
            KFMsg::Previous => THEME_FOCUS_ORDER
                .iter()
                .rev()
                .skip_while(|v| **v != focus_elem)
                .nth(1)
                .unwrap_or(THEME_FOCUS_ORDER.last().unwrap()),
        };

        let _ = self.app.active(&Id::ConfigEditor(focus.into()));
    }

    /// Handle focus of the "Key Global" tab
    fn update_key_focus_global(&mut self, msg: KFMsg) {
        let focus_elem = self.app.focus().and_then(|v| {
            if let Id::ConfigEditor(IdConfigEditor::KeyGlobal(key)) = *v {
                Some(IdKey::Global(key))
            } else {
                None
            }
        });

        // fallback in case somehow the focus gets lost or is on a weird element, reset it to the first element
        let Some(focus_elem) = focus_elem else {
            let _ = self
                .app
                .active(&Id::ConfigEditor(KFGLOBAL_FOCUS_ORDER[0].into()));
            return;
        };

        let focus = match msg {
            KFMsg::Next => KFGLOBAL_FOCUS_ORDER
                .iter()
                .skip_while(|v| **v != focus_elem)
                .nth(1)
                .unwrap_or(&KFGLOBAL_FOCUS_ORDER[0]),
            KFMsg::Previous => KFGLOBAL_FOCUS_ORDER
                .iter()
                .rev()
                .skip_while(|v| **v != focus_elem)
                .nth(1)
                .unwrap_or(KFGLOBAL_FOCUS_ORDER.last().unwrap()),
        };

        let _ = self.app.active(&Id::ConfigEditor(focus.into()));
    }

    /// Handle focus for the "Key Other" tab
    fn update_key_focus_other(&mut self, msg: KFMsg) {
        let focus_elem = self.app.focus().and_then(|v| {
            if let Id::ConfigEditor(IdConfigEditor::KeyOther(key)) = *v {
                Some(IdKey::Other(key))
            } else {
                None
            }
        });

        // fallback in case somehow the focus gets lost or is on a weird element, reset it to the first element
        let Some(focus_elem) = focus_elem else {
            let _ = self
                .app
                .active(&Id::ConfigEditor(KFOTHER_FOCUS_ORDER[0].into()));
            return;
        };

        let focus = match msg {
            KFMsg::Next => KFOTHER_FOCUS_ORDER
                .iter()
                .skip_while(|v| **v != focus_elem)
                .nth(1)
                .unwrap_or(&KFOTHER_FOCUS_ORDER[0]),
            KFMsg::Previous => KFOTHER_FOCUS_ORDER
                .iter()
                .rev()
                .skip_while(|v| **v != focus_elem)
                .nth(1)
                .unwrap_or(KFOTHER_FOCUS_ORDER.last().unwrap()),
        };

        let _ = self.app.active(&Id::ConfigEditor(focus.into()));
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
            IdKey::Global(IdKeyGlobal::Config) => keys.select_view_keys.open_config = binding,
            IdKey::Global(IdKeyGlobal::Down) => keys.navigation_keys.down = binding,
            IdKey::Global(IdKeyGlobal::GotoBottom) => {
                keys.navigation_keys.goto_bottom = binding;
            }
            IdKey::Global(IdKeyGlobal::GotoTop) => keys.navigation_keys.goto_top = binding,
            IdKey::Global(IdKeyGlobal::Help) => keys.select_view_keys.open_help = binding,
            IdKey::Global(IdKeyGlobal::LayoutTreeview) => {
                keys.select_view_keys.view_library = binding;
            }
            IdKey::Global(IdKeyGlobal::LayoutDatabase) => {
                keys.select_view_keys.view_database = binding;
            }
            IdKey::Global(IdKeyGlobal::Left) => keys.navigation_keys.left = binding,
            IdKey::Global(IdKeyGlobal::LyricAdjustForward) => {
                keys.lyric_keys.adjust_offset_forwards = binding;
            }
            IdKey::Global(IdKeyGlobal::LyricAdjustBackward) => {
                keys.lyric_keys.adjust_offset_backwards = binding;
            }
            IdKey::Global(IdKeyGlobal::LyricCycle) => keys.lyric_keys.cycle_frames = binding,
            IdKey::Global(IdKeyGlobal::PlayerToggleGapless) => {
                keys.player_keys.toggle_prefetch = binding;
            }
            IdKey::Global(IdKeyGlobal::PlayerTogglePause) => {
                keys.player_keys.toggle_pause = binding;
            }
            IdKey::Global(IdKeyGlobal::PlayerNext) => keys.player_keys.next_track = binding,
            IdKey::Global(IdKeyGlobal::PlayerPrevious) => {
                keys.player_keys.previous_track = binding;
            }
            IdKey::Global(IdKeyGlobal::PlayerSeekForward) => {
                keys.player_keys.seek_forward = binding;
            }
            IdKey::Global(IdKeyGlobal::PlayerSeekBackward) => {
                keys.player_keys.seek_backward = binding;
            }
            IdKey::Global(IdKeyGlobal::PlayerSpeedUp) => keys.player_keys.speed_up = binding,
            IdKey::Global(IdKeyGlobal::PlayerSpeedDown) => {
                keys.player_keys.speed_down = binding;
            }
            IdKey::Global(IdKeyGlobal::Quit) => keys.quit = binding,
            IdKey::Global(IdKeyGlobal::Right) => keys.navigation_keys.right = binding,
            IdKey::Global(IdKeyGlobal::Up) => keys.navigation_keys.up = binding,
            IdKey::Global(IdKeyGlobal::PlayerVolumeDown) => {
                keys.player_keys.volume_down = binding;
            }
            IdKey::Global(IdKeyGlobal::PlayerVolumeUp) => {
                keys.player_keys.volume_up = binding;
            }
            IdKey::Global(IdKeyGlobal::SavePlaylist) => {
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
            IdKey::Global(IdKeyGlobal::LayoutPodcast) => {
                keys.select_view_keys.view_podcasts = binding;
            }
            IdKey::Global(IdKeyGlobal::XywhMoveLeft) => {
                keys.move_cover_art_keys.move_left = binding;
            }
            IdKey::Global(IdKeyGlobal::XywhMoveRight) => {
                keys.move_cover_art_keys.move_right = binding;
            }
            IdKey::Global(IdKeyGlobal::XywhMoveUp) => {
                keys.move_cover_art_keys.move_up = binding;
            }
            IdKey::Global(IdKeyGlobal::XywhMoveDown) => {
                keys.move_cover_art_keys.move_down = binding;
            }
            IdKey::Global(IdKeyGlobal::XywhZoomIn) => {
                keys.move_cover_art_keys.increase_size = binding;
            }
            IdKey::Global(IdKeyGlobal::XywhZoomOut) => {
                keys.move_cover_art_keys.decrease_size = binding;
            }
            IdKey::Global(IdKeyGlobal::XywhHide) => {
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
            IdConfigEditor::Theme(IdCETheme::LibraryForeground) => {
                style.library.foreground_color = color_config;
            }
            IdConfigEditor::Theme(IdCETheme::LibraryBackground) => {
                style.library.background_color = color_config;
            }
            IdConfigEditor::Theme(IdCETheme::LibraryBorder) => {
                style.library.border_color = color_config;
            }
            IdConfigEditor::Theme(IdCETheme::LibraryHighlight) => {
                style.library.highlight_color = color_config;
            }
            IdConfigEditor::Theme(IdCETheme::PlaylistForeground) => {
                style.playlist.foreground_color = color_config;
            }
            IdConfigEditor::Theme(IdCETheme::PlaylistBackground) => {
                style.playlist.background_color = color_config;
            }
            IdConfigEditor::Theme(IdCETheme::PlaylistBorder) => {
                style.playlist.border_color = color_config;
            }
            IdConfigEditor::Theme(IdCETheme::PlaylistHighlight) => {
                style.playlist.highlight_color = color_config;
            }
            IdConfigEditor::Theme(IdCETheme::ProgressForeground) => {
                style.progress.foreground_color = color_config;
            }
            IdConfigEditor::Theme(IdCETheme::ProgressBackground) => {
                style.progress.background_color = color_config;
            }
            IdConfigEditor::Theme(IdCETheme::ProgressBorder) => {
                style.progress.border_color = color_config;
            }
            IdConfigEditor::Theme(IdCETheme::LyricForeground) => {
                style.lyric.foreground_color = color_config;
            }
            IdConfigEditor::Theme(IdCETheme::LyricBackground) => {
                style.lyric.background_color = color_config;
            }
            IdConfigEditor::Theme(IdCETheme::LyricBorder) => {
                style.lyric.border_color = color_config;
            }

            _ => {}
        }
    }
}
