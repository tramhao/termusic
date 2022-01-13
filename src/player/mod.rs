#[cfg(feature = "gst")]
mod gstreamer_backend;
#[cfg(not(feature = "gst"))]
mod mpv_backend;
use anyhow::Result;
#[cfg(feature = "gst")]
pub use gstreamer_backend::GStreamer;
#[cfg(not(feature = "gst"))]
pub use mpv_backend::Mpv;

pub struct GeneralPl {
    #[cfg(feature = "gst")]
    player: GStreamer,
    #[cfg(not(feature = "gst"))]
    player: Mpv,
}

impl Default for GeneralPl {
    fn default() -> Self {
        #[cfg(feature = "gst")]
        let player = GStreamer::default();
        #[cfg(not(feature = "gst"))]
        let player = Mpv::default();
        Self { player }
    }
}

impl GeneralP for GeneralPl {
    fn add_and_play(&mut self, new: &str) {
        self.player.add_and_play(new);
    }
    fn volume(&self) -> i32 {
        self.player.volume()
    }
    fn volume_up(&mut self) {
        self.player.volume_up();
    }
    fn volume_down(&mut self) {
        self.player.volume_down();
    }
    fn set_volume(&mut self, volume: i32) {
        self.player.set_volume(volume);
    }
    fn pause(&mut self) {
        self.player.pause();
    }
    fn resume(&mut self) {
        self.player.resume();
    }
    fn is_paused(&mut self) -> bool {
        self.player.is_paused()
    }
    fn seek(&mut self, secs: i64) -> Result<()> {
        self.player.seek(secs)
    }
    fn get_progress(&mut self) -> Result<(f64, i64, i64)> {
        self.player.get_progress()
    }
}

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
