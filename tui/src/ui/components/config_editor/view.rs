use crate::ui::Application;
use crate::ui::components::config_editor::update::THEMES_WITHOUT_FILES;
use crate::ui::components::{CEHeader, ConfigSavePopup, GlobalListener};
use crate::ui::model::{ConfigEditorLayout, Model, UserEvent};
use crate::ui::utils::draw_area_in_absolute;
use anyhow::{Result, bail};
use include_dir::DirEntry;
use std::num::{NonZeroU8, NonZeroU32};
use std::path::PathBuf;
use termusiclib::THEME_DIR;
use termusiclib::config::v2::server::{PositionYesNo, PositionYesNoLower, RememberLastPosition};
use termusiclib::config::v2::tui::Alignment as XywhAlign;
use termusiclib::ids::{Id, IdConfigEditor, IdKeyGlobal, IdKeyOther};
use termusiclib::types::Msg;
/**
 * MIT License
 *
 * tuifeed - Copyright (c) 2021 Christian Visintin
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
use termusiclib::utils::{get_app_config_path, get_pin_yin};
use tuirealm::props::{PropPayload, PropValue, TableBuilder, TextSpan};
use tuirealm::ratatui::layout::{Constraint, Layout};
use tuirealm::ratatui::widgets::Clear;
use tuirealm::{AttrValue, Attribute, Frame, State, StateValue};

impl Model {
    pub fn view_config_editor(&mut self) {
        match self.config_editor.layout {
            ConfigEditorLayout::General => self.view_config_editor_general(),
            ConfigEditorLayout::Color => self.view_config_editor_color(),
            ConfigEditorLayout::Key1 => self.view_config_editor_key1(),
            ConfigEditorLayout::Key2 => self.view_config_editor_key2(),
        }
    }

    #[allow(clippy::too_many_lines)]
    fn view_config_editor_general(&mut self) {
        self.terminal
            .raw_mut()
            .draw(|f| {
                let [header, chunks_main, footer] = Layout::vertical([
                    Constraint::Length(3), // config header
                    Constraint::Min(3),
                    Constraint::Length(1), // config footer
                ])
                .areas(f.area());

                let chunks_middle =
                    Layout::horizontal([Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)])
                        .split(chunks_main);

                let chunks_middle_left = Layout::vertical([
                    Constraint::Length(3),
                    Constraint::Length(3),
                    Constraint::Length(3),
                    Constraint::Length(3),
                    Constraint::Length(3),
                    Constraint::Length(3),
                    Constraint::Length(3),
                    Constraint::Length(3),
                    Constraint::Length(3),
                    Constraint::Min(0),
                ])
                .split(chunks_middle[0]);

                let chunks_middle_right = Layout::vertical([
                    Constraint::Length(3),
                    Constraint::Length(3),
                    Constraint::Length(3),
                    Constraint::Length(3),
                    Constraint::Length(3),
                    Constraint::Length(3),
                    Constraint::Length(3),
                    Constraint::Length(3),
                    Constraint::Length(3),
                    Constraint::Min(0),
                ])
                .split(chunks_middle[1]);
                self.app
                    .view(&Id::ConfigEditor(IdConfigEditor::Header), f, header);
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::MusicDir),
                    f,
                    chunks_middle_left[0],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::ExitConfirmation),
                    f,
                    chunks_middle_left[1],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::PlaylistDisplaySymbol),
                    f,
                    chunks_middle_left[2],
                );

                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::PlaylistRandomTrack),
                    f,
                    chunks_middle_left[3],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::PlaylistRandomAlbum),
                    f,
                    chunks_middle_left[4],
                );

                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::PodcastDir),
                    f,
                    chunks_middle_left[5],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::PodcastSimulDownload),
                    f,
                    chunks_middle_left[6],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::PodcastMaxRetries),
                    f,
                    chunks_middle_left[7],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::AlbumPhotoAlign),
                    f,
                    chunks_middle_right[0],
                );

                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::SaveLastPosition),
                    f,
                    chunks_middle_right[1],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::SeekStep),
                    f,
                    chunks_middle_right[2],
                );

                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::KillDamon),
                    f,
                    chunks_middle_right[3],
                );

                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::PlayerUseMpris),
                    f,
                    chunks_middle_right[4],
                );

                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::PlayerUseDiscord),
                    f,
                    chunks_middle_right[5],
                );

                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::PlayerPort),
                    f,
                    chunks_middle_right[6],
                );

                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::ExtraYtdlpArgs),
                    f,
                    chunks_middle_right[7],
                );

                self.app
                    .view(&Id::ConfigEditor(IdConfigEditor::Footer), f, footer);

                Self::view_config_editor_commons(f, &mut self.app);
            })
            .expect("Expected to draw without error");
    }

    fn view_config_editor_commons(f: &mut Frame<'_>, app: &mut Application<Id, Msg, UserEvent>) {
        // -- popups
        if app.mounted(&Id::ConfigEditor(IdConfigEditor::ConfigSavePopup)) {
            let popup = draw_area_in_absolute(f.area(), 50, 3);
            f.render_widget(Clear, popup);
            app.view(&Id::ConfigEditor(IdConfigEditor::ConfigSavePopup), f, popup);
        }
        if app.mounted(&Id::ErrorPopup) {
            let popup = draw_area_in_absolute(f.area(), 50, 4);
            f.render_widget(Clear, popup);
            app.view(&Id::ErrorPopup, f, popup);
        }
    }

    #[allow(clippy::too_many_lines)]
    fn view_config_editor_color(&mut self) {
        let select_library_foreground_len = match self
            .app
            .state(&Id::ConfigEditor(IdConfigEditor::LibraryForeground))
        {
            Ok(State::One(_)) => 3,
            _ => 8,
        };
        let select_library_background_len = match self
            .app
            .state(&Id::ConfigEditor(IdConfigEditor::LibraryBackground))
        {
            Ok(State::One(_)) => 3,
            _ => 8,
        };
        let select_library_border_len = match self
            .app
            .state(&Id::ConfigEditor(IdConfigEditor::LibraryBorder))
        {
            Ok(State::One(_)) => 3,
            _ => 8,
        };
        let select_library_highlight_len = match self
            .app
            .state(&Id::ConfigEditor(IdConfigEditor::LibraryHighlight))
        {
            Ok(State::One(_)) => 3,
            _ => 8,
        };

        let select_playlist_foreground_len = match self
            .app
            .state(&Id::ConfigEditor(IdConfigEditor::PlaylistForeground))
        {
            Ok(State::One(_)) => 3,
            _ => 8,
        };
        let select_playlist_background_len = match self
            .app
            .state(&Id::ConfigEditor(IdConfigEditor::PlaylistBackground))
        {
            Ok(State::One(_)) => 3,
            _ => 8,
        };
        let select_playlist_border_len = match self
            .app
            .state(&Id::ConfigEditor(IdConfigEditor::PlaylistBorder))
        {
            Ok(State::One(_)) => 3,
            _ => 8,
        };
        let select_playlist_highlight_len = match self
            .app
            .state(&Id::ConfigEditor(IdConfigEditor::PlaylistHighlight))
        {
            Ok(State::One(_)) => 3,
            _ => 8,
        };

        let select_progress_foreground_len = match self
            .app
            .state(&Id::ConfigEditor(IdConfigEditor::ProgressForeground))
        {
            Ok(State::One(_)) => 3,
            _ => 8,
        };
        let select_progress_background_len = match self
            .app
            .state(&Id::ConfigEditor(IdConfigEditor::ProgressBackground))
        {
            Ok(State::One(_)) => 3,
            _ => 8,
        };
        let select_progress_border_len = match self
            .app
            .state(&Id::ConfigEditor(IdConfigEditor::ProgressBorder))
        {
            Ok(State::One(_)) => 3,
            _ => 8,
        };

        let select_lyric_foreground_len = match self
            .app
            .state(&Id::ConfigEditor(IdConfigEditor::LyricForeground))
        {
            Ok(State::One(_)) => 3,
            _ => 8,
        };
        let select_lyric_background_len = match self
            .app
            .state(&Id::ConfigEditor(IdConfigEditor::LyricBackground))
        {
            Ok(State::One(_)) => 3,
            _ => 8,
        };
        let select_lyric_border_len = match self
            .app
            .state(&Id::ConfigEditor(IdConfigEditor::LyricBorder))
        {
            Ok(State::One(_)) => 3,
            _ => 8,
        };

        let select_important_popup_foreground_len = match self
            .app
            .state(&Id::ConfigEditor(IdConfigEditor::ImportantPopupForeground))
        {
            Ok(State::One(_)) => 3,
            _ => 8,
        };
        let select_important_popup_background_len = match self
            .app
            .state(&Id::ConfigEditor(IdConfigEditor::ImportantPopupBackground))
        {
            Ok(State::One(_)) => 3,
            _ => 8,
        };
        let select_important_popup_border_len = match self
            .app
            .state(&Id::ConfigEditor(IdConfigEditor::ImportantPopupBorder))
        {
            Ok(State::One(_)) => 3,
            _ => 8,
        };

        let select_fallback_foreground_len = match self
            .app
            .state(&Id::ConfigEditor(IdConfigEditor::FallbackForeground))
        {
            Ok(State::One(_)) => 3,
            _ => 8,
        };
        let select_fallback_background_len = match self
            .app
            .state(&Id::ConfigEditor(IdConfigEditor::FallbackBackground))
        {
            Ok(State::One(_)) => 3,
            _ => 8,
        };
        let select_fallback_border_len = match self
            .app
            .state(&Id::ConfigEditor(IdConfigEditor::FallbackBorder))
        {
            Ok(State::One(_)) => 3,
            _ => 8,
        };
        let select_fallback_highlight_len = match self
            .app
            .state(&Id::ConfigEditor(IdConfigEditor::FallbackHighlight))
        {
            Ok(State::One(_)) => 3,
            _ => 8,
        };

        self.terminal
            .raw_mut()
            .draw(|f| {
                let [header, chunks_main, footer] = Layout::vertical([
                    Constraint::Length(3), // config header
                    Constraint::Min(3),
                    Constraint::Length(1), // config footer
                ])
                .areas(f.area());

                let chunks_middle =
                    Layout::horizontal([Constraint::Ratio(1, 4), Constraint::Ratio(3, 4)])
                        .split(chunks_main);

                let chunks_style =
                    Layout::vertical([Constraint::Percentage(50), Constraint::Percentage(50)])
                        .split(chunks_middle[1]);

                let chunks_style_top = Layout::horizontal([
                    Constraint::Ratio(1, 4), // library
                    Constraint::Ratio(1, 4), // playlist
                    Constraint::Ratio(1, 4), // progress
                    Constraint::Ratio(1, 4), // lyric
                ])
                .split(chunks_style[0]);

                let chunks_style_bottom = Layout::horizontal([
                    Constraint::Ratio(1, 4), // important popup
                    Constraint::Ratio(1, 4), // unused...
                    Constraint::Ratio(1, 4),
                    Constraint::Ratio(1, 4),
                ])
                .split(chunks_style[1]);

                let chunks_library = Layout::vertical([
                    Constraint::Length(1),
                    Constraint::Length(select_library_foreground_len),
                    Constraint::Length(select_library_background_len),
                    Constraint::Length(select_library_border_len),
                    Constraint::Length(select_library_highlight_len),
                    Constraint::Length(3),
                    Constraint::Min(3),
                ])
                .split(chunks_style_top[0]);

                let chunks_playlist = Layout::vertical([
                    Constraint::Length(1),
                    Constraint::Length(select_playlist_foreground_len),
                    Constraint::Length(select_playlist_background_len),
                    Constraint::Length(select_playlist_border_len),
                    Constraint::Length(select_playlist_highlight_len),
                    Constraint::Length(3),
                    Constraint::Length(3),
                    Constraint::Min(3),
                ])
                .split(chunks_style_top[1]);

                let chunks_progress = Layout::vertical([
                    Constraint::Length(1),
                    Constraint::Length(select_progress_foreground_len),
                    Constraint::Length(select_progress_background_len),
                    Constraint::Length(select_progress_border_len),
                    Constraint::Min(3),
                ])
                .split(chunks_style_top[2]);

                let chunks_lyric = Layout::vertical([
                    Constraint::Length(1),
                    Constraint::Length(select_lyric_foreground_len),
                    Constraint::Length(select_lyric_background_len),
                    Constraint::Length(select_lyric_border_len),
                    Constraint::Min(3),
                ])
                .split(chunks_style_top[3]);

                let chunks_important_popup = Layout::vertical([
                    Constraint::Length(1),
                    Constraint::Length(select_important_popup_foreground_len),
                    Constraint::Length(select_important_popup_background_len),
                    Constraint::Length(select_important_popup_border_len),
                    Constraint::Min(3),
                ])
                .split(chunks_style_bottom[0]);

                let chunks_fallback = Layout::vertical([
                    Constraint::Length(1),
                    Constraint::Length(select_fallback_foreground_len),
                    Constraint::Length(select_fallback_background_len),
                    Constraint::Length(select_fallback_border_len),
                    Constraint::Length(select_fallback_highlight_len),
                    Constraint::Min(3),
                ])
                .split(chunks_style_bottom[1]);

                self.app
                    .view(&Id::ConfigEditor(IdConfigEditor::Header), f, header);

                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::CEThemeSelect),
                    f,
                    chunks_middle[0],
                );
                self.app
                    .view(&Id::ConfigEditor(IdConfigEditor::Footer), f, footer);

                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::LibraryLabel),
                    f,
                    chunks_library[0],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::LibraryForeground),
                    f,
                    chunks_library[1],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::LibraryBackground),
                    f,
                    chunks_library[2],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::LibraryBorder),
                    f,
                    chunks_library[3],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::LibraryHighlight),
                    f,
                    chunks_library[4],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::LibraryHighlightSymbol),
                    f,
                    chunks_library[5],
                );

                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::PlaylistLabel),
                    f,
                    chunks_playlist[0],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::PlaylistForeground),
                    f,
                    chunks_playlist[1],
                );

                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::PlaylistBackground),
                    f,
                    chunks_playlist[2],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::PlaylistBorder),
                    f,
                    chunks_playlist[3],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::PlaylistHighlight),
                    f,
                    chunks_playlist[4],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::PlaylistHighlightSymbol),
                    f,
                    chunks_playlist[5],
                );

                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::CurrentlyPlayingTrackSymbol),
                    f,
                    chunks_playlist[6],
                );

                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::ProgressLabel),
                    f,
                    chunks_progress[0],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::ProgressForeground),
                    f,
                    chunks_progress[1],
                );

                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::ProgressBackground),
                    f,
                    chunks_progress[2],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::ProgressBorder),
                    f,
                    chunks_progress[3],
                );

                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::LyricLabel),
                    f,
                    chunks_lyric[0],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::LyricForeground),
                    f,
                    chunks_lyric[1],
                );

                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::LyricBackground),
                    f,
                    chunks_lyric[2],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::LyricBorder),
                    f,
                    chunks_lyric[3],
                );

                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::ImportantPopupLabel),
                    f,
                    chunks_important_popup[0],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::ImportantPopupForeground),
                    f,
                    chunks_important_popup[1],
                );

                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::ImportantPopupBackground),
                    f,
                    chunks_important_popup[2],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::ImportantPopupBorder),
                    f,
                    chunks_important_popup[3],
                );

                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::FallbackLabel),
                    f,
                    chunks_fallback[0],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::FallbackForeground),
                    f,
                    chunks_fallback[1],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::FallbackBackground),
                    f,
                    chunks_fallback[2],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::FallbackBorder),
                    f,
                    chunks_fallback[3],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::FallbackHighlight),
                    f,
                    chunks_fallback[4],
                );

                Self::view_config_editor_commons(f, &mut self.app);
            })
            .expect("Expected to draw without error");
    }

    #[allow(clippy::too_many_lines)]
    fn view_config_editor_key1(&mut self) {
        let select_global_quit_len = match self.app.state(&Id::ConfigEditor(
            IdConfigEditor::KeyGlobal(IdKeyGlobal::Quit),
        )) {
            Ok(State::One(_)) => 3,
            _ => 8,
        };
        let select_global_left_len = match self.app.state(&Id::ConfigEditor(
            IdConfigEditor::KeyGlobal(IdKeyGlobal::Left),
        )) {
            Ok(State::One(_)) => 3,
            _ => 8,
        };
        let select_global_right_len = match self.app.state(&Id::ConfigEditor(
            IdConfigEditor::KeyGlobal(IdKeyGlobal::Right),
        )) {
            Ok(State::One(_)) => 3,
            _ => 8,
        };
        let select_global_up_len = match self.app.state(&Id::ConfigEditor(
            IdConfigEditor::KeyGlobal(IdKeyGlobal::Up),
        )) {
            Ok(State::One(_)) => 3,
            _ => 8,
        };
        let select_global_down_len = match self.app.state(&Id::ConfigEditor(
            IdConfigEditor::KeyGlobal(IdKeyGlobal::Down),
        )) {
            Ok(State::One(_)) => 3,
            _ => 8,
        };
        let select_global_goto_top_len = match self.app.state(&Id::ConfigEditor(
            IdConfigEditor::KeyGlobal(IdKeyGlobal::GotoTop),
        )) {
            Ok(State::One(_)) => 3,
            _ => 8,
        };
        let select_global_goto_bottom_len = match self.app.state(&Id::ConfigEditor(
            IdConfigEditor::KeyGlobal(IdKeyGlobal::GotoBottom),
        )) {
            Ok(State::One(_)) => 3,
            _ => 8,
        };
        let select_global_player_toggle_pause_len = match self.app.state(&Id::ConfigEditor(
            IdConfigEditor::KeyGlobal(IdKeyGlobal::PlayerTogglePause),
        )) {
            Ok(State::One(_)) => 3,
            _ => 8,
        };
        let select_global_player_next_len = match self.app.state(&Id::ConfigEditor(
            IdConfigEditor::KeyGlobal(IdKeyGlobal::PlayerNext),
        )) {
            Ok(State::One(_)) => 3,
            _ => 8,
        };
        let select_global_player_previous_len = match self.app.state(&Id::ConfigEditor(
            IdConfigEditor::KeyGlobal(IdKeyGlobal::PlayerPrevious),
        )) {
            Ok(State::One(_)) => 3,
            _ => 8,
        };

        let select_global_help_len = match self.app.state(&Id::ConfigEditor(
            IdConfigEditor::KeyGlobal(IdKeyGlobal::Help),
        )) {
            Ok(State::One(_)) => 3,
            _ => 8,
        };

        let select_global_volume_up_len = match self.app.state(&Id::ConfigEditor(
            IdConfigEditor::KeyGlobal(IdKeyGlobal::PlayerVolumeUp),
        )) {
            Ok(State::One(_)) => 3,
            _ => 8,
        };

        let select_global_volume_down_len = match self.app.state(&Id::ConfigEditor(
            IdConfigEditor::KeyGlobal(IdKeyGlobal::PlayerVolumeDown),
        )) {
            Ok(State::One(_)) => 3,
            _ => 8,
        };

        let select_global_player_seek_forward_len = match self.app.state(&Id::ConfigEditor(
            IdConfigEditor::KeyGlobal(IdKeyGlobal::PlayerSeekForward),
        )) {
            Ok(State::One(_)) => 3,
            _ => 8,
        };
        let select_global_player_seek_backward_len = match self.app.state(&Id::ConfigEditor(
            IdConfigEditor::KeyGlobal(IdKeyGlobal::PlayerSeekBackward),
        )) {
            Ok(State::One(_)) => 3,
            _ => 8,
        };
        let select_global_player_speed_up_len = match self.app.state(&Id::ConfigEditor(
            IdConfigEditor::KeyGlobal(IdKeyGlobal::PlayerSpeedUp),
        )) {
            Ok(State::One(_)) => 3,
            _ => 8,
        };
        let select_global_player_speed_down_len = match self.app.state(&Id::ConfigEditor(
            IdConfigEditor::KeyGlobal(IdKeyGlobal::PlayerSpeedDown),
        )) {
            Ok(State::One(_)) => 3,
            _ => 8,
        };

        let select_global_lyric_adjust_forward_len = match self.app.state(&Id::ConfigEditor(
            IdConfigEditor::KeyGlobal(IdKeyGlobal::LyricAdjustForward),
        )) {
            Ok(State::One(_)) => 3,
            _ => 8,
        };
        let select_global_lyric_adjust_backward_len = match self.app.state(&Id::ConfigEditor(
            IdConfigEditor::KeyGlobal(IdKeyGlobal::LyricAdjustBackward),
        )) {
            Ok(State::One(_)) => 3,
            _ => 8,
        };
        let select_global_lyric_cycle_len = match self.app.state(&Id::ConfigEditor(
            IdConfigEditor::KeyGlobal(IdKeyGlobal::LyricCycle),
        )) {
            Ok(State::One(_)) => 3,
            _ => 8,
        };
        let select_global_layout_treeview_len = match self.app.state(&Id::ConfigEditor(
            IdConfigEditor::KeyGlobal(IdKeyGlobal::LayoutTreeview),
        )) {
            Ok(State::One(_)) => 3,
            _ => 8,
        };

        let select_global_layout_database_len = match self.app.state(&Id::ConfigEditor(
            IdConfigEditor::KeyGlobal(IdKeyGlobal::LayoutDatabase),
        )) {
            Ok(State::One(_)) => 3,
            _ => 8,
        };

        let select_global_player_toggle_gapless_len = match self.app.state(&Id::ConfigEditor(
            IdConfigEditor::KeyGlobal(IdKeyGlobal::PlayerToggleGapless),
        )) {
            Ok(State::One(_)) => 3,
            _ => 8,
        };

        let select_global_config_len = match self.app.state(&Id::ConfigEditor(
            IdConfigEditor::KeyGlobal(IdKeyGlobal::Config),
        )) {
            Ok(State::One(_)) => 3,
            _ => 8,
        };

        let select_global_save_playlist = match self.app.state(&Id::ConfigEditor(
            IdConfigEditor::KeyGlobal(IdKeyGlobal::SavePlaylist),
        )) {
            Ok(State::One(_)) => 3,
            _ => 8,
        };

        let select_global_layout_podcast = match self.app.state(&Id::ConfigEditor(
            IdConfigEditor::KeyGlobal(IdKeyGlobal::LayoutPodcast),
        )) {
            Ok(State::One(_)) => 3,
            _ => 8,
        };

        let select_global_xywh_move_left = match self.app.state(&Id::ConfigEditor(
            IdConfigEditor::KeyGlobal(IdKeyGlobal::XywhMoveLeft),
        )) {
            Ok(State::One(_)) => 3,
            _ => 8,
        };

        let select_global_xywh_move_right = match self.app.state(&Id::ConfigEditor(
            IdConfigEditor::KeyGlobal(IdKeyGlobal::XywhMoveRight),
        )) {
            Ok(State::One(_)) => 3,
            _ => 8,
        };
        let select_global_xywh_move_up = match self.app.state(&Id::ConfigEditor(
            IdConfigEditor::KeyGlobal(IdKeyGlobal::XywhMoveUp),
        )) {
            Ok(State::One(_)) => 3,
            _ => 8,
        };
        let select_global_xywh_move_down = match self.app.state(&Id::ConfigEditor(
            IdConfigEditor::KeyGlobal(IdKeyGlobal::XywhMoveDown),
        )) {
            Ok(State::One(_)) => 3,
            _ => 8,
        };
        let select_global_xywh_zoom_in = match self.app.state(&Id::ConfigEditor(
            IdConfigEditor::KeyGlobal(IdKeyGlobal::XywhZoomIn),
        )) {
            Ok(State::One(_)) => 3,
            _ => 8,
        };
        let select_global_xywh_zoom_out = match self.app.state(&Id::ConfigEditor(
            IdConfigEditor::KeyGlobal(IdKeyGlobal::XywhZoomOut),
        )) {
            Ok(State::One(_)) => 3,
            _ => 8,
        };
        let select_global_xywh_hide = match self.app.state(&Id::ConfigEditor(
            IdConfigEditor::KeyGlobal(IdKeyGlobal::XywhHide),
        )) {
            Ok(State::One(_)) => 3,
            _ => 8,
        };

        self.terminal
            .raw_mut()
            .draw(|f| {
                let [header, chunks_main, footer] = Layout::vertical([
                    Constraint::Length(3), // config header
                    Constraint::Min(3),
                    Constraint::Length(1), // config footer
                ])
                .areas(f.area());

                let chunks_middle = Layout::horizontal([
                    Constraint::Ratio(1, 4),
                    Constraint::Ratio(1, 4),
                    Constraint::Ratio(1, 4),
                    Constraint::Ratio(1, 4),
                ])
                .split(chunks_main);

                let chunks_middle_column1 = Layout::vertical([
                    Constraint::Length(select_global_quit_len),
                    Constraint::Length(select_global_left_len),
                    Constraint::Length(select_global_down_len),
                    Constraint::Length(select_global_up_len),
                    Constraint::Length(select_global_right_len),
                    Constraint::Length(select_global_goto_top_len),
                    Constraint::Length(select_global_goto_bottom_len),
                    Constraint::Length(select_global_player_toggle_pause_len),
                    Constraint::Length(select_global_player_next_len),
                    Constraint::Min(0),
                ])
                .split(chunks_middle[0]);

                let chunks_middle_column2 = Layout::vertical([
                    Constraint::Length(select_global_player_previous_len),
                    Constraint::Length(select_global_help_len),
                    Constraint::Length(select_global_volume_up_len),
                    Constraint::Length(select_global_volume_down_len),
                    Constraint::Length(select_global_player_seek_forward_len),
                    Constraint::Length(select_global_player_seek_backward_len),
                    Constraint::Length(select_global_player_speed_up_len),
                    Constraint::Length(select_global_player_speed_down_len),
                    Constraint::Length(select_global_lyric_adjust_forward_len),
                    Constraint::Min(0),
                ])
                .split(chunks_middle[1]);
                let chunks_middle_column3 = Layout::vertical([
                    Constraint::Length(select_global_lyric_adjust_backward_len),
                    Constraint::Length(select_global_lyric_cycle_len),
                    Constraint::Length(select_global_layout_treeview_len),
                    Constraint::Length(select_global_layout_database_len),
                    Constraint::Length(select_global_player_toggle_gapless_len),
                    Constraint::Length(select_global_config_len),
                    Constraint::Length(select_global_save_playlist),
                    Constraint::Length(select_global_layout_podcast),
                    Constraint::Length(select_global_xywh_move_left),
                    Constraint::Min(0),
                ])
                .split(chunks_middle[2]);

                let chunks_middle_column4 = Layout::vertical([
                    Constraint::Length(select_global_xywh_move_right),
                    Constraint::Length(select_global_xywh_move_up),
                    Constraint::Length(select_global_xywh_move_down),
                    Constraint::Length(select_global_xywh_zoom_in),
                    Constraint::Length(select_global_xywh_zoom_out),
                    Constraint::Length(select_global_xywh_hide),
                    Constraint::Min(0),
                ])
                .split(chunks_middle[3]);

                self.app
                    .view(&Id::ConfigEditor(IdConfigEditor::Header), f, header);
                self.app
                    .view(&Id::ConfigEditor(IdConfigEditor::Footer), f, footer);

                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::KeyGlobal(IdKeyGlobal::Quit)),
                    f,
                    chunks_middle_column1[0],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::KeyGlobal(IdKeyGlobal::Left)),
                    f,
                    chunks_middle_column1[1],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::KeyGlobal(IdKeyGlobal::Down)),
                    f,
                    chunks_middle_column1[2],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::KeyGlobal(IdKeyGlobal::Up)),
                    f,
                    chunks_middle_column1[3],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::KeyGlobal(IdKeyGlobal::Right)),
                    f,
                    chunks_middle_column1[4],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::KeyGlobal(IdKeyGlobal::GotoTop)),
                    f,
                    chunks_middle_column1[5],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::KeyGlobal(IdKeyGlobal::GotoBottom)),
                    f,
                    chunks_middle_column1[6],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::KeyGlobal(IdKeyGlobal::PlayerTogglePause)),
                    f,
                    chunks_middle_column1[7],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::KeyGlobal(IdKeyGlobal::PlayerNext)),
                    f,
                    chunks_middle_column1[8],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::KeyGlobal(IdKeyGlobal::PlayerPrevious)),
                    f,
                    chunks_middle_column2[0],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::KeyGlobal(IdKeyGlobal::Help)),
                    f,
                    chunks_middle_column2[1],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::KeyGlobal(IdKeyGlobal::PlayerVolumeUp)),
                    f,
                    chunks_middle_column2[2],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::KeyGlobal(IdKeyGlobal::PlayerVolumeDown)),
                    f,
                    chunks_middle_column2[3],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::KeyGlobal(IdKeyGlobal::PlayerSeekForward)),
                    f,
                    chunks_middle_column2[4],
                );

                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::KeyGlobal(IdKeyGlobal::PlayerSeekBackward)),
                    f,
                    chunks_middle_column2[5],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::KeyGlobal(IdKeyGlobal::PlayerSpeedUp)),
                    f,
                    chunks_middle_column2[6],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::KeyGlobal(IdKeyGlobal::PlayerSpeedDown)),
                    f,
                    chunks_middle_column2[7],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::KeyGlobal(IdKeyGlobal::LyricAdjustForward)),
                    f,
                    chunks_middle_column2[8],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::KeyGlobal(IdKeyGlobal::LyricAdjustBackward)),
                    f,
                    chunks_middle_column3[0],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::KeyGlobal(IdKeyGlobal::LyricCycle)),
                    f,
                    chunks_middle_column3[1],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::KeyGlobal(IdKeyGlobal::LayoutTreeview)),
                    f,
                    chunks_middle_column3[2],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::KeyGlobal(IdKeyGlobal::LayoutDatabase)),
                    f,
                    chunks_middle_column3[3],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::KeyGlobal(IdKeyGlobal::PlayerToggleGapless)),
                    f,
                    chunks_middle_column3[4],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::KeyGlobal(IdKeyGlobal::Config)),
                    f,
                    chunks_middle_column3[5],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::KeyGlobal(IdKeyGlobal::SavePlaylist)),
                    f,
                    chunks_middle_column3[6],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::KeyGlobal(IdKeyGlobal::LayoutPodcast)),
                    f,
                    chunks_middle_column3[7],
                );

                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::KeyGlobal(IdKeyGlobal::XywhMoveLeft)),
                    f,
                    chunks_middle_column3[8],
                );

                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::KeyGlobal(IdKeyGlobal::XywhMoveRight)),
                    f,
                    chunks_middle_column4[0],
                );

                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::KeyGlobal(IdKeyGlobal::XywhMoveUp)),
                    f,
                    chunks_middle_column4[1],
                );

                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::KeyGlobal(IdKeyGlobal::XywhMoveDown)),
                    f,
                    chunks_middle_column4[2],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::KeyGlobal(IdKeyGlobal::XywhZoomIn)),
                    f,
                    chunks_middle_column4[3],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::KeyGlobal(IdKeyGlobal::XywhZoomOut)),
                    f,
                    chunks_middle_column4[4],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::KeyGlobal(IdKeyGlobal::XywhHide)),
                    f,
                    chunks_middle_column4[5],
                );
                Self::view_config_editor_commons(f, &mut self.app);
            })
            .expect("Expected to draw without error");
    }

    #[allow(clippy::too_many_lines)]
    fn view_config_editor_key2(&mut self) {
        let select_library_delete_len = match self.app.state(&Id::ConfigEditor(
            IdConfigEditor::KeyOther(IdKeyOther::LibraryDelete),
        )) {
            Ok(State::One(_)) => 3,
            _ => 8,
        };
        let select_library_load_dir_len = match self.app.state(&Id::ConfigEditor(
            IdConfigEditor::KeyOther(IdKeyOther::LibraryLoadDir),
        )) {
            Ok(State::One(_)) => 3,
            _ => 8,
        };
        let select_library_yank_len = match self.app.state(&Id::ConfigEditor(
            IdConfigEditor::KeyOther(IdKeyOther::LibraryYank),
        )) {
            Ok(State::One(_)) => 3,
            _ => 8,
        };
        let select_library_paste_len = match self.app.state(&Id::ConfigEditor(
            IdConfigEditor::KeyOther(IdKeyOther::LibraryPaste),
        )) {
            Ok(State::One(_)) => 3,
            _ => 8,
        };
        let select_library_search_len = match self.app.state(&Id::ConfigEditor(
            IdConfigEditor::KeyOther(IdKeyOther::LibrarySearch),
        )) {
            Ok(State::One(_)) => 3,
            _ => 8,
        };
        let select_library_search_youtube_len = match self.app.state(&Id::ConfigEditor(
            IdConfigEditor::KeyOther(IdKeyOther::LibrarySearchYoutube),
        )) {
            Ok(State::One(_)) => 3,
            _ => 8,
        };
        let select_library_tag_editor_len = match self.app.state(&Id::ConfigEditor(
            IdConfigEditor::KeyOther(IdKeyOther::LibraryTagEditor),
        )) {
            Ok(State::One(_)) => 3,
            _ => 8,
        };
        let select_playlist_delete_len = match self.app.state(&Id::ConfigEditor(
            IdConfigEditor::KeyOther(IdKeyOther::PlaylistDelete),
        )) {
            Ok(State::One(_)) => 3,
            _ => 8,
        };
        let select_playlist_delete_all_len = match self.app.state(&Id::ConfigEditor(
            IdConfigEditor::KeyOther(IdKeyOther::PlaylistDeleteAll),
        )) {
            Ok(State::One(_)) => 3,
            _ => 8,
        };

        let select_playlist_shuffle_len = match self.app.state(&Id::ConfigEditor(
            IdConfigEditor::KeyOther(IdKeyOther::PlaylistShuffle),
        )) {
            Ok(State::One(_)) => 3,
            _ => 8,
        };

        let select_playlist_mode_cycle_len = match self.app.state(&Id::ConfigEditor(
            IdConfigEditor::KeyOther(IdKeyOther::PlaylistModeCycle),
        )) {
            Ok(State::One(_)) => 3,
            _ => 8,
        };
        let select_playlist_search_len = match self.app.state(&Id::ConfigEditor(
            IdConfigEditor::KeyOther(IdKeyOther::PlaylistSearch),
        )) {
            Ok(State::One(_)) => 3,
            _ => 8,
        };
        let select_playlist_play_selected_len = match self.app.state(&Id::ConfigEditor(
            IdConfigEditor::KeyOther(IdKeyOther::PlaylistPlaySelected),
        )) {
            Ok(State::One(_)) => 3,
            _ => 8,
        };

        let select_playlist_swap_down_len = match self.app.state(&Id::ConfigEditor(
            IdConfigEditor::KeyOther(IdKeyOther::PlaylistSwapDown),
        )) {
            Ok(State::One(_)) => 3,
            _ => 8,
        };

        let select_playlist_swap_up_len = match self.app.state(&Id::ConfigEditor(
            IdConfigEditor::KeyOther(IdKeyOther::PlaylistSwapUp),
        )) {
            Ok(State::One(_)) => 3,
            _ => 8,
        };

        let select_database_add_all_len = match self.app.state(&Id::ConfigEditor(
            IdConfigEditor::KeyOther(IdKeyOther::DatabaseAddAll),
        )) {
            Ok(State::One(_)) => 3,
            _ => 8,
        };

        let select_database_add_selected_len = match self.app.state(&Id::ConfigEditor(
            IdConfigEditor::KeyOther(IdKeyOther::DatabaseAddSelected),
        )) {
            Ok(State::One(_)) => 3,
            _ => 8,
        };

        let select_playlist_random_album_len = match self.app.state(&Id::ConfigEditor(
            IdConfigEditor::KeyOther(IdKeyOther::PlaylistAddRandomAlbum),
        )) {
            Ok(State::One(_)) => 3,
            _ => 8,
        };

        let select_playlist_random_tracks_len = match self.app.state(&Id::ConfigEditor(
            IdConfigEditor::KeyOther(IdKeyOther::PlaylistAddRandomTracks),
        )) {
            Ok(State::One(_)) => 3,
            _ => 8,
        };

        let library_switch_root_len = match self.app.state(&Id::ConfigEditor(
            IdConfigEditor::KeyOther(IdKeyOther::LibrarySwitchRoot),
        )) {
            Ok(State::One(_)) => 3,
            _ => 8,
        };

        let library_add_root_len = match self.app.state(&Id::ConfigEditor(
            IdConfigEditor::KeyOther(IdKeyOther::LibraryAddRoot),
        )) {
            Ok(State::One(_)) => 3,
            _ => 8,
        };

        let library_remove_root_len = match self.app.state(&Id::ConfigEditor(
            IdConfigEditor::KeyOther(IdKeyOther::LibraryRemoveRoot),
        )) {
            Ok(State::One(_)) => 3,
            _ => 8,
        };

        let podcast_mark_played_len = match self.app.state(&Id::ConfigEditor(
            IdConfigEditor::KeyOther(IdKeyOther::PodcastMarkPlayed),
        )) {
            Ok(State::One(_)) => 3,
            _ => 8,
        };

        let podcast_mark_all_played_len = match self.app.state(&Id::ConfigEditor(
            IdConfigEditor::KeyOther(IdKeyOther::PodcastMarkAllPlayed),
        )) {
            Ok(State::One(_)) => 3,
            _ => 8,
        };

        let podcast_ep_download_len = match self.app.state(&Id::ConfigEditor(
            IdConfigEditor::KeyOther(IdKeyOther::PodcastEpDownload),
        )) {
            Ok(State::One(_)) => 3,
            _ => 8,
        };

        let podcast_ep_delete_file_len = match self.app.state(&Id::ConfigEditor(
            IdConfigEditor::KeyOther(IdKeyOther::PodcastEpDeleteFile),
        )) {
            Ok(State::One(_)) => 3,
            _ => 8,
        };

        let podcast_delete_feed_len = match self.app.state(&Id::ConfigEditor(
            IdConfigEditor::KeyOther(IdKeyOther::PodcastDeleteFeed),
        )) {
            Ok(State::One(_)) => 3,
            _ => 8,
        };

        let podcast_delete_all_feeds_len = match self.app.state(&Id::ConfigEditor(
            IdConfigEditor::KeyOther(IdKeyOther::PodcastDeleteAllFeeds),
        )) {
            Ok(State::One(_)) => 3,
            _ => 8,
        };

        let podcast_search_add_feed_len = match self.app.state(&Id::ConfigEditor(
            IdConfigEditor::KeyOther(IdKeyOther::PodcastSearchAddFeed),
        )) {
            Ok(State::One(_)) => 3,
            _ => 8,
        };

        let podcast_refresh_feed_len = match self.app.state(&Id::ConfigEditor(
            IdConfigEditor::KeyOther(IdKeyOther::PodcastRefreshFeed),
        )) {
            Ok(State::One(_)) => 3,
            _ => 8,
        };

        let podcast_refresh_all_feeds_len = match self.app.state(&Id::ConfigEditor(
            IdConfigEditor::KeyOther(IdKeyOther::PodcastRefreshAllFeeds),
        )) {
            Ok(State::One(_)) => 3,
            _ => 8,
        };

        self.terminal
            .raw_mut()
            .draw(|f| {
                let [header, chunks_main, footer] = Layout::vertical([
                    Constraint::Length(3), // config header
                    Constraint::Min(3),
                    Constraint::Length(1), // config footer
                ])
                .areas(f.area());

                let chunks_middle = Layout::horizontal([
                    Constraint::Ratio(1, 4),
                    Constraint::Ratio(1, 4),
                    Constraint::Ratio(1, 4),
                    Constraint::Ratio(1, 4),
                ])
                .split(chunks_main);

                let chunks_middle_column1 = Layout::vertical([
                    Constraint::Length(select_library_tag_editor_len),
                    Constraint::Length(select_library_delete_len),
                    Constraint::Length(select_library_load_dir_len),
                    Constraint::Length(select_library_yank_len),
                    Constraint::Length(select_library_paste_len),
                    Constraint::Length(select_library_search_len),
                    Constraint::Length(select_library_search_youtube_len),
                    Constraint::Length(select_playlist_delete_len),
                    Constraint::Length(select_playlist_delete_all_len),
                    Constraint::Min(0),
                ])
                .split(chunks_middle[0]);
                let chunks_middle_column2 = Layout::vertical([
                    Constraint::Length(select_playlist_search_len),
                    Constraint::Length(select_playlist_shuffle_len),
                    Constraint::Length(select_playlist_mode_cycle_len),
                    Constraint::Length(select_playlist_play_selected_len),
                    Constraint::Length(select_playlist_swap_down_len),
                    Constraint::Length(select_playlist_swap_up_len),
                    Constraint::Length(select_database_add_all_len),
                    Constraint::Length(select_database_add_selected_len),
                    Constraint::Length(select_playlist_random_album_len),
                    Constraint::Min(0),
                ])
                .split(chunks_middle[1]);

                let chunks_middle_column3 = Layout::vertical([
                    Constraint::Length(select_playlist_random_tracks_len),
                    Constraint::Length(library_switch_root_len),
                    Constraint::Length(library_add_root_len),
                    Constraint::Length(library_remove_root_len),
                    Constraint::Length(podcast_mark_played_len),
                    Constraint::Length(podcast_mark_all_played_len),
                    Constraint::Length(podcast_ep_download_len),
                    Constraint::Length(podcast_ep_delete_file_len),
                    Constraint::Length(podcast_delete_feed_len),
                    Constraint::Min(0),
                ])
                .split(chunks_middle[2]);

                let chunks_middle_column4 = Layout::vertical([
                    Constraint::Length(podcast_delete_all_feeds_len),
                    Constraint::Length(podcast_refresh_feed_len),
                    Constraint::Length(podcast_refresh_all_feeds_len),
                    Constraint::Length(podcast_search_add_feed_len),
                    Constraint::Min(0),
                ])
                .split(chunks_middle[3]);

                self.app
                    .view(&Id::ConfigEditor(IdConfigEditor::Header), f, header);
                self.app
                    .view(&Id::ConfigEditor(IdConfigEditor::Footer), f, footer);

                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::KeyOther(IdKeyOther::LibraryTagEditor)),
                    f,
                    chunks_middle_column1[0],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::KeyOther(IdKeyOther::LibraryDelete)),
                    f,
                    chunks_middle_column1[1],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::KeyOther(IdKeyOther::LibraryLoadDir)),
                    f,
                    chunks_middle_column1[2],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::KeyOther(IdKeyOther::LibraryYank)),
                    f,
                    chunks_middle_column1[3],
                );

                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::KeyOther(IdKeyOther::LibraryPaste)),
                    f,
                    chunks_middle_column1[4],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::KeyOther(IdKeyOther::LibrarySearch)),
                    f,
                    chunks_middle_column1[5],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::KeyOther(IdKeyOther::LibrarySearchYoutube)),
                    f,
                    chunks_middle_column1[6],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::KeyOther(IdKeyOther::PlaylistDelete)),
                    f,
                    chunks_middle_column1[7],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::KeyOther(IdKeyOther::PlaylistDeleteAll)),
                    f,
                    chunks_middle_column1[8],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::KeyOther(IdKeyOther::PlaylistSearch)),
                    f,
                    chunks_middle_column2[0],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::KeyOther(IdKeyOther::PlaylistShuffle)),
                    f,
                    chunks_middle_column2[1],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::KeyOther(IdKeyOther::PlaylistModeCycle)),
                    f,
                    chunks_middle_column2[2],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::KeyOther(IdKeyOther::PlaylistPlaySelected)),
                    f,
                    chunks_middle_column2[3],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::KeyOther(IdKeyOther::PlaylistSwapDown)),
                    f,
                    chunks_middle_column2[4],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::KeyOther(IdKeyOther::PlaylistSwapUp)),
                    f,
                    chunks_middle_column2[5],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::KeyOther(IdKeyOther::DatabaseAddAll)),
                    f,
                    chunks_middle_column2[6],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::KeyOther(IdKeyOther::DatabaseAddSelected)),
                    f,
                    chunks_middle_column2[7],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::KeyOther(IdKeyOther::PlaylistAddRandomAlbum)),
                    f,
                    chunks_middle_column2[8],
                );

                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::KeyOther(
                        IdKeyOther::PlaylistAddRandomTracks,
                    )),
                    f,
                    chunks_middle_column3[0],
                );

                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::KeyOther(IdKeyOther::LibrarySwitchRoot)),
                    f,
                    chunks_middle_column3[1],
                );

                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::KeyOther(IdKeyOther::LibraryAddRoot)),
                    f,
                    chunks_middle_column3[2],
                );

                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::KeyOther(IdKeyOther::LibraryRemoveRoot)),
                    f,
                    chunks_middle_column3[3],
                );

                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::KeyOther(IdKeyOther::PodcastMarkPlayed)),
                    f,
                    chunks_middle_column3[4],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::KeyOther(IdKeyOther::PodcastMarkAllPlayed)),
                    f,
                    chunks_middle_column3[5],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::KeyOther(IdKeyOther::PodcastEpDownload)),
                    f,
                    chunks_middle_column3[6],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::KeyOther(IdKeyOther::PodcastEpDeleteFile)),
                    f,
                    chunks_middle_column3[7],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::KeyOther(IdKeyOther::PodcastDeleteFeed)),
                    f,
                    chunks_middle_column3[8],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::KeyOther(IdKeyOther::PodcastDeleteAllFeeds)),
                    f,
                    chunks_middle_column4[0],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::KeyOther(IdKeyOther::PodcastRefreshFeed)),
                    f,
                    chunks_middle_column4[1],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::KeyOther(IdKeyOther::PodcastRefreshAllFeeds)),
                    f,
                    chunks_middle_column4[2],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::KeyOther(IdKeyOther::PodcastSearchAddFeed)),
                    f,
                    chunks_middle_column4[3],
                );
                Self::view_config_editor_commons(f, &mut self.app);
            })
            .expect("Expected to draw without error");
    }

    pub fn mount_config_editor(&mut self) {
        self.config_editor.layout = ConfigEditorLayout::General;

        self.remount_config_header_footer().unwrap();

        self.remount_config_general().unwrap();

        self.remount_config_color(&self.config_tui.clone(), None)
            .unwrap();

        self.remount_config_keys().unwrap();

        // Active Config Editor
        assert!(
            self.app
                .active(&Id::ConfigEditor(IdConfigEditor::MusicDir))
                .is_ok()
        );

        if let Err(e) = self.theme_select_load_themes() {
            self.mount_error_popup(e.context("load themes"));
        }
        self.theme_select_sync(None);
        if let Err(e) = self.update_photo() {
            self.mount_error_popup(e.context("update_photo"));
        }
    }

    pub fn umount_config_editor(&mut self) {
        self.library_scan_dir(&self.library.tree_path, None);
        self.playlist_reload();
        self.database_reload();
        self.progress_reload();
        self.mount_label_help();
        self.lyric_reload();

        self.umount_config_header_footer().unwrap();

        self.umount_config_general().unwrap();

        self.umount_config_color().unwrap();

        self.umount_config_keys().unwrap();

        assert!(
            self.app
                .remount(
                    Id::GlobalListener,
                    Box::new(GlobalListener::new(self.config_tui.clone())),
                    Self::subscribe(&self.config_tui.read().settings.keys),
                )
                .is_ok()
        );

        if let Err(e) = self.update_photo() {
            self.mount_error_popup(e.context("update_photo"));
        }
    }

    pub fn action_change_layout(&mut self) {
        match self.config_editor.layout {
            ConfigEditorLayout::General => self.config_editor.layout = ConfigEditorLayout::Color,

            ConfigEditorLayout::Color => self.config_editor.layout = ConfigEditorLayout::Key1,
            ConfigEditorLayout::Key1 => self.config_editor.layout = ConfigEditorLayout::Key2,
            ConfigEditorLayout::Key2 => self.config_editor.layout = ConfigEditorLayout::General,
        }

        assert!(
            self.app
                .remount(
                    Id::ConfigEditor(IdConfigEditor::Header),
                    Box::new(CEHeader::new(
                        self.config_editor.layout,
                        &self.config_tui.read()
                    )),
                    vec![]
                )
                .is_ok()
        );
        match self.config_editor.layout {
            ConfigEditorLayout::General => self
                .app
                .active(&Id::ConfigEditor(IdConfigEditor::MusicDir))
                .ok(),
            ConfigEditorLayout::Color => self
                .app
                .active(&Id::ConfigEditor(IdConfigEditor::CEThemeSelect))
                .ok(),
            ConfigEditorLayout::Key1 => self
                .app
                .active(&Id::ConfigEditor(IdConfigEditor::KeyGlobal(
                    IdKeyGlobal::Quit,
                )))
                .ok(),
            ConfigEditorLayout::Key2 => self
                .app
                .active(&Id::ConfigEditor(IdConfigEditor::KeyOther(
                    IdKeyOther::LibraryTagEditor,
                )))
                .ok(),
        };
    }

    /// Mount quit popup
    pub fn mount_config_save_popup(&mut self) {
        assert!(
            self.app
                .remount(
                    Id::ConfigEditor(IdConfigEditor::ConfigSavePopup),
                    Box::new(ConfigSavePopup::new(self.config_tui.clone())),
                    vec![]
                )
                .is_ok()
        );
        assert!(
            self.app
                .active(&Id::ConfigEditor(IdConfigEditor::ConfigSavePopup))
                .is_ok()
        );
    }

    #[allow(clippy::too_many_lines)]
    pub fn collect_config_data(&mut self) -> Result<()> {
        let mut config_tui = self.config_tui.write();
        match self.config_editor.key_config.check_keys() {
            Ok(()) => config_tui.settings.keys = self.config_editor.key_config.clone(),
            Err(err) => bail!(err),
        }
        config_tui.settings.theme = self.config_editor.theme.clone();

        let mut config_server = self.config_server.write();

        if let Ok(State::One(StateValue::String(music_dir))) =
            self.app.state(&Id::ConfigEditor(IdConfigEditor::MusicDir))
        {
            // config.music_dir = music_dir;
            // let mut vec = Vec::new();
            let vec = music_dir
                .split(';')
                .map(PathBuf::from)
                .filter(|p| {
                    let absolute_dir = shellexpand::path::tilde(p);
                    absolute_dir.exists()
                })
                .collect();
            config_server.settings.player.music_dirs = vec;
        }

        if let Ok(State::One(StateValue::Usize(exit_confirmation))) = self
            .app
            .state(&Id::ConfigEditor(IdConfigEditor::ExitConfirmation))
        {
            config_tui.settings.behavior.confirm_quit = matches!(exit_confirmation, 0);
        }

        if let Ok(State::One(StateValue::Usize(display_symbol))) = self
            .app
            .state(&Id::ConfigEditor(IdConfigEditor::PlaylistDisplaySymbol))
        {
            config_tui
                .settings
                .theme
                .style
                .playlist
                .use_loop_mode_symbol = matches!(display_symbol, 0);
        }

        if let Ok(State::One(StateValue::String(random_track_quantity_str))) = self
            .app
            .state(&Id::ConfigEditor(IdConfigEditor::PlaylistRandomTrack))
        {
            if let Ok(quantity) = random_track_quantity_str.parse::<NonZeroU32>() {
                config_server.settings.player.random_track_quantity = quantity;
            }
        }

        if let Ok(State::One(StateValue::String(random_album_quantity_str))) = self
            .app
            .state(&Id::ConfigEditor(IdConfigEditor::PlaylistRandomAlbum))
        {
            if let Ok(quantity) = random_album_quantity_str.parse::<NonZeroU32>() {
                config_server.settings.player.random_album_min_quantity = quantity;
            }
        }

        if let Ok(State::One(StateValue::String(podcast_dir))) = self
            .app
            .state(&Id::ConfigEditor(IdConfigEditor::PodcastDir))
        {
            let absolute_dir = shellexpand::path::tilde(&podcast_dir);
            if absolute_dir.exists() {
                config_server.settings.podcast.download_dir = absolute_dir.into_owned();
            }
        }
        if let Ok(State::One(StateValue::String(podcast_simul_download))) = self
            .app
            .state(&Id::ConfigEditor(IdConfigEditor::PodcastSimulDownload))
        {
            if let Ok(quantity) = podcast_simul_download.parse::<NonZeroU8>() {
                config_server.settings.podcast.concurrent_downloads_max = quantity;
            }
        }
        if let Ok(State::One(StateValue::String(podcast_max_retries))) = self
            .app
            .state(&Id::ConfigEditor(IdConfigEditor::PodcastMaxRetries))
        {
            if let Ok(quantity) = podcast_max_retries.parse::<u8>() {
                if (1..11).contains(&quantity) {
                    config_server.settings.podcast.max_download_retries = quantity;
                } else {
                    bail!(" It's not recommended to set max retries to more than 10. ");
                }
            }
        }
        if let Ok(State::One(StateValue::Usize(align))) = self
            .app
            .state(&Id::ConfigEditor(IdConfigEditor::AlbumPhotoAlign))
        {
            let align = match align {
                0 => XywhAlign::BottomRight,
                1 => XywhAlign::BottomLeft,
                2 => XywhAlign::TopRight,
                _ => XywhAlign::TopLeft,
            };
            config_tui.settings.coverart.align = align;
        }

        if let Ok(State::One(StateValue::Usize(save_last_position))) = self
            .app
            .state(&Id::ConfigEditor(IdConfigEditor::SaveLastPosition))
        {
            // NOTE: value "0" means to not save the value
            if save_last_position != 0 {
                let new_val = match save_last_position {
                    1 => RememberLastPosition::All(PositionYesNo::Simple(PositionYesNoLower::No)),
                    2 => RememberLastPosition::All(PositionYesNo::Simple(PositionYesNoLower::Yes)),
                    // only 0,1,2 exist here
                    _ => unreachable!(),
                };

                config_server.settings.player.remember_position = new_val;
            }

            // let save_last_position = match save_last_position {
            //     0 => LastPosition::Auto,
            //     1 => LastPosition::No,
            //     2 => LastPosition::Yes,
            //     _ => bail!(" Save last position must be set to auto, yes or no."),
            // };
            // config_server.settings.player.remember_position = save_last_position;
        }

        if let Ok(State::One(StateValue::Usize(seek_step))) =
            self.app.state(&Id::ConfigEditor(IdConfigEditor::SeekStep))
        {
            // NOTE: seek_step is currently unsupported to be set
            let _ = seek_step;

            // let seek_step = match seek_step {
            //     0 => SeekStep::Auto,
            //     1 => SeekStep::Short,
            //     2 => SeekStep::Long,
            //     _ => bail!(" Unknown player step length provided."),
            // };
            // config_server.settings.player.seek_step = seek_step;
        }

        if let Ok(State::One(StateValue::Usize(kill_daemon))) =
            self.app.state(&Id::ConfigEditor(IdConfigEditor::KillDamon))
        {
            config_tui.settings.behavior.quit_server_on_exit = matches!(kill_daemon, 0);
        }

        if let Ok(State::One(StateValue::Usize(player_use_mpris))) = self
            .app
            .state(&Id::ConfigEditor(IdConfigEditor::PlayerUseMpris))
        {
            config_server.settings.player.use_mediacontrols = matches!(player_use_mpris, 0);
        }

        if let Ok(State::One(StateValue::Usize(player_use_discord))) = self
            .app
            .state(&Id::ConfigEditor(IdConfigEditor::PlayerUseDiscord))
        {
            config_server.settings.player.set_discord_status = matches!(player_use_discord, 0);
        }

        if let Ok(State::One(StateValue::String(player_port))) = self
            .app
            .state(&Id::ConfigEditor(IdConfigEditor::PlayerPort))
        {
            if let Ok(port) = player_port.parse::<u16>() {
                if (1024..u16::MAX).contains(&port) {
                    config_server.settings.com.port = port;
                } else {
                    bail!(" It's not recommended to use ports below 1024 for the player. ");
                }
            }
        }

        if let Ok(State::One(StateValue::String(extra_ytdlp_args))) = self
            .app
            .state(&Id::ConfigEditor(IdConfigEditor::ExtraYtdlpArgs))
        {
            config_tui.settings.ytdlp.extra_args = extra_ytdlp_args;
        }
        Ok(())
    }

    /// Extract all Themes to actual locations that can be loaded
    pub fn theme_extract_all() -> Result<()> {
        let mut path = get_app_config_path()?;
        path.push("themes");
        if !path.exists() {
            std::fs::create_dir_all(&path)?;
        }

        let base_path = &path;
        for entry in THEME_DIR.entries() {
            let path = base_path.join(entry.path());

            match entry {
                DirEntry::Dir(d) => {
                    std::fs::create_dir_all(&path)?;
                    d.extract(base_path)?;
                }
                DirEntry::File(f) => {
                    if !path.exists() {
                        std::fs::write(path, f.contents())?;
                    }
                }
            }
        }

        Ok(())
    }

    /// Find all themes in the `config/themes` directory and add them to be selected for preview
    pub fn theme_select_load_themes(&mut self) -> Result<()> {
        let mut path = get_app_config_path()?;
        path.push("themes");

        if let Ok(paths) = std::fs::read_dir(path) {
            self.config_editor.themes.clear();
            let mut paths: Vec<_> = paths.filter_map(std::result::Result::ok).collect();

            paths.sort_by_cached_key(|k| get_pin_yin(&k.file_name().to_string_lossy()));
            for entry in paths {
                let path = entry.path();
                let Some(stem) = path.file_stem() else {
                    warn!("Theme {:#?} does not have a filestem!", path.display());

                    continue;
                };

                self.config_editor
                    .themes
                    .push(stem.to_string_lossy().to_string());
            }
        }

        Ok(())
    }

    /// Build the theme UI table and select the current theme
    pub fn theme_select_sync(&mut self, previous_index: Option<usize>) {
        let mut table: TableBuilder = TableBuilder::default();

        table
            .add_col(TextSpan::new(0.to_string()))
            .add_col(TextSpan::new("Termusic Default"));
        table.add_row();
        table
            .add_col(TextSpan::new(1.to_string()))
            .add_col(TextSpan::new("Native"));

        for (idx, record) in self.config_editor.themes.iter().enumerate() {
            table.add_row();

            // idx + X as 0 until X entries are termusic default, always existing themes
            table
                .add_col(TextSpan::new((idx + THEMES_WITHOUT_FILES).to_string()))
                .add_col(TextSpan::new(record));
        }

        let table = table.build();
        self.app
            .attr(
                &Id::ConfigEditor(IdConfigEditor::CEThemeSelect),
                Attribute::Content,
                AttrValue::Table(table),
            )
            .ok();

        // select theme currently used
        let index = if let Some(index) = previous_index {
            index
        } else {
            let mut index = None;
            if let Some(current_file_name) = self.config_editor.theme.theme.file_name.as_ref() {
                for (idx, name) in self.config_editor.themes.iter().enumerate() {
                    if name == current_file_name {
                        // idx + X as 0 until X entries are termusic default, always existing themes
                        index = Some(idx + THEMES_WITHOUT_FILES);
                        break;
                    }
                }
            }

            index.unwrap_or(0)
        };
        assert!(
            self.app
                .attr(
                    &Id::ConfigEditor(IdConfigEditor::CEThemeSelect),
                    Attribute::Value,
                    AttrValue::Payload(PropPayload::One(PropValue::Usize(index))),
                )
                .is_ok()
        );
    }
}
