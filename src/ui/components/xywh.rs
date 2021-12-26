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
use crate::ui::Model;
use anyhow::{anyhow, Result};
use image::DynamicImage;
use serde::{Deserialize, Serialize};
use std::io::Write;
#[cfg(feature = "cover")]
use std::path::PathBuf;

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct Xywh {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    #[serde(skip)]
    pub height: u32,
    #[serde(skip, default = "Xywh::get_terminal_size_u32_w")]
    pub term_w: u32,
    #[serde(skip, default = "Xywh::get_terminal_size_u32_h")]
    pub term_h: u32,
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
            x,
            y,
            width,
            height,
            term_w: term_width,
            term_h: term_height,
        }
    }
}
impl Xywh {
    fn update_size(&self, image: &DynamicImage) -> Self {
        let (term_width, term_height) = Self::get_terminal_size_u32();
        let (pic_width_orig, pic_height_orig) = image::GenericImageView::dimensions(image);
        let (x, y, width, height) =
            self.calculate_xywh(term_width, term_height, pic_width_orig, pic_height_orig);
        let (x, y) = Self::safe_guard_xy(x, y, term_width, term_height, width, height);
        Self {
            x,
            y,
            width,
            height,
            term_w: self.term_w,
            term_h: self.term_h,
        }
    }
    const fn calculate_xywh(
        &self,
        term_width: u32,
        term_height: u32,
        pic_width_orig: u32,
        pic_height_orig: u32,
    ) -> (u32, u32, u32, u32) {
        let width = self.width * term_width / self.term_w;
        // left for debug
        // eprintln!("{},{},{},{}", self.width, width, self.term_w, term_width);
        let height = (width * pic_height_orig) / (pic_width_orig);
        let x = self.x * term_width / self.term_w - width;
        let y = self.y * term_height / self.term_h - height / 2;
        (x, y, width, height)
    }

    // #[allow(unused)]
    const fn safe_guard_xy(
        x: u32,
        y: u32,
        term_width: u32,
        term_height: u32,
        width: u32,
        height: u32,
    ) -> (u32, u32) {
        let (maximum_x, minimum_x, maximum_y, minimum_y) =
            Self::get_limits(term_width, term_height, width, height);
        let x = if x > maximum_x { maximum_x } else { x };
        let x = if x < minimum_x { minimum_x } else { x };
        let y = if y > maximum_y { maximum_y } else { y };
        let y = if y < minimum_y { minimum_y } else { y };
        (x, y)
    }
    const fn get_limits(
        term_width: u32,
        term_height: u32,
        width: u32,
        height: u32,
    ) -> (u32, u32, u32, u32) {
        let maximum_x = term_width - width - 1;
        let minimum_x = width + 1;
        let maximum_y = term_height - height / 2 - 1;
        let minimum_y = height / 2 + 1;
        (maximum_x, minimum_x, maximum_y, minimum_y)
    }

    fn get_terminal_size_u32() -> (u32, u32) {
        let (term_width, term_height) = viuer::terminal_size();
        (u32::from(term_width), u32::from(term_height))
    }
    fn get_terminal_size_u32_w() -> u32 {
        let (term_width, _term_height) = viuer::terminal_size();
        u32::from(term_width)
    }
    fn get_terminal_size_u32_h() -> u32 {
        let (_term_width, term_height) = viuer::terminal_size();
        u32::from(term_height)
    }
}
impl Model {
    // update picture of album
    #[allow(clippy::cast_possible_truncation)]
    pub fn update_photo(&mut self) -> Result<()> {
        self.clear_photo()?;

        let song = match &self.current_song {
            Some(song) => song,
            None => return Ok(()),
            // None => bail!("no current song"),
        };

        // just show the first photo
        if let Some(picture) = song.picture() {
            if let Ok(image) = image::load_from_memory(&picture.data) {
                // Set desired image dimensions
                // let ratio = f64::from(orig_height) / f64::from(orig_width);
                let xywh = self.config.album_photo_xywh.update_size(&image);
                // debug album photo position
                // eprintln!("{:?}", self.config.album_photo_xywh);
                // eprintln!("{:?}", xywh);
                if self.viuer_supported {
                    let config = viuer::Config {
                        transparent: true,
                        absolute_offset: true,
                        x: xywh.x as u16,
                        y: xywh.y as i16,
                        // x: term_width / 3 - width - 1,
                        // y: (term_height - height / 2) as i16 - 2,
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
        Ok(())
    }

    pub fn clear_photo(&mut self) -> Result<()> {
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
