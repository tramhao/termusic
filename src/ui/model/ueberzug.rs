use super::Model;
use crate::ui::components::Xywh;
use anyhow::Result;
use log::error;
use std::io::Write;
use std::process::Stdio;

impl Model {
    pub fn draw_cover_ueberzug(&self, url: &str, draw_xywh: &Xywh) {
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

    pub fn clear_cover_ueberzug(&self) {
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
}
