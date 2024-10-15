use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::hash::Hash;
use std::iter::once;
use std::str::FromStr;
use tuirealm::event::{Key, KeyEvent, KeyModifiers};

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct Keys {
    pub global_esc: BindingForEvent,
    pub global_quit: BindingForEvent,
    pub global_left: BindingForEvent,
    pub global_down: BindingForEvent,
    pub global_up: BindingForEvent,
    pub global_right: BindingForEvent,
    pub global_goto_top: BindingForEvent,
    pub global_goto_bottom: BindingForEvent,
    pub global_player_toggle_pause: BindingForEvent,
    pub global_player_next: BindingForEvent,
    pub global_player_previous: BindingForEvent,
    pub global_player_volume_plus_1: BindingForEvent,
    pub global_player_volume_plus_2: BindingForEvent,
    pub global_player_volume_minus_1: BindingForEvent,
    pub global_player_volume_minus_2: BindingForEvent,
    pub global_help: BindingForEvent,
    pub global_player_seek_forward: BindingForEvent,
    pub global_player_seek_backward: BindingForEvent,
    pub global_lyric_adjust_forward: BindingForEvent,
    pub global_lyric_adjust_backward: BindingForEvent,
    pub global_player_speed_up: BindingForEvent,
    pub global_player_speed_down: BindingForEvent,
    pub global_lyric_cycle: BindingForEvent,
    pub global_layout_treeview: BindingForEvent,
    pub global_layout_database: BindingForEvent,
    pub global_player_toggle_gapless: BindingForEvent,
    pub global_config_open: BindingForEvent,
    pub global_save_playlist: BindingForEvent,
    pub global_layout_podcast: BindingForEvent,
    pub global_xywh_move_left: BindingForEvent,
    pub global_xywh_move_right: BindingForEvent,
    pub global_xywh_move_up: BindingForEvent,
    pub global_xywh_move_down: BindingForEvent,
    pub global_xywh_zoom_in: BindingForEvent,
    pub global_xywh_zoom_out: BindingForEvent,
    pub global_xywh_hide: BindingForEvent,
    pub library_load_dir: BindingForEvent,
    pub library_delete: BindingForEvent,
    pub library_yank: BindingForEvent,
    pub library_paste: BindingForEvent,
    pub library_search: BindingForEvent,
    pub library_search_youtube: BindingForEvent,
    pub library_tag_editor_open: BindingForEvent,
    pub library_switch_root: BindingForEvent,
    pub library_add_root: BindingForEvent,
    pub library_remove_root: BindingForEvent,
    pub playlist_delete: BindingForEvent,
    pub playlist_delete_all: BindingForEvent,
    pub playlist_shuffle: BindingForEvent,
    pub playlist_mode_cycle: BindingForEvent,
    pub playlist_play_selected: BindingForEvent,
    pub playlist_search: BindingForEvent,
    pub playlist_swap_down: BindingForEvent,
    pub playlist_swap_up: BindingForEvent,
    #[serde(rename = "playlist_cmus_lqueue")] // backwards compat, cannot easily be changed
    pub playlist_add_random_album: BindingForEvent,
    #[serde(rename = "playlist_cmus_tqueue")] // backwards compat, cannot easily be changed
    pub playlist_add_random_tracks: BindingForEvent,
    pub database_add_all: BindingForEvent,
    pub config_save: BindingForEvent,
    pub podcast_mark_played: BindingForEvent,
    pub podcast_mark_all_played: BindingForEvent,
    pub podcast_episode_download: BindingForEvent,
    pub podcast_episode_delete_file: BindingForEvent,
    pub podcast_delete_feed: BindingForEvent,
    pub podcast_delete_all_feeds: BindingForEvent,
    pub podcast_search_add_feed: BindingForEvent,
    pub podcast_refresh_feed: BindingForEvent,
    pub podcast_refresh_all_feeds: BindingForEvent,
}

impl Keys {
    // In order to check if duplicate keys are configured, please ensure all are included here
    fn iter_global(&self) -> impl Iterator<Item = BindingForEvent> {
        once(self.global_esc)
            .chain(once(self.global_quit))
            .chain(once(self.global_left))
            .chain(once(self.global_down))
            .chain(once(self.global_up))
            .chain(once(self.global_right))
            .chain(once(self.global_goto_top))
            .chain(once(self.global_goto_bottom))
            .chain(once(self.global_player_toggle_pause))
            .chain(once(self.global_player_next))
            .chain(once(self.global_player_previous))
            .chain(once(self.global_player_volume_plus_1))
            .chain(once(self.global_player_volume_plus_2))
            .chain(once(self.global_player_volume_minus_1))
            .chain(once(self.global_player_volume_minus_2))
            .chain(once(self.global_help))
            .chain(once(self.global_player_seek_forward))
            .chain(once(self.global_player_seek_backward))
            .chain(once(self.global_lyric_adjust_forward))
            .chain(once(self.global_lyric_adjust_backward))
            .chain(once(self.global_player_speed_up))
            .chain(once(self.global_player_speed_down))
            .chain(once(self.global_lyric_cycle))
            .chain(once(self.global_layout_treeview))
            .chain(once(self.global_layout_database))
            .chain(once(self.global_player_toggle_gapless))
            .chain(once(self.global_config_open))
            .chain(once(self.global_save_playlist))
            .chain(once(self.global_layout_podcast))
            .chain(once(self.global_xywh_move_left))
            .chain(once(self.global_xywh_move_right))
            .chain(once(self.global_xywh_move_up))
            .chain(once(self.global_xywh_move_down))
            .chain(once(self.global_xywh_zoom_in))
            .chain(once(self.global_xywh_zoom_out))
            .chain(once(self.global_xywh_hide))
        // .chain(once(self.config_save))
    }

    fn iter_library(&self) -> impl Iterator<Item = BindingForEvent> {
        once(self.library_load_dir)
            .chain(once(self.library_delete))
            .chain(once(self.library_yank))
            .chain(once(self.library_paste))
            .chain(once(self.library_search))
            .chain(once(self.library_search_youtube))
            .chain(once(self.library_tag_editor_open))
            .chain(once(self.library_switch_root))
            .chain(once(self.library_add_root))
            .chain(once(self.library_remove_root))
    }

    fn iter_playlist(&self) -> impl Iterator<Item = BindingForEvent> {
        once(self.playlist_delete)
            .chain(once(self.playlist_delete_all))
            .chain(once(self.playlist_shuffle))
            .chain(once(self.playlist_mode_cycle))
            .chain(once(self.playlist_play_selected))
            .chain(once(self.playlist_search))
            .chain(once(self.playlist_swap_down))
            .chain(once(self.playlist_swap_up))
            .chain(once(self.playlist_add_random_album))
            .chain(once(self.playlist_add_random_tracks))
    }

    fn iter_podcast(&self) -> impl Iterator<Item = BindingForEvent> {
        once(self.podcast_search_add_feed)
            .chain(once(self.podcast_refresh_feed))
            .chain(once(self.podcast_refresh_all_feeds))
            .chain(once(self.podcast_delete_feed))
            .chain(once(self.podcast_delete_all_feeds))
    }

    fn iter_episode(&self) -> impl Iterator<Item = BindingForEvent> {
        once(self.podcast_mark_played)
            .chain(once(self.podcast_mark_all_played))
            .chain(once(self.podcast_episode_download))
            .chain(once(self.podcast_episode_delete_file))
    }

    pub fn has_unique_elements(&self) -> bool {
        let mut uniq_global = HashSet::new();
        let mut uniq_library = HashSet::new();
        let mut uniq_playlist = HashSet::new();
        let mut uniq_podcast = HashSet::new();
        let mut uniq_episode = HashSet::new();
        self.iter_global().all(move |x| uniq_global.insert(x))
            && self.iter_library().all(move |x| uniq_library.insert(x))
            && self.iter_playlist().all(move |x| uniq_playlist.insert(x))
            && self.iter_podcast().all(move |x| uniq_podcast.insert(x))
            && self.iter_episode().all(move |x| uniq_episode.insert(x))
    }
}

/// Custom Serialize / Deserialize for [`Key`] so that library updates mess with the config layout
mod key_serde_code {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use tuirealm::event::{Key, MediaKeyCode};

    /// Copied from tuirealm 1.8.0
    #[derive(Debug, Eq, PartialEq, Copy, Clone, Serialize, Deserialize)]
    #[serde(tag = "type", content = "args")]
    pub enum MediaKeyCodeSerde {
        /// Play media key.
        Play,
        /// Pause media key.
        Pause,
        /// Play/Pause media key.
        PlayPause,
        /// Reverse media key.
        Reverse,
        /// Stop media key.
        Stop,
        /// Fast-forward media key.
        FastForward,
        /// Rewind media key.
        Rewind,
        /// Next-track media key.
        TrackNext,
        /// Previous-track media key.
        TrackPrevious,
        /// Record media key.
        Record,
        /// Lower-volume media key.
        LowerVolume,
        /// Raise-volume media key.
        RaiseVolume,
        /// Mute media key.
        MuteVolume,
    }

    impl From<&MediaKeyCode> for MediaKeyCodeSerde {
        fn from(value: &MediaKeyCode) -> Self {
            match value {
                MediaKeyCode::Play => MediaKeyCodeSerde::Play,
                MediaKeyCode::Pause => MediaKeyCodeSerde::Pause,
                MediaKeyCode::PlayPause => MediaKeyCodeSerde::PlayPause,
                MediaKeyCode::Reverse => MediaKeyCodeSerde::Reverse,
                MediaKeyCode::Stop => MediaKeyCodeSerde::Stop,
                MediaKeyCode::FastForward => MediaKeyCodeSerde::FastForward,
                MediaKeyCode::Rewind => MediaKeyCodeSerde::Rewind,
                MediaKeyCode::TrackNext => MediaKeyCodeSerde::TrackNext,
                MediaKeyCode::TrackPrevious => MediaKeyCodeSerde::TrackPrevious,
                MediaKeyCode::Record => MediaKeyCodeSerde::Record,
                MediaKeyCode::LowerVolume => MediaKeyCodeSerde::LowerVolume,
                MediaKeyCode::RaiseVolume => MediaKeyCodeSerde::RaiseVolume,
                MediaKeyCode::MuteVolume => MediaKeyCodeSerde::MuteVolume,
            }
        }
    }

    impl From<MediaKeyCodeSerde> for MediaKeyCode {
        fn from(value: MediaKeyCodeSerde) -> Self {
            match value {
                MediaKeyCodeSerde::Play => MediaKeyCode::Play,
                MediaKeyCodeSerde::Pause => MediaKeyCode::Pause,
                MediaKeyCodeSerde::PlayPause => MediaKeyCode::PlayPause,
                MediaKeyCodeSerde::Reverse => MediaKeyCode::Reverse,
                MediaKeyCodeSerde::Stop => MediaKeyCode::Stop,
                MediaKeyCodeSerde::FastForward => MediaKeyCode::FastForward,
                MediaKeyCodeSerde::Rewind => MediaKeyCode::Rewind,
                MediaKeyCodeSerde::TrackNext => MediaKeyCode::TrackNext,
                MediaKeyCodeSerde::TrackPrevious => MediaKeyCode::TrackPrevious,
                MediaKeyCodeSerde::Record => MediaKeyCode::Record,
                MediaKeyCodeSerde::LowerVolume => MediaKeyCode::LowerVolume,
                MediaKeyCodeSerde::RaiseVolume => MediaKeyCode::RaiseVolume,
                MediaKeyCodeSerde::MuteVolume => MediaKeyCode::MuteVolume,
            }
        }
    }

    /// Copied from tuirealm 1.8.0
    #[derive(Debug, Eq, PartialEq, Copy, Clone, Serialize, Deserialize)]
    #[serde(tag = "type", content = "args")]
    pub enum KeySerde {
        /// Backspace key.
        Backspace,
        /// Enter key.
        Enter,
        /// Left arrow key.
        Left,
        /// Right arrow key.
        Right,
        /// Up arrow key.
        Up,
        /// Down arrow key.
        Down,
        /// Home key.
        Home,
        /// End key.
        End,
        /// Page up key.
        PageUp,
        /// Page dow key.
        PageDown,
        /// Tab key.
        Tab,
        /// Shift + Tab key. (sugar)
        BackTab,
        /// Delete key.
        Delete,
        /// Insert key.
        Insert,
        /// Function key followed by index (F1 => `Key::Function(1)`)
        Function(u8),
        /// A character.
        ///
        /// `KeyCode::Char('c')` represents `c` character, etc.
        Char(char),
        /// Null.
        Null,
        /// Caps lock pressed
        CapsLock,
        /// Scroll lock pressed
        ScrollLock,
        /// Num lock pressed
        NumLock,
        /// Print screen key
        PrintScreen,
        /// Pause key
        Pause,
        /// Menu key
        Menu,
        /// keypad begin
        KeypadBegin,
        /// Media key
        Media(MediaKeyCodeSerde),
        /// Escape key.
        Esc,
    }

    impl From<&Key> for KeySerde {
        fn from(value: &Key) -> Self {
            match value {
                Key::Backspace => KeySerde::Backspace,
                Key::Enter => KeySerde::Enter,
                Key::Left => KeySerde::Left,
                Key::Right => KeySerde::Right,
                Key::Up => KeySerde::Up,
                Key::Down => KeySerde::Down,
                Key::Home => KeySerde::Home,
                Key::End => KeySerde::End,
                Key::PageUp => KeySerde::PageUp,
                Key::PageDown => KeySerde::PageDown,
                Key::Tab => KeySerde::Tab,
                Key::BackTab => KeySerde::BackTab,
                Key::Delete => KeySerde::Delete,
                Key::Insert => KeySerde::Insert,
                Key::Function(v) => KeySerde::Function(*v),
                Key::Char(v) => KeySerde::Char(*v),
                Key::Null => KeySerde::Null,
                Key::CapsLock => KeySerde::CapsLock,
                Key::ScrollLock => KeySerde::ScrollLock,
                Key::NumLock => KeySerde::NumLock,
                Key::PrintScreen => KeySerde::PrintScreen,
                Key::Pause => KeySerde::Pause,
                Key::Menu => KeySerde::Menu,
                Key::KeypadBegin => KeySerde::KeypadBegin,
                Key::Media(v) => KeySerde::Media(v.into()),
                Key::Esc => KeySerde::Esc,

                // the following are new events with tuirealm 2.0, but only available in backend "termion", which we dont use
                Key::ShiftLeft
                | Key::AltLeft
                | Key::CtrlLeft
                | Key::ShiftRight
                | Key::AltRight
                | Key::CtrlRight
                | Key::ShiftUp
                | Key::AltUp
                | Key::CtrlUp
                | Key::ShiftDown
                | Key::AltDown
                | Key::CtrlDown
                | Key::CtrlHome
                | Key::CtrlEnd => unimplemented!(),
            }
        }
    }

    impl From<KeySerde> for Key {
        fn from(value: KeySerde) -> Self {
            match value {
                KeySerde::Backspace => Key::Backspace,
                KeySerde::Enter => Key::Enter,
                KeySerde::Left => Key::Left,
                KeySerde::Right => Key::Right,
                KeySerde::Up => Key::Up,
                KeySerde::Down => Key::Down,
                KeySerde::Home => Key::Home,
                KeySerde::End => Key::End,
                KeySerde::PageUp => Key::PageUp,
                KeySerde::PageDown => Key::PageDown,
                KeySerde::Tab => Key::Tab,
                KeySerde::BackTab => Key::BackTab,
                KeySerde::Delete => Key::Delete,
                KeySerde::Insert => Key::Insert,
                KeySerde::Function(v) => Key::Function(v),
                KeySerde::Char(v) => Key::Char(v),
                KeySerde::Null => Key::Null,
                KeySerde::CapsLock => Key::CapsLock,
                KeySerde::ScrollLock => Key::ScrollLock,
                KeySerde::NumLock => Key::NumLock,
                KeySerde::PrintScreen => Key::PrintScreen,
                KeySerde::Pause => Key::Pause,
                KeySerde::Menu => Key::Menu,
                KeySerde::KeypadBegin => Key::KeypadBegin,
                KeySerde::Media(v) => Key::Media(v.into()),
                KeySerde::Esc => Key::Esc,
            }
        }
    }

    #[allow(clippy::trivially_copy_pass_by_ref)] // serde wants to give a reference
    pub fn serialize<S>(val: &Key, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        KeySerde::from(val).serialize(serializer)
    }

    pub fn deserialize<'de, D>(des: D) -> Result<Key, D::Error>
    where
        D: Deserializer<'de>,
    {
        let key = KeySerde::deserialize(des)?;

        Ok(key.into())
    }
}

/// Custom Serialize / Deserialize for [`KeyModifiers`] so that library updates mess with the config layout
mod key_serde_modifier {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use tuirealm::event::KeyModifiers;

    /// Emulate bitflags 1.x layout
    #[derive(Debug, Eq, PartialEq, Copy, Clone, Serialize, Deserialize)]
    #[serde(tag = "type", rename = "KeyModifiers")]
    struct Wrapper {
        bits: u8,
    }

    #[allow(clippy::trivially_copy_pass_by_ref)] // serde wants to give a reference
    pub fn serialize<S>(val: &KeyModifiers, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        Wrapper { bits: val.bits() }.serialize(serializer)
    }

    pub fn deserialize<'de, D>(des: D) -> Result<KeyModifiers, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wrapper = Wrapper::deserialize(des)?;
        let key_modifiers = KeyModifiers::from_bits_truncate(wrapper.bits);

        Ok(key_modifiers)
    }
}

#[derive(Clone, Deserialize, Copy, Eq, PartialEq, Hash, Serialize, Debug)]
pub struct BindingForEvent {
    #[serde(with = "key_serde_code")]
    pub code: Key,
    #[serde(with = "key_serde_modifier")]
    pub modifier: KeyModifiers,
}

impl std::fmt::Display for BindingForEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let code_string = if let Key::Char(char) = self.code {
            char.to_string()
        } else {
            format!("{:?}", self.code)
        };

        let code_string = code_string.replace("Function(", "F");
        let code_string = code_string.replace(')', "");
        let mut code_string = code_string.replace(' ', "Space");

        if self.modifier.intersects(KeyModifiers::CONTROL) {
            write!(f, "CTRL+")?;
        }

        if self.modifier.intersects(KeyModifiers::ALT) {
            write!(f, "ALT+")?;
        }

        if self.modifier.intersects(KeyModifiers::SHIFT) {
            write!(f, "SHIFT+")?;
            code_string = code_string.to_uppercase();
        }

        write!(f, "{code_string}")
    }
}

impl BindingForEvent {
    pub const fn key_event(&self) -> KeyEvent {
        KeyEvent {
            code: self.code,
            modifiers: self.modifier,
        }
    }

    /// Get the Current Modifier, and the string representation of the key
    pub fn mod_key(&self) -> (KeyModifiers, String) {
        (self.modifier, self.key())
    }

    pub fn key(&self) -> String {
        match self.code {
            Key::Backspace => "Backspace".to_string(),
            Key::Enter => "Enter".to_string(),
            Key::Left => "Left".to_string(),
            Key::Right => "Right".to_string(),
            Key::Up => "Up".to_string(),
            Key::Down => "Down".to_string(),
            Key::Home => "Home".to_string(),
            Key::End => "End".to_string(),
            Key::PageUp => "PageUp".to_string(),
            Key::PageDown => "PageDown".to_string(),
            Key::Tab => "Tab".to_string(),
            Key::BackTab => "BackTab".to_string(),
            Key::Delete => "Delete".to_string(),
            Key::Insert => "Insert".to_string(),
            Key::Function(int) => format!("F{int}"),
            Key::Char(char) => {
                if char == ' ' {
                    "Space".to_string()
                } else {
                    format!("{char}")
                }
            }
            Key::Null => "Null".to_string(),
            Key::Esc => "Esc".to_string(),
            Key::CapsLock => "CapsLock".to_string(),
            Key::ScrollLock => "ScrollLock".to_string(),
            Key::NumLock => "NumLock".to_string(),
            Key::PrintScreen => "PrintScreen".to_string(),
            Key::Pause => "Pause".to_string(),
            Key::Menu => "Menu".to_string(),
            Key::KeypadBegin => "KeyPadBegin".to_string(),
            Key::Media(media_key) => format!("Media {media_key:?}"),

            // the following are new events with tuirealm 2.0, but only available in backend "termion", which we dont use
            Key::ShiftLeft
            | Key::AltLeft
            | Key::CtrlLeft
            | Key::ShiftRight
            | Key::AltRight
            | Key::CtrlRight
            | Key::ShiftUp
            | Key::AltUp
            | Key::CtrlUp
            | Key::ShiftDown
            | Key::AltDown
            | Key::CtrlDown
            | Key::CtrlHome
            | Key::CtrlEnd => unimplemented!(),
        }
    }

    pub fn key_from_str(str: &str) -> Result<Key> {
        if str.is_empty() {
            bail!("Empty key")
        }

        if str.len() < 2 {
            let mut chars = str.chars();
            if let Some(char) = chars.next() {
                return Ok(Key::Char(char));
            }
        }
        let str_lower_case = str.to_lowercase();
        if str_lower_case.len() < 4 {
            if let Some(str) = str_lower_case.strip_prefix('f') {
                let my_int = u8::from_str(str)?;
                if my_int > 12 {
                    bail!("Function key should be smaller than F12.");
                }
                return Ok(Key::Function(my_int));
            }
        }
        let special_key = match str_lower_case.as_ref() {
            "backspace" => Key::Backspace,
            "enter" => Key::Enter,
            "left" => Key::Left,
            "right" => Key::Right,
            "up" => Key::Up,
            "down" => Key::Down,
            "home" => Key::Home,
            "end" => Key::End,
            "pageup" => Key::PageUp,
            "pagedown" => Key::PageDown,
            "tab" => Key::Tab,
            "backtab" => Key::BackTab,
            "delete" => Key::Delete,
            "insert" => Key::Insert,
            "esc" => Key::Esc,
            "space" => Key::Char(' '),
            // "null" => Key::Null,
            inv_key => bail!("Provided invalid special key: {}", inv_key),
        };
        Ok(special_key)
    }
}

impl Default for Keys {
    #[allow(clippy::too_many_lines)]
    fn default() -> Self {
        const CONTROL_SHIFT: KeyModifiers = KeyModifiers::CONTROL.union(KeyModifiers::SHIFT);

        Self {
            global_esc: BindingForEvent {
                code: Key::Esc,
                modifier: KeyModifiers::NONE,
            },
            global_quit: BindingForEvent {
                code: Key::Char('q'),
                modifier: KeyModifiers::NONE,
            },
            global_left: BindingForEvent {
                code: Key::Char('h'),
                modifier: KeyModifiers::NONE,
            },
            global_down: BindingForEvent {
                code: Key::Char('j'),
                modifier: KeyModifiers::NONE,
            },
            global_up: BindingForEvent {
                code: Key::Char('k'),
                modifier: KeyModifiers::NONE,
            },
            global_right: BindingForEvent {
                code: Key::Char('l'),
                modifier: KeyModifiers::NONE,
            },
            global_goto_top: BindingForEvent {
                code: Key::Char('g'),
                modifier: KeyModifiers::NONE,
            },
            global_goto_bottom: BindingForEvent {
                code: Key::Char('G'),
                modifier: KeyModifiers::SHIFT,
            },
            global_player_toggle_pause: BindingForEvent {
                code: Key::Char(' '),
                modifier: KeyModifiers::NONE,
            },
            global_player_next: BindingForEvent {
                code: Key::Char('n'),
                modifier: KeyModifiers::NONE,
            },
            global_player_previous: BindingForEvent {
                code: Key::Char('N'),
                modifier: KeyModifiers::SHIFT,
            },
            global_player_volume_plus_1: BindingForEvent {
                code: Key::Char('+'),
                modifier: KeyModifiers::SHIFT,
            },
            global_player_volume_plus_2: BindingForEvent {
                code: Key::Char('='),
                modifier: KeyModifiers::NONE,
            },
            global_player_volume_minus_1: BindingForEvent {
                code: Key::Char('-'),
                modifier: KeyModifiers::SHIFT,
            },
            global_player_volume_minus_2: BindingForEvent {
                code: Key::Char('_'),
                modifier: KeyModifiers::NONE,
            },
            global_help: BindingForEvent {
                code: Key::Char('h'),
                modifier: KeyModifiers::CONTROL,
            },
            global_player_seek_forward: BindingForEvent {
                code: Key::Char('f'),
                modifier: KeyModifiers::NONE,
            },
            global_player_seek_backward: BindingForEvent {
                code: Key::Char('b'),
                modifier: KeyModifiers::NONE,
            },
            global_player_speed_up: BindingForEvent {
                code: Key::Char('f'),
                modifier: KeyModifiers::CONTROL,
            },
            global_player_speed_down: BindingForEvent {
                code: Key::Char('b'),
                modifier: KeyModifiers::CONTROL,
            },

            global_lyric_adjust_forward: BindingForEvent {
                code: Key::Char('F'),
                modifier: KeyModifiers::SHIFT,
            },
            global_lyric_adjust_backward: BindingForEvent {
                code: Key::Char('B'),
                modifier: KeyModifiers::SHIFT,
            },
            global_lyric_cycle: BindingForEvent {
                code: Key::Char('T'),
                modifier: KeyModifiers::SHIFT,
            },
            library_load_dir: BindingForEvent {
                code: Key::Char('L'),
                modifier: KeyModifiers::SHIFT,
            },
            library_delete: BindingForEvent {
                code: Key::Char('d'),
                modifier: KeyModifiers::NONE,
            },
            library_yank: BindingForEvent {
                code: Key::Char('y'),
                modifier: KeyModifiers::NONE,
            },
            library_paste: BindingForEvent {
                code: Key::Char('p'),
                modifier: KeyModifiers::NONE,
            },
            library_search: BindingForEvent {
                code: Key::Char('/'),
                modifier: KeyModifiers::NONE,
            },
            library_search_youtube: BindingForEvent {
                code: Key::Char('s'),
                modifier: KeyModifiers::NONE,
            },
            library_tag_editor_open: BindingForEvent {
                code: Key::Char('t'),
                modifier: KeyModifiers::NONE,
            },
            playlist_delete: BindingForEvent {
                code: Key::Char('d'),
                modifier: KeyModifiers::NONE,
            },
            playlist_delete_all: BindingForEvent {
                code: Key::Char('D'),
                modifier: KeyModifiers::SHIFT,
            },
            playlist_shuffle: BindingForEvent {
                code: Key::Char('r'),
                modifier: KeyModifiers::NONE,
            },
            playlist_mode_cycle: BindingForEvent {
                code: Key::Char('m'),
                modifier: KeyModifiers::NONE,
            },
            playlist_play_selected: BindingForEvent {
                code: Key::Char('l'),
                modifier: KeyModifiers::NONE,
            },
            playlist_search: BindingForEvent {
                code: Key::Char('/'),
                modifier: KeyModifiers::NONE,
            },
            playlist_swap_down: BindingForEvent {
                code: Key::Char('J'),
                modifier: KeyModifiers::SHIFT,
            },
            playlist_swap_up: BindingForEvent {
                code: Key::Char('K'),
                modifier: KeyModifiers::SHIFT,
            },
            playlist_add_random_album: BindingForEvent {
                code: Key::Char('S'),
                modifier: KeyModifiers::SHIFT,
            },
            playlist_add_random_tracks: BindingForEvent {
                code: Key::Char('s'),
                modifier: KeyModifiers::NONE,
            },
            global_layout_treeview: BindingForEvent {
                code: Key::Char('1'),
                modifier: KeyModifiers::NONE,
            },
            global_layout_database: BindingForEvent {
                code: Key::Char('2'),
                modifier: KeyModifiers::NONE,
            },
            database_add_all: BindingForEvent {
                code: Key::Char('L'),
                modifier: KeyModifiers::SHIFT,
            },
            global_player_toggle_gapless: BindingForEvent {
                code: Key::Char('g'),
                modifier: KeyModifiers::CONTROL,
            },
            global_config_open: BindingForEvent {
                code: Key::Char('C'),
                modifier: KeyModifiers::SHIFT,
            },
            config_save: BindingForEvent {
                code: Key::Char('s'),
                modifier: KeyModifiers::CONTROL,
            },
            library_switch_root: BindingForEvent {
                code: Key::Char('o'),
                modifier: KeyModifiers::NONE,
            },
            library_add_root: BindingForEvent {
                code: Key::Char('a'),
                modifier: KeyModifiers::NONE,
            },
            library_remove_root: BindingForEvent {
                code: Key::Char('A'),
                modifier: KeyModifiers::SHIFT,
            },
            global_save_playlist: BindingForEvent {
                code: Key::Char('s'),
                modifier: KeyModifiers::CONTROL,
            },
            global_layout_podcast: BindingForEvent {
                code: Key::Char('3'),
                modifier: KeyModifiers::NONE,
            },
            podcast_search_add_feed: BindingForEvent {
                code: Key::Char('s'),
                modifier: KeyModifiers::NONE,
            },
            podcast_mark_played: BindingForEvent {
                code: Key::Char('m'),
                modifier: KeyModifiers::NONE,
            },
            podcast_mark_all_played: BindingForEvent {
                code: Key::Char('M'),
                modifier: KeyModifiers::SHIFT,
            },
            podcast_refresh_feed: BindingForEvent {
                code: Key::Char('r'),
                modifier: KeyModifiers::NONE,
            },
            podcast_refresh_all_feeds: BindingForEvent {
                code: Key::Char('R'),
                modifier: KeyModifiers::SHIFT,
            },
            podcast_episode_download: BindingForEvent {
                code: Key::Char('d'),
                modifier: KeyModifiers::NONE,
            },
            podcast_episode_delete_file: BindingForEvent {
                code: Key::Char('x'),
                modifier: KeyModifiers::NONE,
            },
            podcast_delete_feed: BindingForEvent {
                code: Key::Char('d'),
                modifier: KeyModifiers::NONE,
            },
            podcast_delete_all_feeds: BindingForEvent {
                code: Key::Char('D'),
                modifier: KeyModifiers::SHIFT,
            },
            global_xywh_move_left: BindingForEvent {
                code: Key::Left,
                modifier: CONTROL_SHIFT,
            },
            global_xywh_move_right: BindingForEvent {
                code: Key::Right,
                modifier: CONTROL_SHIFT,
            },
            global_xywh_move_up: BindingForEvent {
                code: Key::Up,
                modifier: CONTROL_SHIFT,
            },
            global_xywh_move_down: BindingForEvent {
                code: Key::Down,
                modifier: CONTROL_SHIFT,
            },
            global_xywh_zoom_in: BindingForEvent {
                code: Key::PageUp,
                modifier: CONTROL_SHIFT,
            },
            global_xywh_zoom_out: BindingForEvent {
                code: Key::PageDown,
                modifier: CONTROL_SHIFT,
            },
            global_xywh_hide: BindingForEvent {
                code: Key::End,
                modifier: CONTROL_SHIFT,
            },
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    mod keys {
        use super::*;

        #[test]
        fn should_serialize_deserialize_default() {
            let buffer = toml::to_string(&Keys::default()).unwrap();
            let _: Keys = toml::from_str(&buffer).unwrap();
        }
    }

    mod bindings_serde {
        use super::*;
        use pretty_assertions::assert_eq;

        fn helper_tostring<T: Serialize>(val: T) -> String {
            toml::to_string(&val).unwrap()
        }

        fn helper_fromstring(buf: &str) -> BindingForEvent {
            toml::from_str(buf).unwrap()
        }

        #[test]
        fn should_consistently_serialize() {
            let val = BindingForEvent {
                code: Key::Esc,
                modifier: KeyModifiers::NONE,
            };
            assert_eq!(
                "[code]\ntype = \"Esc\"\n\n[modifier]\ntype = \"KeyModifiers\"\nbits = 0\n",
                &helper_tostring(val)
            );

            let val = BindingForEvent {
                code: Key::Char('a'),
                modifier: KeyModifiers::SHIFT,
            };
            assert_eq!("[code]\ntype = \"Char\"\nargs = \"a\"\n\n[modifier]\ntype = \"KeyModifiers\"\nbits = 1\n", &helper_tostring(val));

            let val = BindingForEvent {
                code: Key::Char('x'),
                modifier: KeyModifiers::SHIFT | KeyModifiers::CONTROL | KeyModifiers::ALT,
            };
            assert_eq!("[code]\ntype = \"Char\"\nargs = \"x\"\n\n[modifier]\ntype = \"KeyModifiers\"\nbits = 7\n", &helper_tostring(val));

            let val = BindingForEvent {
                code: Key::Char('s'),
                modifier: KeyModifiers::SHIFT | KeyModifiers::ALT,
            };
            assert_eq!("[code]\ntype = \"Char\"\nargs = \"s\"\n\n[modifier]\ntype = \"KeyModifiers\"\nbits = 5\n", &helper_tostring(val));
        }

        #[test]
        fn should_consistently_deserialize() {
            let val = "[code]\ntype = \"Esc\"\n\n[modifier]\ntype = \"KeyModifiers\"\nbits = 0\n";
            assert_eq!(
                BindingForEvent {
                    code: Key::Esc,
                    modifier: KeyModifiers::NONE,
                },
                helper_fromstring(val)
            );

            let val = "[code]\ntype = \"Char\"\nargs = \"a\"\n\n[modifier]\ntype = \"KeyModifiers\"\nbits = 1\n";
            assert_eq!(
                BindingForEvent {
                    code: Key::Char('a'),
                    modifier: KeyModifiers::SHIFT,
                },
                helper_fromstring(val)
            );

            let val = "[code]\ntype = \"Char\"\nargs = \"x\"\n\n[modifier]\ntype = \"KeyModifiers\"\nbits = 7\n";
            assert_eq!(
                BindingForEvent {
                    code: Key::Char('x'),
                    modifier: KeyModifiers::SHIFT | KeyModifiers::CONTROL | KeyModifiers::ALT,
                },
                helper_fromstring(val)
            );

            let val = "[code]\ntype = \"Char\"\nargs = \"s\"\n\n[modifier]\ntype = \"KeyModifiers\"\nbits = 5\n";
            assert_eq!(
                BindingForEvent {
                    code: Key::Char('s'),
                    modifier: KeyModifiers::SHIFT | KeyModifiers::ALT,
                },
                helper_fromstring(val)
            );
        }
    }
}
