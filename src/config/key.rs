use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
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
    pub global_color_editor_open: BindingForEvent,
    pub global_key_editor_open: BindingForEvent,
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
    pub database_add_all: BindingForEvent,
    pub global_player_toggle_gapless: BindingForEvent,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct BindingForEvent {
    pub code: Key,
    pub modifiers: KeyModifiers,
}

impl std::fmt::Display for BindingForEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let code_string = if let Key::Char(char) = self.code {
            char.to_string()
        } else {
            format!("{:?}", self.code)
        };
        match self.modifiers {
            KeyModifiers::NONE => write!(f, "{}", code_string),
            KeyModifiers::SHIFT => write!(f, "{}", code_string.to_uppercase()),
            KeyModifiers::CONTROL => write!(f, "CTRL+{}", code_string),
            KeyModifiers::ALT => write!(f, "ALT+{}", code_string),
            _ => write!(f, "Wrong Modifiers"),
        }
    }
}

impl BindingForEvent {
    pub const fn key_event(&self) -> KeyEvent {
        KeyEvent {
            code: self.code,
            modifiers: self.modifiers,
        }
    }
    pub const fn modifier(&self) -> usize {
        match self.modifiers {
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
            Key::Function(int) => format!("F{}", int),
            Key::Char(char) => format!("{}", char),
            Key::Null => "Null".to_string(),
            Key::Esc => "Esc".to_string(),
        }
    }

    pub fn key_from_str(str: &str) -> Result<Key> {
        if str.starts_with('F') {
            let mut chars = str.chars();
            chars.next();
            // chars.as_str();
            let my_int = u8::from_str(chars.as_str()).unwrap_or(12);
            return Ok(Key::Function(my_int));
        }
        if str.len() < 2 {
            let mut chars = str.chars();
            let char = chars.next().unwrap();
            return Ok(Key::Char(char));
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
            "null" => Key::Null,
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
                modifiers: KeyModifiers::NONE,
            },
            global_quit: BindingForEvent {
                code: Key::Char('q'),
                modifiers: KeyModifiers::NONE,
            },
            global_left: BindingForEvent {
                code: Key::Char('h'),
                modifiers: KeyModifiers::NONE,
            },
            global_down: BindingForEvent {
                code: Key::Char('j'),
                modifiers: KeyModifiers::NONE,
            },
            global_up: BindingForEvent {
                code: Key::Char('k'),
                modifiers: KeyModifiers::NONE,
            },
            global_right: BindingForEvent {
                code: Key::Char('l'),
                modifiers: KeyModifiers::NONE,
            },
            global_goto_top: BindingForEvent {
                code: Key::Char('g'),
                modifiers: KeyModifiers::NONE,
            },
            global_goto_bottom: BindingForEvent {
                code: Key::Char('G'),
                modifiers: KeyModifiers::SHIFT,
            },
            global_player_toggle_pause: BindingForEvent {
                code: Key::Char(' '),
                modifiers: KeyModifiers::NONE,
            },
            global_player_next: BindingForEvent {
                code: Key::Char('n'),
                modifiers: KeyModifiers::NONE,
            },
            global_player_previous: BindingForEvent {
                code: Key::Char('N'),
                modifiers: KeyModifiers::SHIFT,
            },
            global_player_volume_plus_1: BindingForEvent {
                code: Key::Char('+'),
                modifiers: KeyModifiers::SHIFT,
            },
            global_player_volume_plus_2: BindingForEvent {
                code: Key::Char('='),
                modifiers: KeyModifiers::NONE,
            },
            global_player_volume_minus_1: BindingForEvent {
                code: Key::Char('_'),
                modifiers: KeyModifiers::SHIFT,
            },
            global_player_volume_minus_2: BindingForEvent {
                code: Key::Char('-'),
                modifiers: KeyModifiers::NONE,
            },
            global_help: BindingForEvent {
                code: Key::Char('h'),
                modifiers: KeyModifiers::CONTROL,
            },
            global_player_seek_forward: BindingForEvent {
                code: Key::Char('f'),
                modifiers: KeyModifiers::NONE,
            },
            global_player_seek_backward: BindingForEvent {
                code: Key::Char('b'),
                modifiers: KeyModifiers::NONE,
            },
            global_player_speed_up: BindingForEvent {
                code: Key::Char('f'),
                modifiers: KeyModifiers::CONTROL,
            },
            global_player_speed_down: BindingForEvent {
                code: Key::Char('b'),
                modifiers: KeyModifiers::CONTROL,
            },

            global_lyric_adjust_forward: BindingForEvent {
                code: Key::Char('F'),
                modifiers: KeyModifiers::SHIFT,
            },
            global_lyric_adjust_backward: BindingForEvent {
                code: Key::Char('B'),
                modifiers: KeyModifiers::SHIFT,
            },
            global_lyric_cycle: BindingForEvent {
                code: Key::Char('T'),
                modifiers: KeyModifiers::SHIFT,
            },
            global_color_editor_open: BindingForEvent {
                code: Key::Char('C'),
                modifiers: KeyModifiers::SHIFT,
            },
            global_key_editor_open: BindingForEvent {
                code: Key::Char('K'),
                modifiers: KeyModifiers::SHIFT,
            },
            library_load_dir: BindingForEvent {
                code: Key::Char('L'),
                modifiers: KeyModifiers::SHIFT,
            },
            library_delete: BindingForEvent {
                code: Key::Char('d'),
                modifiers: KeyModifiers::NONE,
            },
            library_yank: BindingForEvent {
                code: Key::Char('y'),
                modifiers: KeyModifiers::NONE,
            },
            library_paste: BindingForEvent {
                code: Key::Char('p'),
                modifiers: KeyModifiers::NONE,
            },
            library_search: BindingForEvent {
                code: Key::Char('/'),
                modifiers: KeyModifiers::NONE,
            },
            library_search_youtube: BindingForEvent {
                code: Key::Char('s'),
                modifiers: KeyModifiers::NONE,
            },
            library_tag_editor_open: BindingForEvent {
                code: Key::Char('t'),
                modifiers: KeyModifiers::NONE,
            },
            playlist_delete: BindingForEvent {
                code: Key::Char('d'),
                modifiers: KeyModifiers::NONE,
            },
            playlist_delete_all: BindingForEvent {
                code: Key::Char('D'),
                modifiers: KeyModifiers::SHIFT,
            },
            playlist_shuffle: BindingForEvent {
                code: Key::Char('s'),
                modifiers: KeyModifiers::NONE,
            },
            playlist_mode_cycle: BindingForEvent {
                code: Key::Char('m'),
                modifiers: KeyModifiers::NONE,
            },
            playlist_play_selected: BindingForEvent {
                code: Key::Char('l'),
                modifiers: KeyModifiers::NONE,
            },
            playlist_add_front: BindingForEvent {
                code: Key::Char('a'),
                modifiers: KeyModifiers::NONE,
            },
            playlist_search: BindingForEvent {
                code: Key::Char('/'),
                modifiers: KeyModifiers::NONE,
            },
            playlist_swap_down: BindingForEvent {
                code: Key::Char('j'),
                modifiers: KeyModifiers::CONTROL,
            },
            playlist_swap_up: BindingForEvent {
                code: Key::Char('k'),
                modifiers: KeyModifiers::CONTROL,
            },
            global_layout_treeview: BindingForEvent {
                code: Key::Char('1'),
                modifiers: KeyModifiers::NONE,
            },
            global_layout_database: BindingForEvent {
                code: Key::Char('2'),
                modifiers: KeyModifiers::NONE,
            },
            database_add_all: BindingForEvent {
                code: Key::Char('L'),
                modifiers: KeyModifiers::SHIFT,
            },
            global_player_toggle_gapless: BindingForEvent {
                code: Key::Char('g'),
                modifiers: KeyModifiers::CONTROL,
            },
        }
    }
}
