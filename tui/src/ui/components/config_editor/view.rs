use crate::ui::components::{
    AlbumPhotoAlign, CEHeader, CEThemeSelectTable, ConfigDatabaseAddAll, ConfigGlobalConfig,
    ConfigGlobalDown, ConfigGlobalGotoBottom, ConfigGlobalGotoTop, ConfigGlobalHelp,
    ConfigGlobalLayoutDatabase, ConfigGlobalLayoutPodcast, ConfigGlobalLayoutTreeview,
    ConfigGlobalLeft, ConfigGlobalLyricAdjustBackward, ConfigGlobalLyricAdjustForward,
    ConfigGlobalLyricCycle, ConfigGlobalPlayerNext, ConfigGlobalPlayerPrevious,
    ConfigGlobalPlayerSeekBackward, ConfigGlobalPlayerSeekForward, ConfigGlobalPlayerSpeedDown,
    ConfigGlobalPlayerSpeedUp, ConfigGlobalPlayerToggleGapless, ConfigGlobalPlayerTogglePause,
    ConfigGlobalQuit, ConfigGlobalRight, ConfigGlobalSavePlaylist, ConfigGlobalUp,
    ConfigGlobalVolumeDown, ConfigGlobalVolumeUp, ConfigGlobalXywhHide, ConfigGlobalXywhMoveDown,
    ConfigGlobalXywhMoveLeft, ConfigGlobalXywhMoveRight, ConfigGlobalXywhMoveUp,
    ConfigGlobalXywhZoomIn, ConfigGlobalXywhZoomOut, ConfigLibraryAddRoot, ConfigLibraryBackground,
    ConfigLibraryBorder, ConfigLibraryDelete, ConfigLibraryForeground, ConfigLibraryHighlight,
    ConfigLibraryHighlightSymbol, ConfigLibraryLoadDir, ConfigLibraryPaste,
    ConfigLibraryRemoveRoot, ConfigLibrarySearch, ConfigLibrarySearchYoutube,
    ConfigLibrarySwitchRoot, ConfigLibraryTagEditor, ConfigLibraryTitle, ConfigLibraryYank,
    ConfigLyricBackground, ConfigLyricBorder, ConfigLyricForeground, ConfigLyricTitle,
    ConfigPlaylistAddFront, ConfigPlaylistBackground, ConfigPlaylistBorder, ConfigPlaylistDelete,
    ConfigPlaylistDeleteAll, ConfigPlaylistForeground, ConfigPlaylistHighlight,
    ConfigPlaylistHighlightSymbol, ConfigPlaylistLqueue, ConfigPlaylistModeCycle,
    ConfigPlaylistPlaySelected, ConfigPlaylistSearch, ConfigPlaylistShuffle,
    ConfigPlaylistSwapDown, ConfigPlaylistSwapUp, ConfigPlaylistTitle, ConfigPlaylistTqueue,
    ConfigPodcastDeleteAllFeeds, ConfigPodcastDeleteFeed, ConfigPodcastEpDeleteFile,
    ConfigPodcastEpDownload, ConfigPodcastMarkAllPlayed, ConfigPodcastMarkPlayed,
    ConfigPodcastRefreshAllFeeds, ConfigPodcastRefreshFeed, ConfigPodcastSearchAddFeed,
    ConfigProgressBackground, ConfigProgressBorder, ConfigProgressForeground, ConfigProgressTitle,
    ConfigSavePopup, ConfigSeekStep, ExitConfirmation, Footer, GlobalListener, KillDaemon,
    MusicDir, PlayerPort, PlayerUseDiscord, PlayerUseMpris, PlaylistDisplaySymbol,
    PlaylistRandomAlbum, PlaylistRandomTrack, PodcastDir, PodcastMaxRetries, PodcastSimulDownload,
    SaveLastPosition,
};
use include_dir::DirEntry;
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
use termusiclib::config::{LastPosition, SeekStep, Settings};
use termusiclib::utils::{draw_area_in_absolute, get_app_config_path, get_pin_yin};
use termusiclib::THEME_DIR;

use crate::ui::model::{ConfigEditorLayout, Model};
use crate::ui::{Application, Id, IdConfigEditor, IdKey, Msg};
use anyhow::{bail, Result};
use std::path::{Path, PathBuf};
use termusiclib::config::Alignment as XywhAlign;
use tuirealm::event::NoUserEvent;
use tuirealm::props::{PropPayload, PropValue, TableBuilder, TextSpan};
use tuirealm::tui::layout::{Constraint, Direction, Layout};
use tuirealm::tui::widgets::Clear;
use tuirealm::{AttrValue, Attribute, Frame, State, StateValue};

impl Model {
    #[allow(clippy::too_many_lines)]
    pub fn view_config_editor_general(&mut self) {
        assert!(self
            .terminal
            .raw_mut()
            .draw(|f| {
                let chunks_main = Layout::default()
                    .direction(Direction::Vertical)
                    .margin(0)
                    .constraints(
                        [
                            Constraint::Length(3),
                            Constraint::Min(3),
                            Constraint::Length(1),
                        ]
                        .as_ref(),
                    )
                    .split(f.size());

                let chunks_middle = Layout::default()
                    .direction(Direction::Horizontal)
                    .margin(0)
                    .constraints([Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)].as_ref())
                    .split(chunks_main[1]);

                let chunks_middle_left = Layout::default()
                    .direction(Direction::Vertical)
                    .margin(0)
                    .constraints(
                        [
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
                        ]
                        .as_ref(),
                    )
                    .split(chunks_middle[0]);

                let chunks_middle_right = Layout::default()
                    .direction(Direction::Vertical)
                    .margin(0)
                    .constraints(
                        [
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
                        ]
                        .as_ref(),
                    )
                    .split(chunks_middle[1]);
                self.app
                    .view(&Id::ConfigEditor(IdConfigEditor::Header), f, chunks_main[0]);
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

                self.app
                    .view(&Id::ConfigEditor(IdConfigEditor::Footer), f, chunks_main[2]);

                Self::view_config_editor_commons(f, &mut self.app);
            })
            .is_ok());
    }

    fn view_config_editor_commons(f: &mut Frame<'_>, app: &mut Application<Id, Msg, NoUserEvent>) {
        // -- popups
        if app.mounted(&Id::ConfigEditor(IdConfigEditor::ConfigSavePopup)) {
            let popup = draw_area_in_absolute(f.size(), 50, 3);
            f.render_widget(Clear, popup);
            app.view(&Id::ConfigEditor(IdConfigEditor::ConfigSavePopup), f, popup);
        }
        if app.mounted(&Id::ErrorPopup) {
            let popup = draw_area_in_absolute(f.size(), 50, 4);
            f.render_widget(Clear, popup);
            app.view(&Id::ErrorPopup, f, popup);
        }
    }

    #[allow(clippy::too_many_lines)]
    pub fn view_config_editor_color(&mut self) {
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
        assert!(self
            .terminal
            .raw_mut()
            .draw(|f| {
                let chunks_main = Layout::default()
                    .direction(Direction::Vertical)
                    .margin(0)
                    .constraints(
                        [
                            Constraint::Length(3),
                            Constraint::Min(3),
                            Constraint::Length(1),
                        ]
                        .as_ref(),
                    )
                    .split(f.size());

                let chunks_middle = Layout::default()
                    .direction(Direction::Horizontal)
                    .margin(0)
                    .constraints([Constraint::Ratio(1, 4), Constraint::Ratio(3, 4)].as_ref())
                    .split(chunks_main[1]);

                let chunks_middle_right = Layout::default()
                    .direction(Direction::Horizontal)
                    .margin(0)
                    .constraints(
                        [
                            Constraint::Ratio(1, 4),
                            Constraint::Ratio(1, 4),
                            Constraint::Ratio(1, 4),
                            Constraint::Ratio(1, 4),
                        ]
                        .as_ref(),
                    )
                    .split(chunks_middle[1]);
                let chunks_library = Layout::default()
                    .direction(Direction::Vertical)
                    .margin(0)
                    .constraints(
                        [
                            Constraint::Length(1),
                            Constraint::Length(select_library_foreground_len),
                            Constraint::Length(select_library_background_len),
                            Constraint::Length(select_library_border_len),
                            Constraint::Length(select_library_highlight_len),
                            Constraint::Length(3),
                            Constraint::Min(3),
                        ]
                        .as_ref(),
                    )
                    .split(chunks_middle_right[0]);

                let chunks_playlist = Layout::default()
                    .direction(Direction::Vertical)
                    .margin(0)
                    .constraints(
                        [
                            Constraint::Length(1),
                            Constraint::Length(select_playlist_foreground_len),
                            Constraint::Length(select_playlist_background_len),
                            Constraint::Length(select_playlist_border_len),
                            Constraint::Length(select_playlist_highlight_len),
                            Constraint::Length(3),
                            Constraint::Min(3),
                        ]
                        .as_ref(),
                    )
                    .split(chunks_middle_right[1]);

                let chunks_progress = Layout::default()
                    .direction(Direction::Vertical)
                    .margin(0)
                    .constraints(
                        [
                            Constraint::Length(1),
                            Constraint::Length(select_progress_foreground_len),
                            Constraint::Length(select_progress_background_len),
                            Constraint::Length(select_progress_border_len),
                            Constraint::Min(3),
                        ]
                        .as_ref(),
                    )
                    .split(chunks_middle_right[2]);

                let chunks_lyric = Layout::default()
                    .direction(Direction::Vertical)
                    .margin(0)
                    .constraints(
                        [
                            Constraint::Length(1),
                            Constraint::Length(select_lyric_foreground_len),
                            Constraint::Length(select_lyric_background_len),
                            Constraint::Length(select_lyric_border_len),
                            Constraint::Min(3),
                        ]
                        .as_ref(),
                    )
                    .split(chunks_middle_right[3]);

                self.app
                    .view(&Id::ConfigEditor(IdConfigEditor::Header), f, chunks_main[0]);

                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::CEThemeSelect),
                    f,
                    chunks_middle[0],
                );
                self.app
                    .view(&Id::ConfigEditor(IdConfigEditor::Footer), f, chunks_main[2]);
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
                Self::view_config_editor_commons(f, &mut self.app);
            })
            .is_ok());
    }

    #[allow(clippy::too_many_lines)]
    pub fn view_config_editor_key1(&mut self) {
        let select_global_quit_len = match self
            .app
            .state(&Id::ConfigEditor(IdConfigEditor::Key(IdKey::GlobalQuit)))
        {
            Ok(State::One(_)) => 3,
            _ => 8,
        };
        let select_global_left_len = match self
            .app
            .state(&Id::ConfigEditor(IdConfigEditor::Key(IdKey::GlobalLeft)))
        {
            Ok(State::One(_)) => 3,
            _ => 8,
        };
        let select_global_right_len = match self
            .app
            .state(&Id::ConfigEditor(IdConfigEditor::Key(IdKey::GlobalRight)))
        {
            Ok(State::One(_)) => 3,
            _ => 8,
        };
        let select_global_up_len = match self
            .app
            .state(&Id::ConfigEditor(IdConfigEditor::Key(IdKey::GlobalUp)))
        {
            Ok(State::One(_)) => 3,
            _ => 8,
        };
        let select_global_down_len = match self
            .app
            .state(&Id::ConfigEditor(IdConfigEditor::Key(IdKey::GlobalDown)))
        {
            Ok(State::One(_)) => 3,
            _ => 8,
        };
        let select_global_goto_top_len = match self
            .app
            .state(&Id::ConfigEditor(IdConfigEditor::Key(IdKey::GlobalGotoTop)))
        {
            Ok(State::One(_)) => 3,
            _ => 8,
        };
        let select_global_goto_bottom_len = match self.app.state(&Id::ConfigEditor(
            IdConfigEditor::Key(IdKey::GlobalGotoBottom),
        )) {
            Ok(State::One(_)) => 3,
            _ => 8,
        };
        let select_global_player_toggle_pause_len = match self.app.state(&Id::ConfigEditor(
            IdConfigEditor::Key(IdKey::GlobalPlayerTogglePause),
        )) {
            Ok(State::One(_)) => 3,
            _ => 8,
        };
        let select_global_player_next_len = match self.app.state(&Id::ConfigEditor(
            IdConfigEditor::Key(IdKey::GlobalPlayerNext),
        )) {
            Ok(State::One(_)) => 3,
            _ => 8,
        };
        let select_global_player_previous_len = match self.app.state(&Id::ConfigEditor(
            IdConfigEditor::Key(IdKey::GlobalPlayerPrevious),
        )) {
            Ok(State::One(_)) => 3,
            _ => 8,
        };

        let select_global_help_len = match self
            .app
            .state(&Id::ConfigEditor(IdConfigEditor::Key(IdKey::GlobalHelp)))
        {
            Ok(State::One(_)) => 3,
            _ => 8,
        };

        let select_global_volume_up_len = match self.app.state(&Id::ConfigEditor(
            IdConfigEditor::Key(IdKey::GlobalVolumeUp),
        )) {
            Ok(State::One(_)) => 3,
            _ => 8,
        };

        let select_global_volume_down_len = match self.app.state(&Id::ConfigEditor(
            IdConfigEditor::Key(IdKey::GlobalVolumeDown),
        )) {
            Ok(State::One(_)) => 3,
            _ => 8,
        };

        let select_global_player_seek_forward_len = match self.app.state(&Id::ConfigEditor(
            IdConfigEditor::Key(IdKey::GlobalPlayerSeekForward),
        )) {
            Ok(State::One(_)) => 3,
            _ => 8,
        };
        let select_global_player_seek_backward_len = match self.app.state(&Id::ConfigEditor(
            IdConfigEditor::Key(IdKey::GlobalPlayerSeekBackward),
        )) {
            Ok(State::One(_)) => 3,
            _ => 8,
        };
        let select_global_player_speed_up_len = match self.app.state(&Id::ConfigEditor(
            IdConfigEditor::Key(IdKey::GlobalPlayerSpeedUp),
        )) {
            Ok(State::One(_)) => 3,
            _ => 8,
        };
        let select_global_player_speed_down_len = match self.app.state(&Id::ConfigEditor(
            IdConfigEditor::Key(IdKey::GlobalPlayerSpeedDown),
        )) {
            Ok(State::One(_)) => 3,
            _ => 8,
        };

        let select_global_lyric_adjust_forward_len = match self.app.state(&Id::ConfigEditor(
            IdConfigEditor::Key(IdKey::GlobalLyricAdjustForward),
        )) {
            Ok(State::One(_)) => 3,
            _ => 8,
        };
        let select_global_lyric_adjust_backward_len = match self.app.state(&Id::ConfigEditor(
            IdConfigEditor::Key(IdKey::GlobalLyricAdjustBackward),
        )) {
            Ok(State::One(_)) => 3,
            _ => 8,
        };
        let select_global_lyric_cycle_len = match self.app.state(&Id::ConfigEditor(
            IdConfigEditor::Key(IdKey::GlobalLyricCycle),
        )) {
            Ok(State::One(_)) => 3,
            _ => 8,
        };
        let select_global_layout_treeview_len = match self.app.state(&Id::ConfigEditor(
            IdConfigEditor::Key(IdKey::GlobalLayoutTreeview),
        )) {
            Ok(State::One(_)) => 3,
            _ => 8,
        };

        let select_global_layout_database_len = match self.app.state(&Id::ConfigEditor(
            IdConfigEditor::Key(IdKey::GlobalLayoutDatabase),
        )) {
            Ok(State::One(_)) => 3,
            _ => 8,
        };

        let select_global_player_toggle_gapless_len = match self.app.state(&Id::ConfigEditor(
            IdConfigEditor::Key(IdKey::GlobalPlayerToggleGapless),
        )) {
            Ok(State::One(_)) => 3,
            _ => 8,
        };

        let select_global_config_len = match self
            .app
            .state(&Id::ConfigEditor(IdConfigEditor::Key(IdKey::GlobalConfig)))
        {
            Ok(State::One(_)) => 3,
            _ => 8,
        };

        let select_global_save_playlist = match self.app.state(&Id::ConfigEditor(
            IdConfigEditor::Key(IdKey::GlobalSavePlaylist),
        )) {
            Ok(State::One(_)) => 3,
            _ => 8,
        };

        let select_global_layout_podcast = match self.app.state(&Id::ConfigEditor(
            IdConfigEditor::Key(IdKey::GlobalLayoutPodcast),
        )) {
            Ok(State::One(_)) => 3,
            _ => 8,
        };

        let select_global_xywh_move_left = match self.app.state(&Id::ConfigEditor(
            IdConfigEditor::Key(IdKey::GlobalXywhMoveLeft),
        )) {
            Ok(State::One(_)) => 3,
            _ => 8,
        };

        let select_global_xywh_move_right = match self.app.state(&Id::ConfigEditor(
            IdConfigEditor::Key(IdKey::GlobalXywhMoveRight),
        )) {
            Ok(State::One(_)) => 3,
            _ => 8,
        };
        let select_global_xywh_move_up = match self.app.state(&Id::ConfigEditor(
            IdConfigEditor::Key(IdKey::GlobalXywhMoveUp),
        )) {
            Ok(State::One(_)) => 3,
            _ => 8,
        };
        let select_global_xywh_move_down = match self.app.state(&Id::ConfigEditor(
            IdConfigEditor::Key(IdKey::GlobalXywhMoveDown),
        )) {
            Ok(State::One(_)) => 3,
            _ => 8,
        };
        let select_global_xywh_zoom_in = match self.app.state(&Id::ConfigEditor(
            IdConfigEditor::Key(IdKey::GlobalXywhZoomIn),
        )) {
            Ok(State::One(_)) => 3,
            _ => 8,
        };
        let select_global_xywh_zoom_out = match self.app.state(&Id::ConfigEditor(
            IdConfigEditor::Key(IdKey::GlobalXywhZoomOut),
        )) {
            Ok(State::One(_)) => 3,
            _ => 8,
        };
        let select_global_xywh_hide = match self.app.state(&Id::ConfigEditor(IdConfigEditor::Key(
            IdKey::GlobalXywhHide,
        ))) {
            Ok(State::One(_)) => 3,
            _ => 8,
        };

        assert!(self
            .terminal
            .raw_mut()
            .draw(|f| {
                let chunks_main = Layout::default()
                    .direction(Direction::Vertical)
                    .margin(0)
                    .constraints(
                        [
                            Constraint::Length(3),
                            Constraint::Min(3),
                            Constraint::Length(1),
                        ]
                        .as_ref(),
                    )
                    .split(f.size());

                let chunks_middle = Layout::default()
                    .direction(Direction::Horizontal)
                    .margin(0)
                    .constraints(
                        [
                            Constraint::Ratio(1, 4),
                            Constraint::Ratio(1, 4),
                            Constraint::Ratio(1, 4),
                            Constraint::Ratio(1, 4),
                        ]
                        .as_ref(),
                    )
                    .split(chunks_main[1]);

                let chunks_middle_column1 = Layout::default()
                    .direction(Direction::Vertical)
                    .margin(0)
                    .constraints(
                        [
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
                        ]
                        .as_ref(),
                    )
                    .split(chunks_middle[0]);

                let chunks_middle_column2 = Layout::default()
                    .direction(Direction::Vertical)
                    .margin(0)
                    .constraints(
                        [
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
                        ]
                        .as_ref(),
                    )
                    .split(chunks_middle[1]);
                let chunks_middle_column3 = Layout::default()
                    .direction(Direction::Vertical)
                    .margin(0)
                    .constraints(
                        [
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
                        ]
                        .as_ref(),
                    )
                    .split(chunks_middle[2]);

                let chunks_middle_column4 = Layout::default()
                    .direction(Direction::Vertical)
                    .margin(0)
                    .constraints(
                        [
                            Constraint::Length(select_global_xywh_move_right),
                            Constraint::Length(select_global_xywh_move_up),
                            Constraint::Length(select_global_xywh_move_down),
                            Constraint::Length(select_global_xywh_zoom_in),
                            Constraint::Length(select_global_xywh_zoom_out),
                            Constraint::Length(select_global_xywh_hide),
                            // Constraint::Length(select_global_xywh_hide),
                            // Constraint::Length(select_global_xywh_hide),
                            // Constraint::Length(select_global_xywh_hide),
                            Constraint::Min(0),
                        ]
                        .as_ref(),
                    )
                    .split(chunks_middle[3]);
                self.app
                    .view(&Id::ConfigEditor(IdConfigEditor::Header), f, chunks_main[0]);
                self.app
                    .view(&Id::ConfigEditor(IdConfigEditor::Footer), f, chunks_main[2]);

                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::Key(IdKey::GlobalQuit)),
                    f,
                    chunks_middle_column1[0],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::Key(IdKey::GlobalLeft)),
                    f,
                    chunks_middle_column1[1],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::Key(IdKey::GlobalDown)),
                    f,
                    chunks_middle_column1[2],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::Key(IdKey::GlobalUp)),
                    f,
                    chunks_middle_column1[3],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::Key(IdKey::GlobalRight)),
                    f,
                    chunks_middle_column1[4],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::Key(IdKey::GlobalGotoTop)),
                    f,
                    chunks_middle_column1[5],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::Key(IdKey::GlobalGotoBottom)),
                    f,
                    chunks_middle_column1[6],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::Key(IdKey::GlobalPlayerTogglePause)),
                    f,
                    chunks_middle_column1[7],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::Key(IdKey::GlobalPlayerNext)),
                    f,
                    chunks_middle_column1[8],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::Key(IdKey::GlobalPlayerPrevious)),
                    f,
                    chunks_middle_column2[0],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::Key(IdKey::GlobalHelp)),
                    f,
                    chunks_middle_column2[1],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::Key(IdKey::GlobalVolumeUp)),
                    f,
                    chunks_middle_column2[2],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::Key(IdKey::GlobalVolumeDown)),
                    f,
                    chunks_middle_column2[3],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::Key(IdKey::GlobalPlayerSeekForward)),
                    f,
                    chunks_middle_column2[4],
                );

                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::Key(IdKey::GlobalPlayerSeekBackward)),
                    f,
                    chunks_middle_column2[5],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::Key(IdKey::GlobalPlayerSpeedUp)),
                    f,
                    chunks_middle_column2[6],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::Key(IdKey::GlobalPlayerSpeedDown)),
                    f,
                    chunks_middle_column2[7],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::Key(IdKey::GlobalLyricAdjustForward)),
                    f,
                    chunks_middle_column2[8],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::Key(IdKey::GlobalLyricAdjustBackward)),
                    f,
                    chunks_middle_column3[0],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::Key(IdKey::GlobalLyricCycle)),
                    f,
                    chunks_middle_column3[1],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::Key(IdKey::GlobalLayoutTreeview)),
                    f,
                    chunks_middle_column3[2],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::Key(IdKey::GlobalLayoutDatabase)),
                    f,
                    chunks_middle_column3[3],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::Key(IdKey::GlobalPlayerToggleGapless)),
                    f,
                    chunks_middle_column3[4],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::Key(IdKey::GlobalConfig)),
                    f,
                    chunks_middle_column3[5],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::Key(IdKey::GlobalSavePlaylist)),
                    f,
                    chunks_middle_column3[6],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::Key(IdKey::GlobalLayoutPodcast)),
                    f,
                    chunks_middle_column3[7],
                );

                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::Key(IdKey::GlobalXywhMoveLeft)),
                    f,
                    chunks_middle_column3[8],
                );

                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::Key(IdKey::GlobalXywhMoveRight)),
                    f,
                    chunks_middle_column4[0],
                );

                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::Key(IdKey::GlobalXywhMoveUp)),
                    f,
                    chunks_middle_column4[1],
                );

                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::Key(IdKey::GlobalXywhMoveDown)),
                    f,
                    chunks_middle_column4[2],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::Key(IdKey::GlobalXywhZoomIn)),
                    f,
                    chunks_middle_column4[3],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::Key(IdKey::GlobalXywhZoomOut)),
                    f,
                    chunks_middle_column4[4],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::Key(IdKey::GlobalXywhHide)),
                    f,
                    chunks_middle_column4[5],
                );
                Self::view_config_editor_commons(f, &mut self.app);
            })
            .is_ok());
    }

    #[allow(clippy::too_many_lines)]
    pub fn view_config_editor_key2(&mut self) {
        let select_library_delete_len = match self
            .app
            .state(&Id::ConfigEditor(IdConfigEditor::Key(IdKey::LibraryDelete)))
        {
            Ok(State::One(_)) => 3,
            _ => 8,
        };
        let select_library_load_dir_len = match self.app.state(&Id::ConfigEditor(
            IdConfigEditor::Key(IdKey::LibraryLoadDir),
        )) {
            Ok(State::One(_)) => 3,
            _ => 8,
        };
        let select_library_yank_len = match self
            .app
            .state(&Id::ConfigEditor(IdConfigEditor::Key(IdKey::LibraryYank)))
        {
            Ok(State::One(_)) => 3,
            _ => 8,
        };
        let select_library_paste_len = match self
            .app
            .state(&Id::ConfigEditor(IdConfigEditor::Key(IdKey::LibraryPaste)))
        {
            Ok(State::One(_)) => 3,
            _ => 8,
        };
        let select_library_search_len = match self
            .app
            .state(&Id::ConfigEditor(IdConfigEditor::Key(IdKey::LibrarySearch)))
        {
            Ok(State::One(_)) => 3,
            _ => 8,
        };
        let select_library_search_youtube_len = match self.app.state(&Id::ConfigEditor(
            IdConfigEditor::Key(IdKey::LibrarySearchYoutube),
        )) {
            Ok(State::One(_)) => 3,
            _ => 8,
        };
        let select_library_tag_editor_len = match self.app.state(&Id::ConfigEditor(
            IdConfigEditor::Key(IdKey::LibraryTagEditor),
        )) {
            Ok(State::One(_)) => 3,
            _ => 8,
        };
        let select_playlist_delete_len = match self.app.state(&Id::ConfigEditor(
            IdConfigEditor::Key(IdKey::PlaylistDelete),
        )) {
            Ok(State::One(_)) => 3,
            _ => 8,
        };
        let select_playlist_delete_all_len = match self.app.state(&Id::ConfigEditor(
            IdConfigEditor::Key(IdKey::PlaylistDeleteAll),
        )) {
            Ok(State::One(_)) => 3,
            _ => 8,
        };

        let select_playlist_shuffle_len = match self.app.state(&Id::ConfigEditor(
            IdConfigEditor::Key(IdKey::PlaylistShuffle),
        )) {
            Ok(State::One(_)) => 3,
            _ => 8,
        };

        let select_playlist_mode_cycle_len = match self.app.state(&Id::ConfigEditor(
            IdConfigEditor::Key(IdKey::PlaylistModeCycle),
        )) {
            Ok(State::One(_)) => 3,
            _ => 8,
        };
        let select_playlist_add_front_len = match self.app.state(&Id::ConfigEditor(
            IdConfigEditor::Key(IdKey::PlaylistAddFront),
        )) {
            Ok(State::One(_)) => 3,
            _ => 8,
        };
        let select_playlist_search_len = match self.app.state(&Id::ConfigEditor(
            IdConfigEditor::Key(IdKey::PlaylistSearch),
        )) {
            Ok(State::One(_)) => 3,
            _ => 8,
        };
        let select_playlist_play_selected_len = match self.app.state(&Id::ConfigEditor(
            IdConfigEditor::Key(IdKey::PlaylistPlaySelected),
        )) {
            Ok(State::One(_)) => 3,
            _ => 8,
        };

        let select_playlist_swap_down_len = match self.app.state(&Id::ConfigEditor(
            IdConfigEditor::Key(IdKey::PlaylistSwapDown),
        )) {
            Ok(State::One(_)) => 3,
            _ => 8,
        };

        let select_playlist_swap_up_len = match self.app.state(&Id::ConfigEditor(
            IdConfigEditor::Key(IdKey::PlaylistSwapUp),
        )) {
            Ok(State::One(_)) => 3,
            _ => 8,
        };

        let select_database_add_all_len = match self.app.state(&Id::ConfigEditor(
            IdConfigEditor::Key(IdKey::DatabaseAddAll),
        )) {
            Ok(State::One(_)) => 3,
            _ => 8,
        };

        let select_playlist_lqueue_len = match self.app.state(&Id::ConfigEditor(
            IdConfigEditor::Key(IdKey::PlaylistLqueue),
        )) {
            Ok(State::One(_)) => 3,
            _ => 8,
        };

        let tqueue_len = match self.app.state(&Id::ConfigEditor(IdConfigEditor::Key(
            IdKey::PlaylistTqueue,
        ))) {
            Ok(State::One(_)) => 3,
            _ => 8,
        };

        let library_switch_root_len = match self.app.state(&Id::ConfigEditor(IdConfigEditor::Key(
            IdKey::LibrarySwitchRoot,
        ))) {
            Ok(State::One(_)) => 3,
            _ => 8,
        };

        let library_add_root_len = match self.app.state(&Id::ConfigEditor(IdConfigEditor::Key(
            IdKey::LibraryAddRoot,
        ))) {
            Ok(State::One(_)) => 3,
            _ => 8,
        };

        let library_remove_root_len = match self.app.state(&Id::ConfigEditor(IdConfigEditor::Key(
            IdKey::LibraryRemoveRoot,
        ))) {
            Ok(State::One(_)) => 3,
            _ => 8,
        };

        let podcast_mark_played_len = match self.app.state(&Id::ConfigEditor(IdConfigEditor::Key(
            IdKey::PodcastMarkPlayed,
        ))) {
            Ok(State::One(_)) => 3,
            _ => 8,
        };

        let podcast_mark_all_played_len = match self.app.state(&Id::ConfigEditor(
            IdConfigEditor::Key(IdKey::PodcastMarkAllPlayed),
        )) {
            Ok(State::One(_)) => 3,
            _ => 8,
        };

        let podcast_ep_download_len = match self.app.state(&Id::ConfigEditor(IdConfigEditor::Key(
            IdKey::PodcastEpDownload,
        ))) {
            Ok(State::One(_)) => 3,
            _ => 8,
        };

        let podcast_ep_delete_file_len = match self.app.state(&Id::ConfigEditor(
            IdConfigEditor::Key(IdKey::PodcastEpDeleteFile),
        )) {
            Ok(State::One(_)) => 3,
            _ => 8,
        };

        let podcast_delete_feed_len = match self.app.state(&Id::ConfigEditor(IdConfigEditor::Key(
            IdKey::PodcastDeleteFeed,
        ))) {
            Ok(State::One(_)) => 3,
            _ => 8,
        };

        let podcast_delete_all_feeds_len = match self.app.state(&Id::ConfigEditor(
            IdConfigEditor::Key(IdKey::PodcastDeleteAllFeeds),
        )) {
            Ok(State::One(_)) => 3,
            _ => 8,
        };

        let podcast_search_add_feed_len = match self.app.state(&Id::ConfigEditor(
            IdConfigEditor::Key(IdKey::PodcastSearchAddFeed),
        )) {
            Ok(State::One(_)) => 3,
            _ => 8,
        };

        let podcast_refresh_feed_len = match self.app.state(&Id::ConfigEditor(IdConfigEditor::Key(
            IdKey::PodcastRefreshFeed,
        ))) {
            Ok(State::One(_)) => 3,
            _ => 8,
        };

        let podcast_refresh_all_feeds_len = match self.app.state(&Id::ConfigEditor(
            IdConfigEditor::Key(IdKey::PodcastRefreshAllFeeds),
        )) {
            Ok(State::One(_)) => 3,
            _ => 8,
        };

        assert!(self
            .terminal
            .raw_mut()
            .draw(|f| {
                let chunks_main = Layout::default()
                    .direction(Direction::Vertical)
                    .margin(0)
                    .constraints(
                        [
                            Constraint::Length(3),
                            Constraint::Min(3),
                            Constraint::Length(1),
                        ]
                        .as_ref(),
                    )
                    .split(f.size());

                let chunks_middle = Layout::default()
                    .direction(Direction::Horizontal)
                    .margin(0)
                    .constraints(
                        [
                            Constraint::Ratio(1, 4),
                            Constraint::Ratio(1, 4),
                            Constraint::Ratio(1, 4),
                            Constraint::Ratio(1, 4),
                        ]
                        .as_ref(),
                    )
                    .split(chunks_main[1]);

                let chunks_middle_column1 = Layout::default()
                    .direction(Direction::Vertical)
                    .margin(0)
                    .constraints(
                        [
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
                        ]
                        .as_ref(),
                    )
                    .split(chunks_middle[0]);
                let chunks_middle_column2 = Layout::default()
                    .direction(Direction::Vertical)
                    .margin(0)
                    .constraints(
                        [
                            Constraint::Length(select_playlist_search_len),
                            Constraint::Length(select_playlist_shuffle_len),
                            Constraint::Length(select_playlist_add_front_len),
                            Constraint::Length(select_playlist_mode_cycle_len),
                            Constraint::Length(select_playlist_play_selected_len),
                            Constraint::Length(select_playlist_swap_down_len),
                            Constraint::Length(select_playlist_swap_up_len),
                            Constraint::Length(select_database_add_all_len),
                            Constraint::Length(select_playlist_lqueue_len),
                            Constraint::Min(0),
                        ]
                        .as_ref(),
                    )
                    .split(chunks_middle[1]);

                let chunks_middle_column3 = Layout::default()
                    .direction(Direction::Vertical)
                    .margin(0)
                    .constraints(
                        [
                            Constraint::Length(tqueue_len),
                            Constraint::Length(library_switch_root_len),
                            Constraint::Length(library_add_root_len),
                            Constraint::Length(library_remove_root_len),
                            Constraint::Length(podcast_mark_played_len),
                            Constraint::Length(podcast_mark_all_played_len),
                            Constraint::Length(podcast_ep_download_len),
                            Constraint::Length(podcast_ep_delete_file_len),
                            Constraint::Length(podcast_delete_feed_len),
                            Constraint::Min(0),
                        ]
                        .as_ref(),
                    )
                    .split(chunks_middle[2]);

                let chunks_middle_column4 = Layout::default()
                    .direction(Direction::Vertical)
                    .margin(0)
                    .constraints(
                        [
                            Constraint::Length(podcast_delete_all_feeds_len),
                            Constraint::Length(podcast_refresh_feed_len),
                            Constraint::Length(podcast_refresh_all_feeds_len),
                            Constraint::Length(podcast_search_add_feed_len),
                            // Constraint::Length(podcast_mark_played_len),
                            // Constraint::Length(podcast_mark_all_played_len),
                            // Constraint::Length(podcast_ep_download_len),
                            // Constraint::Length(podcast_ep_delete_file_len),
                            // Constraint::Length(podcast_delete_feed_len),
                            Constraint::Min(0),
                        ]
                        .as_ref(),
                    )
                    .split(chunks_middle[3]);
                self.app
                    .view(&Id::ConfigEditor(IdConfigEditor::Header), f, chunks_main[0]);
                self.app
                    .view(&Id::ConfigEditor(IdConfigEditor::Footer), f, chunks_main[2]);

                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::Key(IdKey::LibraryTagEditor)),
                    f,
                    chunks_middle_column1[0],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::Key(IdKey::LibraryDelete)),
                    f,
                    chunks_middle_column1[1],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::Key(IdKey::LibraryLoadDir)),
                    f,
                    chunks_middle_column1[2],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::Key(IdKey::LibraryYank)),
                    f,
                    chunks_middle_column1[3],
                );

                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::Key(IdKey::LibraryPaste)),
                    f,
                    chunks_middle_column1[4],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::Key(IdKey::LibrarySearch)),
                    f,
                    chunks_middle_column1[5],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::Key(IdKey::LibrarySearchYoutube)),
                    f,
                    chunks_middle_column1[6],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::Key(IdKey::PlaylistDelete)),
                    f,
                    chunks_middle_column1[7],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::Key(IdKey::PlaylistDeleteAll)),
                    f,
                    chunks_middle_column1[8],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::Key(IdKey::PlaylistSearch)),
                    f,
                    chunks_middle_column2[0],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::Key(IdKey::PlaylistShuffle)),
                    f,
                    chunks_middle_column2[1],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::Key(IdKey::PlaylistAddFront)),
                    f,
                    chunks_middle_column2[2],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::Key(IdKey::PlaylistModeCycle)),
                    f,
                    chunks_middle_column2[3],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::Key(IdKey::PlaylistPlaySelected)),
                    f,
                    chunks_middle_column2[4],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::Key(IdKey::PlaylistSwapDown)),
                    f,
                    chunks_middle_column2[5],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::Key(IdKey::PlaylistSwapUp)),
                    f,
                    chunks_middle_column2[6],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::Key(IdKey::DatabaseAddAll)),
                    f,
                    chunks_middle_column2[7],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::Key(IdKey::PlaylistLqueue)),
                    f,
                    chunks_middle_column2[8],
                );

                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::Key(IdKey::PlaylistTqueue)),
                    f,
                    chunks_middle_column3[0],
                );

                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::Key(IdKey::LibrarySwitchRoot)),
                    f,
                    chunks_middle_column3[1],
                );

                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::Key(IdKey::LibraryAddRoot)),
                    f,
                    chunks_middle_column3[2],
                );

                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::Key(IdKey::LibraryRemoveRoot)),
                    f,
                    chunks_middle_column3[3],
                );

                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::Key(IdKey::PodcastMarkPlayed)),
                    f,
                    chunks_middle_column3[4],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::Key(IdKey::PodcastMarkAllPlayed)),
                    f,
                    chunks_middle_column3[5],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::Key(IdKey::PodcastEpDownload)),
                    f,
                    chunks_middle_column3[6],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::Key(IdKey::PodcastEpDeleteFile)),
                    f,
                    chunks_middle_column3[7],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::Key(IdKey::PodcastDeleteFeed)),
                    f,
                    chunks_middle_column3[8],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::Key(IdKey::PodcastDeleteAllFeeds)),
                    f,
                    chunks_middle_column4[0],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::Key(IdKey::PodcastRefreshFeed)),
                    f,
                    chunks_middle_column4[1],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::Key(IdKey::PodcastRefreshAllFeeds)),
                    f,
                    chunks_middle_column4[2],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::Key(IdKey::PodcastSearchAddFeed)),
                    f,
                    chunks_middle_column4[3],
                );
                Self::view_config_editor_commons(f, &mut self.app);
            })
            .is_ok());
    }

    #[allow(clippy::too_many_lines)]
    pub fn mount_config_editor(&mut self) {
        self.config_layout = ConfigEditorLayout::General;
        let layout = self.config_layout.clone();

        assert!(self
            .app
            .remount(
                Id::ConfigEditor(IdConfigEditor::Header),
                Box::new(CEHeader::new(&layout, &self.config)),
                vec![]
            )
            .is_ok());
        assert!(self
            .app
            .remount(
                Id::ConfigEditor(IdConfigEditor::Footer),
                Box::new(Footer::new(&self.config)),
                vec![]
            )
            .is_ok());

        // Mount general page
        assert!(self
            .app
            .remount(
                Id::ConfigEditor(IdConfigEditor::MusicDir),
                Box::new(MusicDir::new(&self.config)),
                vec![]
            )
            .is_ok());

        assert!(self
            .app
            .remount(
                Id::ConfigEditor(IdConfigEditor::ExitConfirmation),
                Box::new(ExitConfirmation::new(&self.config)),
                vec![]
            )
            .is_ok());

        assert!(self
            .app
            .remount(
                Id::ConfigEditor(IdConfigEditor::PlaylistDisplaySymbol),
                Box::new(PlaylistDisplaySymbol::new(&self.config)),
                vec![]
            )
            .is_ok());

        assert!(self
            .app
            .remount(
                Id::ConfigEditor(IdConfigEditor::PlaylistRandomTrack),
                Box::new(PlaylistRandomTrack::new(&self.config)),
                vec![]
            )
            .is_ok());

        assert!(self
            .app
            .remount(
                Id::ConfigEditor(IdConfigEditor::PlaylistRandomAlbum),
                Box::new(PlaylistRandomAlbum::new(&self.config)),
                vec![]
            )
            .is_ok());

        assert!(self
            .app
            .remount(
                Id::ConfigEditor(IdConfigEditor::PodcastDir),
                Box::new(PodcastDir::new(&self.config)),
                vec![]
            )
            .is_ok());

        assert!(self
            .app
            .remount(
                Id::ConfigEditor(IdConfigEditor::PodcastSimulDownload),
                Box::new(PodcastSimulDownload::new(&self.config)),
                vec![]
            )
            .is_ok());

        assert!(self
            .app
            .remount(
                Id::ConfigEditor(IdConfigEditor::PodcastMaxRetries),
                Box::new(PodcastMaxRetries::new(&self.config)),
                vec![]
            )
            .is_ok());

        assert!(self
            .app
            .remount(
                Id::ConfigEditor(IdConfigEditor::AlbumPhotoAlign),
                Box::new(AlbumPhotoAlign::new(&self.config)),
                vec![]
            )
            .is_ok());

        assert!(self
            .app
            .remount(
                Id::ConfigEditor(IdConfigEditor::SaveLastPosition),
                Box::new(SaveLastPosition::new(&self.config)),
                vec![]
            )
            .is_ok());

        assert!(self
            .app
            .remount(
                Id::ConfigEditor(IdConfigEditor::SeekStep),
                Box::new(ConfigSeekStep::new(&self.config)),
                vec![]
            )
            .is_ok());

        assert!(self
            .app
            .remount(
                Id::ConfigEditor(IdConfigEditor::KillDamon),
                Box::new(KillDaemon::new(&self.config)),
                vec![]
            )
            .is_ok());

        assert!(self
            .app
            .remount(
                Id::ConfigEditor(IdConfigEditor::PlayerUseMpris),
                Box::new(PlayerUseMpris::new(&self.config)),
                vec![]
            )
            .is_ok());

        assert!(self
            .app
            .remount(
                Id::ConfigEditor(IdConfigEditor::PlayerUseDiscord),
                Box::new(PlayerUseDiscord::new(&self.config)),
                vec![]
            )
            .is_ok());

        assert!(self
            .app
            .remount(
                Id::ConfigEditor(IdConfigEditor::PlayerPort),
                Box::new(PlayerPort::new(&self.config)),
                vec![]
            )
            .is_ok());
        let config = self.config.clone();
        self.remount_config_color(&config);

        // Active Config Editor
        assert!(self
            .app
            .active(&Id::ConfigEditor(IdConfigEditor::MusicDir))
            .is_ok());

        if let Err(e) = self.theme_select_load_themes() {
            self.mount_error_popup(format!("Error load themes: {e}"));
        }
        self.theme_select_sync();
        if let Err(e) = self.update_photo() {
            self.mount_error_popup(format!("clear photo error: {e}"));
        }
    }

    #[allow(clippy::too_many_lines)]
    pub fn remount_config_color(&mut self, config: &Settings) {
        // Mount color page
        assert!(self
            .app
            .remount(
                Id::ConfigEditor(IdConfigEditor::CEThemeSelect),
                Box::new(CEThemeSelectTable::new(config)),
                vec![]
            )
            .is_ok());
        assert!(self
            .app
            .remount(
                Id::ConfigEditor(IdConfigEditor::LibraryLabel),
                Box::<ConfigLibraryTitle>::default(),
                vec![]
            )
            .is_ok());

        assert!(self
            .app
            .remount(
                Id::ConfigEditor(IdConfigEditor::LibraryForeground),
                Box::new(ConfigLibraryForeground::new(config)),
                vec![]
            )
            .is_ok());

        assert!(self
            .app
            .remount(
                Id::ConfigEditor(IdConfigEditor::LibraryBackground),
                Box::new(ConfigLibraryBackground::new(config)),
                vec![]
            )
            .is_ok());

        assert!(self
            .app
            .remount(
                Id::ConfigEditor(IdConfigEditor::LibraryBorder),
                Box::new(ConfigLibraryBorder::new(config)),
                vec![]
            )
            .is_ok());

        assert!(self
            .app
            .remount(
                Id::ConfigEditor(IdConfigEditor::LibraryHighlight),
                Box::new(ConfigLibraryHighlight::new(config)),
                vec![]
            )
            .is_ok());

        assert!(self
            .app
            .remount(
                Id::ConfigEditor(IdConfigEditor::PlaylistLabel),
                Box::<ConfigPlaylistTitle>::default(),
                vec![]
            )
            .is_ok());

        assert!(self
            .app
            .remount(
                Id::ConfigEditor(IdConfigEditor::PlaylistForeground),
                Box::new(ConfigPlaylistForeground::new(config)),
                vec![]
            )
            .is_ok());

        assert!(self
            .app
            .remount(
                Id::ConfigEditor(IdConfigEditor::PlaylistBackground),
                Box::new(ConfigPlaylistBackground::new(config)),
                vec![]
            )
            .is_ok());

        assert!(self
            .app
            .remount(
                Id::ConfigEditor(IdConfigEditor::PlaylistBorder),
                Box::new(ConfigPlaylistBorder::new(config)),
                vec![]
            )
            .is_ok());

        assert!(self
            .app
            .remount(
                Id::ConfigEditor(IdConfigEditor::PlaylistHighlight),
                Box::new(ConfigPlaylistHighlight::new(config)),
                vec![]
            )
            .is_ok());

        assert!(self
            .app
            .remount(
                Id::ConfigEditor(IdConfigEditor::ProgressLabel),
                Box::<ConfigProgressTitle>::default(),
                vec![]
            )
            .is_ok());

        assert!(self
            .app
            .remount(
                Id::ConfigEditor(IdConfigEditor::ProgressForeground),
                Box::new(ConfigProgressForeground::new(config)),
                vec![]
            )
            .is_ok());

        assert!(self
            .app
            .remount(
                Id::ConfigEditor(IdConfigEditor::ProgressBackground),
                Box::new(ConfigProgressBackground::new(config)),
                vec![]
            )
            .is_ok());

        assert!(self
            .app
            .remount(
                Id::ConfigEditor(IdConfigEditor::ProgressBorder),
                Box::new(ConfigProgressBorder::new(config)),
                vec![]
            )
            .is_ok());

        assert!(self
            .app
            .remount(
                Id::ConfigEditor(IdConfigEditor::LyricLabel),
                Box::<ConfigLyricTitle>::default(),
                vec![]
            )
            .is_ok());

        assert!(self
            .app
            .remount(
                Id::ConfigEditor(IdConfigEditor::LyricForeground),
                Box::new(ConfigLyricForeground::new(config)),
                vec![]
            )
            .is_ok());

        assert!(self
            .app
            .remount(
                Id::ConfigEditor(IdConfigEditor::LyricBackground),
                Box::new(ConfigLyricBackground::new(config)),
                vec![]
            )
            .is_ok());

        assert!(self
            .app
            .remount(
                Id::ConfigEditor(IdConfigEditor::LyricBorder),
                Box::new(ConfigLyricBorder::new(config)),
                vec![]
            )
            .is_ok());

        assert!(self
            .app
            .remount(
                Id::ConfigEditor(IdConfigEditor::LibraryHighlightSymbol),
                Box::new(ConfigLibraryHighlightSymbol::new(config)),
                vec![]
            )
            .is_ok());

        assert!(self
            .app
            .remount(
                Id::ConfigEditor(IdConfigEditor::PlaylistHighlightSymbol),
                Box::new(ConfigPlaylistHighlightSymbol::new(config)),
                vec![]
            )
            .is_ok());

        // Key 1: Global keys

        assert!(self
            .app
            .remount(
                Id::ConfigEditor(IdConfigEditor::Key(IdKey::GlobalQuit)),
                Box::new(ConfigGlobalQuit::new(config)),
                vec![],
            )
            .is_ok());
        assert!(self
            .app
            .remount(
                Id::ConfigEditor(IdConfigEditor::Key(IdKey::GlobalLeft)),
                Box::new(ConfigGlobalLeft::new(config)),
                vec![],
            )
            .is_ok());
        assert!(self
            .app
            .remount(
                Id::ConfigEditor(IdConfigEditor::Key(IdKey::GlobalRight)),
                Box::new(ConfigGlobalRight::new(config)),
                vec![],
            )
            .is_ok());
        assert!(self
            .app
            .remount(
                Id::ConfigEditor(IdConfigEditor::Key(IdKey::GlobalUp)),
                Box::new(ConfigGlobalUp::new(config)),
                vec![],
            )
            .is_ok());

        assert!(self
            .app
            .remount(
                Id::ConfigEditor(IdConfigEditor::Key(IdKey::GlobalDown)),
                Box::new(ConfigGlobalDown::new(config)),
                vec![],
            )
            .is_ok());
        assert!(self
            .app
            .remount(
                Id::ConfigEditor(IdConfigEditor::Key(IdKey::GlobalGotoTop)),
                Box::new(ConfigGlobalGotoTop::new(config)),
                vec![],
            )
            .is_ok());
        assert!(self
            .app
            .remount(
                Id::ConfigEditor(IdConfigEditor::Key(IdKey::GlobalGotoBottom)),
                Box::new(ConfigGlobalGotoBottom::new(config)),
                vec![],
            )
            .is_ok());
        assert!(self
            .app
            .remount(
                Id::ConfigEditor(IdConfigEditor::Key(IdKey::GlobalPlayerTogglePause)),
                Box::new(ConfigGlobalPlayerTogglePause::new(config)),
                vec![],
            )
            .is_ok());
        assert!(self
            .app
            .remount(
                Id::ConfigEditor(IdConfigEditor::Key(IdKey::GlobalPlayerNext)),
                Box::new(ConfigGlobalPlayerNext::new(config)),
                vec![],
            )
            .is_ok());
        assert!(self
            .app
            .remount(
                Id::ConfigEditor(IdConfigEditor::Key(IdKey::GlobalPlayerPrevious)),
                Box::new(ConfigGlobalPlayerPrevious::new(config)),
                vec![],
            )
            .is_ok());
        assert!(self
            .app
            .remount(
                Id::ConfigEditor(IdConfigEditor::Key(IdKey::GlobalHelp)),
                Box::new(ConfigGlobalHelp::new(config)),
                vec![],
            )
            .is_ok());
        assert!(self
            .app
            .remount(
                Id::ConfigEditor(IdConfigEditor::Key(IdKey::GlobalVolumeUp)),
                Box::new(ConfigGlobalVolumeUp::new(config)),
                vec![],
            )
            .is_ok());
        assert!(self
            .app
            .remount(
                Id::ConfigEditor(IdConfigEditor::Key(IdKey::GlobalVolumeDown)),
                Box::new(ConfigGlobalVolumeDown::new(config)),
                vec![],
            )
            .is_ok());
        assert!(self
            .app
            .remount(
                Id::ConfigEditor(IdConfigEditor::Key(IdKey::GlobalPlayerSeekForward)),
                Box::new(ConfigGlobalPlayerSeekForward::new(config)),
                vec![],
            )
            .is_ok());
        assert!(self
            .app
            .remount(
                Id::ConfigEditor(IdConfigEditor::Key(IdKey::GlobalPlayerSeekBackward)),
                Box::new(ConfigGlobalPlayerSeekBackward::new(config)),
                vec![],
            )
            .is_ok());
        assert!(self
            .app
            .remount(
                Id::ConfigEditor(IdConfigEditor::Key(IdKey::GlobalPlayerSpeedUp)),
                Box::new(ConfigGlobalPlayerSpeedUp::new(config)),
                vec![],
            )
            .is_ok());

        assert!(self
            .app
            .remount(
                Id::ConfigEditor(IdConfigEditor::Key(IdKey::GlobalPlayerSpeedDown)),
                Box::new(ConfigGlobalPlayerSpeedDown::new(config)),
                vec![],
            )
            .is_ok());

        assert!(self
            .app
            .remount(
                Id::ConfigEditor(IdConfigEditor::Key(IdKey::GlobalLyricAdjustForward)),
                Box::new(ConfigGlobalLyricAdjustForward::new(config)),
                vec![],
            )
            .is_ok());
        assert!(self
            .app
            .remount(
                Id::ConfigEditor(IdConfigEditor::Key(IdKey::GlobalLyricAdjustBackward)),
                Box::new(ConfigGlobalLyricAdjustBackward::new(config)),
                vec![],
            )
            .is_ok());
        assert!(self
            .app
            .remount(
                Id::ConfigEditor(IdConfigEditor::Key(IdKey::GlobalLyricCycle)),
                Box::new(ConfigGlobalLyricCycle::new(config)),
                vec![],
            )
            .is_ok());
        assert!(self
            .app
            .remount(
                Id::ConfigEditor(IdConfigEditor::Key(IdKey::GlobalPlayerToggleGapless)),
                Box::new(ConfigGlobalPlayerToggleGapless::new(config)),
                vec![],
            )
            .is_ok());

        assert!(self
            .app
            .remount(
                Id::ConfigEditor(IdConfigEditor::Key(IdKey::GlobalLayoutTreeview)),
                Box::new(ConfigGlobalLayoutTreeview::new(config)),
                vec![],
            )
            .is_ok());

        assert!(self
            .app
            .remount(
                Id::ConfigEditor(IdConfigEditor::Key(IdKey::GlobalLayoutDatabase)),
                Box::new(ConfigGlobalLayoutDatabase::new(config)),
                vec![],
            )
            .is_ok());

        assert!(self
            .app
            .remount(
                Id::ConfigEditor(IdConfigEditor::Key(IdKey::LibraryDelete)),
                Box::new(ConfigLibraryDelete::new(config)),
                vec![],
            )
            .is_ok());
        assert!(self
            .app
            .remount(
                Id::ConfigEditor(IdConfigEditor::Key(IdKey::LibraryLoadDir)),
                Box::new(ConfigLibraryLoadDir::new(config)),
                vec![],
            )
            .is_ok());
        assert!(self
            .app
            .remount(
                Id::ConfigEditor(IdConfigEditor::Key(IdKey::LibraryYank)),
                Box::new(ConfigLibraryYank::new(config)),
                vec![],
            )
            .is_ok());
        assert!(self
            .app
            .remount(
                Id::ConfigEditor(IdConfigEditor::Key(IdKey::LibraryPaste)),
                Box::new(ConfigLibraryPaste::new(config)),
                vec![],
            )
            .is_ok());
        assert!(self
            .app
            .remount(
                Id::ConfigEditor(IdConfigEditor::Key(IdKey::LibrarySearch)),
                Box::new(ConfigLibrarySearch::new(config)),
                vec![],
            )
            .is_ok());
        assert!(self
            .app
            .remount(
                Id::ConfigEditor(IdConfigEditor::Key(IdKey::LibrarySearchYoutube)),
                Box::new(ConfigLibrarySearchYoutube::new(config)),
                vec![],
            )
            .is_ok());
        assert!(self
            .app
            .remount(
                Id::ConfigEditor(IdConfigEditor::Key(IdKey::LibraryTagEditor)),
                Box::new(ConfigLibraryTagEditor::new(config)),
                vec![],
            )
            .is_ok());
        assert!(self
            .app
            .remount(
                Id::ConfigEditor(IdConfigEditor::Key(IdKey::PlaylistDelete)),
                Box::new(ConfigPlaylistDelete::new(config)),
                vec![],
            )
            .is_ok());
        assert!(self
            .app
            .remount(
                Id::ConfigEditor(IdConfigEditor::Key(IdKey::PlaylistDeleteAll)),
                Box::new(ConfigPlaylistDeleteAll::new(config)),
                vec![],
            )
            .is_ok());
        assert!(self
            .app
            .remount(
                Id::ConfigEditor(IdConfigEditor::Key(IdKey::PlaylistAddFront)),
                Box::new(ConfigPlaylistAddFront::new(config)),
                vec![],
            )
            .is_ok());
        assert!(self
            .app
            .remount(
                Id::ConfigEditor(IdConfigEditor::Key(IdKey::PlaylistShuffle)),
                Box::new(ConfigPlaylistShuffle::new(config)),
                vec![],
            )
            .is_ok());
        assert!(self
            .app
            .remount(
                Id::ConfigEditor(IdConfigEditor::Key(IdKey::PlaylistSearch)),
                Box::new(ConfigPlaylistSearch::new(config)),
                vec![],
            )
            .is_ok());
        assert!(self
            .app
            .remount(
                Id::ConfigEditor(IdConfigEditor::Key(IdKey::PlaylistPlaySelected)),
                Box::new(ConfigPlaylistPlaySelected::new(config)),
                vec![],
            )
            .is_ok());
        assert!(self
            .app
            .remount(
                Id::ConfigEditor(IdConfigEditor::Key(IdKey::PlaylistModeCycle)),
                Box::new(ConfigPlaylistModeCycle::new(config)),
                vec![],
            )
            .is_ok());

        assert!(self
            .app
            .remount(
                Id::ConfigEditor(IdConfigEditor::Key(IdKey::PlaylistSwapDown)),
                Box::new(ConfigPlaylistSwapDown::new(config)),
                vec![],
            )
            .is_ok());

        assert!(self
            .app
            .remount(
                Id::ConfigEditor(IdConfigEditor::Key(IdKey::PlaylistSwapUp)),
                Box::new(ConfigPlaylistSwapUp::new(config)),
                vec![],
            )
            .is_ok());

        assert!(self
            .app
            .remount(
                Id::ConfigEditor(IdConfigEditor::Key(IdKey::DatabaseAddAll)),
                Box::new(ConfigDatabaseAddAll::new(config)),
                vec![],
            )
            .is_ok());

        assert!(self
            .app
            .remount(
                Id::ConfigEditor(IdConfigEditor::Key(IdKey::GlobalConfig)),
                Box::new(ConfigGlobalConfig::new(config)),
                vec![],
            )
            .is_ok());

        assert!(self
            .app
            .remount(
                Id::ConfigEditor(IdConfigEditor::Key(IdKey::PlaylistLqueue)),
                Box::new(ConfigPlaylistLqueue::new(config)),
                vec![],
            )
            .is_ok());

        assert!(self
            .app
            .remount(
                Id::ConfigEditor(IdConfigEditor::Key(IdKey::PlaylistTqueue)),
                Box::new(ConfigPlaylistTqueue::new(config)),
                vec![],
            )
            .is_ok());

        assert!(self
            .app
            .remount(
                Id::ConfigEditor(IdConfigEditor::Key(IdKey::LibrarySwitchRoot)),
                Box::new(ConfigLibrarySwitchRoot::new(config)),
                vec![],
            )
            .is_ok());

        assert!(self
            .app
            .remount(
                Id::ConfigEditor(IdConfigEditor::Key(IdKey::LibraryAddRoot)),
                Box::new(ConfigLibraryAddRoot::new(config)),
                vec![],
            )
            .is_ok());

        assert!(self
            .app
            .remount(
                Id::ConfigEditor(IdConfigEditor::Key(IdKey::LibraryRemoveRoot)),
                Box::new(ConfigLibraryRemoveRoot::new(config)),
                vec![],
            )
            .is_ok());

        assert!(self
            .app
            .remount(
                Id::ConfigEditor(IdConfigEditor::Key(IdKey::GlobalSavePlaylist)),
                Box::new(ConfigGlobalSavePlaylist::new(config)),
                vec![],
            )
            .is_ok());

        assert!(self
            .app
            .remount(
                Id::ConfigEditor(IdConfigEditor::Key(IdKey::GlobalLayoutPodcast)),
                Box::new(ConfigGlobalLayoutPodcast::new(config)),
                vec![],
            )
            .is_ok());

        assert!(self
            .app
            .remount(
                Id::ConfigEditor(IdConfigEditor::Key(IdKey::GlobalXywhMoveLeft)),
                Box::new(ConfigGlobalXywhMoveLeft::new(config)),
                vec![],
            )
            .is_ok());
        assert!(self
            .app
            .remount(
                Id::ConfigEditor(IdConfigEditor::Key(IdKey::GlobalXywhMoveRight)),
                Box::new(ConfigGlobalXywhMoveRight::new(config)),
                vec![],
            )
            .is_ok());
        assert!(self
            .app
            .remount(
                Id::ConfigEditor(IdConfigEditor::Key(IdKey::GlobalXywhMoveUp)),
                Box::new(ConfigGlobalXywhMoveUp::new(config)),
                vec![],
            )
            .is_ok());
        assert!(self
            .app
            .remount(
                Id::ConfigEditor(IdConfigEditor::Key(IdKey::GlobalXywhMoveDown)),
                Box::new(ConfigGlobalXywhMoveDown::new(config)),
                vec![],
            )
            .is_ok());
        assert!(self
            .app
            .remount(
                Id::ConfigEditor(IdConfigEditor::Key(IdKey::GlobalXywhZoomIn)),
                Box::new(ConfigGlobalXywhZoomIn::new(config)),
                vec![],
            )
            .is_ok());
        assert!(self
            .app
            .remount(
                Id::ConfigEditor(IdConfigEditor::Key(IdKey::GlobalXywhZoomOut)),
                Box::new(ConfigGlobalXywhZoomOut::new(config)),
                vec![],
            )
            .is_ok());
        assert!(self
            .app
            .remount(
                Id::ConfigEditor(IdConfigEditor::Key(IdKey::GlobalXywhHide)),
                Box::new(ConfigGlobalXywhHide::new(config)),
                vec![],
            )
            .is_ok());
        assert!(self
            .app
            .remount(
                Id::ConfigEditor(IdConfigEditor::Key(IdKey::PodcastMarkPlayed)),
                Box::new(ConfigPodcastMarkPlayed::new(config)),
                vec![],
            )
            .is_ok());
        assert!(self
            .app
            .remount(
                Id::ConfigEditor(IdConfigEditor::Key(IdKey::PodcastMarkAllPlayed)),
                Box::new(ConfigPodcastMarkAllPlayed::new(config)),
                vec![],
            )
            .is_ok());
        assert!(self
            .app
            .remount(
                Id::ConfigEditor(IdConfigEditor::Key(IdKey::PodcastEpDownload)),
                Box::new(ConfigPodcastEpDownload::new(config)),
                vec![],
            )
            .is_ok());
        assert!(self
            .app
            .remount(
                Id::ConfigEditor(IdConfigEditor::Key(IdKey::PodcastEpDeleteFile)),
                Box::new(ConfigPodcastEpDeleteFile::new(config)),
                vec![],
            )
            .is_ok());
        assert!(self
            .app
            .remount(
                Id::ConfigEditor(IdConfigEditor::Key(IdKey::PodcastDeleteFeed)),
                Box::new(ConfigPodcastDeleteFeed::new(config)),
                vec![],
            )
            .is_ok());
        assert!(self
            .app
            .remount(
                Id::ConfigEditor(IdConfigEditor::Key(IdKey::PodcastDeleteAllFeeds)),
                Box::new(ConfigPodcastDeleteAllFeeds::new(config)),
                vec![],
            )
            .is_ok());
        assert!(self
            .app
            .remount(
                Id::ConfigEditor(IdConfigEditor::Key(IdKey::PodcastRefreshFeed)),
                Box::new(ConfigPodcastRefreshFeed::new(config)),
                vec![],
            )
            .is_ok());
        assert!(self
            .app
            .remount(
                Id::ConfigEditor(IdConfigEditor::Key(IdKey::PodcastRefreshAllFeeds)),
                Box::new(ConfigPodcastRefreshAllFeeds::new(config)),
                vec![],
            )
            .is_ok());
        assert!(self
            .app
            .remount(
                Id::ConfigEditor(IdConfigEditor::Key(IdKey::PodcastSearchAddFeed)),
                Box::new(ConfigPodcastSearchAddFeed::new(config)),
                vec![],
            )
            .is_ok());
        self.theme_select_sync();
    }

    #[allow(clippy::too_many_lines)]
    pub fn umount_config_editor(&mut self) {
        self.library_reload_tree();
        self.playlist_reload();
        self.database_reload();
        self.progress_reload();
        self.mount_label_help();
        self.lyric_reload();

        assert!(self
            .app
            .umount(&Id::ConfigEditor(IdConfigEditor::Header))
            .is_ok());

        assert!(self
            .app
            .umount(&Id::ConfigEditor(IdConfigEditor::Footer))
            .is_ok());

        assert!(self
            .app
            .umount(&Id::ConfigEditor(IdConfigEditor::MusicDir))
            .is_ok());

        assert!(self
            .app
            .umount(&Id::ConfigEditor(IdConfigEditor::ExitConfirmation))
            .is_ok());

        assert!(self
            .app
            .umount(&Id::ConfigEditor(IdConfigEditor::PlaylistDisplaySymbol))
            .is_ok());

        assert!(self
            .app
            .umount(&Id::ConfigEditor(IdConfigEditor::PlaylistRandomAlbum))
            .is_ok());
        assert!(self
            .app
            .umount(&Id::ConfigEditor(IdConfigEditor::PlaylistRandomTrack))
            .is_ok());

        assert!(self
            .app
            .umount(&Id::ConfigEditor(IdConfigEditor::PodcastDir))
            .is_ok());
        assert!(self
            .app
            .umount(&Id::ConfigEditor(IdConfigEditor::PodcastSimulDownload))
            .is_ok());
        assert!(self
            .app
            .umount(&Id::ConfigEditor(IdConfigEditor::PodcastMaxRetries))
            .is_ok());
        assert!(self
            .app
            .umount(&Id::ConfigEditor(IdConfigEditor::AlbumPhotoAlign))
            .is_ok());

        assert!(self
            .app
            .umount(&Id::ConfigEditor(IdConfigEditor::SaveLastPosition))
            .is_ok());

        assert!(self
            .app
            .umount(&Id::ConfigEditor(IdConfigEditor::SeekStep))
            .is_ok());

        assert!(self
            .app
            .umount(&Id::ConfigEditor(IdConfigEditor::KillDamon))
            .is_ok());

        assert!(self
            .app
            .umount(&Id::ConfigEditor(IdConfigEditor::PlayerUseMpris))
            .is_ok());

        assert!(self
            .app
            .umount(&Id::ConfigEditor(IdConfigEditor::PlayerUseDiscord))
            .is_ok());

        assert!(self
            .app
            .umount(&Id::ConfigEditor(IdConfigEditor::PlayerPort))
            .is_ok());

        assert!(self
            .app
            .umount(&Id::ConfigEditor(IdConfigEditor::CEThemeSelect))
            .is_ok());

        assert!(self
            .app
            .umount(&Id::ConfigEditor(IdConfigEditor::LibraryLabel))
            .is_ok());

        assert!(self
            .app
            .umount(&Id::ConfigEditor(IdConfigEditor::LibraryForeground))
            .is_ok());

        assert!(self
            .app
            .umount(&Id::ConfigEditor(IdConfigEditor::LibraryBackground))
            .is_ok());
        assert!(self
            .app
            .umount(&Id::ConfigEditor(IdConfigEditor::LibraryBorder))
            .is_ok());
        assert!(self
            .app
            .umount(&Id::ConfigEditor(IdConfigEditor::LibraryHighlight))
            .is_ok());

        assert!(self
            .app
            .umount(&Id::ConfigEditor(IdConfigEditor::PlaylistLabel))
            .is_ok());

        assert!(self
            .app
            .umount(&Id::ConfigEditor(IdConfigEditor::PlaylistForeground))
            .is_ok());

        assert!(self
            .app
            .umount(&Id::ConfigEditor(IdConfigEditor::PlaylistBackground))
            .is_ok());
        assert!(self
            .app
            .umount(&Id::ConfigEditor(IdConfigEditor::PlaylistBorder))
            .is_ok());
        assert!(self
            .app
            .umount(&Id::ConfigEditor(IdConfigEditor::PlaylistHighlight))
            .is_ok());
        assert!(self
            .app
            .umount(&Id::ConfigEditor(IdConfigEditor::ProgressLabel))
            .is_ok());

        assert!(self
            .app
            .umount(&Id::ConfigEditor(IdConfigEditor::ProgressForeground))
            .is_ok());

        assert!(self
            .app
            .umount(&Id::ConfigEditor(IdConfigEditor::ProgressBackground))
            .is_ok());
        assert!(self
            .app
            .umount(&Id::ConfigEditor(IdConfigEditor::ProgressBorder))
            .is_ok());
        assert!(self
            .app
            .umount(&Id::ConfigEditor(IdConfigEditor::LyricLabel))
            .is_ok());

        assert!(self
            .app
            .umount(&Id::ConfigEditor(IdConfigEditor::LyricForeground))
            .is_ok());

        assert!(self
            .app
            .umount(&Id::ConfigEditor(IdConfigEditor::LyricBackground))
            .is_ok());
        assert!(self
            .app
            .umount(&Id::ConfigEditor(IdConfigEditor::LyricBorder))
            .is_ok());
        assert!(self
            .app
            .umount(&Id::ConfigEditor(IdConfigEditor::LibraryHighlightSymbol))
            .is_ok());
        assert!(self
            .app
            .umount(&Id::ConfigEditor(IdConfigEditor::PlaylistHighlightSymbol))
            .is_ok());

        // umount keys global

        self.app
            .umount(&Id::ConfigEditor(IdConfigEditor::Key(IdKey::GlobalQuit)))
            .ok();
        self.app
            .umount(&Id::ConfigEditor(IdConfigEditor::Key(IdKey::GlobalLeft)))
            .ok();
        self.app
            .umount(&Id::ConfigEditor(IdConfigEditor::Key(IdKey::GlobalRight)))
            .ok();
        self.app
            .umount(&Id::ConfigEditor(IdConfigEditor::Key(IdKey::GlobalUp)))
            .ok();
        self.app
            .umount(&Id::ConfigEditor(IdConfigEditor::Key(IdKey::GlobalDown)))
            .ok();
        self.app
            .umount(&Id::ConfigEditor(IdConfigEditor::Key(IdKey::GlobalGotoTop)))
            .ok();
        self.app
            .umount(&Id::ConfigEditor(IdConfigEditor::Key(
                IdKey::GlobalGotoBottom,
            )))
            .ok();
        self.app
            .umount(&Id::ConfigEditor(IdConfigEditor::Key(
                IdKey::GlobalPlayerTogglePause,
            )))
            .ok();
        self.app
            .umount(&Id::ConfigEditor(IdConfigEditor::Key(
                IdKey::GlobalPlayerNext,
            )))
            .ok();
        self.app
            .umount(&Id::ConfigEditor(IdConfigEditor::Key(
                IdKey::GlobalPlayerPrevious,
            )))
            .ok();
        self.app
            .umount(&Id::ConfigEditor(IdConfigEditor::Key(IdKey::GlobalHelp)))
            .ok();
        self.app
            .umount(&Id::ConfigEditor(IdConfigEditor::Key(
                IdKey::GlobalVolumeUp,
            )))
            .ok();
        self.app
            .umount(&Id::ConfigEditor(IdConfigEditor::Key(
                IdKey::GlobalVolumeDown,
            )))
            .ok();

        self.app
            .umount(&Id::ConfigEditor(IdConfigEditor::Key(
                IdKey::GlobalPlayerSeekForward,
            )))
            .ok();
        self.app
            .umount(&Id::ConfigEditor(IdConfigEditor::Key(
                IdKey::GlobalPlayerSeekBackward,
            )))
            .ok();
        self.app
            .umount(&Id::ConfigEditor(IdConfigEditor::Key(
                IdKey::GlobalPlayerSpeedUp,
            )))
            .ok();
        self.app
            .umount(&Id::ConfigEditor(IdConfigEditor::Key(
                IdKey::GlobalPlayerSpeedDown,
            )))
            .ok();
        self.app
            .umount(&Id::ConfigEditor(IdConfigEditor::Key(
                IdKey::GlobalLyricAdjustForward,
            )))
            .ok();
        self.app
            .umount(&Id::ConfigEditor(IdConfigEditor::Key(
                IdKey::GlobalLyricAdjustBackward,
            )))
            .ok();
        self.app
            .umount(&Id::ConfigEditor(IdConfigEditor::Key(
                IdKey::GlobalLyricCycle,
            )))
            .ok();
        self.app
            .umount(&Id::ConfigEditor(IdConfigEditor::Key(
                IdKey::GlobalLayoutDatabase,
            )))
            .ok();
        self.app
            .umount(&Id::ConfigEditor(IdConfigEditor::Key(
                IdKey::GlobalLayoutTreeview,
            )))
            .ok();
        self.app
            .umount(&Id::ConfigEditor(IdConfigEditor::Key(
                IdKey::GlobalPlayerToggleGapless,
            )))
            .ok();

        // umount keys other
        self.app
            .umount(&Id::ConfigEditor(IdConfigEditor::Key(IdKey::LibraryDelete)))
            .ok();
        self.app
            .umount(&Id::ConfigEditor(IdConfigEditor::Key(
                IdKey::LibraryLoadDir,
            )))
            .ok();
        self.app
            .umount(&Id::ConfigEditor(IdConfigEditor::Key(IdKey::LibraryYank)))
            .ok();
        self.app
            .umount(&Id::ConfigEditor(IdConfigEditor::Key(IdKey::LibraryPaste)))
            .ok();
        self.app
            .umount(&Id::ConfigEditor(IdConfigEditor::Key(IdKey::LibrarySearch)))
            .ok();
        self.app
            .umount(&Id::ConfigEditor(IdConfigEditor::Key(
                IdKey::LibrarySearchYoutube,
            )))
            .ok();
        self.app
            .umount(&Id::ConfigEditor(IdConfigEditor::Key(
                IdKey::LibraryTagEditor,
            )))
            .ok();
        self.app
            .umount(&Id::ConfigEditor(IdConfigEditor::Key(
                IdKey::PlaylistDelete,
            )))
            .ok();
        self.app
            .umount(&Id::ConfigEditor(IdConfigEditor::Key(
                IdKey::PlaylistDeleteAll,
            )))
            .ok();
        self.app
            .umount(&Id::ConfigEditor(IdConfigEditor::Key(
                IdKey::PlaylistShuffle,
            )))
            .ok();
        self.app
            .umount(&Id::ConfigEditor(IdConfigEditor::Key(
                IdKey::PlaylistModeCycle,
            )))
            .ok();
        self.app
            .umount(&Id::ConfigEditor(IdConfigEditor::Key(
                IdKey::PlaylistPlaySelected,
            )))
            .ok();
        self.app
            .umount(&Id::ConfigEditor(IdConfigEditor::Key(
                IdKey::PlaylistAddFront,
            )))
            .ok();
        self.app
            .umount(&Id::ConfigEditor(IdConfigEditor::Key(
                IdKey::PlaylistSearch,
            )))
            .ok();
        self.app
            .umount(&Id::ConfigEditor(IdConfigEditor::Key(
                IdKey::PlaylistSwapDown,
            )))
            .ok();
        self.app
            .umount(&Id::ConfigEditor(IdConfigEditor::Key(
                IdKey::PlaylistSwapUp,
            )))
            .ok();
        self.app
            .umount(&Id::ConfigEditor(IdConfigEditor::Key(
                IdKey::DatabaseAddAll,
            )))
            .ok();
        self.app
            .umount(&Id::ConfigEditor(IdConfigEditor::Key(IdKey::GlobalConfig)))
            .ok();
        self.app
            .umount(&Id::ConfigEditor(IdConfigEditor::Key(
                IdKey::PlaylistLqueue,
            )))
            .ok();

        self.app
            .umount(&Id::ConfigEditor(IdConfigEditor::Key(
                IdKey::PlaylistTqueue,
            )))
            .ok();

        self.app
            .umount(&Id::ConfigEditor(IdConfigEditor::Key(
                IdKey::LibrarySwitchRoot,
            )))
            .ok();

        self.app
            .umount(&Id::ConfigEditor(IdConfigEditor::Key(
                IdKey::LibraryAddRoot,
            )))
            .ok();

        self.app
            .umount(&Id::ConfigEditor(IdConfigEditor::Key(
                IdKey::LibraryRemoveRoot,
            )))
            .ok();

        self.app
            .umount(&Id::ConfigEditor(IdConfigEditor::Key(
                IdKey::GlobalSavePlaylist,
            )))
            .ok();
        self.app
            .umount(&Id::ConfigEditor(IdConfigEditor::Key(
                IdKey::GlobalLayoutPodcast,
            )))
            .ok();

        self.app
            .umount(&Id::ConfigEditor(IdConfigEditor::Key(
                IdKey::GlobalXywhMoveLeft,
            )))
            .ok();
        self.app
            .umount(&Id::ConfigEditor(IdConfigEditor::Key(
                IdKey::GlobalXywhMoveRight,
            )))
            .ok();
        self.app
            .umount(&Id::ConfigEditor(IdConfigEditor::Key(
                IdKey::GlobalXywhMoveUp,
            )))
            .ok();
        self.app
            .umount(&Id::ConfigEditor(IdConfigEditor::Key(
                IdKey::GlobalXywhMoveDown,
            )))
            .ok();
        self.app
            .umount(&Id::ConfigEditor(IdConfigEditor::Key(
                IdKey::GlobalXywhZoomIn,
            )))
            .ok();
        self.app
            .umount(&Id::ConfigEditor(IdConfigEditor::Key(
                IdKey::GlobalXywhZoomOut,
            )))
            .ok();
        self.app
            .umount(&Id::ConfigEditor(IdConfigEditor::Key(
                IdKey::GlobalXywhHide,
            )))
            .ok();

        self.app
            .umount(&Id::ConfigEditor(IdConfigEditor::Key(
                IdKey::PodcastMarkPlayed,
            )))
            .ok();
        self.app
            .umount(&Id::ConfigEditor(IdConfigEditor::Key(
                IdKey::PodcastMarkAllPlayed,
            )))
            .ok();
        self.app
            .umount(&Id::ConfigEditor(IdConfigEditor::Key(
                IdKey::PodcastEpDownload,
            )))
            .ok();
        self.app
            .umount(&Id::ConfigEditor(IdConfigEditor::Key(
                IdKey::PodcastEpDeleteFile,
            )))
            .ok();
        self.app
            .umount(&Id::ConfigEditor(IdConfigEditor::Key(
                IdKey::PodcastDeleteFeed,
            )))
            .ok();
        self.app
            .umount(&Id::ConfigEditor(IdConfigEditor::Key(
                IdKey::PodcastDeleteAllFeeds,
            )))
            .ok();
        self.app
            .umount(&Id::ConfigEditor(IdConfigEditor::Key(
                IdKey::PodcastRefreshFeed,
            )))
            .ok();
        self.app
            .umount(&Id::ConfigEditor(IdConfigEditor::Key(
                IdKey::PodcastRefreshAllFeeds,
            )))
            .ok();
        self.app
            .umount(&Id::ConfigEditor(IdConfigEditor::Key(
                IdKey::PodcastSearchAddFeed,
            )))
            .ok();
        assert!(self
            .app
            .remount(
                Id::GlobalListener,
                Box::new(GlobalListener::new(&self.config.keys)),
                Self::subscribe(&self.config.keys),
            )
            .is_ok());

        if let Err(e) = self.update_photo() {
            self.mount_error_popup(format!("update photo error: {e}"));
        }
    }

    pub fn action_change_layout(&mut self) {
        match self.config_layout {
            ConfigEditorLayout::General => self.config_layout = ConfigEditorLayout::Color,

            ConfigEditorLayout::Color => self.config_layout = ConfigEditorLayout::Key1,
            ConfigEditorLayout::Key1 => self.config_layout = ConfigEditorLayout::Key2,
            ConfigEditorLayout::Key2 => self.config_layout = ConfigEditorLayout::General,
        }

        let layout = self.config_layout.clone();
        assert!(self
            .app
            .remount(
                Id::ConfigEditor(IdConfigEditor::Header),
                Box::new(CEHeader::new(&layout, &self.config)),
                vec![]
            )
            .is_ok());
        match self.config_layout {
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
                .active(&Id::ConfigEditor(IdConfigEditor::Key(IdKey::GlobalQuit)))
                .ok(),
            ConfigEditorLayout::Key2 => self
                .app
                .active(&Id::ConfigEditor(IdConfigEditor::Key(
                    IdKey::LibraryTagEditor,
                )))
                .ok(),
        };
    }

    /// Mount quit popup
    pub fn mount_config_save_popup(&mut self) {
        assert!(self
            .app
            .remount(
                Id::ConfigEditor(IdConfigEditor::ConfigSavePopup),
                Box::new(ConfigSavePopup::new(&self.config)),
                vec![]
            )
            .is_ok());
        assert!(self
            .app
            .active(&Id::ConfigEditor(IdConfigEditor::ConfigSavePopup))
            .is_ok());
    }

    #[allow(clippy::too_many_lines)]
    pub fn collect_config_data(&mut self) -> Result<()> {
        if self.ke_key_config.has_unique_elements() {
            self.config.keys = self.ke_key_config.clone();
        } else {
            bail!("Duplicate key config found, no changes are saved.");
        }
        self.config.style_color_symbol = self.ce_style_color_symbol.clone();
        if let Ok(State::One(StateValue::String(music_dir))) =
            self.app.state(&Id::ConfigEditor(IdConfigEditor::MusicDir))
        {
            // self.config.music_dir = music_dir;
            // let mut vec = Vec::new();
            let vec = music_dir
                .split(';')
                .map(std::string::ToString::to_string)
                .filter(|p| {
                    let absolute_dir = shellexpand::tilde(p).to_string();
                    let path = Path::new(&absolute_dir);
                    path.exists()
                })
                .collect();
            self.config.music_dir = vec;
        }

        if let Ok(State::One(StateValue::Usize(exit_confirmation))) = self
            .app
            .state(&Id::ConfigEditor(IdConfigEditor::ExitConfirmation))
        {
            self.config.enable_exit_confirmation = matches!(exit_confirmation, 0);
        }

        if let Ok(State::One(StateValue::Usize(display_symbol))) = self
            .app
            .state(&Id::ConfigEditor(IdConfigEditor::PlaylistDisplaySymbol))
        {
            self.config.playlist_display_symbol = matches!(display_symbol, 0);
        }

        if let Ok(State::One(StateValue::String(random_track_quantity_str))) = self
            .app
            .state(&Id::ConfigEditor(IdConfigEditor::PlaylistRandomTrack))
        {
            if let Ok(quantity) = random_track_quantity_str.parse::<u32>() {
                self.config.playlist_select_random_track_quantity = quantity;
            }
        }

        if let Ok(State::One(StateValue::String(random_album_quantity_str))) = self
            .app
            .state(&Id::ConfigEditor(IdConfigEditor::PlaylistRandomAlbum))
        {
            if let Ok(quantity) = random_album_quantity_str.parse::<u32>() {
                self.config.playlist_select_random_album_quantity = quantity;
            }
        }

        if let Ok(State::One(StateValue::String(podcast_dir))) = self
            .app
            .state(&Id::ConfigEditor(IdConfigEditor::PodcastDir))
        {
            let absolute_dir = shellexpand::tilde(&podcast_dir).to_string();
            let path = Path::new(&absolute_dir);
            if path.exists() {
                self.config.podcast_dir = absolute_dir;
            }
        }
        if let Ok(State::One(StateValue::String(podcast_simul_download))) = self
            .app
            .state(&Id::ConfigEditor(IdConfigEditor::PodcastSimulDownload))
        {
            if let Ok(quantity) = podcast_simul_download.parse::<usize>() {
                if (1..101).contains(&quantity) {
                    self.config.podcast_simultanious_download = quantity;
                } else {
                    bail!(" It's not suggested to set simultanious download to bigger than 100. ");
                }
            }
        }
        if let Ok(State::One(StateValue::String(podcast_max_retries))) = self
            .app
            .state(&Id::ConfigEditor(IdConfigEditor::PodcastMaxRetries))
        {
            if let Ok(quantity) = podcast_max_retries.parse::<usize>() {
                if (1..11).contains(&quantity) {
                    self.config.podcast_max_retries = quantity;
                } else {
                    bail!(" It's not recommended to set max retries to bigger than 10. ");
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
            self.config.album_photo_xywh.align = align;
        }

        if let Ok(State::One(StateValue::Usize(save_last_position))) = self
            .app
            .state(&Id::ConfigEditor(IdConfigEditor::SaveLastPosition))
        {
            let save_last_position = match save_last_position {
                0 => LastPosition::Auto,
                1 => LastPosition::No,
                2 => LastPosition::Yes,
                _ => bail!("Remember playing position must be set to auto, yes or no."),
            };
            self.config.player_remember_last_played_position = save_last_position;
        }

        if let Ok(State::One(StateValue::Usize(seek_step))) =
            self.app.state(&Id::ConfigEditor(IdConfigEditor::SeekStep))
        {
            let seek_step = match seek_step {
                0 => SeekStep::Auto,
                1 => SeekStep::Short,
                2 => SeekStep::Long,
                _ => bail!("Shouldn't happend here."),
            };
            self.config.player_seek_step = seek_step;
        }

        if let Ok(State::One(StateValue::Usize(kill_daemon))) =
            self.app.state(&Id::ConfigEditor(IdConfigEditor::KillDamon))
        {
            self.config.kill_daemon_when_quit = matches!(kill_daemon, 0);
        }

        if let Ok(State::One(StateValue::Usize(player_use_mpris))) = self
            .app
            .state(&Id::ConfigEditor(IdConfigEditor::PlayerUseMpris))
        {
            self.config.player_use_mpris = matches!(player_use_mpris, 0);
        }

        if let Ok(State::One(StateValue::Usize(player_use_discord))) = self
            .app
            .state(&Id::ConfigEditor(IdConfigEditor::PlayerUseDiscord))
        {
            self.config.player_use_discord = matches!(player_use_discord, 0);
        }

        if let Ok(State::One(StateValue::String(player_port))) = self
            .app
            .state(&Id::ConfigEditor(IdConfigEditor::PlayerPort))
        {
            if let Ok(port) = player_port.parse::<u16>() {
                if (1000..u16::MAX).contains(&port) {
                    self.config.player_port = port;
                } else {
                    bail!(" It's not recommended to set player port less than 1000. ");
                }
            }
        }
        Ok(())
    }

    pub fn theme_select_save() -> Result<()> {
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
    pub fn theme_select_load_themes(&mut self) -> Result<()> {
        let mut path = get_app_config_path()?;
        path.push("themes");
        if let Ok(paths) = std::fs::read_dir(path) {
            let mut paths: Vec<_> = paths.filter_map(std::result::Result::ok).collect();

            paths.sort_by_cached_key(|k| get_pin_yin(&k.file_name().to_string_lossy()));
            for p in paths {
                self.ce_themes.push(p.path().to_string_lossy().to_string());
            }
        }

        Ok(())
    }

    pub fn theme_select_sync(&mut self) {
        let mut table: TableBuilder = TableBuilder::default();

        for (idx, record) in self.ce_themes.iter().enumerate() {
            if idx > 0 {
                table.add_row();
            }

            let path = PathBuf::from(record);
            let name = path.file_stem();

            if let Some(n) = name {
                table
                    .add_col(TextSpan::new(idx.to_string()))
                    .add_col(TextSpan::new(n.to_string_lossy()));
            }
        }
        if self.ce_themes.is_empty() {
            table.add_col(TextSpan::from("0"));
            table.add_col(TextSpan::from("empty theme list"));
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
        let mut index = 0;
        for (idx, v) in self.ce_themes.iter().enumerate() {
            if *v == self.ce_style_color_symbol.alacritty_theme.path {
                index = idx;
                break;
            }
        }
        assert!(self
            .app
            .attr(
                &Id::ConfigEditor(IdConfigEditor::CEThemeSelect),
                Attribute::Value,
                AttrValue::Payload(PropPayload::One(PropValue::Usize(index))),
            )
            .is_ok());
    }
}
