use termusiclib::config::v2::tui::keys::Keys;
use termusiclib::config::SharedTuiSettings;
use termusiclib::ids::{Id, IdConfigEditor, IdTagEditor};
use termusiclib::types::{
    ConfigEditorMsg, MainLayoutMsg, Msg, PLMsg, PlayerMsg, SavePlaylistMsg, XYWHMsg,
};

use crate::ui::Model;
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
            // "escape" should always just close the dialogs or similar, but should never quit so escape can be "spammed" to exit everything
            // Event::Keyboard(keyevent) if keyevent == keys.escape.get() => Some(Msg::QuitPopupShow),
            Event::Keyboard(keyevent) if keyevent == keys.quit.get() => Some(Msg::QuitPopupShow),
            Event::Keyboard(keyevent) if keyevent == keys.player_keys.toggle_pause.get() => {
                Some(Msg::Player(PlayerMsg::TogglePause))
            }
            Event::Keyboard(keyevent) if keyevent == keys.player_keys.next_track.get() => {
                Some(Msg::Playlist(PLMsg::NextSong))
            }
            Event::Keyboard(keyevent) if keyevent == keys.player_keys.previous_track.get() => {
                Some(Msg::Playlist(PLMsg::PrevSong))
            }
            Event::Keyboard(keyevent) if keyevent == keys.player_keys.volume_down.get() => {
                Some(Msg::Player(PlayerMsg::VolumeDown))
            }
            Event::Keyboard(keyevent) if keyevent == keys.player_keys.volume_up.get() => {
                Some(Msg::Player(PlayerMsg::VolumeUp))
            }
            Event::Keyboard(keyevent) if keyevent == keys.select_view_keys.open_help.get() => {
                Some(Msg::HelpPopupShow)
            }
            Event::Keyboard(keyevent) if keyevent == keys.player_keys.seek_forward.get() => {
                Some(Msg::Player(PlayerMsg::SeekForward))
            }
            Event::Keyboard(keyevent) if keyevent == keys.player_keys.seek_backward.get() => {
                Some(Msg::Player(PlayerMsg::SeekBackward))
            }
            Event::Keyboard(keyevent) if keyevent == keys.player_keys.speed_up.get() => {
                Some(Msg::Player(PlayerMsg::SpeedUp))
            }
            Event::Keyboard(keyevent) if keyevent == keys.player_keys.speed_down.get() => {
                Some(Msg::Player(PlayerMsg::SpeedDown))
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
                Some(Msg::Layout(MainLayoutMsg::TreeView))
            }

            Event::Keyboard(keyevent) if keyevent == keys.select_view_keys.view_database.get() => {
                Some(Msg::Layout(MainLayoutMsg::DataBase))
            }

            Event::Keyboard(keyevent) if keyevent == keys.select_view_keys.view_podcasts.get() => {
                Some(Msg::Layout(MainLayoutMsg::Podcast))
            }

            Event::Keyboard(keyevent) if keyevent == keys.player_keys.toggle_prefetch.get() => {
                Some(Msg::Player(PlayerMsg::ToggleGapless))
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
            // Sub::new(
            //     SubEventClause::Keyboard(keys.escape.get()),
            //     Self::no_popup_mounted_clause(),
            // ),
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

    /// Generate the Clause for any popups to not be mounted.
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
        SubClause::Not(Box::new(Self::everywhere_popups(Box::new(
            Self::delete_confirm_popups(Box::new(SubClause::Or(
                Box::new(SubClause::IsMounted(Id::GeneralSearchInput)),
                Box::new(SubClause::Or(
                    Box::new(SubClause::IsMounted(Id::TagEditor(IdTagEditor::LabelHint))),
                    Box::new(SubClause::Or(
                        Box::new(SubClause::IsMounted(Id::ConfigEditor(
                            IdConfigEditor::Footer,
                        ))),
                        Box::new(Self::youtube_search_popups(Box::new(SubClause::Or(
                            Box::new(SubClause::IsMounted(Id::SavePlaylistPopup)),
                            Box::new(SubClause::Or(
                                Box::new(SubClause::IsMounted(Id::SavePlaylistConfirm)),
                                Box::new(SubClause::Or(
                                    Box::new(SubClause::IsMounted(Id::PodcastAddPopup)),
                                    Box::new(SubClause::IsMounted(Id::DatabaseAddConfirmPopup)),
                                )),
                            )),
                        )))),
                    )),
                )),
            ))),
        ))))
    }

    /// Youtube search popups, see [youtube search](super::popups::youtube_search)
    #[inline]
    fn youtube_search_popups(or: Box<SubClause<Id>>) -> SubClause<Id> {
        SubClause::Or(
            Box::new(SubClause::IsMounted(Id::YoutubeSearchInputPopup)),
            Box::new(SubClause::Or(
                Box::new(SubClause::IsMounted(Id::YoutubeSearchTablePopup)),
                Box::new(SubClause::Or(
                    Box::new(SubClause::IsMounted(Id::YoutubeSearchTablePopup)),
                    or,
                )),
            )),
        )
    }

    /// Delete confirmation popups, anything from the `deleteconfirm` module
    #[inline]
    fn delete_confirm_popups(or: Box<SubClause<Id>>) -> SubClause<Id> {
        SubClause::Or(
            Box::new(SubClause::IsMounted(Id::DeleteConfirmInputPopup)),
            Box::new(SubClause::Or(
                Box::new(SubClause::IsMounted(Id::DeleteConfirmRadioPopup)),
                or,
            )),
        )
    }

    /// Popups that could happen everywhere
    #[inline]
    fn everywhere_popups(or: Box<SubClause<Id>>) -> SubClause<Id> {
        SubClause::Or(
            Box::new(SubClause::IsMounted(Id::HelpPopup)),
            Box::new(SubClause::Or(
                Box::new(SubClause::IsMounted(Id::ErrorPopup)),
                Box::new(SubClause::Or(
                    Box::new(SubClause::IsMounted(Id::QuitPopup)),
                    or,
                )),
            )),
        )
    }
}
