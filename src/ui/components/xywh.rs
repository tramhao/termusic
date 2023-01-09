use crate::track::MediaType;
/**
 * MIT License
 *
 * tui-realm - Copyright (C) 2021 Christian Visintin
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
// -- modules
// mod clock;
// mod counter;
// -- export
// pub use clock::Clock;
// pub use counter::{Digit, Letter};
use crate::ui::{model::ViuerSupported, Id, IdConfigEditor, IdTagEditor, Model};
use anyhow::{anyhow, bail, Result};
use image::io::Reader as ImageReader;
use image::DynamicImage;
use lofty::Picture;
use serde::{Deserialize, Serialize};
use std::io::Write;

#[derive(Clone, Deserialize, Serialize)]
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

#[derive(Clone, Deserialize, Serialize)]
pub enum Alignment {
    BottomRight,
    BottomLeft,
    TopRight,
    TopLeft,
}

impl Alignment {
    const fn x(&self, absolute_x: u32, width: u32, term_width: u32) -> u32 {
        match self {
            Self::BottomRight | Self::TopRight => {
                Self::get_size_substract(absolute_x, width, term_width)
            }
            Self::BottomLeft | Self::TopLeft => absolute_x,
        }
    }
    const fn y(&self, absolute_y: u32, height: u32, term_height: u32) -> u32 {
        match self {
            Self::BottomRight | Self::BottomLeft => {
                Self::get_size_substract(absolute_y, height / 2, term_height)
            }
            Self::TopRight | Self::TopLeft => absolute_y,
        }
    }

    const fn get_size_substract(absolute_size: u32, size: u32, term_size: u32) -> u32 {
        if absolute_size > size {
            return absolute_size - size;
        }
        term_size - size
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
    fn update_size(&self, image: &DynamicImage) -> Result<Self> {
        let (term_width, term_height) = Self::get_terminal_size_u32();
        let (x, y, width, height) = self.calculate_xywh(term_width, term_height, image)?;

        let (x, y) = Self::safe_guard_xy(x, y, term_width, term_height, width, height);
        Ok(Self {
            x_between_1_100: self.x_between_1_100,
            y_between_1_100: self.y_between_1_100,
            width_between_1_100: self.width_between_1_100,
            x,
            y,
            width,
            height,
            align: self.align.clone(),
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
            self.align.x(absolute_x, width, term_width),
            self.align.y(absolute_y, height, term_height),
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
        // let width = width + width % 2;
        let height = (width * pic_height_orig) / (pic_width_orig);
        Self::safe_guard_width_or_height(height, term_height * 2)
    }

    const fn safe_guard_xy(
        x: u32,
        y: u32,
        term_width: u32,
        term_height: u32,
        width: u32,
        height: u32,
    ) -> (u32, u32) {
        let (max_x, min_x, max_y, min_y) = Self::get_limits(term_width, term_height, width, height);
        let (x, y) = (
            Self::safe_guard_max(x, max_x),
            Self::safe_guard_max(y, max_y),
        );
        let (x, y) = (
            Self::safe_guard_min(x, min_x),
            Self::safe_guard_min(y, min_y),
        );
        (x, y)
    }
    const fn safe_guard_max(position: u32, max: u32) -> u32 {
        if position > max {
            return max;
        }
        position
    }
    const fn safe_guard_min(position: u32, min: u32) -> u32 {
        if position < min {
            return min;
        }
        position
    }

    const fn get_limits(
        term_width: u32,
        term_height: u32,
        width: u32,
        height: u32,
    ) -> (u32, u32, u32, u32) {
        let max_x = term_width - width - 1;
        let min_x = 1;
        let max_y = term_height - height / 2 - 1;
        let min_y = 1;
        (max_x, min_x, max_y, min_y)
    }

    pub fn get_terminal_size_u32() -> (u32, u32) {
        let (term_width, term_height) = viuer::terminal_size();
        (u32::from(term_width), u32::from(term_height))
    }
}
impl Model {
    fn should_not_show_photo(&self) -> bool {
        if self.app.mounted(&Id::TagEditor(IdTagEditor::InputTitle)) {
            return true;
        }

        if self.app.mounted(&Id::YoutubeSearchTablePopup) {
            return true;
        }

        if self.app.mounted(&Id::GeneralSearchInput) {
            return true;
        }

        if self.player.playlist.is_stopped() {
            return true;
        }

        if self.app.mounted(&Id::ConfigEditor(IdConfigEditor::Header)) {
            return true;
        }

        false
    }

    #[allow(clippy::cast_possible_truncation)]
    pub fn update_photo(&mut self) -> Result<()> {
        #[cfg(feature = "cover")]
        if self.config.disable_album_art_from_cli {
            return Ok(());
        }
        self.clear_photo()?;

        if self.should_not_show_photo() {
            return Ok(());
        }
        let track = match self.player.playlist.current_track() {
            Some(track) => track,
            None => return Ok(()),
        };

        match track.media_type {
            Some(MediaType::Music) => {
                // just show the first photo
                if let Some(picture) = track.picture() {
                    if let Ok(image) = image::load_from_memory(picture.data()) {
                        self.show_image(&image)?;
                        return Ok(());
                    }
                }

                if let Some(album_photo) = track.album_photo() {
                    let img = ImageReader::open(album_photo)?.decode()?;
                    self.show_image(&img)?;
                }
            }
            Some(MediaType::Podcast) => {
                let mut url = String::new();
                if let Some(episode_photo_url) = track.album_photo() {
                    url = episode_photo_url.to_string();
                } else if let Some(pod_photo_url) =
                    self.podcast_get_album_photo_by_url(track.file().unwrap_or(""))
                {
                    url = pod_photo_url;
                }

                if url.is_empty() {
                    return Ok(());
                }
                if let Ok(result) = ureq::get(&url).call() {
                    let picture = Picture::from_reader(&mut result.into_reader())?;
                    if let Ok(image) = image::load_from_memory(picture.data()) {
                        self.show_image(&image)?;
                    }
                }
            }
            None => {}
        }

        Ok(())
    }

    #[allow(clippy::cast_possible_truncation)]
    fn show_image(&mut self, img: &DynamicImage) -> Result<()> {
        match self.config.album_photo_xywh.update_size(img) {
            Err(e) => self.mount_error_popup(e.to_string()),
            Ok(xywh) => {
                match self.viuer_supported {
                    ViuerSupported::Kitty | ViuerSupported::ITerm => {
                        let config = viuer::Config {
                            transparent: true,
                            absolute_offset: true,
                            x: xywh.x as u16,
                            y: xywh.y as i16,
                            width: Some(xywh.width),
                            height: None,
                            ..viuer::Config::default()
                        };
                        viuer::print(img, &config)
                            .map_err(|e| anyhow!("viuer print error: {}", e))?;
                    }
                    ViuerSupported::NotSupported => {
                        #[cfg(feature = "cover")]
                        {
                            let mut cache_file =
                                dirs::cache_dir().unwrap_or_else(std::env::temp_dir);
                            cache_file.push("termusic");
                            if !cache_file.exists() {
                                std::fs::create_dir_all(&cache_file)?;
                            }
                            cache_file.push("termusic_cover.jpg");
                            img.save(cache_file.clone())?;
                            if !cache_file.exists() {
                                bail!("cover file is not saved correctly");
                            }
                            if let Some(file) = cache_file.as_path().to_str() {
                                self.ueberzug_instance.draw_cover_ueberzug(file, &xywh)?;
                            }
                        }
                    }
                };
            }
        }
        Ok(())
    }

    fn clear_photo(&mut self) -> Result<()> {
        match self.viuer_supported {
            ViuerSupported::Kitty | ViuerSupported::ITerm => {
                self.clear_image_viuer_kitty()
                    .map_err(|e| anyhow!("Clear album photo error: {}", e))?;
            }
            // ViuerSupported::ITerm => {
            //     // FIXME: This is a total clear of the whole screen. I haven't found a better way to clear
            //     // iterm images
            //     self.terminal.raw_mut().clear()?;
            // }
            ViuerSupported::NotSupported => {
                #[cfg(feature = "cover")]
                self.ueberzug_instance.clear_cover_ueberzug()?;
            }
        }
        Ok(())
    }
    fn clear_image_viuer_kitty(&mut self) -> Result<()> {
        write!(self.terminal.raw_mut().backend_mut(), "\x1b_Ga=d\x1b\\")?;
        // write!(self.terminal.raw_mut().backend_mut(), "\x1b_Ga=d\x1b\\")?;
        self.terminal.raw_mut().backend_mut().flush()?;
        Ok(())
    }
}
