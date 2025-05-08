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
mod footer;
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
pub use footer::Footer;
pub use labels::{DownloadSpinner, LabelGeneric, LabelSpan};
pub use lyric::Lyric;
pub use music_library::MusicLibrary;
pub use playlist::Playlist;
pub use podcast::{EpisodeList, FeedsList};
pub use popups::general_search::{GSInputPopup, GSTablePopup, Source};
pub use progress::Progress;
pub use tag_editor::*;
use termusiclib::config::v2::tui::keys::Keys;
use termusiclib::config::SharedTuiSettings;
use termusiclib::types::SavePlaylistMsg;

use crate::ui::{ConfigEditorMsg, Id, IdConfigEditor, IdTagEditor, Model, Msg, PLMsg, XYWHMsg};
use tui_realm_stdlib::Phantom;
use tuirealm::event::NoUserEvent;
use tuirealm::{Component, Event, MockComponent, Sub, SubClause, SubEventClause};

#[derive(MockComponent)]
pub struct GlobalListener {
    component: Phantom,
    config: SharedTuiSettings,
}

impl GlobalListener {
    pub fn new(config: SharedTuiSettings) -> Self {
        Self {
            component: Phantom::default(),
            config,
        }
    }
}

impl Component<Msg, NoUserEvent> for GlobalListener {
    #[allow(clippy::too_many_lines)]
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        let keys = &self.config.read().settings.keys;
        match ev {
            Event::WindowResize(..) => Some(Msg::UpdatePhoto),
            Event::Keyboard(keyevent) if keyevent == keys.escape.get() => Some(Msg::QuitPopupShow),
            Event::Keyboard(keyevent) if keyevent == keys.quit.get() => Some(Msg::QuitPopupShow),
            Event::Keyboard(keyevent) if keyevent == keys.player_keys.toggle_pause.get() => {
                Some(Msg::PlayerTogglePause)
            }
            Event::Keyboard(keyevent) if keyevent == keys.player_keys.next_track.get() => {
                Some(Msg::Playlist(PLMsg::NextSong))
            }
            Event::Keyboard(keyevent) if keyevent == keys.player_keys.previous_track.get() => {
                Some(Msg::Playlist(PLMsg::PrevSong))
            }
            Event::Keyboard(keyevent) if keyevent == keys.player_keys.volume_down.get() => {
                Some(Msg::PlayerVolumeDown)
            }
            // Event::Keyboard(keyevent)
            //     if keyevent == keys.player_keys.volume_minus_2.get() =>
            // {
            //     Some(Msg::PlayerVolumeDown)
            // }
            Event::Keyboard(keyevent) if keyevent == keys.player_keys.volume_up.get() => {
                Some(Msg::PlayerVolumeUp)
            }
            // Event::Keyboard(keyevent)
            //     if keyevent == keys.player_keys.volume_plus_2.get() =>
            // {
            //     Some(Msg::PlayerVolumeUp)
            // }
            Event::Keyboard(keyevent) if keyevent == keys.select_view_keys.open_help.get() => {
                Some(Msg::HelpPopupShow)
            }
            Event::Keyboard(keyevent) if keyevent == keys.player_keys.seek_forward.get() => {
                Some(Msg::PlayerSeekForward)
            }
            Event::Keyboard(keyevent) if keyevent == keys.player_keys.seek_backward.get() => {
                Some(Msg::PlayerSeekBackward)
            }
            Event::Keyboard(keyevent) if keyevent == keys.player_keys.speed_up.get() => {
                Some(Msg::PlayerSpeedUp)
            }
            Event::Keyboard(keyevent) if keyevent == keys.player_keys.speed_down.get() => {
                Some(Msg::PlayerSpeedDown)
            }

            Event::Keyboard(keyevent)
                if keyevent == keys.lyric_keys.adjust_offset_forwards.get() =>
            {
                Some(Msg::LyricAdjustDelay(1000))
            }
            Event::Keyboard(keyevent)
                if keyevent == keys.lyric_keys.adjust_offset_backwards.get() =>
            {
                Some(Msg::LyricAdjustDelay(-1000))
            }
            Event::Keyboard(keyevent) if keyevent == keys.lyric_keys.cycle_frames.get() => {
                Some(Msg::LyricCycle)
            }

            Event::Keyboard(keyevent) if keyevent == keys.select_view_keys.view_library.get() => {
                Some(Msg::LayoutTreeView)
            }

            Event::Keyboard(keyevent) if keyevent == keys.select_view_keys.view_database.get() => {
                Some(Msg::LayoutDataBase)
            }

            Event::Keyboard(keyevent) if keyevent == keys.select_view_keys.view_podcasts.get() => {
                Some(Msg::LayoutPodCast)
            }

            Event::Keyboard(keyevent) if keyevent == keys.player_keys.toggle_prefetch.get() => {
                Some(Msg::PlayerToggleGapless)
            }

            Event::Keyboard(keyevent) if keyevent == keys.select_view_keys.open_config.get() => {
                Some(Msg::ConfigEditor(ConfigEditorMsg::Open))
            }

            Event::Keyboard(keyevent) if keyevent == keys.player_keys.save_playlist.get() => {
                Some(Msg::SavePlaylist(SavePlaylistMsg::PopupShow))
            }
            Event::Keyboard(keyevent) if keyevent == keys.move_cover_art_keys.move_left.get() => {
                Some(Msg::Xywh(XYWHMsg::MoveLeft))
            }
            Event::Keyboard(keyevent) if keyevent == keys.move_cover_art_keys.move_right.get() => {
                Some(Msg::Xywh(XYWHMsg::MoveRight))
            }
            Event::Keyboard(keyevent) if keyevent == keys.move_cover_art_keys.move_up.get() => {
                Some(Msg::Xywh(XYWHMsg::MoveUp))
            }
            Event::Keyboard(keyevent) if keyevent == keys.move_cover_art_keys.move_down.get() => {
                Some(Msg::Xywh(XYWHMsg::MoveDown))
            }
            Event::Keyboard(keyevent)
                if keyevent == keys.move_cover_art_keys.increase_size.get() =>
            {
                Some(Msg::Xywh(XYWHMsg::ZoomIn))
            }
            Event::Keyboard(keyevent)
                if keyevent == keys.move_cover_art_keys.decrease_size.get() =>
            {
                Some(Msg::Xywh(XYWHMsg::ZoomOut))
            }
            Event::Keyboard(keyevent) if keyevent == keys.move_cover_art_keys.toggle_hide.get() => {
                Some(Msg::Xywh(XYWHMsg::ToggleHidden))
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
                SubEventClause::Keyboard(keys.escape.get()),
                Self::no_popup_mounted_clause(),
            ),
            Sub::new(
                SubEventClause::Keyboard(keys.quit.get()),
                Self::no_popup_mounted_clause(),
            ),
            Sub::new(
                SubEventClause::Keyboard(keys.player_keys.toggle_pause.get()),
                Self::no_popup_mounted_clause(),
            ),
            Sub::new(
                SubEventClause::Keyboard(keys.player_keys.next_track.get()),
                Self::no_popup_mounted_clause(),
            ),
            Sub::new(
                SubEventClause::Keyboard(keys.player_keys.previous_track.get()),
                Self::no_popup_mounted_clause(),
            ),
            Sub::new(
                SubEventClause::Keyboard(keys.player_keys.speed_up.get()),
                Self::no_popup_mounted_clause(),
            ),
            Sub::new(
                SubEventClause::Keyboard(keys.player_keys.speed_down.get()),
                Self::no_popup_mounted_clause(),
            ),
            Sub::new(
                SubEventClause::Keyboard(keys.player_keys.volume_down.get()),
                Self::no_popup_mounted_clause(),
            ),
            // Sub::new(
            //     SubEventClause::Keyboard(keys.player_keys.volume_minus_2.get()),
            //     Self::no_popup_mounted_clause(),
            // ),
            Sub::new(
                SubEventClause::Keyboard(keys.player_keys.volume_up.get()),
                Self::no_popup_mounted_clause(),
            ),
            // Sub::new(
            //     SubEventClause::Keyboard(keys.player_keys.volume_plus_2.get()),
            //     Self::no_popup_mounted_clause(),
            // ),
            Sub::new(
                SubEventClause::Keyboard(keys.select_view_keys.open_help.get()),
                Self::no_popup_mounted_clause(),
            ),
            Sub::new(
                SubEventClause::Keyboard(keys.player_keys.seek_forward.get()),
                Self::no_popup_mounted_clause(),
            ),
            Sub::new(
                SubEventClause::Keyboard(keys.player_keys.seek_backward.get()),
                Self::no_popup_mounted_clause(),
            ),
            Sub::new(
                SubEventClause::Keyboard(keys.lyric_keys.adjust_offset_forwards.get()),
                Self::no_popup_mounted_clause(),
            ),
            Sub::new(
                SubEventClause::Keyboard(keys.lyric_keys.adjust_offset_backwards.get()),
                Self::no_popup_mounted_clause(),
            ),
            Sub::new(
                SubEventClause::Keyboard(keys.lyric_keys.cycle_frames.get()),
                Self::no_popup_mounted_clause(),
            ),
            Sub::new(
                SubEventClause::Keyboard(keys.select_view_keys.view_library.get()),
                Self::no_popup_mounted_clause(),
            ),
            Sub::new(
                SubEventClause::Keyboard(keys.select_view_keys.view_database.get()),
                Self::no_popup_mounted_clause(),
            ),
            Sub::new(
                SubEventClause::Keyboard(keys.player_keys.toggle_prefetch.get()),
                Self::no_popup_mounted_clause(),
            ),
            Sub::new(
                SubEventClause::Keyboard(keys.select_view_keys.open_config.get()),
                Self::no_popup_mounted_clause(),
            ),
            Sub::new(
                SubEventClause::Keyboard(keys.player_keys.save_playlist.get()),
                Self::no_popup_mounted_clause(),
            ),
            Sub::new(
                SubEventClause::Keyboard(keys.select_view_keys.view_podcasts.get()),
                Self::no_popup_mounted_clause(),
            ),
            Sub::new(
                SubEventClause::Keyboard(keys.move_cover_art_keys.move_left.get()),
                SubClause::Always,
            ),
            Sub::new(
                SubEventClause::Keyboard(keys.move_cover_art_keys.move_right.get()),
                SubClause::Always,
            ),
            Sub::new(
                SubEventClause::Keyboard(keys.move_cover_art_keys.move_up.get()),
                SubClause::Always,
            ),
            Sub::new(
                SubEventClause::Keyboard(keys.move_cover_art_keys.move_down.get()),
                SubClause::Always,
            ),
            Sub::new(
                SubEventClause::Keyboard(keys.move_cover_art_keys.increase_size.get()),
                SubClause::Always,
            ),
            Sub::new(
                SubEventClause::Keyboard(keys.move_cover_art_keys.decrease_size.get()),
                SubClause::Always,
            ),
            Sub::new(
                SubEventClause::Keyboard(keys.move_cover_art_keys.toggle_hide.get()),
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
