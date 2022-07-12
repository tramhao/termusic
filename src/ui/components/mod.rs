mod color_editor;
mod config_editor;
mod database;
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
mod general_search;
mod key_editor;
mod labels;
mod lyric;
mod music_library;
mod playlist;
mod popups;
mod progress;
mod tag_editor;
mod xywh;
mod youtube_search;

// -- export
pub use general_search::{GSInputPopup, GSTablePopup, Source};
pub use labels::LabelGeneric;
pub use lyric::Lyric;
pub use music_library::MusicLibrary;
pub use playlist::Playlist;
pub use popups::{
    DeleteConfirmInputPopup, DeleteConfirmRadioPopup, ErrorPopup, HelpPopup, MessagePopup,
    QuitPopup,
};
pub use progress::Progress;
pub use youtube_search::{YSInputPopup, YSTablePopup};
//Tag Editor Controls
pub use color_editor::{
    CEHelpPopup, CELibraryBackground, CELibraryBorder, CELibraryForeground, CELibraryHighlight,
    CELibraryHighlightSymbol, CELibraryTitle, CELyricBackground, CELyricBorder, CELyricForeground,
    CELyricTitle, CEPlaylistBackground, CEPlaylistBorder, CEPlaylistForeground,
    CEPlaylistHighlight, CEPlaylistHighlightSymbol, CEPlaylistTitle, CEProgressBackground,
    CEProgressBorder, CEProgressForeground, CEProgressTitle, CERadioOk, CESelectColor,
    ThemeSelectTable,
};
pub use config_editor::*;
pub use database::{DBListCriteria, DBListSearchResult, DBListSearchTracks};
pub use key_editor::*;
pub use tag_editor::{
    TECounterDelete, TEHelpPopup, TEInputArtist, TEInputTitle, TERadioTag, TESelectLyric,
    TETableLyricOptions, TETextareaLyric,
};
pub use xywh::{Alignment, Xywh};

use crate::config::Keys;
use crate::player::{Loop, PlayerTrait, Status};
// #[cfg(any(feature = "mpris", feature = "discord"))]
// use crate::track::Track;
use crate::ui::model::TermusicLayout;
use crate::ui::{CEMsg, ConfigEditorMsg, GSMsg, Id, KEMsg, Model, Msg, PLMsg, YSMsg};
use tui_realm_stdlib::Phantom;
use tuirealm::event::NoUserEvent;
use tuirealm::props::{AttrValue, Attribute};
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
    #[allow(clippy::too_many_lines)]
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
                if keyevent == self.keys.global_player_speed_up.key_event() =>
            {
                Some(Msg::PlayerSpeedUp)
            }
            Event::Keyboard(keyevent)
                if keyevent == self.keys.global_player_speed_down.key_event() =>
            {
                Some(Msg::PlayerSpeedDown)
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

            Event::Keyboard(keyevent)
                if keyevent == self.keys.global_layout_treeview.key_event() =>
            {
                Some(Msg::LayoutTreeView)
            }

            Event::Keyboard(keyevent)
                if keyevent == self.keys.global_layout_database.key_event() =>
            {
                Some(Msg::LayoutDataBase)
            }

            Event::Keyboard(keyevent)
                if keyevent == self.keys.global_player_toggle_gapless.key_event() =>
            {
                Some(Msg::PlayerToggleGapless)
            }

            Event::Keyboard(keyevent) if keyevent == self.keys.global_config_open.key_event() => {
                Some(Msg::ConfigEditor(ConfigEditorMsg::Open))
            }

            _ => None,
        }
    }
}

impl Model {
    /// global listener subscriptions
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
                SubEventClause::Keyboard(keys.global_player_speed_up.key_event()),
                SubClause::Always,
            ),
            Sub::new(
                SubEventClause::Keyboard(keys.global_player_speed_down.key_event()),
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
            Sub::new(
                SubEventClause::Keyboard(keys.global_layout_treeview.key_event()),
                SubClause::Always,
            ),
            Sub::new(
                SubEventClause::Keyboard(keys.global_layout_database.key_event()),
                SubClause::Always,
            ),
            Sub::new(
                SubEventClause::Keyboard(keys.global_player_toggle_gapless.key_event()),
                SubClause::Always,
            ),
            Sub::new(
                SubEventClause::Keyboard(keys.global_config_open.key_event()),
                SubClause::Always,
            ),
            Sub::new(SubEventClause::WindowResize, SubClause::Always),
        ]
    }

    pub fn player_stop(&mut self) {
        self.time_pos = 0;
        self.player.set_status(Status::Stopped);
        self.player.playlist.current_track = None;
        self.player.stop();
        self.player
            .message_tx
            .send(crate::player::PlayerMsg::Progress(0, 60))
            .ok();
        if let Err(e) = self.update_photo() {
            self.mount_error_popup(format!("update photo error: {}", e).as_str());
        };
        self.progress_update_title();
        self.lyric_update_title();
        self.update_lyric();
        self.force_redraw();
    }

    pub fn player_update_current_track_after(&mut self) {
        #[cfg(any(feature = "mpris", feature = "discord"))]
        if let Some(song) = &self.player.playlist.current_track {
            #[cfg(feature = "mpris")]
            if let Some(file) = song.file() {
                self.mpris.add_and_play(file);
            }
            #[cfg(feature = "discord")]
            self.discord.update(&song);
        }
        self.time_pos = 0;
        self.playlist_sync();
        if let Err(e) = self.update_photo() {
            self.mount_error_popup(format!("update photo error: {}", e).as_str());
        };
        self.progress_update_title();
        self.lyric_update_title();
        self.update_playing_song();
    }

    pub fn player_previous(&mut self) {
        if let Loop::Single | Loop::Queue = self.config.loop_mode {
            return;
        }

        if self.player.playlist.is_empty() {
            self.player_stop();
            return;
        }

        if let Some(song) = self.player.playlist.tracks.pop_back() {
            self.player.playlist.tracks.push_front(song);
        }
        if let Some(song) = self.player.playlist.tracks.pop_back() {
            self.player.playlist.tracks.push_front(song);
        }
        self.player.skip();
    }

    pub fn player_toggle_pause(&mut self) {
        if self.player.playlist.is_empty() && self.player.playlist.current_track.is_none() {
            return;
        }
        if self.player.is_paused() {
            self.player.set_status(Status::Running);
            self.player.resume();
            #[cfg(feature = "mpris")]
            self.mpris.resume();
            #[cfg(feature = "discord")]
            self.discord.resume(self.time_pos);
        } else {
            // self.player.status = Status::Paused;
            self.player.set_status(Status::Paused);
            self.player.pause();
            #[cfg(feature = "mpris")]
            self.mpris.pause();
            #[cfg(feature = "discord")]
            self.discord.pause();
        }
        self.progress_update_title();
    }

    pub fn player_seek(&mut self, offset: i64) {
        // FIXME: dirty fix for seeking when paused with symphonia,basically set it to play
        // in rusty sink code, and seek, and then set it back to pause.
        #[cfg(not(any(feature = "mpv", feature = "gst")))]
        let paused = self.player.is_paused();
        #[cfg(not(any(feature = "mpv", feature = "gst")))]
        if paused {
            self.player.set_volume(0);
        }

        self.player.seek(offset).ok();

        #[cfg(not(any(feature = "mpv", feature = "gst")))]
        if paused {
            self.force_redraw();
            std::thread::sleep(std::time::Duration::from_millis(50));
            self.player.pause();
            self.player.set_volume(self.config.volume);
        }
    }

    pub fn global_fix_focus(&mut self) {
        let mut focus = false;
        if let Ok(f) = self.app.query(&Id::Library, Attribute::Focus) {
            if Some(AttrValue::Flag(true)) == f {
                focus = true;
            }
        }

        if let Ok(f) = self.app.query(&Id::Playlist, Attribute::Focus) {
            if Some(AttrValue::Flag(true)) == f {
                focus = true;
            }
        }

        if let Ok(f) = self.app.query(&Id::DBListCriteria, Attribute::Focus) {
            if Some(AttrValue::Flag(true)) == f {
                focus = true;
            }
        }

        if let Ok(f) = self.app.query(&Id::DBListSearchResult, Attribute::Focus) {
            if Some(AttrValue::Flag(true)) == f {
                focus = true;
            }
        }

        if let Ok(f) = self.app.query(&Id::DBListSearchTracks, Attribute::Focus) {
            if Some(AttrValue::Flag(true)) == f {
                focus = true;
            }
        }

        if !focus {
            match self.layout {
                TermusicLayout::TreeView => self.app.active(&Id::Library).ok(),
                TermusicLayout::DataBase => self.app.active(&Id::DBListCriteria).ok(),
            };
        }
    }
}
