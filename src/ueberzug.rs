use crate::ui::components::Xywh;
use anyhow::{bail, Result};
use std::io::Write;
use std::process::Child;
use std::process::Stdio;
use std::sync::RwLock;

pub struct UeInstance {
    ueberzug: RwLock<Option<Child>>,
}

impl Default for UeInstance {
    fn default() -> Self {
        Self {
            ueberzug: RwLock::new(None),
        }
    }
}

impl UeInstance {
    pub fn draw_cover_ueberzug(&self, url: &str, draw_xywh: &Xywh) -> Result<()> {
        if draw_xywh.width <= 1 || draw_xywh.height <= 1 {
            return Ok(());
        }

        // Ueberzug takes an area given in chars and fits the image to
        // that area (from the top left).
        //   draw_offset.y += (draw_size.y - size.y) - (draw_size.y - size.y) / 2;
        let cmd = format!("{{\"action\":\"add\",\"scaler\":\"forced_cover\",\"identifier\":\"cover\",\"x\":{},\"y\":{},\"width\":{},\"height\":{},\"path\":\"{}\"}}\n",
        // let cmd = format!("{{\"action\":\"add\",\"scaler\":\"fit_contain\",\"identifier\":\"cover\",\"x\":{},\"y\":{},\"width\":{},\"height\":{},\"path\":\"{}\"}}\n",
        // TODO: right now the y position of ueberzug is not consistent, and could be a 0.5 difference
                // draw_xywh.x, draw_xywh.y-1,
                draw_xywh.x, draw_xywh.y,//-1 + (draw_xywh.width-draw_xywh.height) % 2,
                draw_xywh.width,draw_xywh.height/2,//+ (draw_xywh.width-draw_xywh.height)%2,
                url,
            );

        // println!(
        //     "draw_xywh.x = {}, draw_xywh.y = {}, draw_wyxh.width = {}, draw_wyxh.height = {}",
        //     draw_xywh.x, draw_xywh.y, draw_xywh.width, draw_xywh.height,
        // );
        if let Err(e) = self.run_ueberzug_cmd(&cmd) {
            bail!("Failed to run Ueberzug: {}", e)
            // Ok(())
        }
        Ok(())
    }

    pub fn clear_cover_ueberzug(&self) -> Result<()> {
        let cmd = "{\"action\": \"remove\", \"identifier\": \"cover\"}\n";
        if let Err(e) = self.run_ueberzug_cmd(cmd) {
            bail!("Failed to run Ueberzug: {}", e)
            // Ok(())
        }
        Ok(())
    }

    fn run_ueberzug_cmd(&self, cmd: &str) -> Result<()> {
        let mut ueberzug = self.ueberzug.write().unwrap();

        if ueberzug.is_none() {
            *ueberzug = Some(
                std::process::Command::new("ueberzug")
                    .args(["layer", "--silent"])
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
