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
mod key;
mod theme;

use crate::player::Loop;
use crate::ui::components::Xywh;
use anyhow::{anyhow, Result};
pub use key::{BindingForEvent, Keys, ALT_SHIFT, CONTROL_ALT, CONTROL_ALT_SHIFT, CONTROL_SHIFT};
use serde::{Deserialize, Serialize};
use std::fs::{self, read_to_string};
use std::path::{Path, PathBuf};
pub use theme::{load_alacritty, ColorTermusic, StyleColorSymbol};

pub const MUSIC_DIR: [&str; 2] = ["~/Music/mp3", "~/Music"];

#[derive(Clone, Deserialize, Serialize)]
#[allow(clippy::struct_excessive_bools)]
pub struct Settings {
    pub music_dir: Vec<String>,
    #[serde(skip)]
    pub music_dir_from_cli: Option<String>,
    #[serde(skip)]
    pub disable_album_art_from_cli: bool,
    #[serde(skip)]
    pub disable_discord_rpc_from_cli: bool,
    #[serde(skip)]
    pub max_depth_cli: usize,
    pub loop_mode: Loop,
    pub volume: i32,
    pub speed: i32,
    pub add_playlist_front: bool,
    pub gapless: bool,
    pub enable_exit_confirmation: bool,
    pub playlist_display_symbol: bool,
    pub playlist_select_random_track_quantity: u32,
    pub playlist_select_random_album_quantity: u32,
    pub theme_selected: String,
    pub album_photo_xywh: Xywh,
    pub style_color_symbol: StyleColorSymbol,
    pub keys: Keys,
}

impl Default for Settings {
    fn default() -> Self {
        let mut music_dir = Vec::new();
        for dir in &MUSIC_DIR {
            let absolute_dir = shellexpand::tilde(dir).to_string();
            let path = Path::new(&absolute_dir);
            if path.exists() {
                music_dir.push((*dir).to_string());
            }
        }
        Self {
            music_dir,
            music_dir_from_cli: None,
            loop_mode: Loop::Queue,
            volume: 70,
            speed: 10,
            add_playlist_front: false,
            gapless: true,
            enable_exit_confirmation: true,
            playlist_display_symbol: true,
            keys: Keys::default(),
            theme_selected: "default".to_string(),
            style_color_symbol: StyleColorSymbol::default(),
            album_photo_xywh: Xywh::default(),
            playlist_select_random_track_quantity: 20,
            playlist_select_random_album_quantity: 1,
            disable_album_art_from_cli: false,
            disable_discord_rpc_from_cli: false,
            max_depth_cli: 4,
        }
    }
}

impl Settings {
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
    let mut path = dirs::config_dir().ok_or_else(|| anyhow!("failed to find os config dir."))?;
    path.push("termusic");

    if !path.exists() {
        fs::create_dir_all(&path)?;
    }
    Ok(path)
}
