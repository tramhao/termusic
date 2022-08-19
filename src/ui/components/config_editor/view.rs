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
use crate::config::Settings;
use crate::ui::components::{
    AlbumPhotoAlign, AlbumPhotoWidth, AlbumPhotoX, AlbumPhotoY, CEHeader, CEThemeSelectTable,
    ConfigDatabaseAddAll, ConfigGlobalConfig, ConfigGlobalDown, ConfigGlobalGotoBottom,
    ConfigGlobalGotoTop, ConfigGlobalHelp, ConfigGlobalLayoutDatabase, ConfigGlobalLayoutTreeview,
    ConfigGlobalLeft, ConfigGlobalLyricAdjustBackward, ConfigGlobalLyricAdjustForward,
    ConfigGlobalLyricCycle, ConfigGlobalPlayerNext, ConfigGlobalPlayerPrevious,
    ConfigGlobalPlayerSeekBackward, ConfigGlobalPlayerSeekForward, ConfigGlobalPlayerSpeedDown,
    ConfigGlobalPlayerSpeedUp, ConfigGlobalPlayerToggleGapless, ConfigGlobalPlayerTogglePause,
    ConfigGlobalQuit, ConfigGlobalRight, ConfigGlobalUp, ConfigGlobalVolumeDown,
    ConfigGlobalVolumeUp, ConfigLibraryAddRoot, ConfigLibraryBackground, ConfigLibraryBorder,
    ConfigLibraryDelete, ConfigLibraryForeground, ConfigLibraryHighlight,
    ConfigLibraryHighlightSymbol, ConfigLibraryLoadDir, ConfigLibraryPaste,
    ConfigLibraryRemoveRoot, ConfigLibrarySearch, ConfigLibrarySearchYoutube,
    ConfigLibrarySwitchRoot, ConfigLibraryTagEditor, ConfigLibraryTitle, ConfigLibraryYank,
    ConfigLyricBackground, ConfigLyricBorder, ConfigLyricForeground, ConfigLyricTitle,
    ConfigPlaylistAddFront, ConfigPlaylistBackground, ConfigPlaylistBorder, ConfigPlaylistDelete,
    ConfigPlaylistDeleteAll, ConfigPlaylistForeground, ConfigPlaylistHighlight,
    ConfigPlaylistHighlightSymbol, ConfigPlaylistLqueue, ConfigPlaylistModeCycle,
    ConfigPlaylistPlaySelected, ConfigPlaylistSearch, ConfigPlaylistShuffle,
    ConfigPlaylistSwapDown, ConfigPlaylistSwapUp, ConfigPlaylistTitle, ConfigPlaylistTqueue,
    ConfigProgressBackground, ConfigProgressBorder, ConfigProgressForeground, ConfigProgressTitle,
    ConfigSavePopup, ExitConfirmation, Footer, GlobalListener, MusicDir, PlaylistDisplaySymbol,
    PlaylistRandomAlbum, PlaylistRandomTrack,
};
use crate::utils::draw_area_in_absolute;

use crate::ui::components::Alignment as XywhAlign;
use crate::ui::model::{ConfigEditorLayout, Model};
use crate::ui::{Application, Id, IdConfigEditor, IdKey, Msg};
use anyhow::{bail, Result};
use std::path::Path;
use tuirealm::event::NoUserEvent;
use tuirealm::tui::layout::{Constraint, Direction, Layout};
use tuirealm::tui::widgets::Clear;
use tuirealm::Frame;
use tuirealm::{State, StateValue};

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
                            Constraint::Min(2),
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
                            Constraint::Min(2),
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
                    &Id::ConfigEditor(IdConfigEditor::AlbumPhotoX),
                    f,
                    chunks_middle_right[0],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::AlbumPhotoY),
                    f,
                    chunks_middle_right[1],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::AlbumPhotoWidth),
                    f,
                    chunks_middle_right[2],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::AlbumPhotoAlign),
                    f,
                    chunks_middle_right[3],
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
                            // Constraint::Length(),
                            // Constraint::Length(),
                            // Constraint::Length(),
                            Constraint::Min(0),
                        ]
                        .as_ref(),
                    )
                    .split(chunks_middle[2]);

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
                            Constraint::Min(0),
                        ]
                        .as_ref(),
                    )
                    .split(chunks_middle[2]);

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
                Id::ConfigEditor(IdConfigEditor::AlbumPhotoX),
                Box::new(AlbumPhotoX::new(&self.config)),
                vec![]
            )
            .is_ok());

        assert!(self
            .app
            .remount(
                Id::ConfigEditor(IdConfigEditor::AlbumPhotoY),
                Box::new(AlbumPhotoY::new(&self.config)),
                vec![]
            )
            .is_ok());

        assert!(self
            .app
            .remount(
                Id::ConfigEditor(IdConfigEditor::AlbumPhotoWidth),
                Box::new(AlbumPhotoWidth::new(&self.config)),
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

        let config = self.config.clone();
        self.remount_config_color(&config);

        // Active Config Editor
        assert!(self
            .app
            .active(&Id::ConfigEditor(IdConfigEditor::MusicDir))
            .is_ok());

        if let Err(e) = self.theme_select_load_themes() {
            self.mount_error_popup(format!("Error load themes: {}", e));
        }
        self.theme_select_sync();
        if let Err(e) = self.update_photo() {
            self.mount_error_popup(format!("clear photo error: {}", e));
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
                Box::new(ConfigLibraryTitle::default()),
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
                Box::new(ConfigPlaylistTitle::default()),
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
                Box::new(ConfigProgressTitle::default()),
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
                Box::new(ConfigLyricTitle::default()),
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

        self.theme_select_sync();
    }

    #[allow(clippy::too_many_lines)]
    pub fn umount_config_editor(&mut self) {
        self.library_reload_tree();
        self.playlist_reload();
        self.database_reload();
        self.progress_reload();
        self.remount_label_help(None, None, None);
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
            .umount(&Id::ConfigEditor(IdConfigEditor::AlbumPhotoX))
            .is_ok());
        assert!(self
            .app
            .umount(&Id::ConfigEditor(IdConfigEditor::AlbumPhotoY))
            .is_ok());
        assert!(self
            .app
            .umount(&Id::ConfigEditor(IdConfigEditor::AlbumPhotoWidth))
            .is_ok());
        assert!(self
            .app
            .umount(&Id::ConfigEditor(IdConfigEditor::AlbumPhotoAlign))
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

        assert!(self
            .app
            .remount(
                Id::GlobalListener,
                Box::new(GlobalListener::new(&self.config.keys)),
                Self::subscribe(&self.config.keys),
            )
            .is_ok());

        if let Err(e) = self.update_photo() {
            self.mount_error_popup(format!("update photo error: {}", e));
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

        if let Ok(State::One(StateValue::String(album_photo_x_between_1_100_str))) = self
            .app
            .state(&Id::ConfigEditor(IdConfigEditor::AlbumPhotoX))
        {
            if let Ok(quantity) = album_photo_x_between_1_100_str.parse::<u32>() {
                if (1..101).contains(&quantity) {
                    self.config.album_photo_xywh.x_between_1_100 = quantity;
                } else {
                    bail!("album photo x should be between 1 and 100");
                }
            }
        }
        if let Ok(State::One(StateValue::String(album_photo_y_between_1_100_str))) = self
            .app
            .state(&Id::ConfigEditor(IdConfigEditor::AlbumPhotoY))
        {
            if let Ok(quantity) = album_photo_y_between_1_100_str.parse::<u32>() {
                if (1..101).contains(&quantity) {
                    self.config.album_photo_xywh.y_between_1_100 = quantity;
                } else {
                    bail!("album photo y should be between 1 and 100");
                }
            }
        }
        if let Ok(State::One(StateValue::String(album_photo_width_between_1_100_str))) = self
            .app
            .state(&Id::ConfigEditor(IdConfigEditor::AlbumPhotoWidth))
        {
            if let Ok(quantity) = album_photo_width_between_1_100_str.parse::<u32>() {
                if (1..101).contains(&quantity) {
                    self.config.album_photo_xywh.width_between_1_100 = quantity;
                } else {
                    bail!("album photo width should be between 1 and 100");
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
        Ok(())
    }
}
