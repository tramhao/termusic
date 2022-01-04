use crate::ui::components::StyleColorSymbol;
use crate::ui::components::Xywh;
use crate::ui::Loop;
use anyhow::{anyhow, Result};
/**
 * MIT License
 *
 * termusic - Copyright (c) 2021 Larry Hao
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
use serde::{Deserialize, Serialize};
use std::fs::{self, read_to_string};
use std::path::PathBuf;
use tuirealm::event::{Key, KeyEvent, KeyModifiers};

pub const MUSIC_DIR: &str = "~/Music";

#[derive(Clone, Deserialize, Serialize)]
pub struct Termusic {
    pub music_dir: String,
    #[serde(skip_serializing)]
    pub music_dir_from_cli: Option<String>,
    pub loop_mode: Loop,
    pub volume: i32,
    pub add_playlist_front: bool,
    pub disable_exit_confirmation: bool,
    pub theme_selected: String,
    pub style_color_symbol: StyleColorSymbol,
    pub album_photo_xywh: Xywh,
    pub keys: Keys,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct Keys {
    pub global_esc: KeyBind,
    pub global_quit: KeyBind,
    pub global_left: KeyBind,
    pub global_down: KeyBind,
    pub global_up: KeyBind,
    pub global_right: KeyBind,
    pub global_goto_top: KeyBind,
    pub global_goto_bottom: KeyBind,
    pub global_player_toggle_pause: KeyBind,
    pub global_player_next: KeyBind,
    pub global_player_previous: KeyBind,
    pub global_player_volume_plus_1: KeyBind,
    pub global_player_volume_plus_2: KeyBind,
    pub global_player_volume_minus_1: KeyBind,
    pub global_player_volume_minus_2: KeyBind,
    pub global_help: KeyBind,
    pub global_player_seek_forward: KeyBind,
    pub global_player_seek_backward: KeyBind,
    pub global_lyric_adjust_forward: KeyBind,
    pub global_lyric_adjust_backward: KeyBind,
    pub global_lyric_cycle: KeyBind,
    pub global_color_editor_open: KeyBind,
    pub library_load_dir: KeyBind,
    pub library_delete: KeyBind,
    pub library_yank: KeyBind,
    pub library_paste: KeyBind,
    pub library_search: KeyBind,
    pub library_search_youtube: KeyBind,
    pub library_tag_editor_open: KeyBind,
    pub playlist_delete: KeyBind,
    pub playlist_delete_all: KeyBind,
    pub playlist_shuffle: KeyBind,
    pub playlist_mode_cycle: KeyBind,
    pub playlist_play_selected: KeyBind,
    pub playlist_add_front: KeyBind,
    pub playlist_search: KeyBind,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct KeyBind {
    code: Key,
    modifiers: KeyModifiers,
}

impl std::fmt::Display for KeyBind {
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

impl KeyBind {
    pub const fn key_event(&self) -> KeyEvent {
        KeyEvent {
            code: self.code,
            modifiers: self.modifiers,
        }
    }
}

impl Default for Keys {
    #[allow(clippy::too_many_lines)]
    fn default() -> Self {
        Self {
            global_esc: KeyBind {
                code: Key::Esc,
                modifiers: KeyModifiers::NONE,
            },
            global_quit: KeyBind {
                code: Key::Char('q'),
                modifiers: KeyModifiers::NONE,
            },
            global_left: KeyBind {
                code: Key::Char('h'),
                modifiers: KeyModifiers::NONE,
            },
            global_down: KeyBind {
                code: Key::Char('j'),
                modifiers: KeyModifiers::NONE,
            },
            global_up: KeyBind {
                code: Key::Char('k'),
                modifiers: KeyModifiers::NONE,
            },
            global_right: KeyBind {
                code: Key::Char('l'),
                modifiers: KeyModifiers::NONE,
            },
            global_goto_top: KeyBind {
                code: Key::Char('g'),
                modifiers: KeyModifiers::NONE,
            },
            global_goto_bottom: KeyBind {
                code: Key::Char('G'),
                modifiers: KeyModifiers::SHIFT,
            },
            global_player_toggle_pause: KeyBind {
                code: Key::Char(' '),
                modifiers: KeyModifiers::NONE,
            },
            global_player_next: KeyBind {
                code: Key::Char('n'),
                modifiers: KeyModifiers::NONE,
            },
            global_player_previous: KeyBind {
                code: Key::Char('N'),
                modifiers: KeyModifiers::SHIFT,
            },
            global_player_volume_plus_1: KeyBind {
                code: Key::Char('+'),
                modifiers: KeyModifiers::SHIFT,
            },
            global_player_volume_plus_2: KeyBind {
                code: Key::Char('='),
                modifiers: KeyModifiers::NONE,
            },
            global_player_volume_minus_1: KeyBind {
                code: Key::Char('_'),
                modifiers: KeyModifiers::SHIFT,
            },
            global_player_volume_minus_2: KeyBind {
                code: Key::Char('-'),
                modifiers: KeyModifiers::NONE,
            },
            global_help: KeyBind {
                code: Key::Char('h'),
                modifiers: KeyModifiers::CONTROL,
            },
            global_player_seek_forward: KeyBind {
                code: Key::Char('f'),
                modifiers: KeyModifiers::NONE,
            },
            global_player_seek_backward: KeyBind {
                code: Key::Char('b'),
                modifiers: KeyModifiers::NONE,
            },
            global_lyric_adjust_forward: KeyBind {
                code: Key::Char('F'),
                modifiers: KeyModifiers::SHIFT,
            },
            global_lyric_adjust_backward: KeyBind {
                code: Key::Char('B'),
                modifiers: KeyModifiers::SHIFT,
            },
            global_lyric_cycle: KeyBind {
                code: Key::Char('T'),
                modifiers: KeyModifiers::SHIFT,
            },
            global_color_editor_open: KeyBind {
                code: Key::Char('C'),
                modifiers: KeyModifiers::SHIFT,
            },
            library_load_dir: KeyBind {
                code: Key::Char('L'),
                modifiers: KeyModifiers::SHIFT,
            },
            library_delete: KeyBind {
                code: Key::Char('d'),
                modifiers: KeyModifiers::NONE,
            },
            library_yank: KeyBind {
                code: Key::Char('y'),
                modifiers: KeyModifiers::NONE,
            },
            library_paste: KeyBind {
                code: Key::Char('p'),
                modifiers: KeyModifiers::NONE,
            },
            library_search: KeyBind {
                code: Key::Char('/'),
                modifiers: KeyModifiers::NONE,
            },
            library_search_youtube: KeyBind {
                code: Key::Char('s'),
                modifiers: KeyModifiers::NONE,
            },
            library_tag_editor_open: KeyBind {
                code: Key::Char('t'),
                modifiers: KeyModifiers::NONE,
            },
            playlist_delete: KeyBind {
                code: Key::Char('d'),
                modifiers: KeyModifiers::NONE,
            },
            playlist_delete_all: KeyBind {
                code: Key::Char('D'),
                modifiers: KeyModifiers::SHIFT,
            },
            playlist_shuffle: KeyBind {
                code: Key::Char('s'),
                modifiers: KeyModifiers::NONE,
            },
            playlist_mode_cycle: KeyBind {
                code: Key::Char('m'),
                modifiers: KeyModifiers::NONE,
            },
            playlist_play_selected: KeyBind {
                code: Key::Char('l'),
                modifiers: KeyModifiers::NONE,
            },
            playlist_add_front: KeyBind {
                code: Key::Char('a'),
                modifiers: KeyModifiers::NONE,
            },
            playlist_search: KeyBind {
                code: Key::Char('s'),
                modifiers: KeyModifiers::NONE,
            },
        }
    }
}
impl Default for Termusic {
    fn default() -> Self {
        Self {
            music_dir: MUSIC_DIR.to_string(),
            music_dir_from_cli: None,
            loop_mode: Loop::Queue,
            volume: 70,
            add_playlist_front: false,
            disable_exit_confirmation: false,
            keys: Keys::default(),
            theme_selected: "default".to_string(),
            style_color_symbol: StyleColorSymbol::default(),
            album_photo_xywh: Xywh::default(),
        }
    }
}

impl Termusic {
    pub fn save(&self) -> Result<()> {
        let mut path = get_app_config_path()?;
        path.push("config.toml");

        let string = toml::to_string(self)?;

        fs::write(path.to_string_lossy().as_ref(), string)?;

        Ok(())
    }

    pub fn load(&mut self) -> Result<()> {
        let mut path = get_app_config_path()?;
        path.push("config.toml");
        if !path.exists() {
            let config = Self::default();
            config.save()?;
        }

        let string = read_to_string(path.to_string_lossy().as_ref())?;
        let config: Self = toml::from_str(&string)?;
        *self = config;
        Ok(())
    }
}

pub fn get_app_config_path() -> Result<PathBuf> {
    let mut path =
        dirs_next::config_dir().ok_or_else(|| anyhow!("failed to find os config dir."))?;
    path.push("termusic");

    if !path.exists() {
        fs::create_dir_all(&path)?;
    }
    Ok(path)
}
