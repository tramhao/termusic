// mod internal_backend;
// mod crossbeam;
// mod symphonia_backend;
#[cfg(all(feature = "gst", not(feature = "mpv")))]
mod gstreamer_backend;
#[cfg(feature = "mpv")]
mod mpv_backend;
#[cfg(not(any(feature = "mpv", feature = "gst")))]
mod rusty_backend;
// #[cfg(not(any(feature = "mpv", feature = "gst")))]
// mod rodio_backend;
use anyhow::Result;
#[cfg(feature = "mpv")]
use mpv_backend::Mpv;
// #[cfg(not(any(feature = "mpv", feature = "gst")))]
// use rodio_backend::RodioPlayer;
// use symphonia_backend::Symphonia;

pub struct GeneralPl {
    #[cfg(all(feature = "gst", not(feature = "mpv")))]
    player: gstreamer_backend::GStreamer,
    #[cfg(feature = "mpv")]
    player: Mpv,
    // player: RodioPlayer,
    // player: Symphonia,
    // player: crossbeam::Player,
    // player: symphonia_backend::Symphonia,
    #[cfg(not(any(feature = "mpv", feature = "gst")))]
    player: rusty_backend::Player,
}

impl Default for GeneralPl {
    fn default() -> Self {
        #[cfg(all(feature = "gst", not(feature = "mpv")))]
        let player = gstreamer_backend::GStreamer::default();
        #[cfg(feature = "mpv")]
        let player = Mpv::default();
        #[cfg(not(any(feature = "mpv", feature = "gst")))]
        let player = rusty_backend::Player::default();
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
