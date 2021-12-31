//! ## Components
//!
//! demo example components

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
use crate::ui::{Id, IdColorEditor, IdTagEditor, Model};
use anyhow::{anyhow, bail, Result};
use image::DynamicImage;
use serde::{Deserialize, Serialize};
use std::io::Write;
#[cfg(feature = "cover")]
use std::path::PathBuf;

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
    align: Alignment,
}

#[derive(Clone, Deserialize, Serialize, Debug)]
enum Alignment {
    BottomRight,
    BottomLeft,
    TopRight,
    TopLeft,
}

impl Alignment {
    const fn x(&self, absolute_x: u32, width: u32, term_width: u32) -> u32 {
        match self {
            Alignment::BottomRight | Alignment::TopRight => {
                Self::get_size_substract(absolute_x, width, term_width)
            }
            Alignment::BottomLeft | Alignment::TopLeft => absolute_x,
        }
    }
    const fn y(&self, absolute_y: u32, height: u32, term_height: u32) -> u32 {
        match self {
            Alignment::BottomRight | Alignment::BottomLeft => {
                Self::get_size_substract(absolute_y, height / 2, term_height)
            }
            Alignment::TopRight | Alignment::TopLeft => absolute_y,
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

    fn get_terminal_size_u32() -> (u32, u32) {
        let (term_width, term_height) = viuer::terminal_size();
        (u32::from(term_width), u32::from(term_height))
    }
}
impl Model {
    // update picture of album
    #[allow(clippy::cast_possible_truncation)]
    pub fn update_photo(&mut self) -> Result<()> {
        self.clear_photo()?;

        if self.app.mounted(&Id::TagEditor(IdTagEditor::TEInputTitle))
            | self
                .app
                .mounted(&Id::ColorEditor(IdColorEditor::ThemeSelect))
        {
            return Ok(());
        }
        let song = match &self.current_song {
            Some(song) => song,
            None => return Ok(()),
        };

        // just show the first photo
        if let Some(picture) = song.picture() {
            if let Ok(image) = image::load_from_memory(picture.data()) {
                // Set desired image dimensions
                match self.config.album_photo_xywh.update_size(&image) {
                    Err(e) => self.mount_error_popup(&e.to_string()),
                    Ok(xywh) => {
                        // debug album photo position
                        // eprintln!("{:?}", self.config.album_photo_xywh);
                        // eprintln!("{:?}", xywh);
                        if self.viuer_supported {
                            let config = viuer::Config {
                                transparent: true,
                                absolute_offset: true,
                                x: xywh.x as u16,
                                y: xywh.y as i16,
                                width: Some(xywh.width),
                                height: None,
                                ..viuer::Config::default()
                            };
                            viuer::print(&image, &config)
                                .map_err(|e| anyhow!("viuer print error: {}", e))?;

                            return Ok(());
                        };
                        #[cfg(feature = "cover")]
                        {
                            let mut cache_file =
                                dirs_next::cache_dir().unwrap_or_else(|| PathBuf::from("/tmp"));
                            cache_file.push("termusic_cover.jpg");
                            image.save(cache_file.clone())?;
                            // image.save(Path::new("/tmp/termusic_cover.jpg"))?;
                            if let Some(file) = cache_file.as_path().to_str() {
                                self.ueberzug_instance.draw_cover_ueberzug(file, &xywh)?;
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn clear_photo(&mut self) -> Result<()> {
        // clear all previous image
        // if (viuer::KittySupport::Local == viuer::get_kitty_support()) || viuer::is_iterm_supported()
        // {

        if self.viuer_supported {
            self.clear_image_viuer()
                .map_err(|e| anyhow!("Clear album photo error: {}", e))?;
            return Ok(());
        }
        #[cfg(feature = "cover")]
        self.ueberzug_instance.clear_cover_ueberzug()?;
        Ok(())
    }
    fn clear_image_viuer(&mut self) -> Result<()> {
        write!(self.terminal.raw_mut().backend_mut(), "\x1b_Ga=d\x1b\\")?;
        self.terminal.raw_mut().backend_mut().flush()?;
        Ok(())
    }
}
