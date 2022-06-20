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
pub use key::{BindingForEvent, Keys};
use serde::{Deserialize, Serialize};
use std::fs::{self, read_to_string};
use std::path::PathBuf;
pub use theme::{load_alacritty, ColorTermusic, StyleColorSymbol};

pub const MUSIC_DIR: &str = "~/Music";

#[derive(Clone, Deserialize, Serialize)]
pub struct Termusic {
    pub music_dir: String,
    #[serde(skip_serializing)]
    pub music_dir_from_cli: Option<String>,
    pub loop_mode: Loop,
    pub volume: i32,
    pub speed: f32,
    pub add_playlist_front: bool,
    pub gapless: bool,
    pub disable_exit_confirmation: bool,
    pub playlist_display_symbol: bool,
    pub theme_selected: String,
    pub album_photo_xywh: Xywh,
    pub style_color_symbol: StyleColorSymbol,
    pub keys: Keys,
}

impl Default for Termusic {
    fn default() -> Self {
        Self {
            music_dir: MUSIC_DIR.to_string(),
            music_dir_from_cli: None,
            loop_mode: Loop::Queue,
            volume: 70,
            speed: 1.0,
            add_playlist_front: false,
            gapless: true,
            disable_exit_confirmation: false,
            playlist_display_symbol: true,
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
    let mut path = dirs::config_dir().ok_or_else(|| anyhow!("failed to find os config dir."))?;
    path.push("termusic");

    if !path.exists() {
        fs::create_dir_all(&path)?;
    }
    Ok(path)
}
