use std::time::Duration;

use anyhow::Result;
use termusiclib::{
    player::{PlayerProgress, PlayerTimeUnit},
    track::Track,
};

/// Some Track information the backend may have parsed to be available.
/// For example, for radio urls this may contain the ICY metadata parsed from the byte stream.
///
/// This is different compared to [`Track`] as this contains data parsed by the backend instead
/// of our dedicated metadata parser, which currently is `lofty`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct MediaInfo {
    /// The title of the current media playing (if present)
    pub media_title: Option<String>,
}

pub type Volume = u16;
/// The type of [`Volume::saturating_add_signed`]
pub type VolumeSigned = i16;
// TODO: Speed should likely be "NonZero"
// TODO: Document what speed is actually meaning, as it currently does not mean "0" or "1" is normal, but "10" is.
// TODO: Is signed integer really correct here? We currently only deal with positive numbers here.
pub type Speed = i32;
// This is currently the same type as [`Speed`], but for consistentcy with `VolumeSigned` the alias exists.
pub type SpeedSigned = Speed;

pub const MIN_SPEED: Speed = 1;
pub const MAX_SPEED: Speed = 30;

#[allow(clippy::module_name_repetitions)]
pub trait PlayerTrait {
    /// Add the given track, skip to it (if not already) and start playing.
    fn add_and_play(&mut self, track: &Track);
    /// Add the given track to the queue.
    ///
    /// Unlike [`add_and_play`](Self::add_and_play), this function only pre-loads the track and does not skip to it if there is another playing.
    /// This function should also not change the play state.
    fn enqueue_next(&mut self, track: &Track);
    /// Skip the currently playing track, if there is one.
    ///
    /// It is expected that this function will fire a [`PlayerCmd::Eos`](crate::PlayerCmd::Eos) message.
    fn skip_one(&mut self);

    /// Pause the playback, does nothing if in any other state than `Running`.
    fn pause(&mut self);
    /// Resume the playback, does nothing if in any other state than `Paused` (including `Stopped`).
    fn resume(&mut self);
    /// Change the state to `Stopped`, does nothing if already stopped.
    ///
    /// `Stopped` in this case means unloading everything in the current queue / any loaded track, but not unloading the backend itself.
    /// This may also be interpreted as having alias `clear`.
    fn stop(&mut self);
    /// Determines if the playback is currently State `Paused` (not `Stopped`).
    // TODO: this is seemingly unused and hardcoded in practically all backends, do we still need that?
    fn is_paused(&self) -> bool;

    /// Get the currently set volume.
    fn volume(&self) -> Volume;
    /// Add a relative amount to the current volume.
    ///
    /// Returns the new volume.
    fn add_volume(&mut self, volume: VolumeSigned) -> Volume {
        // TODO: maybe remove this from the trait and instead only have "volume" & "set_volume"?
        let volume = self.volume().saturating_add_signed(volume);
        self.set_volume(volume)
    }
    /// Set the volume to a specific amount.
    ///
    /// Returns the new volume.
    fn set_volume(&mut self, volume: Volume) -> Volume;

    /// Get the currently set speed.
    fn speed(&self) -> Speed;
    /// Add a relative amount to the current speed.
    ///
    /// Returns the new speed.
    fn add_speed(&mut self, speed: SpeedSigned) -> Speed {
        // TODO: maybe remove this from the trait and instead only have "speed" & "set_speed"?
        // NOTE: the clamping should likely be done in `set_speed` instead of here
        let speed = (self.speed() + speed).clamp(MIN_SPEED, MAX_SPEED);
        self.set_speed(speed)
    }
    /// Set the speed to a specific amount.
    ///
    /// Returns the new speed.
    fn set_speed(&mut self, speed: Speed) -> Speed;

    /// Seek relatively to the current time
    ///
    /// # Errors
    ///
    /// Depending on different backend, there could be different errors during seek.
    fn seek(&mut self, secs: i64) -> Result<()>;
    // TODO: sync return types between "seek" and "seek_to"?
    /// Seek to a absolute position
    fn seek_to(&mut self, position: Duration);

    /// Get current track time position
    fn get_progress(&self) -> Option<PlayerProgress>;
    /// Quickly access the position.
    ///
    /// This should ALWAYS match up with [`PlayerTrait::get_progress`]'s `.position`!
    fn position(&self) -> Option<PlayerTimeUnit> {
        // TODO: maybe remove this from the trait and instead only have "get_progress"?
        self.get_progress()?.position
    }

    /// Get the state of gapless playback.
    fn gapless(&self) -> bool;
    /// Set the gapless state.
    fn set_gapless(&mut self, to: bool);

    /// Get info of the current media
    fn media_info(&self) -> MediaInfo;
}
