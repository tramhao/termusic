use super::Model;
#[cfg(feature = "cover")]
use log::error;
// #[cfg(feature = "cover")]
use std::io::Write;
#[cfg(feature = "cover")]
use std::path::Path;
#[cfg(feature = "cover")]
use std::process::Stdio;

use anyhow::Result;

#[allow(dead_code)]
pub struct Xywh {
    x: u16,
    y: u16,
    width: u32,
    height: u32,
}

impl Model {
    #[cfg(feature = "cover")]
    fn draw_cover_ueberzug(&self, url: &str, draw_xywh: &Xywh) {
        if draw_xywh.width <= 1 || draw_xywh.height <= 1 {
            return;
        }

        // Ueberzug takes an area given in chars and fits the image to
        // that area (from the top left).

        // Round up since the bottom might have empty space within
        // the designated box

        let cmd = format!("{{\"action\":\"add\",\"scaler\":\"fit_contain\",\"identifier\":\"cover\",\"x\":{},\"y\":{},\"width\":{},\"height\":{},\"path\":\"{}\"}}\n",
            draw_xywh.x, draw_xywh.y-1,
            draw_xywh.width, draw_xywh.height,
            // path.to_str().unwrap()
            url,
        );

        if let Err(e) = self.run_ueberzug_cmd(&cmd) {
            error!("Failed to run Ueberzug: {}", e);
        }
    }

    #[cfg(feature = "cover")]
    fn clear_cover_ueberzug(&self) {
        let cmd = "{\"action\": \"remove\", \"identifier\": \"cover\"}\n";
        if let Err(e) = self.run_ueberzug_cmd(cmd) {
            error!("Failed to run Ueberzug: {}", e);
        }
    }

    #[cfg(feature = "cover")]
    fn run_ueberzug_cmd(&self, cmd: &str) -> Result<(), std::io::Error> {
        let mut ueberzug = self.ueberzug.write().unwrap();

        if ueberzug.is_none() {
            *ueberzug = Some(
                std::process::Command::new("ueberzug")
                    .args(&["layer", "--silent"])
                    .stdin(Stdio::piped())
                    .stdout(Stdio::piped())
                    .spawn()?,
            );
        }

        let stdin = (*ueberzug).as_mut().unwrap().stdin.as_mut().unwrap();
        stdin.write_all(cmd.as_bytes())?;

        Ok(())
    }
    // update picture of album
    #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
    pub fn update_photo(&mut self) {
        self.clear_photo();

        let song = match &self.current_song {
            Some(song) => song,
            None => return,
        };

        // just show the first photo
        if let Some(picture) = song.picture() {
            if let Ok(image) = image::load_from_memory(&picture.data) {
                let (term_width, term_height) = viuer::terminal_size();
                // Set desired image dimensions
                let (orig_width, orig_height) = image::GenericImageView::dimensions(&image);
                // let ratio = f64::from(orig_height) / f64::from(orig_width);
                let width = 20_u16;
                let height = (width * orig_height as u16).checked_div(orig_width as u16);
                if let Some(height) = height {
                    let xywh = Xywh {
                        x: term_width - width - 1,
                        y: (term_height - height / 2 - 8) - 1,
                        width: u32::from(width),
                        height: u32::from(height),
                    };
                    // if terminal is not kitty or item, show photo with ueberzug
                    if (viuer::KittySupport::Local != viuer::get_kitty_support())
                        && !viuer::is_iterm_supported()
                    {
                        #[cfg(feature = "cover")]
                        image.save(Path::new("/tmp/termusic_cover.jpg")).ok();
                        #[cfg(feature = "cover")]
                        self.draw_cover_ueberzug("/tmp/termusic_cover.jpg", &xywh);
                        return;
                    };
                    let config = viuer::Config {
                        transparent: true,
                        absolute_offset: true,
                        x: xywh.x,
                        y: xywh.y as i16,
                        // x: term_width / 3 - width - 1,
                        // y: (term_height - height / 2) as i16 - 2,
                        width: Some(xywh.width),
                        height: None,
                        ..viuer::Config::default()
                    };
                    viuer::print(&image, &config).ok();
                }
            }
        }
    }

    pub fn clear_photo(&mut self) {
        // clear all previous image
        if (viuer::KittySupport::Local != viuer::get_kitty_support())
            && !viuer::is_iterm_supported()
        {
            #[cfg(feature = "cover")]
            self.clear_cover_ueberzug();
        } else if let Err(e) = self.clear_image_viuer() {
            self.mount_error_popup(format!("Clear album photo error: {}", e).as_str());
        }
    }

    pub fn clear_image_viuer(&mut self) -> Result<()> {
        // write!(terminal.raw_mut().backend_mut(), "\x1b_Ga=d\x1b\\").ok();
        write!(self.terminal.raw_mut().backend_mut(), "\x1b_Ga=d\x1b\\")?;
        self.terminal.raw_mut().backend_mut().flush()?;
        Ok(())
    }
}
