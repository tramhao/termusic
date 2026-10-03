use termusiclib::{
    player::{RunningStatus, UpdateEvents},
    track::Track,
};

use crate::StreamTX;

/// Contains all the status for the current playback, which is not backend dependent.
#[derive(Debug)]
pub struct RunInfo {
    /// The current running status that we are targeting, not fully representing what the backend is actually doing.
    status: RunningStatus,

    /// The currently playing [`Track`], if there is one.
    ///
    /// This should only be UNSET if `status == RunningStatus::Stopped`.
    ///
    /// This may or may not be present in any playlist.
    current_track: Option<Track>,
    /// The track that has been enqueued.
    ///
    /// This is used to know whether something had been enqueued and if something changed in-between.
    enqueued: Option<Track>,
}

impl Default for RunInfo {
    fn default() -> Self {
        Self {
            status: RunningStatus::Stopped,
            current_track: None,
            enqueued: None,
        }
    }
}

impl RunInfo {
    /// Set the [`RunningStatus`] of the playlist, also sends a stream event.
    fn set_status(&mut self, status: RunningStatus, tx: &StreamTX) {
        self.status = status;
        Self::send_stream_ev(
            UpdateEvents::PlayStateChanged {
                playing: status.as_u32(),
            },
            tx,
        );
    }

    /// Start playing, if not already.
    pub fn play(&mut self, tx: &StreamTX) {
        self.set_status(RunningStatus::Running, tx);
    }

    /// Pause playback.
    pub fn pause(&mut self, tx: &StreamTX) {
        self.set_status(RunningStatus::Paused, tx);
    }

    /// Stop the current playback by setting [`RunningStatus::Stopped`], preventing going to the next track
    /// and finally, stop the currently playing track.
    pub fn stop(&mut self, tx: &StreamTX) {
        self.set_status(RunningStatus::Stopped, tx);
        self.clear_current_track();
    }

    /// Get the current running status.
    #[must_use]
    pub fn status(&self) -> RunningStatus {
        self.status
    }

    /// Get whether the current running status is playing or not.
    #[must_use]
    pub fn is_playing(&self) -> bool {
        self.status == RunningStatus::Running
    }

    /// Get whether the current running status is paused or not.
    #[must_use]
    pub fn is_paused(&self) -> bool {
        self.status == RunningStatus::Paused
    }

    /// Get wether the current running status is stopped or not.
    #[must_use]
    pub fn is_stopped(&self) -> bool {
        self.status == RunningStatus::Stopped
    }

    /// Set a new currently playing track.
    pub fn set_current_track(&mut self, track: Track) {
        self.current_track = Some(track);
    }

    /// Get the current track, if there is one.
    #[must_use]
    pub fn current_track(&self) -> Option<&Track> {
        self.current_track.as_ref()
    }

    /// Clear the currently playing track.
    fn clear_current_track(&mut self) {
        let _ = self.current_track.take();
    }

    /// Set the track that has been enqueued for seamless playback
    pub fn set_enqueued(&mut self, track: Track) {
        self.enqueued = Some(track);
    }

    /// Get the current enqueued Track value and reset it.
    pub fn take_enqueued(&mut self) -> Option<Track> {
        self.enqueued.take()
    }

    /// Send stream events with consistent error handling.
    fn send_stream_ev(ev: UpdateEvents, tx: &StreamTX) {
        // there is only one error case: no receivers
        if tx.send(ev).is_err() {
            debug!("Stream Event not send: No Receivers");
        }
    }
}
