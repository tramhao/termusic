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
use crate::ui::model::{Model, ViuerSupported};
#[cfg(feature = "cover")]
use anyhow::bail;
use anyhow::{anyhow, Result};
use image::io::Reader as ImageReader;
use image::DynamicImage;
use lofty::Picture;
use std::io::Write;
use termusiclib::types::{DLMsg, Id, IdConfigEditor, IdTagEditor, ImageWrapper, Msg};

impl Model {
    pub fn xywh_move_left(&mut self) {
        self.config.album_photo_xywh.move_left();
        self.update_photo().ok();
    }

    pub fn xywh_move_right(&mut self) {
        self.config.album_photo_xywh.move_right();
        self.update_photo().ok();
    }

    pub fn xywh_move_up(&mut self) {
        self.config.album_photo_xywh.move_up();
        self.update_photo().ok();
    }

    pub fn xywh_move_down(&mut self) {
        self.config.album_photo_xywh.move_down();
        self.update_photo().ok();
    }
    pub fn xywh_zoom_in(&mut self) {
        self.config.album_photo_xywh.zoom_in();
        self.update_photo().ok();
    }
    pub fn xywh_zoom_out(&mut self) {
        self.config.album_photo_xywh.zoom_out();
        self.update_photo().ok();
    }
    pub fn xywh_toggle_hide(&mut self) {
        self.clear_photo().ok();
        self.config.disable_album_art_from_cli = !self.config.disable_album_art_from_cli;
        self.update_photo().ok();
    }
    fn should_not_show_photo(&self) -> bool {
        if self.app.mounted(&Id::HelpPopup) {
            return true;
        }
        if self.app.mounted(&Id::PodcastSearchTablePopup) {
            return true;
        }

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
        let Some(track) = self.player.playlist.current_track() else {
            return Ok(())
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
                let tx = self.tx_to_main.clone();
                std::thread::spawn(move || {
                    match ureq::get(&url).call() {
                        Ok(result) => match Picture::from_reader(&mut result.into_reader()) {
                            Ok(picture) => match image::load_from_memory(picture.data()) {
                                Ok(image) => {
                                    let image_wrapper = ImageWrapper { data: image };
                                    tx.send(Msg::Download(DLMsg::FetchPhotoSuccess(image_wrapper)))
                                        .ok()
                                }
                                Err(e) => tx
                                    .send(Msg::Download(DLMsg::FetchPhotoErr(format!(
                                        "Error in load_from_memory: {e}"
                                    ))))
                                    .ok(),
                            },
                            Err(e) => tx
                                .send(Msg::Download(DLMsg::FetchPhotoErr(format!(
                                    "Error in picture from_reader: {e}"
                                ))))
                                .ok(),
                        },
                        Err(e) => tx
                            .send(Msg::Download(DLMsg::FetchPhotoErr(format!(
                                "Error in ureq get: {e}"
                            ))))
                            .ok(),
                    }

                    // if let Ok(result) = ureq::get(&url).call() {
                    //     let picture = Picture::from_reader(&mut result.into_reader())?;
                    //     if let Ok(image) = image::load_from_memory(picture.data()) {
                    //         self.show_image(&image)?;
                    //     }
                    // }
                });
            }
            None => {}
        }

        Ok(())
    }

    #[allow(clippy::cast_possible_truncation)]
    pub fn show_image(&mut self, img: &DynamicImage) -> Result<()> {
        match self.config.album_photo_xywh.update_size(img) {
            Err(e) => self.mount_error_popup(e.to_string()),
            Ok(xywh) => {
                // eprintln!("{:?}", self.viuer_supported);
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
                                self.ueberzug_instance
                                    .draw_cover_ueberzug(file, &xywh, false)?;
                            }
                        }
                    } // ViuerSupported::Sixel => {
                      //     #[cfg(feature = "cover")]
                      //     {
                      //         let mut cache_file =
                      //             dirs::cache_dir().unwrap_or_else(std::env::temp_dir);
                      //         cache_file.push("termusic");
                      //         if !cache_file.exists() {
                      //             std::fs::create_dir_all(&cache_file)?;
                      //         }
                      //         cache_file.push("termusic_cover.jpg");
                      //         img.save(cache_file.clone())?;
                      //         if !cache_file.exists() {
                      //             bail!("cover file is not saved correctly");
                      //         }
                      //         if let Some(file) = cache_file.as_path().to_str() {
                      //             self.ueberzug_instance
                      //                 .draw_cover_ueberzug(file, &xywh, true)?;
                      //         }
                      //     }
                      // }
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
                Self::remove_temp_files()?;
            }
            // ViuerSupported::Sixel => {
            //     self.clear_image_viuer_kitty()
            //         .map_err(|e| anyhow!("Clear album photo error: {}", e))?;
            // }
            // ViuerSupported::ITerm => {
            //     // FIXME: This is a total clear of the whole screen. I haven't found a better way to clear
            //     // iterm images
            //     self.terminal.raw_mut().clear()?;
            // }
            // ViuerSupported::NotSupported | ViuerSupported::Sixel => {
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

    fn remove_temp_files() -> Result<()> {
        // Clean up temp files created by `viuer`'s kitty printer to avoid
        // possible freeze because of too many temp files in the temp folder.
        // Context: https://github.com/aome510/spotify-player/issues/148
        let tmp_dir = std::env::temp_dir();
        for path in (std::fs::read_dir(tmp_dir)?).flatten() {
            let path = path.path();
            if path.display().to_string().contains(".tmp.viuer") {
                std::fs::remove_file(path)?;
            }
        }

        Ok(())
    }
}
