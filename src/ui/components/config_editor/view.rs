use crate::ui::components::{
    AlbumPhotoAlign, AlbumPhotoWidth, AlbumPhotoX, AlbumPhotoY, CEHeader, CEThemeSelectTable,
    ConfigLibraryBackground, ConfigLibraryBorder, ConfigLibraryForeground, ConfigLibraryHighlight,
    ConfigLibraryHighlightSymbol, ConfigLibraryTitle, ConfigLyricBackground, ConfigLyricBorder,
    ConfigLyricForeground, ConfigLyricTitle, ConfigPlaylistBackground, ConfigPlaylistBorder,
    ConfigPlaylistForeground, ConfigPlaylistHighlight, ConfigPlaylistHighlightSymbol,
    ConfigPlaylistTitle, ConfigProgressBackground, ConfigProgressBorder, ConfigProgressForeground,
    ConfigProgressTitle, ConfigSavePopup, ExitConfirmation, Footer, MusicDir,
    PlaylistDisplaySymbol, PlaylistRandomAlbum, PlaylistRandomTrack,
};
// use crate::ui::components::*;
use crate::utils::draw_area_in_absolute;

use crate::ui::components::Alignment as XywhAlign;
use crate::ui::model::{ConfigEditorLayout, Model};
use crate::ui::{Application, Id, IdConfigEditor, Msg};
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
                    .margin(1)
                    .constraints(
                        [
                            Constraint::Length(3),
                            Constraint::Min(3),
                            Constraint::Length(2),
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
                    .margin(1)
                    .constraints(
                        [
                            Constraint::Length(3),
                            Constraint::Min(3),
                            Constraint::Length(2),
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

    pub fn view_config_editor_key1(&mut self) {
        assert!(self
            .terminal
            .raw_mut()
            .draw(|f| {
                let chunks_main = Layout::default()
                    .direction(Direction::Vertical)
                    .margin(1)
                    .constraints(
                        [
                            Constraint::Length(3),
                            Constraint::Min(3),
                            Constraint::Length(2),
                        ]
                        .as_ref(),
                    )
                    .split(f.size());

                self.app
                    .view(&Id::ConfigEditor(IdConfigEditor::Header), f, chunks_main[0]);
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::MusicDir),
                    f,
                    chunks_main[1],
                );
                self.app
                    .view(&Id::ConfigEditor(IdConfigEditor::Footer), f, chunks_main[2]);

                Self::view_config_editor_commons(f, &mut self.app);
            })
            .is_ok());
    }

    pub fn view_config_editor_key2(&mut self) {
        assert!(self
            .terminal
            .raw_mut()
            .draw(|f| {
                let chunks_main = Layout::default()
                    .direction(Direction::Vertical)
                    .margin(1)
                    .constraints(
                        [
                            Constraint::Length(3),
                            Constraint::Min(3),
                            Constraint::Length(2),
                        ]
                        .as_ref(),
                    )
                    .split(f.size());

                self.app
                    .view(&Id::ConfigEditor(IdConfigEditor::Header), f, chunks_main[0]);
                self.app.view(
                    &Id::ConfigEditor(IdConfigEditor::MusicDir),
                    f,
                    chunks_main[1],
                );
                self.app
                    .view(&Id::ConfigEditor(IdConfigEditor::Footer), f, chunks_main[2]);

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
                Box::new(Footer::default()),
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

        // Mount color page
        assert!(self
            .app
            .remount(
                Id::ConfigEditor(IdConfigEditor::CEThemeSelect),
                Box::new(CEThemeSelectTable::default()),
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
                Box::new(ConfigLibraryForeground::new(&self.config)),
                vec![]
            )
            .is_ok());

        assert!(self
            .app
            .remount(
                Id::ConfigEditor(IdConfigEditor::LibraryBackground),
                Box::new(ConfigLibraryBackground::new(&self.config)),
                vec![]
            )
            .is_ok());

        assert!(self
            .app
            .remount(
                Id::ConfigEditor(IdConfigEditor::LibraryBorder),
                Box::new(ConfigLibraryBorder::new(&self.config)),
                vec![]
            )
            .is_ok());

        assert!(self
            .app
            .remount(
                Id::ConfigEditor(IdConfigEditor::LibraryHighlight),
                Box::new(ConfigLibraryHighlight::new(&self.config)),
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
                Box::new(ConfigPlaylistForeground::new(&self.config)),
                vec![]
            )
            .is_ok());

        assert!(self
            .app
            .remount(
                Id::ConfigEditor(IdConfigEditor::PlaylistBackground),
                Box::new(ConfigPlaylistBackground::new(&self.config)),
                vec![]
            )
            .is_ok());

        assert!(self
            .app
            .remount(
                Id::ConfigEditor(IdConfigEditor::PlaylistBorder),
                Box::new(ConfigPlaylistBorder::new(&self.config)),
                vec![]
            )
            .is_ok());

        assert!(self
            .app
            .remount(
                Id::ConfigEditor(IdConfigEditor::PlaylistHighlight),
                Box::new(ConfigPlaylistHighlight::new(&self.config)),
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
                Box::new(ConfigProgressForeground::new(&self.config)),
                vec![]
            )
            .is_ok());

        assert!(self
            .app
            .remount(
                Id::ConfigEditor(IdConfigEditor::ProgressBackground),
                Box::new(ConfigProgressBackground::new(&self.config)),
                vec![]
            )
            .is_ok());

        assert!(self
            .app
            .remount(
                Id::ConfigEditor(IdConfigEditor::ProgressBorder),
                Box::new(ConfigProgressBorder::new(&self.config)),
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
                Box::new(ConfigLyricForeground::new(&self.config)),
                vec![]
            )
            .is_ok());

        assert!(self
            .app
            .remount(
                Id::ConfigEditor(IdConfigEditor::LyricBackground),
                Box::new(ConfigLyricBackground::new(&self.config)),
                vec![]
            )
            .is_ok());

        assert!(self
            .app
            .remount(
                Id::ConfigEditor(IdConfigEditor::LyricBorder),
                Box::new(ConfigLyricBorder::new(&self.config)),
                vec![]
            )
            .is_ok());

        assert!(self
            .app
            .remount(
                Id::ConfigEditor(IdConfigEditor::LibraryHighlightSymbol),
                Box::new(ConfigLibraryHighlightSymbol::new(&self.config)),
                vec![]
            )
            .is_ok());

        assert!(self
            .app
            .remount(
                Id::ConfigEditor(IdConfigEditor::PlaylistHighlightSymbol),
                Box::new(ConfigPlaylistHighlightSymbol::new(&self.config)),
                vec![]
            )
            .is_ok());

        // Active Config Editor
        assert!(self
            .app
            .active(&Id::ConfigEditor(IdConfigEditor::MusicDir))
            .is_ok());

        if let Err(e) = Self::theme_select_save() {
            self.mount_error_popup(format!("theme save error: {}", e).as_str());
        }
        if let Err(e) = self.theme_select_load_themes() {
            self.mount_error_popup(format!("Error load themes: {}", e).as_str());
        }
        self.ce_theme_select_sync();
        self.app.lock_subs();
        if let Err(e) = self.update_photo() {
            self.mount_error_popup(format!("clear photo error: {}", e).as_str());
        }
    }

    pub fn umount_config_editor(&mut self) {
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
        if let Err(e) = self.update_photo() {
            self.mount_error_popup(format!("update photo error: {}", e).as_ref());
        }
        self.app.unlock_subs();
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
            ConfigEditorLayout::Key1 => Some(()),
            ConfigEditorLayout::Key2 => None,
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

    pub fn collect_config_data(&mut self) {
        if let Ok(State::One(StateValue::String(music_dir))) =
            self.app.state(&Id::ConfigEditor(IdConfigEditor::MusicDir))
        {
            self.config.music_dir = music_dir;
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
                self.config.album_photo_xywh.x_between_1_100 = quantity;
            }
        }
        if let Ok(State::One(StateValue::String(album_photo_y_between_1_100_str))) = self
            .app
            .state(&Id::ConfigEditor(IdConfigEditor::AlbumPhotoY))
        {
            if let Ok(quantity) = album_photo_y_between_1_100_str.parse::<u32>() {
                self.config.album_photo_xywh.y_between_1_100 = quantity;
            }
        }
        if let Ok(State::One(StateValue::String(album_photo_width_between_1_100_str))) = self
            .app
            .state(&Id::ConfigEditor(IdConfigEditor::AlbumPhotoWidth))
        {
            if let Ok(quantity) = album_photo_width_between_1_100_str.parse::<u32>() {
                self.config.album_photo_xywh.width_between_1_100 = quantity;
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
    }
}
