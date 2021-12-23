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

pub const MUSIC_DIR: &str = "~/Music";

#[derive(Clone, Deserialize, Serialize)]
pub struct Termusic {
    pub music_dir: String,
    #[serde(skip_serializing)]
    pub music_dir_from_cli: Option<String>,
    pub loop_mode: Loop,
    pub volume: i32,
    pub add_playlist_front: bool,
    pub theme_selected: String,
    pub style_color_symbol: StyleColorSymbol,
    pub album_photo_xywh: Xywh,
}
impl Default for Termusic {
    fn default() -> Self {
        Self {
            music_dir: MUSIC_DIR.to_string(),
            music_dir_from_cli: None,
            loop_mode: Loop::Queue,
            volume: 70,
            add_playlist_front: false,
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
