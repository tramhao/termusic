use crate::config::Xywh;
use anyhow::Context;
use anyhow::{bail, Result};
use std::ffi::OsStr;
use std::io::Read as _;
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
        self.run_ueberzug_cmd(cmd)
            .map_err(map_err)
            .context("clear_cover")?;
        Ok(())
    }

    fn run_ueberzug_cmd(&mut self, cmd: &str) -> Result<()> {
        // error!("using x11 output for ueberzugpp");

        let Some(ueberzug) = self.try_wait_spawn(["layer"])? else {
            return Ok(());
        };

        let stdin = ueberzug.stdin.as_mut().unwrap();
        stdin
            .write_all(cmd.as_bytes())
            .context("ueberzug command writing")?;

        Ok(())
    }

    fn run_ueberzug_cmd_sixel(&mut self, cmd: &str) -> Result<()> {
        // error!("using sixel output for ueberzugpp");

        let Some(ueberzug) = self.try_wait_spawn(
            ["layer"],
            // ["layer", "--silent", "--no-cache", "--output", "sixel"]
            // ["layer", "--sixel"]
            // ["--sixel"]
        )?
        else {
            return Ok(());
        };

        let stdin = ueberzug.stdin.as_mut().unwrap();
        stdin
            .write_all(cmd.as_bytes())
            .context("ueberzug command writing")?;

        Ok(())
    }

    /// Spawn the given `cmd`, and set `self.ueberzug` and return a reference to the child for direct use
    ///
    /// On fail, also set `set.ueberzug` to [`UeInstanceState::Error`]
    fn spawn_cmd<I, S>(&mut self, args: I) -> Result<&mut Child>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        let mut cmd = Command::new("ueberzug");
        cmd.args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::null()) // ueberzug does not output to stdout
            .stderr(Stdio::piped());

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

    /// If ueberzug instance does not exist, create it. Otherwise take the existing one
    ///
    /// Do a [`Child::try_wait`] on the existing instance and return a error if the instance has exited
    fn try_wait_spawn<I, S>(&mut self, args: I) -> Result<Option<&mut Child>>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        let child = match self.ueberzug {
            UeInstanceState::New => self.spawn_cmd(args)?,
            UeInstanceState::Child(ref mut v) => v,
            UeInstanceState::Error => return on_error().map(|()| None),
        };

        if let Some(exit_status) = child.try_wait()? {
            let mut stderr_buf = String::new();
            child
                .stderr
                .as_mut()
                .map(|v| v.read_to_string(&mut stderr_buf));

            // using a permanent-Error because it is likely the error will happen again on restart (like being on wayland instead of x11)
            self.ueberzug = UeInstanceState::Error;

            if stderr_buf.is_empty() {
                stderr_buf.push_str("<empty>");
            }

            // special handling for unix as that only contains the ".signal" extension, which is important there
            #[cfg(not(target_family = "unix"))]
            {
                bail!(
                    "ueberzug command closed unexpectedly, (code {:?}), stderr:\n{}",
                    exit_status.code(),
                    stderr_buf
                );
            }
            #[cfg(target_family = "unix")]
            {
                use std::os::unix::process::ExitStatusExt as _;
                bail!(
                    "ueberzug command closed unexpectedly, (code {:?}, signal {:?}), stderr:\n{}",
                    exit_status.code(),
                    exit_status.signal(),
                    stderr_buf
                );
            }
        }

        // out of some reason local variable "child" cannot be returned here because it is modified in the "try_wait" branch
        // even though that branch never reaches here
        Ok(Some(self.ueberzug.unwrap_child_mut()))
    }
}

/// Small helper to always print a message and return a consistent return
#[inline]
#[allow(clippy::unnecessary_wraps)]
fn on_error() -> Result<()> {
    trace!("Not re-trying ueberzug, because it has a permanent error!");

    Ok(())
}

/// Map a given error to include extra context
#[inline]
#[allow(clippy::needless_pass_by_value)]
fn map_err(err: anyhow::Error) -> anyhow::Error {
    err.context("Failed to run Ueberzug")
}
