use termusiclib::config::SharedTuiSettings;
use termusiclib::config::v2::tui::keys::Keys;
use tui_realm_stdlib::Phantom;
use tuirealm::{Component, Event, MockComponent, Sub, SubClause, SubEventClause};

use crate::ui::Model;
use crate::ui::ids::{Id, IdConfigEditor, IdTagEditor};
use crate::ui::model::UserEvent;
use crate::ui::msg::{
    ConfigEditorMsg, HelpPopupMsg, LyricMsg, MainLayoutMsg, Msg, PLMsg, PlayerMsg, QuitPopupMsg,
    SavePlaylistMsg, XYWHMsg,
};

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

impl Component<Msg, UserEvent> for GlobalListener {
    #[allow(clippy::too_many_lines)]
    fn on(&mut self, ev: Event<UserEvent>) -> Option<Msg> {
        let keys = &self.config.read().settings.keys;
        match ev {
            Event::WindowResize(..) => Some(Msg::UpdatePhoto),
            // "escape" should always just close the dialogs or similar, but should never quit so escape can be "spammed" to exit everything
            // Event::Keyboard(keyevent) if keyevent == keys.escape.get() => Some(Msg::QuitPopupShow),
            Event::Keyboard(keyevent) if keyevent == keys.quit.get() => {
                Some(Msg::QuitPopup(QuitPopupMsg::Show))
            }
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
                Some(Msg::HelpPopup(HelpPopupMsg::Show))
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
                Some(Msg::LyricMessage(LyricMsg::AdjustDelay(1000)))
            }
            Event::Keyboard(keyevent)
                if keyevent == keys.lyric_keys.adjust_offset_backwards.get() =>
            {
                Some(Msg::LyricMessage(LyricMsg::AdjustDelay(-1000)))
            }
            Event::Keyboard(keyevent) if keyevent == keys.lyric_keys.cycle_frames.get() => {
                Some(Msg::LyricMessage(LyricMsg::Cycle))
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
    pub fn subscribe(keys: &Keys) -> Vec<Sub<Id, UserEvent>> {
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
        let mut collection = Vec::new();

        Self::podcast_popups(&mut collection);
        Self::general_popups(&mut collection);

        // dont leave much unused space, as this vec will basically stay for the entire duration of the app
        collection.shrink_to_fit();

        SubClause::Not(Box::new(SubClause::OrMany(collection)))
    }

    /// Podcast related popups.
    ///
    /// The values added to `storage` are meant to be used in a [`SubClause::OrMany`].
    #[inline]
    fn podcast_popups(storage: &mut Vec<SubClause<Id>>) {
        storage.extend([
            SubClause::IsMounted(Id::FeedDeleteConfirmRadioPopup),
            SubClause::IsMounted(Id::FeedDeleteConfirmInputPopup),
            SubClause::IsMounted(Id::PodcastSearchTablePopup),
            SubClause::IsMounted(Id::PodcastAddPopup),
        ]);
    }

    /// Popups that dont relate to any other place specifically.
    ///
    /// The values added to `storage` are meant to be used in a [`SubClause::OrMany`].
    #[inline]
    fn general_popups(storage: &mut Vec<SubClause<Id>>) {
        storage.extend(Self::everywhere_popups());
        storage.extend(Self::delete_confirm_popups());
        storage.extend(Self::youtube_search_popups());

        storage.extend([
            SubClause::IsMounted(Id::GeneralSearchInput),
            SubClause::IsMounted(Id::TagEditor(IdTagEditor::LabelHint)),
            SubClause::IsMounted(Id::ConfigEditor(IdConfigEditor::Footer)),
            SubClause::IsMounted(Id::SavePlaylistPopup),
            SubClause::IsMounted(Id::SavePlaylistConfirm),
            SubClause::IsMounted(Id::DatabaseAddConfirmPopup),
        ]);
    }

    /// Youtube search popups, see [youtube search](super::popups::youtube_search).
    ///
    /// The values returned are meant to be used in a [`SubClause::OrMany`].
    #[inline]
    fn youtube_search_popups() -> [SubClause<Id>; 3] {
        [
            SubClause::IsMounted(Id::YoutubeSearchInputPopup),
            SubClause::IsMounted(Id::YoutubeSearchTablePopup),
            SubClause::IsMounted(Id::YoutubeSearchTablePopup),
        ]
    }

    /// Delete confirmation popups, anything from the `deleteconfirm` module.
    ///
    /// The values returned are meant to be used in a [`SubClause::OrMany`].
    #[inline]
    fn delete_confirm_popups() -> [SubClause<Id>; 2] {
        [
            SubClause::IsMounted(Id::DeleteConfirmInputPopup),
            SubClause::IsMounted(Id::DeleteConfirmRadioPopup),
        ]
    }

    /// Popups that could happen everywhere.
    ///
    /// The values returned are meant to be used in a [`SubClause::OrMany`].
    #[inline]
    fn everywhere_popups() -> [SubClause<Id>; 3] {
        [
            SubClause::IsMounted(Id::HelpPopup),
            SubClause::IsMounted(Id::ErrorPopup),
            SubClause::IsMounted(Id::QuitPopup),
        ]
    }
}
