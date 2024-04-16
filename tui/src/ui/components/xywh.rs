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
use anyhow::{Context, Result};
use bytes::Buf;
use image::io::Reader as ImageReader;
use image::DynamicImage;
use lofty::Picture;
#[cfg(any(
    feature = "cover-viuer-iterm",
    feature = "cover-viuer-kitty",
    feature = "cover-viuer-sixel"
))]
use std::io::Write;
use termusiclib::track::MediaType;
use termusiclib::types::{DLMsg, Id, IdConfigEditor, IdTagEditor, ImageWrapper, Msg};
use tokio::runtime::Handle;

impl Model {
    pub fn xywh_move_left(&mut self) {
        self.config.write().album_photo_xywh.move_left();
        self.update_photo().ok();
    }

    pub fn xywh_move_right(&mut self) {
        self.config.write().album_photo_xywh.move_right();
        self.update_photo().ok();
    }

    pub fn xywh_move_up(&mut self) {
        self.config.write().album_photo_xywh.move_up();
        self.update_photo().ok();
    }

    pub fn xywh_move_down(&mut self) {
        self.config.write().album_photo_xywh.move_down();
        self.update_photo().ok();
    }
    pub fn xywh_zoom_in(&mut self) {
        self.config.write().album_photo_xywh.zoom_in();
        self.update_photo().ok();
    }
    pub fn xywh_zoom_out(&mut self) {
        self.config.write().album_photo_xywh.zoom_out();
        self.update_photo().ok();
    }
    pub fn xywh_toggle_hide(&mut self) {
        self.clear_photo().ok();
        let mut config = self.config.write();
        config.disable_album_art_from_cli = !config.disable_album_art_from_cli;
        drop(config);
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

        if self.playlist.is_stopped() {
            return true;
        }

        if self.app.mounted(&Id::ConfigEditor(IdConfigEditor::Header)) {
            return true;
        }

        false
    }

    /// Get and show a image for the current playing media
    ///
    /// Requires that the current thread has a entered runtime
    #[allow(clippy::cast_possible_truncation)]
    pub fn update_photo(&mut self) -> Result<()> {
        if self.config.read().disable_album_art_from_cli {
            return Ok(());
        }
        self.clear_photo()?;

        if self.should_not_show_photo() {
            return Ok(());
        }
        let Some(track) = self.playlist.current_track() else {
            return Ok(());
        };

        match track.media_type {
            MediaType::Music => {
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
            MediaType::Podcast => {
                let url = {
                    if let Some(episode_photo_url) = track.album_photo() {
                        episode_photo_url.to_string()
                    } else if let Some(pod_photo_url) = track
                        .file()
                        .and_then(|file| self.podcast_get_album_photo_by_url(file))
                    {
                        pod_photo_url
                    } else {
                        return Ok(());
                    }
                };

                if url.is_empty() {
                    return Ok(());
                }
                let tx = self.tx_to_main.clone();

                Handle::current().spawn(async move {
                    match reqwest::get(&url).await {
                        Ok(result) => {
                            if result.status() != reqwest::StatusCode::OK {
                                tx.send(Msg::Download(DLMsg::FetchPhotoErr(format!(
                                    "Error non-OK Status code: {}",
                                    result.status()
                                ))))
                                .ok();
                                return;
                            }

                            let mut reader = {
                                let bytes = match result.bytes().await {
                                    Ok(v) => v,
                                    Err(err) => {
                                        tx.send(Msg::Download(DLMsg::FetchPhotoErr(format!(
                                            "Error in reqest::Response::bytes: {err}"
                                        ))))
                                        .ok();
                                        return;
                                    }
                                };

                                bytes.reader()
                            };

                            let picture = match Picture::from_reader(&mut reader) {
                                Ok(v) => v,
                                Err(e) => {
                                    tx.send(Msg::Download(DLMsg::FetchPhotoErr(format!(
                                        "Error in picture from_reader: {e}"
                                    ))))
                                    .ok();
                                    return;
                                }
                            };

                            match image::load_from_memory(picture.data()) {
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
                            }
                        }
                        Err(e) => tx
                            .send(Msg::Download(DLMsg::FetchPhotoErr(format!(
                                "Error in ureq get: {e}"
                            ))))
                            .ok(),
                    };
                });
            }
            MediaType::LiveRadio => {}
        }

        Ok(())
    }

    #[allow(clippy::cast_possible_truncation, clippy::unnecessary_wraps)]
    pub fn show_image(&mut self, img: &DynamicImage) -> Result<()> {
        #[allow(unused_variables)]
        let xywh = self.config.read().album_photo_xywh.update_size(img)?;

        // error!("{:?}", self.viuer_supported);
        match self.viuer_supported {
            ViuerSupported::NotSupported => {
                #[cfg(all(feature = "cover-ueberzug", not(target_os = "windows")))]
                {
                    let mut cache_file = dirs::cache_dir().unwrap_or_else(std::env::temp_dir);
                    cache_file.push("termusic");
                    if !cache_file.exists() {
                        std::fs::create_dir_all(&cache_file)?;
                    }
                    cache_file.push("termusic_cover.jpg");
                    img.save(&cache_file)?;
                    if !cache_file.exists() {
                        anyhow::bail!("cover file is not saved correctly");
                    }
                    if let Some(file) = cache_file.as_path().to_str() {
                        self.ueberzug_instance
                            .draw_cover_ueberzug(file, &xywh, false)?;
                    }
                }
            }
            #[cfg(any(
                feature = "cover-viuer-iterm",
                feature = "cover-viuer-kitty",
                feature = "cover-viuer-sixel"
            ))]
            _ => {
                let config = viuer::Config {
                    transparent: true,
                    absolute_offset: true,
                    x: xywh.x as u16,
                    y: xywh.y as i16,
                    width: Some(xywh.width),
                    height: None,
                    ..viuer::Config::default()
                };
                viuer::print(img, &config).context("viuer::print")?;
            }
        };

        Ok(())
    }

    #[allow(clippy::unnecessary_wraps)]
    fn clear_photo(&mut self) -> Result<()> {
        match self.viuer_supported {
            #[cfg(feature = "cover-viuer-kitty")]
            ViuerSupported::Kitty => {
                self.clear_image_viuer_kitty()
                    .context("clear_photo kitty")?;
                Self::remove_temp_files()?;
            }
            #[cfg(feature = "cover-viuer-iterm")]
            ViuerSupported::ITerm => {
                self.clear_image_viuer_kitty()
                    .context("clear_photo iterm")?;
                Self::remove_temp_files()?;
            }
            #[cfg(feature = "cover-viuer-sixel")]
            ViuerSupported::Sixel => {
                self.clear_image_viuer_kitty()
                    .context("clear_photo sixel")?;
                // sixel does not use temp-files, so no cleaning necessary
            }
            ViuerSupported::NotSupported => {
                #[cfg(all(feature = "cover-ueberzug", not(target_os = "windows")))]
                self.ueberzug_instance.clear_cover_ueberzug()?;
            }
        }
        Ok(())
    }

    #[cfg(any(
        feature = "cover-viuer-iterm",
        feature = "cover-viuer-kitty",
        feature = "cover-viuer-sixel"
    ))]
    fn clear_image_viuer_kitty(&mut self) -> Result<()> {
        write!(self.terminal.raw_mut().backend_mut(), "\x1b_Ga=d\x1b\\")?;
        self.terminal.raw_mut().backend_mut().flush()?;
        Ok(())
    }

    #[cfg(any(feature = "cover-viuer-iterm", feature = "cover-viuer-kitty"))]
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
