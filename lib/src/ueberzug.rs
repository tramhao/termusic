use crate::config::Xywh;
use anyhow::{bail, Result};
use std::io::Write;
use std::process::Child;
use std::process::Command;
use std::process::Stdio;

#[derive(Debug)]
pub enum UeInstanceState {
    New,
    Child(Child),
    /// Permanent Error
    Error,
}

impl PartialEq for UeInstanceState {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Child(_), Self::Child(_)) => true,
            _ => core::mem::discriminant(self) == core::mem::discriminant(other),
        }
    }
}

impl UeInstanceState {
    /// unwrap value in [`UeInstanceState::Child`], panicing if not that variant
    fn unwrap_child_mut(&mut self) -> &mut Child {
        if let Self::Child(v) = self {
            return v;
        }
        unreachable!()
    }
}

/// Run `ueberzug` commands
///
/// If there is a permanent error (like `ueberzug` not being installed), will silently ignore all commands after initial error
#[derive(Debug)]
pub struct UeInstance {
    ueberzug: UeInstanceState,
}

impl Default for UeInstance {
    fn default() -> Self {
        Self {
            ueberzug: UeInstanceState::New,
        }
    }
}

impl UeInstance {
    pub fn draw_cover_ueberzug(
        &mut self,
        url: &str,
        draw_xywh: &Xywh,
        use_sixel: bool,
    ) -> Result<()> {
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

        // debug!(
        //     "draw_xywh.x = {}, draw_xywh.y = {}, draw_wyxh.width = {}, draw_wyxh.height = {}",
        //     draw_xywh.x, draw_xywh.y, draw_xywh.width, draw_xywh.height,
        // );
        if use_sixel {
            self.run_ueberzug_cmd_sixel(&cmd).map_err(map_err)?;
        } else {
            self.run_ueberzug_cmd(&cmd).map_err(map_err)?;
        };

        Ok(())
    }

    pub fn clear_cover_ueberzug(&mut self) -> Result<()> {
        let cmd = "{\"action\": \"remove\", \"identifier\": \"cover\"}\n";
        self.run_ueberzug_cmd(cmd).map_err(map_err)?;
        Ok(())
    }

    fn run_ueberzug_cmd(&mut self, cmd: &str) -> Result<()> {
        // error!("using x11 output for ueberzugpp");

        let ueberzug = match self.ueberzug {
            UeInstanceState::New => self.spawn_cmd(
                Command::new("ueberzug")
                    .args(["layer", "--silent"])
                    .stdin(Stdio::piped())
                    .stdout(Stdio::piped()),
            )?,
            UeInstanceState::Child(ref mut v) => v,
            UeInstanceState::Error => return on_error(),
        };

        let stdin = ueberzug.stdin.as_mut().unwrap();
        stdin.write_all(cmd.as_bytes())?;

        Ok(())
    }

    fn run_ueberzug_cmd_sixel(&mut self, cmd: &str) -> Result<()> {
        // error!("using sixel output for ueberzugpp");

        let ueberzug = match self.ueberzug {
            UeInstanceState::New => {
                self.spawn_cmd(
                    Command::new("ueberzug")
                        .args(["layer", "--silent"])
                        // .args(["layer", "--silent", "--no-cache", "--output", "sixel"])
                        // .args(["layer", "--sixel"])
                        // .args(["--sixel"])
                        .stdin(Stdio::piped())
                        .stdout(Stdio::piped()),
                )?
            }
            UeInstanceState::Child(ref mut v) => v,
            UeInstanceState::Error => return on_error(),
        };

        let stdin = ueberzug.stdin.as_mut().unwrap();
        stdin.write_all(cmd.as_bytes())?;

        Ok(())
    }

    /// Spawn the given `cmd`, and set `self.ueberzug` and return a reference to the child for direct use
    ///
    /// On fail, also set `set.ueberzug` to [`UeInstanceState::Error`]
    fn spawn_cmd(&mut self, cmd: &mut Command) -> Result<&mut Child> {
        match cmd.spawn() {
            Ok(child) => {
                self.ueberzug = UeInstanceState::Child(child);
                return Ok(self.ueberzug.unwrap_child_mut());
            }
            Err(err) => {
                if err.kind() == std::io::ErrorKind::NotFound {
                    self.ueberzug = UeInstanceState::Error;
                }
                bail!(err)
            }
        }
    }
}

/// Small helper to always print a message and return a consistent return
#[inline]
#[allow(clippy::unnecessary_wraps)]
fn on_error() -> Result<()> {
    info!("Not re-trying ueberzug, because it has a permanent error!");

    Ok(())
}

/// Map a given error to include extra context
#[inline]
#[allow(clippy::needless_pass_by_value)]
fn map_err(err: anyhow::Error) -> anyhow::Error {
    err.context("Failed to run Ueberzug")
}
