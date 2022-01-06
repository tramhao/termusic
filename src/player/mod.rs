#[cfg(feature = "gst")]
mod gstreamer_backend;
#[cfg(not(feature = "gst"))]
mod mpv_backend;
use anyhow::Result;
#[cfg(feature = "gst")]
pub use gstreamer_backend::GStreamer;
#[cfg(not(feature = "gst"))]
pub use mpv_backend::Mpv;

pub trait GeneralP {
    fn add_and_play(&mut self, new: &str);
    fn volume(&self) -> i32;
    fn volume_up(&mut self);
    fn volume_down(&mut self);
    fn set_volume(&mut self, volume: i32);
    fn pause(&mut self);
    fn resume(&mut self);
    fn is_paused(&mut self) -> bool;
    fn seek(&mut self, secs: i64) -> Result<()>;
    fn get_progress(&mut self) -> Result<(f64, i64, i64)>;
}
