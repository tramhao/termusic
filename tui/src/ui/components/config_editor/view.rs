use crate::ui::Application;
use crate::ui::components::config_editor::update::THEMES_WITHOUT_FILES;
use crate::ui::components::raw::uniform_dynamic_grid::UniformDynamicGrid;
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

// NOTE: the macros either have to be in a different file OR be defined *before* they are used, otherwise they are not in scope

/// Chain many values together and `max` value from it.
///
/// Equivalent to manually chaining `val1.max(val2.max(val3))`.
macro_rules! max {
    ($first:ident$(,)?) => {
        $first
    };
    (
        $first:ident
        $(
            , $second:ident
        )* $(,)?
    ) => {
        $first.max(max!($($second,)*))
    }
}

/// Chain many values together and satured added value from it.
///
/// Equivalent to manually chaining `val1.saturating_add(val2.saturating_add(val3))`.
macro_rules! sat_add {
    ($first:expr$(,)?) => {
        $first
    };
    (
        $first:expr
        $(
            , $second:expr
        )* $(,)?
    ) => {
        $first.saturating_add(sat_add!($($second,)*))
    }
}

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
        /// Gets the state of `Id::ConfigEditor(id)` and if it has a `State::One`, returns `yes`, otherwise `no`.
        ///
        /// Macro to apply "DRY"(Dont-Repeat-Yourself) to reduce function length.
        macro_rules! is_expanded {
            ($id:expr, $yes:expr, $no:expr) => {
                match self.app.state(&Id::ConfigEditor($id)) {
                    Ok(State::One(_)) => $no,
                    _ => $yes,
                }
            };
        }

        let library_foreground_len: u16 = is_expanded!(IdConfigEditor::LibraryForeground, 8, 3);
        let library_background_len: u16 = is_expanded!(IdConfigEditor::LibraryBackground, 8, 3);
        let library_border_len: u16 = is_expanded!(IdConfigEditor::LibraryBorder, 8, 3);
        let library_highlight_len: u16 = is_expanded!(IdConfigEditor::LibraryHighlight, 8, 3);

        let playlist_foreground_len: u16 = is_expanded!(IdConfigEditor::PlaylistForeground, 8, 3);
        let playlist_background_len: u16 = is_expanded!(IdConfigEditor::PlaylistBackground, 8, 3);
        let playlist_border_len: u16 = is_expanded!(IdConfigEditor::PlaylistBorder, 8, 3);
        let playlist_highlight_len: u16 = is_expanded!(IdConfigEditor::PlaylistHighlight, 8, 3);

        let progress_foreground_len: u16 = is_expanded!(IdConfigEditor::ProgressForeground, 8, 3);
        let progress_background_len: u16 = is_expanded!(IdConfigEditor::ProgressBackground, 8, 3);
        let progress_border_len: u16 = is_expanded!(IdConfigEditor::ProgressBorder, 8, 3);

        let lyric_foreground_len: u16 = is_expanded!(IdConfigEditor::LyricForeground, 8, 3);
        let lyric_background_len: u16 = is_expanded!(IdConfigEditor::LyricBackground, 8, 3);
        let lyric_border_len: u16 = is_expanded!(IdConfigEditor::LyricBorder, 8, 3);

        let important_popup_foreground_len: u16 =
            is_expanded!(IdConfigEditor::ImportantPopupForeground, 8, 3);
        let important_popup_background_len: u16 =
            is_expanded!(IdConfigEditor::ImportantPopupBackground, 8, 3);
        let important_popup_border_len: u16 =
            is_expanded!(IdConfigEditor::ImportantPopupBorder, 8, 3);

        let fallback_foreground_len: u16 = is_expanded!(IdConfigEditor::FallbackForeground, 8, 3);
        let fallback_background_len: u16 = is_expanded!(IdConfigEditor::FallbackBackground, 8, 3);
        let fallback_border_len: u16 = is_expanded!(IdConfigEditor::FallbackBorder, 8, 3);
        let fallback_highlight_len: u16 = is_expanded!(IdConfigEditor::FallbackHighlight, 8, 3);

        self.terminal
            .raw_mut()
            .draw(|f| {
                let [header, chunks_main, footer] = Layout::vertical([
                    Constraint::Length(3), // config header
                    Constraint::Min(3),
                    Constraint::Length(1), // config footer
                ])
                .areas(f.area());

                let [left, right] =
                    Layout::horizontal([Constraint::Ratio(1, 4), Constraint::Ratio(3, 4)])
                        .areas(chunks_main);

                let library_height = sat_add! {
                    1u16, // label
                    library_foreground_len,
                    library_background_len,
                    library_border_len,
                    library_highlight_len,
                    3u16, // highlight symbol
                };
                let playlist_height = sat_add! {
                    1u16, // label
                    playlist_foreground_len,
                    playlist_background_len,
                    playlist_border_len,
                    playlist_highlight_len,
                    3u16, // highlight symbol
                    3u16, // current track symbol,
                };
                let progress_height = sat_add! {
                    1u16, // label
                    progress_foreground_len,
                    progress_background_len,
                    progress_border_len
                };
                let lyric_height = sat_add! {
                    1u16, // label
                    lyric_foreground_len,
                    lyric_background_len,
                    lyric_border_len
                };
                let important_popup_height = sat_add! {
                    1u16, // label
                    important_popup_foreground_len,
                    important_popup_background_len,
                    important_popup_border_len
                };
                let fallback_height = sat_add! {
                    1u16, // label
                    fallback_foreground_len,
                    fallback_background_len,
                    fallback_border_len,
                    fallback_highlight_len
                };

                let max_height = max! {
                    library_height,
                    playlist_height,
                    progress_height,
                    lyric_height,
                    important_popup_height,
                    fallback_height
                };

                let cells = UniformDynamicGrid::new(6, max_height, 16 + 2)
                    .with_row_spacing(1)
                    .draw_row_low_space()
                    .split(right);

                let chunks_library = Layout::vertical([
                    Constraint::Length(1),
                    Constraint::Length(library_foreground_len),
                    Constraint::Length(library_background_len),
                    Constraint::Length(library_border_len),
                    Constraint::Length(library_highlight_len),
                    Constraint::Length(3),
                    Constraint::Min(0),
                ])
                .split(cells[0]);

                let chunks_playlist = Layout::vertical([
                    Constraint::Length(1),
                    Constraint::Length(playlist_foreground_len),
                    Constraint::Length(playlist_background_len),
                    Constraint::Length(playlist_border_len),
                    Constraint::Length(playlist_highlight_len),
                    Constraint::Length(3),
                    Constraint::Length(3),
                    Constraint::Min(0),
                ])
                .split(cells[1]);

                let chunks_progress = Layout::vertical([
                    Constraint::Length(1),
                    Constraint::Length(progress_foreground_len),
                    Constraint::Length(progress_background_len),
                    Constraint::Length(progress_border_len),
                    Constraint::Min(0),
                ])
                .split(cells[2]);

                let chunks_lyric = Layout::vertical([
                    Constraint::Length(1),
                    Constraint::Length(lyric_foreground_len),
                    Constraint::Length(lyric_background_len),
                    Constraint::Length(lyric_border_len),
                    Constraint::Min(0),
                ])
                .split(cells[3]);

                let chunks_important_popup = Layout::vertical([
                    Constraint::Length(1),
                    Constraint::Length(important_popup_foreground_len),
                    Constraint::Length(important_popup_background_len),
                    Constraint::Length(important_popup_border_len),
                    Constraint::Min(0),
                ])
                .split(cells[4]);

                let chunks_fallback = Layout::vertical([
                    Constraint::Length(1),
                    Constraint::Length(fallback_foreground_len),
                    Constraint::Length(fallback_background_len),
                    Constraint::Length(fallback_border_len),
                    Constraint::Length(fallback_highlight_len),
                    Constraint::Min(0),
                ])
                .split(cells[5]);

                self.app
                    .view(&Id::ConfigEditor(IdConfigEditor::Header), f, header);

                self.app
                    .view(&Id::ConfigEditor(IdConfigEditor::CEThemeSelect), f, left);
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
        /// Gets the state of `Id::ConfigEditor(id)` and if it has a `State::One`, returns `yes`, otherwise `no`.
        ///
        /// Macro to apply "DRY"(Dont-Repeat-Yourself) to reduce function length.
        macro_rules! is_expanded {
            ($id:expr, $yes:expr, $no:expr) => {
                match self.app.state(&Id::ConfigEditor($id)) {
                    Ok(State::One(_)) => $no,
                    _ => $yes,
                }
            };
        }

        let global_quit_len = is_expanded!(IdConfigEditor::KeyGlobal(IdKeyGlobal::Quit), 8, 3);
        let global_left_len = is_expanded!(IdConfigEditor::KeyGlobal(IdKeyGlobal::Left), 8, 3);
        let global_right_len = is_expanded!(IdConfigEditor::KeyGlobal(IdKeyGlobal::Right), 8, 3);
        let global_up_len = is_expanded!(IdConfigEditor::KeyGlobal(IdKeyGlobal::Up), 8, 3);
        let global_down_len = is_expanded!(IdConfigEditor::KeyGlobal(IdKeyGlobal::Down), 8, 3);
        let global_goto_top_len =
            is_expanded!(IdConfigEditor::KeyGlobal(IdKeyGlobal::GotoTop), 8, 3);
        let global_goto_bottom_len =
            is_expanded!(IdConfigEditor::KeyGlobal(IdKeyGlobal::GotoBottom), 8, 3);
        let global_player_toggle_pause_len = is_expanded!(
            IdConfigEditor::KeyGlobal(IdKeyGlobal::PlayerTogglePause),
            8,
            3
        );
        let global_player_next_len =
            is_expanded!(IdConfigEditor::KeyGlobal(IdKeyGlobal::PlayerNext), 8, 3);
        let global_player_previous_len =
            is_expanded!(IdConfigEditor::KeyGlobal(IdKeyGlobal::PlayerPrevious), 8, 3);

        let global_help_len = is_expanded!(IdConfigEditor::KeyGlobal(IdKeyGlobal::Help), 8, 3);

        let global_volume_up_len =
            is_expanded!(IdConfigEditor::KeyGlobal(IdKeyGlobal::PlayerVolumeUp), 8, 3);

        let global_volume_down_len = is_expanded!(
            IdConfigEditor::KeyGlobal(IdKeyGlobal::PlayerVolumeDown),
            8,
            3
        );

        let global_player_seek_forward_len = is_expanded!(
            IdConfigEditor::KeyGlobal(IdKeyGlobal::PlayerSeekForward),
            8,
            3
        );
        let global_player_seek_backward_len = is_expanded!(
            IdConfigEditor::KeyGlobal(IdKeyGlobal::PlayerSeekBackward),
            8,
            3
        );
        let global_player_speed_up_len =
            is_expanded!(IdConfigEditor::KeyGlobal(IdKeyGlobal::PlayerSpeedUp), 8, 3);
        let global_player_speed_down_len = is_expanded!(
            IdConfigEditor::KeyGlobal(IdKeyGlobal::PlayerSpeedDown),
            8,
            3
        );

        let global_lyric_adjust_forward_len = is_expanded!(
            IdConfigEditor::KeyGlobal(IdKeyGlobal::LyricAdjustForward),
            8,
            3
        );
        let global_lyric_adjust_backward_len = is_expanded!(
            IdConfigEditor::KeyGlobal(IdKeyGlobal::LyricAdjustBackward),
            8,
            3
        );
        let global_lyric_cycle_len =
            is_expanded!(IdConfigEditor::KeyGlobal(IdKeyGlobal::LyricCycle), 8, 3);
        let global_layout_treeview_len =
            is_expanded!(IdConfigEditor::KeyGlobal(IdKeyGlobal::LayoutTreeview), 8, 3);

        let global_layout_database_len =
            is_expanded!(IdConfigEditor::KeyGlobal(IdKeyGlobal::LayoutDatabase), 8, 3);

        let global_player_toggle_gapless_len = is_expanded!(
            IdConfigEditor::KeyGlobal(IdKeyGlobal::PlayerToggleGapless),
            8,
            3
        );

        let global_config_len = is_expanded!(IdConfigEditor::KeyGlobal(IdKeyGlobal::Config), 8, 3);

        let global_save_playlist =
            is_expanded!(IdConfigEditor::KeyGlobal(IdKeyGlobal::SavePlaylist), 8, 3);

        let global_layout_podcast =
            is_expanded!(IdConfigEditor::KeyGlobal(IdKeyGlobal::LayoutPodcast), 8, 3);

        let global_xywh_move_left =
            is_expanded!(IdConfigEditor::KeyGlobal(IdKeyGlobal::XywhMoveLeft), 8, 3);

        let global_xywh_move_right =
            is_expanded!(IdConfigEditor::KeyGlobal(IdKeyGlobal::XywhMoveRight), 8, 3);
        let global_xywh_move_up =
            is_expanded!(IdConfigEditor::KeyGlobal(IdKeyGlobal::XywhMoveUp), 8, 3);
        let global_xywh_move_down =
            is_expanded!(IdConfigEditor::KeyGlobal(IdKeyGlobal::XywhMoveDown), 8, 3);
        let global_xywh_zoom_in =
            is_expanded!(IdConfigEditor::KeyGlobal(IdKeyGlobal::XywhZoomIn), 8, 3);
        let global_xywh_zoom_out =
            is_expanded!(IdConfigEditor::KeyGlobal(IdKeyGlobal::XywhZoomOut), 8, 3);
        let global_xywh_hide =
            is_expanded!(IdConfigEditor::KeyGlobal(IdKeyGlobal::XywhHide,), 8, 3);

        self.terminal
            .raw_mut()
            .draw(|f| {
                let [header, chunks_main, footer] = Layout::vertical([
                    Constraint::Length(3), // config header
                    Constraint::Min(3),
                    Constraint::Length(1), // config footer
                ])
                .areas(f.area());

                let max_height = max! {
                    global_layout_treeview_len,
                    global_layout_database_len,
                    global_layout_podcast,

                    global_quit_len,
                    global_left_len,
                    global_right_len,
                    global_up_len,
                    global_down_len,
                    global_goto_top_len,
                    global_goto_bottom_len,

                    global_help_len,
                    global_volume_up_len,
                    global_volume_down_len,

                    global_player_seek_forward_len,
                    global_player_seek_backward_len,
                    global_player_speed_up_len,
                    global_player_speed_down_len,
                    global_player_toggle_gapless_len,
                    global_player_toggle_pause_len,
                    global_player_next_len,
                    global_player_previous_len,

                    global_lyric_adjust_forward_len,
                    global_lyric_adjust_backward_len,
                    global_lyric_cycle_len,

                    global_config_len,

                    global_save_playlist,

                    global_xywh_move_left,
                    global_xywh_move_right,
                    global_xywh_move_up,
                    global_xywh_move_down,
                    global_xywh_zoom_in,
                    global_xywh_zoom_out,
                    global_xywh_hide
                };

                let cells = UniformDynamicGrid::new(33, max_height, 23 + 2)
                    .draw_row_low_space()
                    .split(chunks_main);

                self.app
                    .view(&Id::ConfigEditor(IdConfigEditor::Header), f, header);
                self.app
                    .view(&Id::ConfigEditor(IdConfigEditor::Footer), f, footer);

                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::KeyGlobal(IdKeyGlobal::Quit)),
                    f,
                    cells[0],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::KeyGlobal(IdKeyGlobal::Left)),
                    f,
                    cells[1],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::KeyGlobal(IdKeyGlobal::Down)),
                    f,
                    cells[2],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::KeyGlobal(IdKeyGlobal::Up)),
                    f,
                    cells[3],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::KeyGlobal(IdKeyGlobal::Right)),
                    f,
                    cells[4],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::KeyGlobal(IdKeyGlobal::GotoTop)),
                    f,
                    cells[5],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::KeyGlobal(IdKeyGlobal::GotoBottom)),
                    f,
                    cells[6],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::KeyGlobal(IdKeyGlobal::PlayerTogglePause)),
                    f,
                    cells[7],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::KeyGlobal(IdKeyGlobal::PlayerNext)),
                    f,
                    cells[8],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::KeyGlobal(IdKeyGlobal::PlayerPrevious)),
                    f,
                    cells[9],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::KeyGlobal(IdKeyGlobal::Help)),
                    f,
                    cells[10],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::KeyGlobal(IdKeyGlobal::PlayerVolumeUp)),
                    f,
                    cells[11],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::KeyGlobal(IdKeyGlobal::PlayerVolumeDown)),
                    f,
                    cells[12],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::KeyGlobal(IdKeyGlobal::PlayerSeekForward)),
                    f,
                    cells[13],
                );

                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::KeyGlobal(IdKeyGlobal::PlayerSeekBackward)),
                    f,
                    cells[14],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::KeyGlobal(IdKeyGlobal::PlayerSpeedUp)),
                    f,
                    cells[15],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::KeyGlobal(IdKeyGlobal::PlayerSpeedDown)),
                    f,
                    cells[16],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::KeyGlobal(IdKeyGlobal::LyricAdjustForward)),
                    f,
                    cells[17],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::KeyGlobal(IdKeyGlobal::LyricAdjustBackward)),
                    f,
                    cells[18],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::KeyGlobal(IdKeyGlobal::LyricCycle)),
                    f,
                    cells[19],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::KeyGlobal(IdKeyGlobal::LayoutTreeview)),
                    f,
                    cells[20],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::KeyGlobal(IdKeyGlobal::LayoutDatabase)),
                    f,
                    cells[21],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::KeyGlobal(IdKeyGlobal::PlayerToggleGapless)),
                    f,
                    cells[22],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::KeyGlobal(IdKeyGlobal::Config)),
                    f,
                    cells[23],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::KeyGlobal(IdKeyGlobal::SavePlaylist)),
                    f,
                    cells[24],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::KeyGlobal(IdKeyGlobal::LayoutPodcast)),
                    f,
                    cells[25],
                );

                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::KeyGlobal(IdKeyGlobal::XywhMoveLeft)),
                    f,
                    cells[26],
                );

                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::KeyGlobal(IdKeyGlobal::XywhMoveRight)),
                    f,
                    cells[27],
                );

                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::KeyGlobal(IdKeyGlobal::XywhMoveUp)),
                    f,
                    cells[28],
                );

                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::KeyGlobal(IdKeyGlobal::XywhMoveDown)),
                    f,
                    cells[29],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::KeyGlobal(IdKeyGlobal::XywhZoomIn)),
                    f,
                    cells[30],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::KeyGlobal(IdKeyGlobal::XywhZoomOut)),
                    f,
                    cells[31],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::KeyGlobal(IdKeyGlobal::XywhHide)),
                    f,
                    cells[32],
                );
                Self::view_config_editor_commons(f, &mut self.app);
            })
            .expect("Expected to draw without error");
    }

    #[allow(clippy::too_many_lines)]
    fn view_config_editor_key2(&mut self) {
        /// Gets the state of `Id::ConfigEditor(id)` and if it has a `State::One`, returns `yes`, otherwise `no`.
        ///
        /// Macro to apply "DRY"(Dont-Repeat-Yourself) to reduce function length.
        macro_rules! is_expanded {
            ($id:expr, $yes:expr, $no:expr) => {
                match self.app.state(&Id::ConfigEditor($id)) {
                    Ok(State::One(_)) => $no,
                    _ => $yes,
                }
            };
        }

        let library_delete_len =
            is_expanded!(IdConfigEditor::KeyOther(IdKeyOther::LibraryDelete), 8, 3);
        let library_load_dir_len =
            is_expanded!(IdConfigEditor::KeyOther(IdKeyOther::LibraryLoadDir), 8, 3);
        let library_yank_len =
            is_expanded!(IdConfigEditor::KeyOther(IdKeyOther::LibraryYank), 8, 3);
        let library_paste_len =
            is_expanded!(IdConfigEditor::KeyOther(IdKeyOther::LibraryPaste), 8, 3);
        let library_search_len =
            is_expanded!(IdConfigEditor::KeyOther(IdKeyOther::LibrarySearch), 8, 3);
        let library_search_youtube_len = is_expanded!(
            IdConfigEditor::KeyOther(IdKeyOther::LibrarySearchYoutube),
            8,
            3
        );
        let library_tag_editor_len =
            is_expanded!(IdConfigEditor::KeyOther(IdKeyOther::LibraryTagEditor), 8, 3);
        let playlist_delete_len =
            is_expanded!(IdConfigEditor::KeyOther(IdKeyOther::PlaylistDelete), 8, 3);
        let playlist_delete_all_len = is_expanded!(
            IdConfigEditor::KeyOther(IdKeyOther::PlaylistDeleteAll),
            8,
            3
        );

        let playlist_shuffle_len =
            is_expanded!(IdConfigEditor::KeyOther(IdKeyOther::PlaylistShuffle), 8, 3);

        let playlist_mode_cycle_len = is_expanded!(
            IdConfigEditor::KeyOther(IdKeyOther::PlaylistModeCycle),
            8,
            3
        );
        let playlist_search_len =
            is_expanded!(IdConfigEditor::KeyOther(IdKeyOther::PlaylistSearch), 8, 3);
        let playlist_play_selected_len = is_expanded!(
            IdConfigEditor::KeyOther(IdKeyOther::PlaylistPlaySelected),
            8,
            3
        );

        let playlist_swap_down_len =
            is_expanded!(IdConfigEditor::KeyOther(IdKeyOther::PlaylistSwapDown), 8, 3);

        let playlist_swap_up_len =
            is_expanded!(IdConfigEditor::KeyOther(IdKeyOther::PlaylistSwapUp), 8, 3);

        let database_add_all_len =
            is_expanded!(IdConfigEditor::KeyOther(IdKeyOther::DatabaseAddAll), 8, 3);

        let database_add_selected_len = is_expanded!(
            IdConfigEditor::KeyOther(IdKeyOther::DatabaseAddSelected),
            8,
            3
        );

        let playlist_random_album_len = is_expanded!(
            IdConfigEditor::KeyOther(IdKeyOther::PlaylistAddRandomAlbum),
            8,
            3
        );

        let playlist_random_tracks_len = is_expanded!(
            IdConfigEditor::KeyOther(IdKeyOther::PlaylistAddRandomTracks),
            8,
            3
        );

        let library_switch_root_len = is_expanded!(
            IdConfigEditor::KeyOther(IdKeyOther::LibrarySwitchRoot,),
            8,
            3
        );

        let library_add_root_len =
            is_expanded!(IdConfigEditor::KeyOther(IdKeyOther::LibraryAddRoot,), 8, 3);

        let library_remove_root_len = is_expanded!(
            IdConfigEditor::KeyOther(IdKeyOther::LibraryRemoveRoot,),
            8,
            3
        );

        let podcast_mark_played_len = is_expanded!(
            IdConfigEditor::KeyOther(IdKeyOther::PodcastMarkPlayed,),
            8,
            3
        );

        let podcast_mark_all_played_len = is_expanded!(
            IdConfigEditor::KeyOther(IdKeyOther::PodcastMarkAllPlayed),
            8,
            3
        );

        let podcast_ep_download_len = is_expanded!(
            IdConfigEditor::KeyOther(IdKeyOther::PodcastEpDownload,),
            8,
            3
        );

        let podcast_ep_delete_file_len = is_expanded!(
            IdConfigEditor::KeyOther(IdKeyOther::PodcastEpDeleteFile),
            8,
            3
        );

        let podcast_delete_feed_len = is_expanded!(
            IdConfigEditor::KeyOther(IdKeyOther::PodcastDeleteFeed,),
            8,
            3
        );

        let podcast_delete_all_feeds_len = is_expanded!(
            IdConfigEditor::KeyOther(IdKeyOther::PodcastDeleteAllFeeds),
            8,
            3
        );

        let podcast_search_add_feed_len = is_expanded!(
            IdConfigEditor::KeyOther(IdKeyOther::PodcastSearchAddFeed),
            8,
            3
        );

        let podcast_refresh_feed_len = is_expanded!(
            IdConfigEditor::KeyOther(IdKeyOther::PodcastRefreshFeed,),
            8,
            3
        );

        let podcast_refresh_all_feeds_len = is_expanded!(
            IdConfigEditor::KeyOther(IdKeyOther::PodcastRefreshAllFeeds),
            8,
            3
        );

        self.terminal
            .raw_mut()
            .draw(|f| {
                let [header, chunks_main, footer] = Layout::vertical([
                    Constraint::Length(3), // config header
                    Constraint::Min(3),
                    Constraint::Length(1), // config footer
                ])
                .areas(f.area());

                let max_height = max! {
                    library_delete_len,
                    library_load_dir_len,
                    library_yank_len,
                    library_paste_len,
                    library_search_len,
                    library_search_youtube_len,
                    library_tag_editor_len,
                    library_switch_root_len,
                    library_add_root_len,
                    library_remove_root_len,

                    playlist_delete_len,
                    playlist_delete_all_len,
                    playlist_shuffle_len,
                    playlist_mode_cycle_len,
                    playlist_search_len,
                    playlist_play_selected_len,
                    playlist_swap_down_len,
                    playlist_swap_up_len,
                    playlist_random_album_len,
                    playlist_random_tracks_len,

                    database_add_all_len,
                    database_add_selected_len,

                    podcast_mark_played_len,
                    podcast_mark_all_played_len,
                    podcast_ep_download_len,
                    podcast_ep_delete_file_len,
                    podcast_delete_feed_len,
                    podcast_delete_all_feeds_len,
                    podcast_search_add_feed_len,
                    podcast_refresh_feed_len,
                    podcast_refresh_all_feeds_len,
                };

                let cells = UniformDynamicGrid::new(31, max_height, 25 + 2)
                    .draw_row_low_space()
                    .split(chunks_main);

                self.app
                    .view(&Id::ConfigEditor(IdConfigEditor::Header), f, header);
                self.app
                    .view(&Id::ConfigEditor(IdConfigEditor::Footer), f, footer);

                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::KeyOther(IdKeyOther::LibraryTagEditor)),
                    f,
                    cells[0],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::KeyOther(IdKeyOther::LibraryDelete)),
                    f,
                    cells[1],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::KeyOther(IdKeyOther::LibraryLoadDir)),
                    f,
                    cells[2],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::KeyOther(IdKeyOther::LibraryYank)),
                    f,
                    cells[3],
                );

                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::KeyOther(IdKeyOther::LibraryPaste)),
                    f,
                    cells[4],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::KeyOther(IdKeyOther::LibrarySearch)),
                    f,
                    cells[5],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::KeyOther(IdKeyOther::LibrarySearchYoutube)),
                    f,
                    cells[6],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::KeyOther(IdKeyOther::PlaylistDelete)),
                    f,
                    cells[7],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::KeyOther(IdKeyOther::PlaylistDeleteAll)),
                    f,
                    cells[8],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::KeyOther(IdKeyOther::PlaylistSearch)),
                    f,
                    cells[9],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::KeyOther(IdKeyOther::PlaylistShuffle)),
                    f,
                    cells[10],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::KeyOther(IdKeyOther::PlaylistModeCycle)),
                    f,
                    cells[11],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::KeyOther(IdKeyOther::PlaylistPlaySelected)),
                    f,
                    cells[12],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::KeyOther(IdKeyOther::PlaylistSwapDown)),
                    f,
                    cells[13],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::KeyOther(IdKeyOther::PlaylistSwapUp)),
                    f,
                    cells[14],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::KeyOther(IdKeyOther::DatabaseAddAll)),
                    f,
                    cells[15],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::KeyOther(IdKeyOther::DatabaseAddSelected)),
                    f,
                    cells[16],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::KeyOther(IdKeyOther::PlaylistAddRandomAlbum)),
                    f,
                    cells[17],
                );

                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::KeyOther(
                        IdKeyOther::PlaylistAddRandomTracks,
                    )),
                    f,
                    cells[18],
                );

                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::KeyOther(IdKeyOther::LibrarySwitchRoot)),
                    f,
                    cells[19],
                );

                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::KeyOther(IdKeyOther::LibraryAddRoot)),
                    f,
                    cells[20],
                );

                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::KeyOther(IdKeyOther::LibraryRemoveRoot)),
                    f,
                    cells[21],
                );

                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::KeyOther(IdKeyOther::PodcastMarkPlayed)),
                    f,
                    cells[22],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::KeyOther(IdKeyOther::PodcastMarkAllPlayed)),
                    f,
                    cells[23],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::KeyOther(IdKeyOther::PodcastEpDownload)),
                    f,
                    cells[24],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::KeyOther(IdKeyOther::PodcastEpDeleteFile)),
                    f,
                    cells[25],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::KeyOther(IdKeyOther::PodcastDeleteFeed)),
                    f,
                    cells[26],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::KeyOther(IdKeyOther::PodcastDeleteAllFeeds)),
                    f,
                    cells[27],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::KeyOther(IdKeyOther::PodcastRefreshFeed)),
                    f,
                    cells[28],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::KeyOther(IdKeyOther::PodcastRefreshAllFeeds)),
                    f,
                    cells[29],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::KeyOther(IdKeyOther::PodcastSearchAddFeed)),
                    f,
                    cells[30],
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
