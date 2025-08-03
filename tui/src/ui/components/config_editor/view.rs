use crate::ui::Application;
use crate::ui::components::config_editor::update::THEMES_WITHOUT_FILES;
use crate::ui::components::raw::dynamic_height_grid::DynamicHeightGrid;
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
use termusiclib::types::{IdKey, KFGLOBAL_FOCUS_ORDER, KFOTHER_FOCUS_ORDER, Msg};
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
use tuirealm::ratatui::layout::{Constraint, Layout, Rect};
use tuirealm::ratatui::widgets::Clear;
use tuirealm::{AttrValue, Attribute, Frame, State, StateValue};

// NOTE: the macros either have to be in a different file OR be defined *before* they are used, otherwise they are not in scope

/// Chain many values together and saturated added value from it.
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

/// Convert to a `Box<[]>`, but allow spacing without formatting messing it up.
///
/// Equivalent to manually doing `Box::from([val1, val2])`.
macro_rules! to_boxed_slice {
    ($first:expr$(,)?) => {
        $first
    };
    (
        $first:expr
        $(
            , $second:expr
        )* $(,)?
    ) => {
        Box::from([$first, $($second,)*])
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

    /// Split the frame area into header, main and footer,
    /// also Draw the Header and footer and return the main area.
    fn view_config_editor_common(
        app: &mut Application<Id, Msg, UserEvent>,
        f: &mut Frame<'_>,
    ) -> Rect {
        let [header, chunk_main, footer] = Layout::vertical([
            Constraint::Length(3), // config header
            Constraint::Min(3),
            Constraint::Length(1), // config footer
        ])
        .areas(f.area());

        app.view(&Id::ConfigEditor(IdConfigEditor::Header), f, header);

        // draw before main chunk, to easily tell if something is overdrawing
        app.view(&Id::ConfigEditor(IdConfigEditor::Footer), f, footer);

        chunk_main
    }

    /// Draw the keys for tab "General"
    #[allow(clippy::too_many_lines)]
    fn view_config_editor_general(&mut self) {
        self.terminal
            .raw_mut()
            .draw(|f| {
                let chunk_main = Self::view_config_editor_common(&mut self.app, f);

                let focus_elem = self
                    .app
                    .focus()
                    .and_then(|v| {
                        if let Id::ConfigEditor(id) = *v {
                            Some(id)
                        } else {
                            None
                        }
                    })
                    .and_then(|v| {
                        Some(match v {
                            IdConfigEditor::MusicDir => 0,
                            IdConfigEditor::ExitConfirmation => 1,
                            IdConfigEditor::PlaylistDisplaySymbol => 2,
                            IdConfigEditor::PlaylistRandomTrack => 3,
                            IdConfigEditor::PlaylistRandomAlbum => 4,
                            IdConfigEditor::PodcastDir => 5,
                            IdConfigEditor::PodcastSimulDownload => 6,
                            IdConfigEditor::PodcastMaxRetries => 7,
                            IdConfigEditor::AlbumPhotoAlign => 8,
                            IdConfigEditor::SaveLastPosition => 9,
                            IdConfigEditor::SeekStep => 10,
                            IdConfigEditor::KillDamon => 11,
                            IdConfigEditor::PlayerUseMpris => 12,
                            IdConfigEditor::PlayerUseDiscord => 13,
                            IdConfigEditor::PlayerPort => 14,
                            IdConfigEditor::ExtraYtdlpArgs => 15,
                            _ => return None,
                        })
                    });

                let cells = UniformDynamicGrid::new(16, 3, 56 + 2)
                    .draw_row_low_space()
                    .distribute_row_space()
                    .focus_node(focus_elem)
                    .split(chunk_main);

                self.app
                    .view(&Id::ConfigEditor(IdConfigEditor::MusicDir), f, cells[0]);
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::ExitConfirmation),
                    f,
                    cells[1],
                );

                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::PlaylistDisplaySymbol),
                    f,
                    cells[2],
                );

                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::PlaylistRandomTrack),
                    f,
                    cells[3],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::PlaylistRandomAlbum),
                    f,
                    cells[4],
                );

                self.app
                    .view(&Id::ConfigEditor(IdConfigEditor::PodcastDir), f, cells[5]);
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::PodcastSimulDownload),
                    f,
                    cells[6],
                );
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::PodcastMaxRetries),
                    f,
                    cells[7],
                );

                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::AlbumPhotoAlign),
                    f,
                    cells[8],
                );

                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::SaveLastPosition),
                    f,
                    cells[9],
                );
                self.app
                    .view(&Id::ConfigEditor(IdConfigEditor::SeekStep), f, cells[10]);

                self.app
                    .view(&Id::ConfigEditor(IdConfigEditor::KillDamon), f, cells[11]);

                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::PlayerUseMpris),
                    f,
                    cells[12],
                );

                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::PlayerUseDiscord),
                    f,
                    cells[13],
                );

                self.app
                    .view(&Id::ConfigEditor(IdConfigEditor::PlayerPort), f, cells[14]);

                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::ExtraYtdlpArgs),
                    f,
                    cells[15],
                );

                Self::view_config_editor_popups(&mut self.app, f);
            })
            .expect("Expected to draw without error");
    }

    /// Draw common Popups while in the config editor
    fn view_config_editor_popups(app: &mut Application<Id, Msg, UserEvent>, f: &mut Frame<'_>) {
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

    /// Draw the keys for tab "Themes and Colors"
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
                let chunk_main = Self::view_config_editor_common(&mut self.app, f);

                let [left, right] =
                    Layout::horizontal([Constraint::Ratio(1, 4), Constraint::Ratio(3, 4)])
                        .areas(chunk_main);

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

                // NOTE: the elements below have to be in the order they are draw and blurred(focused) in:
                let elem_height = to_boxed_slice! {
                    library_height,
                    playlist_height,
                    progress_height,
                    lyric_height,
                    important_popup_height,
                    fallback_height,
                };

                let focus_elem = self
                    .app
                    .focus()
                    .and_then(|v| {
                        if let Id::ConfigEditor(id) = *v {
                            Some(id)
                        } else {
                            None
                        }
                    })
                    .and_then(|v| {
                        Some(match v {
                            IdConfigEditor::LibraryLabel
                            | IdConfigEditor::LibraryForeground
                            | IdConfigEditor::LibraryBackground
                            | IdConfigEditor::LibraryBorder
                            | IdConfigEditor::LibraryHighlight
                            | IdConfigEditor::LibraryHighlightSymbol => 0,
                            IdConfigEditor::PlaylistLabel
                            | IdConfigEditor::PlaylistForeground
                            | IdConfigEditor::PlaylistBackground
                            | IdConfigEditor::PlaylistBorder
                            | IdConfigEditor::PlaylistHighlight
                            | IdConfigEditor::PlaylistHighlightSymbol
                            | IdConfigEditor::CurrentlyPlayingTrackSymbol => 1,
                            IdConfigEditor::ProgressLabel
                            | IdConfigEditor::ProgressForeground
                            | IdConfigEditor::ProgressBackground
                            | IdConfigEditor::ProgressBorder => 2,
                            IdConfigEditor::LyricLabel
                            | IdConfigEditor::LyricForeground
                            | IdConfigEditor::LyricBackground
                            | IdConfigEditor::LyricBorder => 3,
                            IdConfigEditor::ImportantPopupLabel
                            | IdConfigEditor::ImportantPopupForeground
                            | IdConfigEditor::ImportantPopupBackground
                            | IdConfigEditor::ImportantPopupBorder => 4,
                            IdConfigEditor::FallbackLabel
                            | IdConfigEditor::FallbackForeground
                            | IdConfigEditor::FallbackBackground
                            | IdConfigEditor::FallbackBorder
                            | IdConfigEditor::FallbackHighlight => 5,
                            _ => return None,
                        })
                    });

                let cells = DynamicHeightGrid::new(elem_height, 16 + 2)
                    .with_row_spacing(1)
                    .draw_row_low_space()
                    .distribute_row_space()
                    .focus_node(focus_elem)
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
                    .view(&Id::ConfigEditor(IdConfigEditor::CEThemeSelect), f, left);

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

                Self::view_config_editor_popups(&mut self.app, f);
            })
            .expect("Expected to draw without error");
    }

    /// Draw the keys for tab "Key Global"
    fn view_config_editor_key1(&mut self) {
        self.terminal
            .raw_mut()
            .draw(|f| {
                let chunk_main = Self::view_config_editor_common(&mut self.app, f);

                KeyDisplay::new(KFGLOBAL_FOCUS_ORDER, 23 + 2).view(&mut self.app, chunk_main, f);

                Self::view_config_editor_popups(&mut self.app, f);
            })
            .expect("Expected to draw without error");
    }

    /// Draw the keys for tab "Key Other"
    fn view_config_editor_key2(&mut self) {
        self.terminal
            .raw_mut()
            .draw(|f| {
                let chunk_main = Self::view_config_editor_common(&mut self.app, f);

                KeyDisplay::new(KFOTHER_FOCUS_ORDER, 25 + 2).view(&mut self.app, chunk_main, f);

                Self::view_config_editor_popups(&mut self.app, f);
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
                .active(&Id::ConfigEditor(KFGLOBAL_FOCUS_ORDER[0].into()))
                .ok(),
            ConfigEditorLayout::Key2 => self
                .app
                .active(&Id::ConfigEditor(KFOTHER_FOCUS_ORDER[0].into()))
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

/// Enum which determines what [`KeyDisplay`] will and look for focus.
#[derive(Debug, Clone, Copy, PartialEq)]
enum KeyDisplayType {
    Global,
    Other,
}

/// Helper type to draw "Key" tabs.
#[derive(Debug)]
struct KeyDisplay<'a> {
    elems: &'a [IdKey],
    discriminant: KeyDisplayType,
    width: u16,
}

impl<'a> KeyDisplay<'a> {
    /// Create a new instance, where the elements `elems` are drawn in that order with at least a width of `width`.
    ///
    /// NOTE: all given [`IdKey`]s need to be of the same discriminant!
    pub fn new(elems: &'a [IdKey], width: u16) -> Self {
        // figure out which kind of tab is draw by looking at the first element
        let discriminant = std::mem::discriminant(&IdConfigEditor::from(&elems[0]));

        let discriminant = if discriminant
            == std::mem::discriminant(&IdConfigEditor::KeyGlobal(IdKeyGlobal::Config))
        {
            KeyDisplayType::Global
        } else if discriminant
            == std::mem::discriminant(&IdConfigEditor::KeyOther(IdKeyOther::DatabaseAddAll))
        {
            KeyDisplayType::Other
        } else {
            unimplemented!("Invalid Discriminant: {:#?}", discriminant)
        };

        Self {
            elems,
            discriminant,
            width,
        }
    }

    /// Actually draw the elements in the current instance.
    pub fn view(&self, model: &mut Application<Id, Msg, UserEvent>, area: Rect, f: &mut Frame<'_>) {
        /// Gets the state of `Id::ConfigEditor(id)` and if it has a `State::One`, returns `yes`, otherwise `no`.
        ///
        /// Macro to apply "DRY"(Dont-Repeat-Yourself) to reduce function length.
        macro_rules! is_expanded {
            ($id:expr, $yes:expr, $no:expr) => {
                match model.state(&Id::ConfigEditor($id)) {
                    Ok(State::One(_)) => $no,
                    _ => $yes,
                }
            };
        }

        // determine what heights each element should have
        let mut elems_heights = Vec::with_capacity(self.elems.len());

        for id in self.elems {
            elems_heights.push(is_expanded!(IdConfigEditor::from(id), 8, 3));
        }

        // find the focused element, if any
        let focus_elem = model
            .focus()
            .and_then(|v| match self.discriminant {
                KeyDisplayType::Global => {
                    if let Id::ConfigEditor(IdConfigEditor::KeyGlobal(key)) = *v {
                        Some(IdKey::Global(key))
                    } else {
                        None
                    }
                }
                KeyDisplayType::Other => {
                    if let Id::ConfigEditor(IdConfigEditor::KeyOther(key)) = *v {
                        Some(IdKey::Other(key))
                    } else {
                        None
                    }
                }
            })
            .and_then(|focus| {
                self.elems
                    .iter()
                    .enumerate()
                    .find(|(_, v)| **v == focus)
                    .map(|(idx, _)| idx)
            });

        let cells = DynamicHeightGrid::new(elems_heights, self.width)
            .draw_row_low_space()
            .distribute_row_space()
            .focus_node(focus_elem)
            .split(area);

        // actually draw each element
        for (id, cell) in self.elems.iter().zip(cells.iter()) {
            model.view(&Id::ConfigEditor(id.into()), f, *cell);
        }
    }
}
