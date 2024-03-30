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
mod config_editor;
mod database;
mod labels;
mod lyric;
mod music_library;
mod playlist;
mod podcast;
mod popups;
mod progress;
#[allow(
    clippy::match_bool,
    clippy::redundant_closure_for_method_calls,
    clippy::doc_markdown,
    clippy::module_name_repetitions
)]
/// Tag Editor Controls
mod tag_editor;
mod xywh;

// -- export
pub use config_editor::*;
pub use database::{DBListCriteria, DBListSearchResult, DBListSearchTracks};
pub use labels::{DownloadSpinner, LabelGeneric, LabelSpan};
pub use lyric::Lyric;
pub use music_library::MusicLibrary;
pub use playlist::Playlist;
pub use podcast::{EpisodeList, FeedsList};
pub use popups::general_search::{GSInputPopup, GSTablePopup, Source};
pub use progress::Progress;
pub use tag_editor::*;
use termusicplayback::SharedSettings;

use crate::config::Keys;
use crate::ui::{ConfigEditorMsg, Id, IdConfigEditor, IdTagEditor, Model, Msg, PLMsg, XYWHMsg};
use tui_realm_stdlib::Phantom;
use tuirealm::event::NoUserEvent;
use tuirealm::{Component, Event, MockComponent, Sub, SubClause, SubEventClause};

#[derive(MockComponent)]
pub struct GlobalListener {
    component: Phantom,
    config: SharedSettings,
}

impl GlobalListener {
    pub fn new(config: SharedSettings) -> Self {
        Self {
            component: Phantom::default(),
            config,
        }
    }
}

impl Component<Msg, NoUserEvent> for GlobalListener {
    #[allow(clippy::too_many_lines)]
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        let keys = &self.config.read().keys;
        match ev {
            Event::WindowResize(..) => Some(Msg::UpdatePhoto),
            Event::Keyboard(keyevent) if keyevent == keys.global_esc.key_event() => {
                Some(Msg::QuitPopupShow)
            }
            Event::Keyboard(keyevent) if keyevent == keys.global_quit.key_event() => {
                Some(Msg::QuitPopupShow)
            }
            Event::Keyboard(keyevent)
                if keyevent == keys.global_player_toggle_pause.key_event() =>
            {
                Some(Msg::PlayerTogglePause)
            }
            Event::Keyboard(keyevent) if keyevent == keys.global_player_next.key_event() => {
                Some(Msg::Playlist(PLMsg::NextSong))
            }
            Event::Keyboard(keyevent) if keyevent == keys.global_player_previous.key_event() => {
                Some(Msg::Playlist(PLMsg::PrevSong))
            }
            Event::Keyboard(keyevent)
                if keyevent == keys.global_player_volume_minus_1.key_event() =>
            {
                Some(Msg::PlayerVolumeDown)
            }
            Event::Keyboard(keyevent)
                if keyevent == keys.global_player_volume_minus_2.key_event() =>
            {
                Some(Msg::PlayerVolumeDown)
            }
            Event::Keyboard(keyevent)
                if keyevent == keys.global_player_volume_plus_1.key_event() =>
            {
                Some(Msg::PlayerVolumeUp)
            }
            Event::Keyboard(keyevent)
                if keyevent == keys.global_player_volume_plus_2.key_event() =>
            {
                Some(Msg::PlayerVolumeUp)
            }
            Event::Keyboard(keyevent) if keyevent == keys.global_help.key_event() => {
                Some(Msg::HelpPopupShow)
            }
            Event::Keyboard(keyevent)
                if keyevent == keys.global_player_seek_forward.key_event() =>
            {
                Some(Msg::PlayerSeekForward)
            }
            Event::Keyboard(keyevent)
                if keyevent == keys.global_player_seek_backward.key_event() =>
            {
                Some(Msg::PlayerSeekBackward)
            }
            Event::Keyboard(keyevent) if keyevent == keys.global_player_speed_up.key_event() => {
                Some(Msg::PlayerSpeedUp)
            }
            Event::Keyboard(keyevent) if keyevent == keys.global_player_speed_down.key_event() => {
                Some(Msg::PlayerSpeedDown)
            }

            Event::Keyboard(keyevent)
                if keyevent == keys.global_lyric_adjust_forward.key_event() =>
            {
                Some(Msg::LyricAdjustDelay(1000))
            }
            Event::Keyboard(keyevent)
                if keyevent == keys.global_lyric_adjust_backward.key_event() =>
            {
                Some(Msg::LyricAdjustDelay(-1000))
            }
            Event::Keyboard(keyevent) if keyevent == keys.global_lyric_cycle.key_event() => {
                Some(Msg::LyricCycle)
            }

            Event::Keyboard(keyevent) if keyevent == keys.global_layout_treeview.key_event() => {
                Some(Msg::LayoutTreeView)
            }

            Event::Keyboard(keyevent) if keyevent == keys.global_layout_database.key_event() => {
                Some(Msg::LayoutDataBase)
            }

            Event::Keyboard(keyevent) if keyevent == keys.global_layout_podcast.key_event() => {
                Some(Msg::LayoutPodCast)
            }

            Event::Keyboard(keyevent)
                if keyevent == keys.global_player_toggle_gapless.key_event() =>
            {
                Some(Msg::PlayerToggleGapless)
            }

            Event::Keyboard(keyevent) if keyevent == keys.global_config_open.key_event() => {
                Some(Msg::ConfigEditor(ConfigEditorMsg::Open))
            }

            Event::Keyboard(keyevent) if keyevent == keys.global_save_playlist.key_event() => {
                Some(Msg::SavePlaylistPopupShow)
            }
            Event::Keyboard(keyevent) if keyevent == keys.global_xywh_move_left.key_event() => {
                Some(Msg::Xywh(XYWHMsg::MoveLeft))
            }
            Event::Keyboard(keyevent) if keyevent == keys.global_xywh_move_right.key_event() => {
                Some(Msg::Xywh(XYWHMsg::MoveRight))
            }
            Event::Keyboard(keyevent) if keyevent == keys.global_xywh_move_up.key_event() => {
                Some(Msg::Xywh(XYWHMsg::MoveUp))
            }
            Event::Keyboard(keyevent) if keyevent == keys.global_xywh_move_down.key_event() => {
                Some(Msg::Xywh(XYWHMsg::MoveDown))
            }
            Event::Keyboard(keyevent) if keyevent == keys.global_xywh_zoom_in.key_event() => {
                Some(Msg::Xywh(XYWHMsg::ZoomIn))
            }
            Event::Keyboard(keyevent) if keyevent == keys.global_xywh_zoom_out.key_event() => {
                Some(Msg::Xywh(XYWHMsg::ZoomOut))
            }
            Event::Keyboard(keyevent) if keyevent == keys.global_xywh_hide.key_event() => {
                Some(Msg::Xywh(XYWHMsg::Hide))
            }
            _ => None,
        }
    }
}

impl Model {
    /// global listener subscriptions
    #[allow(clippy::too_many_lines)]
    pub fn subscribe(keys: &Keys) -> Vec<Sub<Id, NoUserEvent>> {
        vec![
            Sub::new(
                SubEventClause::Keyboard(keys.global_esc.key_event()),
                Self::no_popup_mounted_clause(),
            ),
            Sub::new(
                SubEventClause::Keyboard(keys.global_quit.key_event()),
                Self::no_popup_mounted_clause(),
            ),
            Sub::new(
                SubEventClause::Keyboard(keys.global_player_toggle_pause.key_event()),
                Self::no_popup_mounted_clause(),
            ),
            Sub::new(
                SubEventClause::Keyboard(keys.global_player_next.key_event()),
                Self::no_popup_mounted_clause(),
            ),
            Sub::new(
                SubEventClause::Keyboard(keys.global_player_previous.key_event()),
                Self::no_popup_mounted_clause(),
            ),
            Sub::new(
                SubEventClause::Keyboard(keys.global_player_speed_up.key_event()),
                Self::no_popup_mounted_clause(),
            ),
            Sub::new(
                SubEventClause::Keyboard(keys.global_player_speed_down.key_event()),
                Self::no_popup_mounted_clause(),
            ),
            Sub::new(
                SubEventClause::Keyboard(keys.global_player_volume_minus_1.key_event()),
                Self::no_popup_mounted_clause(),
            ),
            Sub::new(
                SubEventClause::Keyboard(keys.global_player_volume_minus_2.key_event()),
                Self::no_popup_mounted_clause(),
            ),
            Sub::new(
                SubEventClause::Keyboard(keys.global_player_volume_plus_1.key_event()),
                Self::no_popup_mounted_clause(),
            ),
            Sub::new(
                SubEventClause::Keyboard(keys.global_player_volume_plus_2.key_event()),
                Self::no_popup_mounted_clause(),
            ),
            Sub::new(
                SubEventClause::Keyboard(keys.global_help.key_event()),
                Self::no_popup_mounted_clause(),
            ),
            Sub::new(
                SubEventClause::Keyboard(keys.global_player_seek_forward.key_event()),
                Self::no_popup_mounted_clause(),
            ),
            Sub::new(
                SubEventClause::Keyboard(keys.global_player_seek_backward.key_event()),
                Self::no_popup_mounted_clause(),
            ),
            Sub::new(
                SubEventClause::Keyboard(keys.global_lyric_adjust_forward.key_event()),
                Self::no_popup_mounted_clause(),
            ),
            Sub::new(
                SubEventClause::Keyboard(keys.global_lyric_adjust_backward.key_event()),
                Self::no_popup_mounted_clause(),
            ),
            Sub::new(
                SubEventClause::Keyboard(keys.global_lyric_cycle.key_event()),
                Self::no_popup_mounted_clause(),
            ),
            Sub::new(
                SubEventClause::Keyboard(keys.global_layout_treeview.key_event()),
                Self::no_popup_mounted_clause(),
            ),
            Sub::new(
                SubEventClause::Keyboard(keys.global_layout_database.key_event()),
                Self::no_popup_mounted_clause(),
            ),
            Sub::new(
                SubEventClause::Keyboard(keys.global_player_toggle_gapless.key_event()),
                Self::no_popup_mounted_clause(),
            ),
            Sub::new(
                SubEventClause::Keyboard(keys.global_config_open.key_event()),
                Self::no_popup_mounted_clause(),
            ),
            Sub::new(
                SubEventClause::Keyboard(keys.global_save_playlist.key_event()),
                Self::no_popup_mounted_clause(),
            ),
            Sub::new(
                SubEventClause::Keyboard(keys.global_layout_podcast.key_event()),
                Self::no_popup_mounted_clause(),
            ),
            Sub::new(
                SubEventClause::Keyboard(keys.global_xywh_move_left.key_event()),
                SubClause::Always,
            ),
            Sub::new(
                SubEventClause::Keyboard(keys.global_xywh_move_right.key_event()),
                SubClause::Always,
            ),
            Sub::new(
                SubEventClause::Keyboard(keys.global_xywh_move_up.key_event()),
                SubClause::Always,
            ),
            Sub::new(
                SubEventClause::Keyboard(keys.global_xywh_move_down.key_event()),
                SubClause::Always,
            ),
            Sub::new(
                SubEventClause::Keyboard(keys.global_xywh_zoom_in.key_event()),
                SubClause::Always,
            ),
            Sub::new(
                SubEventClause::Keyboard(keys.global_xywh_zoom_out.key_event()),
                SubClause::Always,
            ),
            Sub::new(
                SubEventClause::Keyboard(keys.global_xywh_hide.key_event()),
                SubClause::Always,
            ),
            Sub::new(SubEventClause::WindowResize, SubClause::Always),
        ]
    }

    fn no_popup_mounted_clause() -> SubClause<Id> {
        let subclause1 = Self::no_popup_mounted_clause_1();
        let subclause2 = Self::no_popup_mounted_clause_2();
        SubClause::And(Box::new(subclause1), Box::new(subclause2))
    }
    fn no_popup_mounted_clause_2() -> SubClause<Id> {
        SubClause::Not(Box::new(SubClause::Or(
            Box::new(SubClause::IsMounted(Id::FeedDeleteConfirmRadioPopup)),
            Box::new(SubClause::Or(
                Box::new(SubClause::IsMounted(Id::FeedDeleteConfirmInputPopup)),
                Box::new(SubClause::IsMounted(Id::PodcastSearchTablePopup)),
            )),
        )))
    }
    fn no_popup_mounted_clause_1() -> SubClause<Id> {
        SubClause::Not(Box::new(SubClause::Or(
            Box::new(SubClause::IsMounted(Id::HelpPopup)),
            Box::new(SubClause::Or(
                Box::new(SubClause::IsMounted(Id::ErrorPopup)),
                Box::new(SubClause::Or(
                    Box::new(SubClause::IsMounted(Id::QuitPopup)),
                    Box::new(SubClause::Or(
                        Box::new(SubClause::IsMounted(Id::DeleteConfirmInputPopup)),
                        Box::new(SubClause::Or(
                            Box::new(SubClause::IsMounted(Id::DeleteConfirmRadioPopup)),
                            Box::new(SubClause::Or(
                                Box::new(SubClause::IsMounted(Id::GeneralSearchInput)),
                                Box::new(SubClause::Or(
                                    Box::new(SubClause::IsMounted(Id::TagEditor(
                                        IdTagEditor::LabelHint,
                                    ))),
                                    Box::new(SubClause::Or(
                                        Box::new(SubClause::IsMounted(Id::ConfigEditor(
                                            IdConfigEditor::Footer,
                                        ))),
                                        Box::new(SubClause::Or(
                                            Box::new(SubClause::IsMounted(
                                                Id::YoutubeSearchInputPopup,
                                            )),
                                            Box::new(SubClause::Or(
                                                Box::new(SubClause::IsMounted(
                                                    Id::YoutubeSearchTablePopup,
                                                )),
                                                Box::new(SubClause::Or(
                                                    Box::new(SubClause::IsMounted(
                                                        Id::YoutubeSearchTablePopup,
                                                    )),
                                                    Box::new(SubClause::Or(
                                                        Box::new(SubClause::IsMounted(
                                                            Id::SavePlaylistPopup,
                                                        )),
                                                        Box::new(SubClause::Or(
                                                            Box::new(SubClause::IsMounted(
                                                                Id::SavePlaylistConfirm,
                                                            )),
                                                            Box::new(SubClause::IsMounted(
                                                                Id::PodcastAddPopup,
                                                            )),
                                                        )),
                                                    )),
                                                )),
                                            )),
                                        )),
                                    )),
                                )),
                            )),
                        )),
                    )),
                )),
            )),
        )))
    }
}
