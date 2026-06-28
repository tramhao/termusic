use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context, Result};
use async_trait::async_trait;
use parking_lot::RwLock;
pub use playlist::Playlist;
use termusiclib::config::SharedServerSettings;
use termusiclib::config::v2::server::config_extra::ServerConfigVersionedDefaulted;
use termusiclib::new_database::{Database, track_ops};
use termusiclib::player::playlist_helpers::{
    PlaylistAddTrack, PlaylistPlaySpecific, PlaylistRemoveTrackIndexed, PlaylistSwapTrack,
};
use termusiclib::player::{
    PlayerProgress, PlayerTimeUnit, RunningStatus, TrackChangedInfo, UpdateEvents,
};
use termusiclib::podcast::db::Database as DBPod;
use termusiclib::track::{MediaTypes, Track};
use termusiclib::utils::get_app_config_path;
use tokio::runtime::Handle;
use tokio::sync::mpsc::error::SendError;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio::sync::{broadcast, oneshot};

pub use backends::{Backend, BackendSelect};

mod discord;
mod mpris;
pub mod playlist;

#[macro_use]
extern crate log;

mod backends;

/// Private module for benchmarking only, should never be used outside.
///
/// This is necessary as benchmarking via criterion can only access public lib(crate) level function, like any other outside binary / crate.
pub mod __bench {
    pub use super::backends::rusty::source::async_ring;
}

pub type PlayerCmdCallback = oneshot::Receiver<()>;
pub type PlayerCmdReciever = UnboundedReceiver<(PlayerCmd, PlayerCmdCallbackSender)>;

/// Wrapper around the potential oneshot sender to implement convenience functions.
#[derive(Debug)]
pub struct PlayerCmdCallbackSender(Option<oneshot::Sender<()>>);

impl PlayerCmdCallbackSender {
    /// Send on the oneshot, if there is any.
    pub fn call(self) {
        let Some(sender) = self.0 else {
            return;
        };
        let _ = sender.send(());
    }
}

/// Wrapper for the actual sender, to make it easier to implement new functions.
#[derive(Debug, Clone)]
pub struct PlayerCmdSender(UnboundedSender<(PlayerCmd, PlayerCmdCallbackSender)>);

impl PlayerCmdSender {
    /// Send a given [`PlayerCmd`] without any callback.
    ///
    /// # Errors
    /// Also see [`oneshot::Sender::send`].
    pub fn send(
        &self,
        cmd: PlayerCmd,
    ) -> Result<(), SendError<(PlayerCmd, PlayerCmdCallbackSender)>> {
        self.0.send((cmd, PlayerCmdCallbackSender(None)))
    }

    /// Send a given [`PlayerCmd`] with a callback, returning the receiver.
    ///
    /// # Errors
    /// Also see [`oneshot::Sender::send`].
    pub fn send_cb(
        &self,
        cmd: PlayerCmd,
    ) -> Result<PlayerCmdCallback, SendError<(PlayerCmd, PlayerCmdCallbackSender)>> {
        let (tx, rx) = oneshot::channel();
        self.0.send((cmd, PlayerCmdCallbackSender(Some(tx))))?;
        Ok(rx)
    }

    #[must_use]
    pub fn new(tx: UnboundedSender<(PlayerCmd, PlayerCmdCallbackSender)>) -> Self {
        Self(tx)
    }
}

#[derive(Clone, Debug, Copy, PartialEq)]
pub enum PlayerErrorType {
    /// The error happened for the currently playing track.
    Current,
    /// The error happened for the track that was tried to be enqueued.
    Enqueue,
}

#[derive(Clone, Debug)]
pub enum PlayerCmd {
    // Mainly called from the backends
    /// The Backend indicates the current track is about to end.
    AboutToFinish,
    /// The Backend indicates that the current track has ended.
    Eos,
    /// The Backend indicates new metadata is available.
    MetadataChanged,
    /// A Error happened in the backend (for example `NotFound`) that makes it unrecoverable to continue to play the current track.
    /// This will basically be treated as a [`Eos`](PlayerCmd::Eos), with some extra handling.
    ///
    /// This should **not** be used if the whole backend is unrecoverable.
    Error(PlayerErrorType),

    // Internal only
    Tick,

    // Mainly called from outside sources (client, mpris)
    CycleLoop,
    SkipPrevious,
    Pause,
    Play,
    /// Quit the server process. Includes the source triggering the quit.
    Quit(&'static str),
    ReloadConfig,
    ReloadPlaylist,
    RestartTrack,
    SeekBackward,
    SeekForward,
    SkipNext,
    SpeedDown,
    SpeedUp,
    ToggleGapless,
    TogglePause,
    VolumeDown,
    VolumeUp,
    VolumeSet(Volume),

    PlaylistPlaySpecific(PlaylistPlaySpecific),
    PlaylistAddTrack(PlaylistAddTrack),
    PlaylistRemoveTrack(PlaylistRemoveTrackIndexed),
    PlaylistClear,
    PlaylistSwapTrack(PlaylistSwapTrack),
    PlaylistShuffle,
    PlaylistRemoveDeletedTracks,
}

/// Sources for [`PlayerCmd::Quit`].
pub mod quit_sources {
    pub const CLIENT: &str = "client quit";
    pub const TICK: &str = "tick";
    pub const CTRLC: &str = "ctrl c";
    pub const MPRIS: &str = "mpris";
}

pub type StreamTX = broadcast::Sender<UpdateEvents>;
pub type SharedPlaylist = Arc<RwLock<Playlist>>;
pub type SharedRunInfo = Arc<RwLock<RunInfo>>;

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

#[allow(clippy::module_name_repetitions)]
pub struct GeneralPlayer {
    /// The backend in use where audio will be output to.
    pub backend: Backend,
    /// A reference to the server config.
    pub config: SharedServerSettings,

    /// The playlist.
    pub playlist: SharedPlaylist,
    /// The information for the current playback.
    pub run_info: SharedRunInfo,

    /// Media Control (mpris) instance.
    pub mpris: Option<mpris::Mpris>,
    /// Discord status display RPC.
    pub discord: Option<discord::Rpc>,

    /// Track database.
    pub db: Database,
    /// Podcast database.
    pub db_podcast: DBPod,

    /// Sender for Player commands.
    pub cmd_tx: PlayerCmdSender,
    /// Sender for the Streaming Updates.
    pub stream_tx: StreamTX,

    /// Keep track of continues backend errors (like `NotFound`) to not keep trying infinitely.
    pub errors_since_last_progress: usize,
    /// Track whether the current track is being skipped.
    pub in_skip: bool, // TODO: evaluate if this option can be omitted somehow
}

impl GeneralPlayer {
    /// Create a new [`GeneralPlayer`], with the selected `backend`
    ///
    /// # Errors
    ///
    /// - if connecting to the database fails
    /// - if config path creation fails
    pub fn new_backend(
        backend: BackendSelect,
        config: SharedServerSettings,
        cmd_tx: PlayerCmdSender,
        stream_tx: StreamTX,
        playlist: SharedPlaylist,
        run_info: SharedRunInfo,
    ) -> Result<Self> {
        let backend = Backend::new_select(backend, config.clone(), cmd_tx.clone());

        let db_path = get_app_config_path().with_context(|| "failed to get podcast db path.")?;

        let db_podcast = DBPod::new(&db_path).with_context(|| "error connecting to podcast db.")?;
        let config_read = config.read();
        let db = Database::new_default_path()?;

        let mpris = if config.read().settings.player.use_mediacontrols {
            let mut mpris = mpris::Mpris::new(cmd_tx.clone());

            // set volume on start, as souvlaki (0.8.2) defaults to 1.0 until set by us
            // also otherwise we only set this once the volume actually changes or mpris is re-started via config reload
            mpris.update_volume(backend.as_player().volume());

            Some(mpris)
        } else {
            None
        };
        let discord = if config.read().get_discord_status_enable() {
            Some(discord::Rpc::default())
        } else {
            None
        };

        drop(config_read);

        Ok(Self {
            backend,
            config,

            playlist,
            run_info,

            mpris,
            discord,

            db,
            db_podcast,

            cmd_tx,
            stream_tx,

            errors_since_last_progress: 0,
            in_skip: false,
        })
    }

    /// Increment the errors that happened by one.
    pub fn increment_errors(&mut self) {
        self.errors_since_last_progress += 1;
    }

    /// Reset errors that happened back to 0
    pub fn reset_errors(&mut self) {
        self.errors_since_last_progress = 0;
    }

    /// Reload the config from file, on fail continue to use the old
    ///
    /// # Errors
    ///
    /// - if Config could not be parsed
    pub fn reload_config(&mut self) -> Result<()> {
        info!("Reloading config");
        let mut config = self.config.write();
        let parsed = ServerConfigVersionedDefaulted::from_config_path()?.into_settings();
        config.settings = parsed;

        if config.settings.player.use_mediacontrols && self.mpris.is_none() {
            // start mpris if new config has it enabled, but is not active yet
            let mut mpris = mpris::Mpris::new(self.cmd_tx.clone());
            // actually set the metadata of the currently playing track, otherwise the controls will work but no title or coverart will be set until next track
            if let Some(track) = self.run_info.read().current_track() {
                mpris.set_track(track);
            }
            // the same for volume
            mpris.update_volume(self.volume());
            self.mpris.replace(mpris);
        } else if !config.settings.player.use_mediacontrols && self.mpris.is_some() {
            // stop mpris if new config does not have it enabled, but is currently active
            self.mpris.take();
        }

        if config.get_discord_status_enable() && self.discord.is_none() {
            // start discord ipc if new config has it enabled, but is not active yet
            let discord = discord::Rpc::default();

            // actually set the metadata of the currently playing track, otherwise the controls will work but no title or coverart will be set until next track
            if let Some(track) = self.run_info.read().current_track() {
                discord.set_track(track);
            }

            self.discord.replace(discord);
        } else if !config.get_discord_status_enable() && self.discord.is_some() {
            // stop discord ipc if new config does not have it enabled, but is currently active
            self.discord.take();
        }

        info!("Config Reloaded");

        Ok(())
    }

    fn get_player(&self) -> &dyn PlayerTrait {
        self.backend.as_player()
    }

    fn get_player_mut(&mut self) -> &mut (dyn PlayerTrait + Send) {
        self.backend.as_player_mut()
    }

    pub fn toggle_gapless(&mut self) -> bool {
        let new_gapless = !<Self as PlayerTrait>::gapless(self);
        <Self as PlayerTrait>::set_gapless(self, new_gapless);
        self.config.write().settings.player.gapless = new_gapless;
        new_gapless
    }

    /// Requires that the function is called on a thread with a entered tokio runtime
    ///
    /// # Panics
    ///
    /// if `current_track_index` in playlist is above u32
    pub fn start_play(&mut self, from_eos: bool) {
        let mut run_info = self.run_info.write();
        if run_info.is_stopped() | run_info.is_paused() {
            run_info.play(&self.stream_tx);
        }

        let mut playlist = self.playlist.write();

        let next_track = if self.in_skip || !from_eos {
            // When in-skip or not from EOS, get the current track, as "next" or "previous" has already set it (in_skip), or we want to play the current one as we just loaded the playlist
            self.in_skip = false;
            playlist.get_current_track()
        } else {
            // When from EOS, we want to respect the LoopMode decision, so no force here.
            // Force should only be done in Skip / Next.
            playlist.next(false)
        };

        if let Some(enqueued_track) = run_info.take_enqueued() {
            let Some(track) = next_track else {
                error!("Desync: enqueue flag set, but playlist did not have a next anymore!");
                drop(playlist);
                drop(run_info);
                self.stop();
                return;
            };
            if enqueued_track.as_track_source() == track.as_track_source() {
                info!("Starting seamless Track {track:#?}");
                run_info.set_current_track(track.clone());
                drop(playlist);
                drop(run_info);
                self.set_track_mpris_discord();

                self.send_track_changed();

                return;
            }

            info!("Enqueued Track did not match next track from playlist, skipping enqueued!");
        }

        if let Some(track) = next_track.cloned() {
            drop(playlist);
            drop(run_info);
            info!("Starting Track {track:#?}");

            let wait = async {
                self.add_and_play(&track).await;
            };
            Handle::current().block_on(wait);
            self.run_info.write().set_current_track(track);

            self.set_track_mpris_discord();
            self.player_restore_last_position();

            self.send_track_changed();
        } else {
            info!("Stopping due to not having a next track");
            drop(playlist);
            drop(run_info);
            self.stop();
        }
    }

    /// Handle [`PlayerCmd::MetadataChanged`] for all things the [`GeneralPlayer`] controls.
    pub fn metadata_changed(&mut self) {
        self.send_track_changed();
    }

    /// Send event [`UpdateEvents::TrackChanged`]. In a function to de-duplicate calls.
    fn send_track_changed(&mut self) {
        self.send_stream_ev(UpdateEvents::TrackChanged(TrackChangedInfo {
            current_track_index: u64::try_from(self.playlist.read().get_current_track_index())
                .unwrap(),
            title: self.media_info().media_title,
            progress: self.get_progress(),
        }));
    }

    /// Set the current track for extra services like Media Control or discord.
    fn set_track_mpris_discord(&mut self) {
        if let Some(track) = self.run_info.read().current_track() {
            if let Some(ref mut mpris) = self.mpris {
                mpris.set_track(track);
            }

            if let Some(ref discord) = self.discord {
                discord.set_track(track);
            }
        }
    }

    /// Try to enqueue the next track to play, so that it can be seamlessly played.
    pub fn enqueue_next_from_playlist(&mut self) {
        let mut playlist = self.playlist.write();
        let Some(next_track) = playlist.fetch_next().cloned() else {
            debug!("No next track, not enqueuing!");
            return;
        };
        drop(playlist);

        self.enqueue_next(&next_track);

        info!("Next track enqueued: {next_track:#?}");
    }

    /// Skip to a specific track, if there is one
    ///
    /// # Errors
    ///
    /// - if converting u64 to usize fails
    /// - if the given info's tracks mismatch with the actual playlist
    pub fn play_specific(&mut self, info: &PlaylistPlaySpecific) -> Result<()> {
        let has_next = self.playlist.write().set_play_specific(info)?.is_some();
        debug!("play_specific: has_next: {has_next:#?}");

        if has_next {
            self.skip_one();
        } else {
            self.stop();
        }

        Ok(())
    }

    /// Skip to the next track, if there is one
    pub fn next(&mut self) {
        let has_next = self.playlist.write().next(true).is_some();
        debug!("next: has_next: {has_next:#?}");

        if has_next {
            self.skip_one();
        } else {
            self.stop();
        }
    }

    /// Switch & Play the previous track in the playlist
    pub fn previous(&mut self) {
        let mut playlist = self.playlist.write();
        let has_previous = playlist.previous().is_some();
        drop(playlist);
        debug!("previous: has_previous: {has_previous:#?}");

        if has_previous {
            self.skip_one();
        } else {
            self.stop();
        }
    }

    /// Resume playback if paused, pause playback if running.
    ///
    /// Also starts playback if the state was "stopped".
    pub fn toggle_pause(&mut self) {
        // NOTE: if this ".read()" call is in a match's statement, it will not be unlocked until the end of the match
        // see https://github.com/rust-lang/rust/issues/93883
        let status = self.run_info.read().status();
        match status {
            RunningStatus::Running => {
                <Self as PlayerTrait>::pause(self);
            }
            RunningStatus::Stopped => {
                self.resume_from_stopped();
            }
            RunningStatus::Paused => {
                <Self as PlayerTrait>::resume(self);
            }
        }
    }

    /// Pause playback if running
    pub fn pause(&mut self) {
        // NOTE: if this ".read()" call is in a match's statement, it will not be unlocked until the end of the match
        // see https://github.com/rust-lang/rust/issues/93883
        let status = self.run_info.read().status();
        match status {
            RunningStatus::Running => {
                <Self as PlayerTrait>::pause(self);
            }
            RunningStatus::Stopped | RunningStatus::Paused => {}
        }
    }

    /// Resume playback if paused
    pub fn play(&mut self) {
        // NOTE: if this ".read()" call is in a match's statement, it will not be unlocked until the end of the match
        // see https://github.com/rust-lang/rust/issues/93883
        let status = self.run_info.read().status();
        match status {
            RunningStatus::Running => {}
            RunningStatus::Stopped => {
                self.resume_from_stopped();
            }
            RunningStatus::Paused => {
                <Self as PlayerTrait>::resume(self);
            }
        }
    }

    /// Resume playback if stopped.
    ///
    /// Tries to transition from a [`RunningStatus::Stopped`] to a [`RunningStatus::Running`].
    pub fn resume_from_stopped(&mut self) {
        // NOTE: if this ".read()" call is in a match's statement, it will not be unlocked until the end of the match
        // see https://github.com/rust-lang/rust/issues/93883
        let playlist_read = self.playlist.read();
        let status = self.run_info.read().status();
        match status {
            RunningStatus::Running | RunningStatus::Paused => {}
            RunningStatus::Stopped => {
                // nothing to play
                if playlist_read.is_empty() {
                    return;
                }

                drop(playlist_read);
                self.run_info.write().stop(&self.stream_tx);

                info!("Resuming from stopped status");

                self.start_play(false);
            }
        }
    }

    /// Seeks to 0 seconds, thereby practically restarting the current track.
    ///
    /// # Panics
    ///
    /// if the underlying "seek" returns a error (which current never happens)
    pub fn restart_track(&mut self) {
        self.seek_to(Duration::from_secs(0));
    }

    /// # Panics
    ///
    /// if the underlying "seek" returns a error (which current never happens)
    pub fn seek_relative(&mut self, forward: bool) {
        // fallback to 5 instead of not seeking at all
        let track_len = self
            .run_info
            .read()
            .current_track()
            .and_then(Track::duration)
            .unwrap_or(Duration::from_secs(5))
            .as_secs();

        let mut offset = self
            .config
            .read()
            .settings
            .player
            .seek_step
            .get_step(track_len);

        if !forward {
            offset = -offset;
        }
        self.seek(offset).expect("Error in player seek.");
    }

    /// Helper function to de-duplicate setting last position for a given track.
    fn set_last_position(&self, track: &Track, to: Option<Duration>) -> Result<()> {
        match track.inner() {
            MediaTypes::Track(track_data) => {
                track_ops::set_last_position(&self.db.get_connection(), track_data.path(), to)
                    .with_context(|| track_data.path().to_string_lossy().to_string())?;
            }
            MediaTypes::Radio(_) => (),
            MediaTypes::Podcast(_podcast_track_data) => {
                let to = to.unwrap_or_default();
                self.db_podcast
                    .set_last_position(track, to)
                    .context("Podcast Episode")?;
            }
        }

        Ok(())
    }

    /// Save the current track position to the database.
    #[allow(clippy::cast_sign_loss)]
    pub fn player_save_last_position(&mut self) {
        let run_info = self.run_info.read();
        let Some(track) = run_info.current_track() else {
            info!("Not saving Last position as there is no current track");
            return;
        };
        let Some(position) = self.position() else {
            info!("Not saving Last position as there is no position");
            return;
        };

        let Some(time_before_save) = self
            .config
            .read()
            .settings
            .player
            .remember_position
            .get_time(track.media_type())
        else {
            info!(
                "Not saving Last position as \"Remember last position\" is not enabled for {:#?}",
                track.media_type()
            );
            return;
        };

        if time_before_save < position.as_secs() {
            if let Err(err) = self.set_last_position(track, Some(position)) {
                error!("Saving last_position failed. Error: {err:#?}");
            }
        } else {
            info!("Not saving Last position as the position is lower than time_before_save");
        }
    }

    /// Restore the last known track position for the current track.
    pub fn player_restore_last_position(&mut self) {
        let run_info = self.run_info.read();
        let Some(track) = run_info.current_track().cloned() else {
            info!("Not restoring Last position as there is no current track");
            return;
        };
        drop(run_info);

        let mut restored = false;

        if self
            .config
            .read()
            .settings
            .player
            .remember_position
            .is_enabled_for(track.media_type())
        {
            match track.inner() {
                MediaTypes::Track(track_data) => {
                    let res =
                        track_ops::get_last_position(&self.db.get_connection(), track_data.path());
                    if let Ok(Some(last_pos)) = res {
                        self.seek_to(last_pos);
                        restored = true;
                    }
                }
                MediaTypes::Radio(_) => (),
                MediaTypes::Podcast(_podcast_track_data) => {
                    if let Ok(last_pos) = self.db_podcast.get_last_position(&track) {
                        self.seek_to(last_pos);
                        restored = true;
                    }
                }
            }
        } else {
            info!(
                "Not restoring Last position as it is not enabled for {:#?}",
                track.media_type()
            );
        }

        // should we really reset here already instead of just waiting until either next track or exit?
        if restored && let Err(err) = self.set_last_position(&track, None) {
            error!("Resetting last_position failed. Error: {err:#?}");
        }
    }

    /// Update all the places that should be updated on a new Progress report.
    pub fn update_progress(&mut self, progress: &PlayerProgress) {
        self.mpris_update_progress(progress);

        self.send_stream_ev_no_err(UpdateEvents::Progress(*progress));
    }

    /// Send stream events with consistent error handling
    fn send_stream_ev(&self, ev: UpdateEvents) {
        // there is only one error case: no receivers
        if self.stream_tx.send(ev).is_err() {
            debug!("Stream Event not send: No Receivers");
        }
    }

    /// Send stream events with no error handling.
    ///
    /// Useful for events which would otherwise spam the logs but we dont care about (like progress updates).
    fn send_stream_ev_no_err(&self, ev: UpdateEvents) {
        let _ = self.stream_tx.send(ev);
    }
}

#[async_trait]
impl PlayerTrait for GeneralPlayer {
    async fn add_and_play(&mut self, track: &Track) {
        self.get_player_mut().add_and_play(track).await;
    }
    fn volume(&self) -> Volume {
        self.get_player().volume()
    }
    fn add_volume(&mut self, volume: VolumeSigned) -> Volume {
        let vol = self.get_player_mut().add_volume(volume);
        self.mpris_volume_update();
        self.send_stream_ev(UpdateEvents::VolumeChanged { volume: vol });

        vol
    }
    fn set_volume(&mut self, volume: Volume) -> Volume {
        let vol = self.get_player_mut().set_volume(volume);
        self.mpris_volume_update();
        self.send_stream_ev(UpdateEvents::VolumeChanged { volume: vol });

        vol
    }
    /// This function should not be used directly, use `GeneralPlayer::pause`
    fn pause(&mut self) {
        self.run_info.write().pause(&self.stream_tx);
        self.get_player_mut().pause();
        let time_pos = self.get_player().position();
        if let Some(ref mut mpris) = self.mpris {
            mpris.pause(time_pos);
        }
        if let Some(ref discord) = self.discord {
            discord.pause();
        }
    }
    /// This function should not be used directly, use `GeneralPlayer::play`
    fn resume(&mut self) {
        self.run_info.write().play(&self.stream_tx);
        self.get_player_mut().resume();

        let time_pos = self.get_player().position();
        if let Some(ref mut mpris) = self.mpris {
            mpris.resume(time_pos);
        }
        if let Some(ref discord) = self.discord {
            discord.resume(time_pos);
        }
    }
    fn is_paused(&self) -> bool {
        self.get_player().is_paused()
    }
    fn seek(&mut self, secs: i64) -> Result<()> {
        self.get_player_mut().seek(secs)
    }
    fn seek_to(&mut self, position: Duration) {
        self.get_player_mut().seek_to(position);
    }

    fn set_speed(&mut self, speed: Speed) -> Speed {
        let speed = self.get_player_mut().set_speed(speed);
        self.send_stream_ev(UpdateEvents::SpeedChanged { speed });

        speed
    }

    fn add_speed(&mut self, speed: SpeedSigned) -> Speed {
        let speed = self.get_player_mut().add_speed(speed);
        self.send_stream_ev(UpdateEvents::SpeedChanged { speed });

        speed
    }

    fn speed(&self) -> Speed {
        self.get_player().speed()
    }

    fn stop(&mut self) {
        self.in_skip = false;
        self.run_info.write().stop(&self.stream_tx);
        self.get_player_mut().stop();

        if let Some(ref mut mpris) = self.mpris {
            mpris.stop();
        }
        if let Some(ref discord) = self.discord {
            discord.stop();
        }
    }

    fn get_progress(&self) -> Option<PlayerProgress> {
        self.get_player().get_progress()
    }

    fn gapless(&self) -> bool {
        self.get_player().gapless()
    }

    fn set_gapless(&mut self, to: bool) {
        self.get_player_mut().set_gapless(to);
        self.send_stream_ev(UpdateEvents::GaplessChanged { gapless: to });
    }

    fn skip_one(&mut self) {
        self.in_skip = true;
        self.get_player_mut().skip_one();
    }

    fn position(&self) -> Option<PlayerTimeUnit> {
        self.get_player().position()
    }

    fn enqueue_next(&mut self, track: &Track) {
        self.run_info.write().set_enqueued(track.clone());
        self.get_player_mut().enqueue_next(track);
    }

    fn media_info(&self) -> MediaInfo {
        self.get_player().media_info()
    }
}

/// Some information that may be available from the backend
/// This is different from [`Track`] as this is everything parsed from the decoder's metadata
/// and [`Track`] stores some different extra stuff
#[derive(Debug, Clone, PartialEq, Default)]
pub struct MediaInfo {
    /// The title of the current media playing (if present)
    pub media_title: Option<String>,
}

pub type Volume = u16;
/// The type of [`Volume::saturating_add_signed`]
pub type VolumeSigned = i16;
pub type Speed = i32;
// yes this is currently the same as speed, but for consistentcy with VolumeSigned (and maybe other types)
pub type SpeedSigned = Speed;

pub const MIN_SPEED: Speed = 1;
pub const MAX_SPEED: Speed = 30;

#[allow(clippy::module_name_repetitions)]
#[async_trait]
pub trait PlayerTrait {
    /// Add the given track, skip to it (if not already) and start playing
    async fn add_and_play(&mut self, track: &Track);
    /// Get the currently set volume
    fn volume(&self) -> Volume;
    /// Add a relative amount to the current volume
    ///
    /// Returns the new volume
    fn add_volume(&mut self, volume: VolumeSigned) -> Volume {
        let volume = self.volume().saturating_add_signed(volume);
        self.set_volume(volume)
    }
    /// Set the volume to a specific amount.
    ///
    /// Returns the new volume
    fn set_volume(&mut self, volume: Volume) -> Volume;
    fn pause(&mut self);
    fn resume(&mut self);
    fn is_paused(&self) -> bool;
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
    /// Set the speed to a specific amount.
    ///
    /// Returns the new speed
    fn set_speed(&mut self, speed: Speed) -> Speed;
    /// Add a relative amount to the current speed
    ///
    /// Returns the new speed
    fn add_speed(&mut self, speed: SpeedSigned) -> Speed {
        // NOTE: the clamping should likely be done in `set_speed` instead of here
        let speed = (self.speed() + speed).clamp(MIN_SPEED, MAX_SPEED);
        self.set_speed(speed)
    }
    /// Get the currently set speed
    fn speed(&self) -> Speed;
    fn stop(&mut self);
    fn gapless(&self) -> bool;
    fn set_gapless(&mut self, to: bool);
    fn skip_one(&mut self);
    /// Quickly access the position.
    ///
    /// This should ALWAYS match up with [`PlayerTrait::get_progress`]'s `.position`!
    fn position(&self) -> Option<PlayerTimeUnit> {
        self.get_progress()?.position
    }
    /// Add the given URI to be played, but do not skip currently playing track
    fn enqueue_next(&mut self, track: &Track);
    /// Get info of the current media
    fn media_info(&self) -> MediaInfo;
}
