//! ## Components
//!
//! demo example components

mod general_search;
/**
 * MIT License
 *
 * tui-realm - Copyright (C) 2021 Christian Visintin
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
// -- modules
// mod clock;
// mod counter;
mod label;
mod lyric;
mod music_library;
mod playlist;
mod popups;
mod progress;
// mod table_playlist;
mod color_editor;
mod key_editor;
mod tag_editor;
mod xywh;
mod youtube_search;

// -- export
// pub use clock::Clock;
// pub use counter::{Digit, Letter};
pub use general_search::{GSInputPopup, GSTablePopup, Source};
pub use label::Label;
pub use lyric::Lyric;
pub use music_library::MusicLibrary;
pub use playlist::Playlist;
pub use popups::{
    DeleteConfirmInputPopup, DeleteConfirmRadioPopup, ErrorPopup, HelpPopup, MessagePopup,
    QuitPopup,
};
pub use progress::Progress;
// pub use table_playlist::Table;
pub use youtube_search::{YSInputPopup, YSTablePopup};
//Tag Edotor Controls
pub use color_editor::{
    load_alacritty_theme, AlacrittyTheme, CEHelpPopup, CELibraryBackground, CELibraryBorder,
    CELibraryForeground, CELibraryHighlight, CELibraryHighlightSymbol, CELibraryTitle,
    CELyricBackground, CELyricBorder, CELyricForeground, CELyricTitle, CEPlaylistBackground,
    CEPlaylistBorder, CEPlaylistForeground, CEPlaylistHighlight, CEPlaylistHighlightSymbol,
    CEPlaylistTitle, CEProgressBackground, CEProgressBorder, CEProgressForeground, CEProgressTitle,
    CERadioOk, CESelectColor, ColorConfig, StyleColorSymbol, ThemeSelectTable,
};
// pub use key_editor::{
//     KEGlobalDown, KEGlobalDownInput, KEGlobalGotoBottom, KEGlobalGotoBottomInput, KEGlobalGotoTop,
//     KEGlobalGotoTopInput, KEGlobalHelp, KEGlobalHelpInput, KEGlobalLeft, KEGlobalLeftInput,
//     KEGlobalPlayerNext, KEGlobalPlayerNextInput, KEGlobalPlayerPrevious,
//     KEGlobalPlayerPreviousInput, KEGlobalPlayerTogglePause, KEGlobalPlayerTogglePauseInput,
//     KEGlobalQuit, KEGlobalQuitInput, KEGlobalRight, KEGlobalRightInput, KEGlobalUp,
//     KEGlobalUpInput, KEGlobalVolumeDown, KEGlobalVolumeDownInput, KEGlobalVolumeUp,
//     KEGlobalVolumeUpInput, KEHelpPopup, KERadioOk, KeyBind, Keys, MODIFIER_LIST,
// };
pub use key_editor::*;
pub use tag_editor::{
    TECounterDelete, TEHelpPopup, TEInputArtist, TEInputTitle, TERadioTag, TESelectLyric,
    TETableLyricOptions, TETextareaLyric,
};
pub use xywh::Xywh;

use crate::player::GeneralP;
use crate::ui::{CEMsg, GSMsg, Id, KEMsg, Loop, Model, Msg, PLMsg, Status, YSMsg};
use tui_realm_stdlib::Phantom;
use tuirealm::event::NoUserEvent;
use tuirealm::props::{Alignment, Borders, Color, Style};
use tuirealm::tui::layout::{Constraint, Direction, Layout, Rect};
use tuirealm::tui::widgets::Block;
use tuirealm::{Component, Event, MockComponent, Sub, SubClause, SubEventClause};

#[derive(MockComponent)]
pub struct GlobalListener {
    component: Phantom,
    keys: Keys,
}

impl GlobalListener {
    pub fn new(keys: &Keys) -> Self {
        Self {
            component: Phantom::default(),
            keys: keys.clone(),
        }
    }
}

impl Component<Msg, NoUserEvent> for GlobalListener {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        match ev {
            Event::WindowResize(..) => Some(Msg::UpdatePhoto),
            Event::Keyboard(keyevent) if keyevent == self.keys.global_esc.key_event() => {
                Some(Msg::QuitPopupShow)
            }
            Event::Keyboard(keyevent) if keyevent == self.keys.global_quit.key_event() => {
                Some(Msg::QuitPopupShow)
            }
            Event::Keyboard(keyevent)
                if keyevent == self.keys.global_player_toggle_pause.key_event() =>
            {
                Some(Msg::PlayerTogglePause)
            }
            Event::Keyboard(keyevent) if keyevent == self.keys.global_player_next.key_event() => {
                Some(Msg::Playlist(PLMsg::NextSong))
            }
            Event::Keyboard(keyevent)
                if keyevent == self.keys.global_player_previous.key_event() =>
            {
                Some(Msg::Playlist(PLMsg::PrevSong))
            }
            Event::Keyboard(keyevent)
                if keyevent == self.keys.global_player_volume_minus_1.key_event() =>
            {
                Some(Msg::PlayerVolumeDown)
            }
            Event::Keyboard(keyevent)
                if keyevent == self.keys.global_player_volume_minus_2.key_event() =>
            {
                Some(Msg::PlayerVolumeDown)
            }
            Event::Keyboard(keyevent)
                if keyevent == self.keys.global_player_volume_plus_1.key_event() =>
            {
                Some(Msg::PlayerVolumeUp)
            }
            Event::Keyboard(keyevent)
                if keyevent == self.keys.global_player_volume_plus_2.key_event() =>
            {
                Some(Msg::PlayerVolumeUp)
            }
            Event::Keyboard(keyevent) if keyevent == self.keys.global_help.key_event() => {
                Some(Msg::HelpPopupShow)
            }
            Event::Keyboard(keyevent)
                if keyevent == self.keys.global_player_seek_forward.key_event() =>
            {
                Some(Msg::PlayerSeek(5))
            }
            Event::Keyboard(keyevent)
                if keyevent == self.keys.global_player_seek_backward.key_event() =>
            {
                Some(Msg::PlayerSeek(-5))
            }
            Event::Keyboard(keyevent)
                if keyevent == self.keys.global_lyric_adjust_forward.key_event() =>
            {
                Some(Msg::LyricAdjustDelay(1000))
            }
            Event::Keyboard(keyevent)
                if keyevent == self.keys.global_lyric_adjust_backward.key_event() =>
            {
                Some(Msg::LyricAdjustDelay(-1000))
            }
            Event::Keyboard(keyevent) if keyevent == self.keys.global_lyric_cycle.key_event() => {
                Some(Msg::LyricCycle)
            }

            Event::Keyboard(keyevent)
                if keyevent == self.keys.global_color_editor_open.key_event() =>
            {
                Some(Msg::ColorEditor(CEMsg::ColorEditorShow))
            }
            Event::Keyboard(keyevent)
                if keyevent == self.keys.global_key_editor_open.key_event() =>
            {
                Some(Msg::KeyEditor(KEMsg::KeyEditorShow))
            }

            _ => None,
        }
    }
}

impl Model {
    /// global listener subscriptions
    // #[allow(clippy::too_many_lines)]
    pub fn subscribe(keys: &Keys) -> Vec<Sub<Id, NoUserEvent>> {
        vec![
            Sub::new(
                SubEventClause::Keyboard(keys.global_esc.key_event()),
                SubClause::Always,
            ),
            Sub::new(
                SubEventClause::Keyboard(keys.global_quit.key_event()),
                SubClause::Always,
            ),
            Sub::new(
                SubEventClause::Keyboard(keys.global_player_toggle_pause.key_event()),
                SubClause::Always,
            ),
            Sub::new(
                SubEventClause::Keyboard(keys.global_player_next.key_event()),
                SubClause::Always,
            ),
            Sub::new(
                SubEventClause::Keyboard(keys.global_player_previous.key_event()),
                SubClause::Always,
            ),
            Sub::new(
                SubEventClause::Keyboard(keys.global_player_volume_minus_1.key_event()),
                SubClause::Always,
            ),
            Sub::new(
                SubEventClause::Keyboard(keys.global_player_volume_minus_2.key_event()),
                SubClause::Always,
            ),
            Sub::new(
                SubEventClause::Keyboard(keys.global_player_volume_plus_1.key_event()),
                SubClause::Always,
            ),
            Sub::new(
                SubEventClause::Keyboard(keys.global_player_volume_plus_2.key_event()),
                SubClause::Always,
            ),
            Sub::new(
                SubEventClause::Keyboard(keys.global_help.key_event()),
                SubClause::Always,
            ),
            Sub::new(
                SubEventClause::Keyboard(keys.global_player_seek_forward.key_event()),
                SubClause::Always,
            ),
            Sub::new(
                SubEventClause::Keyboard(keys.global_player_seek_backward.key_event()),
                SubClause::Always,
            ),
            Sub::new(
                SubEventClause::Keyboard(keys.global_lyric_adjust_forward.key_event()),
                SubClause::Always,
            ),
            Sub::new(
                SubEventClause::Keyboard(keys.global_lyric_adjust_backward.key_event()),
                SubClause::Always,
            ),
            Sub::new(
                SubEventClause::Keyboard(keys.global_lyric_cycle.key_event()),
                SubClause::Always,
            ),
            Sub::new(
                SubEventClause::Keyboard(keys.global_color_editor_open.key_event()),
                SubClause::Always,
            ),
            Sub::new(
                SubEventClause::Keyboard(keys.global_key_editor_open.key_event()),
                SubClause::Always,
            ),
            Sub::new(SubEventClause::WindowResize, SubClause::Always),
        ]
    }
    pub fn player_next(&mut self) {
        if self.playlist_items.is_empty() {
            return;
        }
        self.time_pos = 0;
        self.time_pos_elapsed = std::time::Instant::now();
        self.status = Some(Status::Running);
        if let Some(song) = self.playlist_items.pop_front() {
            if let Some(file) = song.file() {
                self.player.add_and_play(file);
                #[cfg(feature = "mpris")]
                self.mpris.add_and_play(file);
            }
            match self.config.loop_mode {
                Loop::Playlist => self.playlist_items.push_back(song.clone()),
                Loop::Single => self.playlist_items.push_front(song.clone()),
                Loop::Queue => {}
            }
            self.playlist_sync();
            self.current_song = Some(song);
            if let Err(e) = self.update_photo() {
                self.mount_error_popup(format!("update photo error: {}", e).as_str());
            };
            self.progress_update_title();
            self.update_playing_song();
        }
    }

    pub fn player_previous(&mut self) {
        if let Loop::Single | Loop::Queue = self.config.loop_mode {
            return;
        }

        if self.playlist_items.is_empty() {
            return;
        }

        if let Some(song) = self.playlist_items.pop_back() {
            self.playlist_items.push_front(song);
        }
        if let Some(song) = self.playlist_items.pop_back() {
            self.playlist_items.push_front(song);
        }
        self.player_next();
    }

    pub fn player_toggle_pause(&mut self) {
        if self.player.is_paused() {
            self.status = Some(Status::Running);
            self.player.resume();
            #[cfg(feature = "mpris")]
            self.mpris.resume();
        } else {
            self.status = Some(Status::Paused);
            self.player.pause();
            #[cfg(feature = "mpris")]
            self.mpris.pause();
        }
    }

    pub fn player_seek(&mut self, offset: i64) {
        self.player.seek(offset).ok();
        if let Ok((_, time_pos, _)) = self.player.get_progress() {
            if let Some(t) = std::time::Instant::now()
                .checked_sub(std::time::Duration::from_secs(time_pos.try_into().unwrap()))
            {
                self.time_pos_elapsed = t;
            }
        }
        self.progress_update();
    }
}
///
/// Get block
pub fn get_block<'a>(props: &Borders, title: (String, Alignment), focus: bool) -> Block<'a> {
    Block::default()
        .borders(props.sides)
        .border_style(if focus {
            props.style()
        } else {
            Style::default().fg(Color::Reset).bg(Color::Reset)
        })
        .border_type(props.modifiers)
        .title(title.0)
        .title_alignment(title.1)
}

// Draw an area (WxH / 3) in the middle of the parent area
pub fn draw_area_in_relative(parent: Rect, width: u16, height: u16) -> Rect {
    let new_area = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - height) / 2),
                Constraint::Percentage(height),
                Constraint::Percentage((100 - height) / 2),
            ]
            .as_ref(),
        )
        .split(parent);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - width) / 2),
                Constraint::Percentage(width),
                Constraint::Percentage((100 - width) / 2),
            ]
            .as_ref(),
        )
        .split(new_area[1])[1]
}

pub fn draw_area_in_absolute(parent: Rect, width: u16, height: u16) -> Rect {
    let new_area = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length((parent.height - height) / 2),
                Constraint::Length(height),
                Constraint::Length((parent.height - height) / 2),
            ]
            .as_ref(),
        )
        .split(parent);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Length((parent.width - width) / 2),
                Constraint::Length(width),
                Constraint::Length((parent.width - width) / 2),
            ]
            .as_ref(),
        )
        .split(new_area[1])[1]
}

pub fn draw_area_top_right_absolute(parent: Rect, width: u16, height: u16) -> Rect {
    let new_area = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(1),
                Constraint::Length(height),
                Constraint::Length(parent.height - height - 1),
            ]
            .as_ref(),
        )
        .split(parent);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Length(parent.width - width - 1),
                Constraint::Length(width),
                Constraint::Length(1),
            ]
            .as_ref(),
        )
        .split(new_area[1])[1]
}

#[cfg(test)]
mod tests {

    use super::*;

    use pretty_assertions::assert_eq;

    #[test]
    fn test_utils_ui_draw_area_in() {
        let area: Rect = Rect::new(0, 0, 1024, 512);
        let child: Rect = draw_area_in_relative(area, 75, 30);
        assert_eq!(child.x, 43);
        assert_eq!(child.y, 63);
        assert_eq!(child.width, 271);
        assert_eq!(child.height, 54);
    }
}
