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

use crate::utils::get_app_config_path;
use anyhow::{bail, Result};
use figment::{
    providers::{Format, Serialized, Toml},
    Figment,
};
use image::DynamicImage;
pub use key::{BindingForEvent, Keys};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::{fs, net::IpAddr};
pub use theme::{Alacritty, AlacrittyColor, ColorTermusic, StyleColorSymbol};

/// The filename of the config
pub const FILE_NAME: &str = "config.toml";

lazy_static! {
    static ref MUSIC_DIR: Vec<PathBuf> = {
        let mut vec = Vec::new();
        let mut path =
            dirs::audio_dir().unwrap_or_else(|| PathBuf::from(shellexpand::path::tilde("~/Music")));
        vec.push(path.clone());
        path.push("mp3");
        vec.push(path);
        vec
    };
    static ref PODCAST_DIR: PathBuf = {
        let mut path =
            dirs::audio_dir().unwrap_or_else(|| PathBuf::from(shellexpand::path::tilde("~/Music")));
        path.push("podcast");
        path
    };
}

#[derive(Clone, Copy, Default, Deserialize, Serialize, Debug)]
pub enum Loop {
    Single,
    #[default]
    Playlist,
    Random,
}

impl Loop {
    pub fn display(self, display_symbol: bool) -> String {
        if display_symbol {
            match self {
                Self::Single => "🔂".to_string(),
                Self::Playlist => "🔁".to_string(),
                Self::Random => "🔀".to_string(),
            }
        } else {
            match self {
                Self::Single => "single".to_string(),
                Self::Playlist => "playlist".to_string(),
                Self::Random => "random".to_string(),
            }
        }
    }
}

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct Xywh {
    pub x_between_1_100: u32,
    pub y_between_1_100: u32,
    pub width_between_1_100: u32,
    #[serde(skip)]
    pub x: u32,
    #[serde(skip)]
    pub y: u32,
    #[serde(skip)]
    pub width: u32,
    #[serde(skip)]
    pub height: u32,
    pub align: Alignment,
}

#[derive(Clone, Copy, Deserialize, Serialize, Debug)]
pub enum Alignment {
    BottomRight,
    BottomLeft,
    TopRight,
    TopLeft,
}

impl Alignment {
    const fn x(self, absolute_x: u32, width: u32) -> u32 {
        match self {
            Self::BottomRight | Self::TopRight => Self::get_size_substract(absolute_x, width),
            Self::BottomLeft | Self::TopLeft => absolute_x,
        }
    }
    const fn y(self, absolute_y: u32, height: u32) -> u32 {
        match self {
            Self::BottomRight | Self::BottomLeft => {
                Self::get_size_substract(absolute_y, height / 2)
            }
            Self::TopRight | Self::TopLeft => absolute_y,
        }
    }

    const fn get_size_substract(absolute_size: u32, size: u32) -> u32 {
        if absolute_size > size {
            return absolute_size - size;
        }
        0
    }
}

impl Default for Xywh {
    #[allow(clippy::cast_lossless, clippy::cast_possible_truncation)]
    fn default() -> Self {
        let width = 20_u32;
        let height = 20_u32;
        let (term_width, term_height) = Self::get_terminal_size_u32();
        let x = term_width - 1;
        let y = term_height - 9;

        Self {
            x_between_1_100: 100,
            y_between_1_100: 77,
            width_between_1_100: width,
            x,
            y,
            width,
            height,
            align: Alignment::BottomRight,
        }
    }
}
impl Xywh {
    pub fn move_left(&mut self) {
        self.x_between_1_100 = self.x_between_1_100.saturating_sub(1);
    }

    pub fn move_right(&mut self) {
        self.x_between_1_100 += 1;
        self.x_between_1_100 = self.x_between_1_100.min(100);
    }

    pub fn move_up(&mut self) {
        self.y_between_1_100 = self.y_between_1_100.saturating_sub(2);
    }

    pub fn move_down(&mut self) {
        self.y_between_1_100 += 2;
        self.y_between_1_100 = self.y_between_1_100.min(100);
    }
    pub fn zoom_in(&mut self) {
        self.width_between_1_100 += 1;
        self.width_between_1_100 = self.width_between_1_100.min(100);
    }

    pub fn zoom_out(&mut self) {
        self.width_between_1_100 = self.width_between_1_100.saturating_sub(1);
    }

    pub fn update_size(&self, image: &DynamicImage) -> Result<Self> {
        let (term_width, term_height) = Self::get_terminal_size_u32();
        let (x, y, width, height) = self.calculate_xywh(term_width, term_height, image)?;
        Ok(Self {
            x_between_1_100: self.x_between_1_100,
            y_between_1_100: self.y_between_1_100,
            width_between_1_100: self.width_between_1_100,
            x,
            y,
            width,
            height,
            align: self.align,
        })
    }
    fn calculate_xywh(
        &self,
        term_width: u32,
        term_height: u32,
        image: &DynamicImage,
    ) -> Result<(u32, u32, u32, u32)> {
        let width = self.get_width(term_width)?;
        let height = Self::get_height(width, term_height, image)?;
        let (absolute_x, absolute_y) = (
            self.x_between_1_100 * term_width / 100,
            self.y_between_1_100 * term_height / 100,
        );
        let (x, y) = (
            self.align.x(absolute_x, width),
            self.align.y(absolute_y, height),
        );
        Ok((x, y, width, height))
    }

    fn get_width(&self, term_width: u32) -> Result<u32> {
        let width = self.width_between_1_100 * term_width / 100;
        Self::safe_guard_width_or_height(width, term_width)
    }

    fn safe_guard_width_or_height(size: u32, size_max: u32) -> Result<u32> {
        if size > size_max {
            bail!("image width is too big, please reduce image width");
        }
        Ok(size)
    }

    fn get_height(width: u32, term_height: u32, image: &DynamicImage) -> Result<u32> {
        let (pic_width_orig, pic_height_orig) = image::GenericImageView::dimensions(image);
        let height = (width * pic_height_orig) / (pic_width_orig);
        Self::safe_guard_width_or_height(height, term_height * 2)
    }

    pub fn get_terminal_size_u32() -> (u32, u32) {
        let (term_width, term_height) = viuer::terminal_size();
        (u32::from(term_width), u32::from(term_height))
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub enum SeekStep {
    Short,
    Long,
    Auto,
}

impl std::fmt::Display for SeekStep {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let seek_step = match self {
            Self::Short => "short(5 seconds)",
            Self::Long => "long(30 seconds)",
            Self::Auto => "auto(depend on audio length)",
        };
        write!(f, "{seek_step}")
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub enum LastPosition {
    Yes,
    No,
    Auto,
}

impl std::fmt::Display for LastPosition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let save_last_position = match self {
            Self::Yes => "yes",
            Self::No => "no",
            Self::Auto => "auto",
        };
        write!(f, "{save_last_position}")
    }
}

#[derive(Clone, Deserialize, Serialize, Debug)]
#[allow(clippy::struct_excessive_bools)]
pub struct Settings {
    pub music_dir: Vec<PathBuf>,
    #[serde(skip)]
    pub music_dir_from_cli: Option<PathBuf>,
    #[serde(skip)]
    pub disable_album_art_from_cli: bool,
    #[serde(skip)]
    pub disable_discord_rpc_from_cli: bool,
    #[serde(skip)]
    pub max_depth_cli: usize,
    pub player_port: u16,
    pub player_interface: IpAddr,
    pub player_loop_mode: Loop,
    pub player_volume: u16,
    pub player_speed: i32,
    pub player_gapless: bool,
    pub podcast_simultanious_download: usize,
    pub podcast_max_retries: usize,
    pub podcast_dir: PathBuf,
    pub player_seek_step: SeekStep,
    pub player_remember_last_played_position: LastPosition,
    pub enable_exit_confirmation: bool,
    pub playlist_display_symbol: bool,
    pub playlist_select_random_track_quantity: u32,
    pub playlist_select_random_album_quantity: u32,
    pub theme_selected: String,
    pub kill_daemon_when_quit: bool,
    pub player_use_mpris: bool,
    pub player_use_discord: bool,
    pub album_photo_xywh: Xywh,
    pub style_color_symbol: StyleColorSymbol,
    pub keys: Keys,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            music_dir: MUSIC_DIR.clone(),
            music_dir_from_cli: None,
            player_loop_mode: Loop::Random,
            player_volume: 70,
            player_speed: 10,
            player_gapless: true,
            player_remember_last_played_position: LastPosition::Auto,
            enable_exit_confirmation: true,
            playlist_display_symbol: true,
            keys: Keys::default(),
            theme_selected: "default".to_string(),
            style_color_symbol: StyleColorSymbol::default(),
            album_photo_xywh: Xywh::default(),
            playlist_select_random_track_quantity: 20,
            playlist_select_random_album_quantity: 5,
            disable_album_art_from_cli: false,
            disable_discord_rpc_from_cli: false,
            max_depth_cli: 4,
            podcast_simultanious_download: 3,
            podcast_dir: PODCAST_DIR.clone(),
            podcast_max_retries: 3,
            player_seek_step: SeekStep::Auto,
            kill_daemon_when_quit: true,
            player_use_mpris: true,
            player_use_discord: true,
            player_port: 50101,
            player_interface: "::1".parse().unwrap(),
        }
    }
}

impl Settings {
    pub fn save(&self) -> Result<()> {
        let mut path = get_app_config_path()?;
        path.push(FILE_NAME);
        let string = toml::to_string(self)?;

        fs::write(path, string)?;

        Ok(())
    }

    pub fn load(&mut self) -> Result<()> {
        let mut path = get_app_config_path()?;
        path.push(FILE_NAME);
        if !path.exists() {
            let config = Self::default();
            config.save()?;
        }

        let figment = Figment::new()
            .merge(Serialized::defaults(Settings::default()))
            .merge(Toml::file(path));
        let config: Settings = figment.extract()?;
        *self = config;
        Ok(())
    }
}
