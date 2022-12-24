use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::hash::Hash;
use std::iter::once;
use std::str::FromStr;
use tuirealm::event::{Key, KeyEvent, KeyModifiers};

pub const CONTROL_SHIFT: KeyModifiers =
    KeyModifiers::from_bits_truncate(KeyModifiers::CONTROL.bits() | KeyModifiers::SHIFT.bits());
pub const ALT_SHIFT: KeyModifiers =
    KeyModifiers::from_bits_truncate(KeyModifiers::ALT.bits() | KeyModifiers::SHIFT.bits());
pub const CONTROL_ALT: KeyModifiers =
    KeyModifiers::from_bits_truncate(KeyModifiers::ALT.bits() | KeyModifiers::CONTROL.bits());
pub const CONTROL_ALT_SHIFT: KeyModifiers = KeyModifiers::from_bits_truncate(
    KeyModifiers::ALT.bits() | KeyModifiers::CONTROL.bits() | KeyModifiers::SHIFT.bits(),
);

#[derive(Clone, Deserialize, Serialize)]
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
    pub library_load_dir: BindingForEvent,
    pub library_delete: BindingForEvent,
    pub library_yank: BindingForEvent,
    pub library_paste: BindingForEvent,
    pub library_search: BindingForEvent,
    pub library_search_youtube: BindingForEvent,
    pub library_tag_editor_open: BindingForEvent,
    pub playlist_delete: BindingForEvent,
    pub playlist_delete_all: BindingForEvent,
    pub playlist_shuffle: BindingForEvent,
    pub playlist_mode_cycle: BindingForEvent,
    pub playlist_play_selected: BindingForEvent,
    pub playlist_add_front: BindingForEvent,
    pub playlist_search: BindingForEvent,
    pub playlist_swap_down: BindingForEvent,
    pub playlist_swap_up: BindingForEvent,
    pub playlist_cmus_lqueue: BindingForEvent,
    pub playlist_cmus_tqueue: BindingForEvent,
    pub database_add_all: BindingForEvent,
    pub global_player_toggle_gapless: BindingForEvent,
    pub global_config_open: BindingForEvent,
    pub config_save: BindingForEvent,
    pub library_switch_root: BindingForEvent,
    pub library_add_root: BindingForEvent,
    pub library_remove_root: BindingForEvent,
    pub global_save_playlist: BindingForEvent,
}

impl Keys {
    // In order to check if duplicate keys are configured, please ensure all are included here
    pub fn iter_global(&self) -> impl Iterator<Item = BindingForEvent> {
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
        // .chain(once(self.config_save))
    }

    pub fn iter_library(&self) -> impl Iterator<Item = BindingForEvent> {
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
        // This is not necessary
        // .chain(once(self.database_add_all))
    }

    pub fn iter_playlist(&self) -> impl Iterator<Item = BindingForEvent> {
        once(self.playlist_delete)
            .chain(once(self.playlist_delete_all))
            .chain(once(self.playlist_shuffle))
            .chain(once(self.playlist_mode_cycle))
            .chain(once(self.playlist_play_selected))
            .chain(once(self.playlist_add_front))
            .chain(once(self.playlist_search))
            .chain(once(self.playlist_swap_down))
            .chain(once(self.playlist_swap_up))
            .chain(once(self.playlist_cmus_lqueue))
            .chain(once(self.playlist_cmus_tqueue))
    }

    pub fn has_unique_elements(&self) -> bool {
        let mut uniq_global = HashSet::new();
        let mut uniq_library = HashSet::new();
        let mut uniq_playlist = HashSet::new();
        self.iter_global().all(move |x| uniq_global.insert(x))
            && self.iter_library().all(move |x| uniq_library.insert(x))
            && self.iter_playlist().all(move |x| uniq_playlist.insert(x))
    }
}

#[derive(Clone, Deserialize, Copy, Eq, PartialEq, Hash, Serialize)]
pub struct BindingForEvent {
    pub code: Key,
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
        let code_string = code_string.replace(' ', "Space");
        match self.modifier {
            KeyModifiers::NONE => write!(f, "{code_string}"),
            KeyModifiers::SHIFT => write!(f, "SHIFT+{}", code_string.to_uppercase()),
            KeyModifiers::CONTROL => write!(f, "CTRL+{code_string}"),
            KeyModifiers::ALT => write!(f, "ALT+{code_string}"),
            CONTROL_SHIFT => write!(f, "CTRL+SHIFT+{code_string}"),
            ALT_SHIFT => write!(f, "ALT+SHIFT+{code_string}"),
            CONTROL_ALT => write!(f, "CTRL+ALT+{code_string}"),
            CONTROL_ALT_SHIFT => write!(f, "CTRL+ALT+SHIFT+{code_string}"),
            _ => write!(f, "Wrong Modifiers"),
        }
    }
}

impl BindingForEvent {
    pub const fn key_event(&self) -> KeyEvent {
        KeyEvent {
            code: self.code,
            modifiers: self.modifier,
        }
    }

    pub fn mod_key(&self) -> (usize, String) {
        (self.modifier(), self.key())
    }

    pub const fn modifier(&self) -> usize {
        match self.modifier {
            // KeyModifiers::NONE => 0,
            KeyModifiers::SHIFT => 1,
            KeyModifiers::CONTROL => 2,
            KeyModifiers::ALT => 3,
            CONTROL_SHIFT => 4,
            ALT_SHIFT => 5,
            CONTROL_ALT => 6,
            CONTROL_ALT_SHIFT => 7,
            _ => 0,
            // _ => 0,
        }
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
        }
    }

    pub fn key_from_str(str: &str) -> Result<Key> {
        if str.is_empty() {
            bail!("Empty key")
        }

        if str.len() < 2 {
            let mut chars = str.chars();
            let char = chars.next().unwrap();
            return Ok(Key::Char(char));
        }
        if str.starts_with('F') {
            let mut chars = str.chars();
            chars.next();
            let my_int = u8::from_str(chars.as_str())?;
            if my_int > 12 {
                bail!("Function key should be smaller than F12.");
            }
            return Ok(Key::Function(my_int));
        }
        let str_lower_case = str.to_lowercase();
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
            "f1" => Key::Function(1),
            "f2" => Key::Function(2),
            "f3" => Key::Function(3),
            "f4" => Key::Function(4),
            "f5" => Key::Function(5),
            "f6" => Key::Function(6),
            "f7" => Key::Function(7),
            "f8" => Key::Function(8),
            "f9" => Key::Function(9),
            "f10" => Key::Function(10),
            "f11" => Key::Function(11),
            "f12" => Key::Function(12),
            "space" => Key::Char(' '),
            // "null" => Key::Null,
            &_ => bail!("Error key configured"),
        };
        Ok(special_key)
    }
}

impl Default for Keys {
    #[allow(clippy::too_many_lines)]
    fn default() -> Self {
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
                code: Key::Char('_'),
                modifier: KeyModifiers::SHIFT,
            },
            global_player_volume_minus_2: BindingForEvent {
                code: Key::Char('-'),
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
            playlist_add_front: BindingForEvent {
                code: Key::Char('a'),
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
            playlist_cmus_lqueue: BindingForEvent {
                code: Key::Char('S'),
                modifier: KeyModifiers::SHIFT,
            },
            playlist_cmus_tqueue: BindingForEvent {
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
        }
    }
}
