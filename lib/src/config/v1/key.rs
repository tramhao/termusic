use serde::{Deserialize, Serialize};
use std::hash::Hash;
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

/// Custom Serialize / Deserialize for [`Key`] so that library updates dont mess with the config layout
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

/// Custom Serialize / Deserialize for [`KeyModifiers`] so that library updates dont mess with the config layout
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
    #[inline]
    pub const fn key_event(&self) -> KeyEvent {
        KeyEvent {
            code: self.code,
            modifiers: self.modifier,
        }
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
            assert_eq!(
                "[code]\ntype = \"Char\"\nargs = \"a\"\n\n[modifier]\ntype = \"KeyModifiers\"\nbits = 1\n",
                &helper_tostring(val)
            );

            let val = BindingForEvent {
                code: Key::Char('x'),
                modifier: KeyModifiers::SHIFT | KeyModifiers::CONTROL | KeyModifiers::ALT,
            };
            assert_eq!(
                "[code]\ntype = \"Char\"\nargs = \"x\"\n\n[modifier]\ntype = \"KeyModifiers\"\nbits = 7\n",
                &helper_tostring(val)
            );

            let val = BindingForEvent {
                code: Key::Char('s'),
                modifier: KeyModifiers::SHIFT | KeyModifiers::ALT,
            };
            assert_eq!(
                "[code]\ntype = \"Char\"\nargs = \"s\"\n\n[modifier]\ntype = \"KeyModifiers\"\nbits = 5\n",
                &helper_tostring(val)
            );
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
