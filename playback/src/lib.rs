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
    AboutToFinish,
    CycleLoop,
    Eos,
    GetProgress,
    SkipPrevious,
    Pause,
    Play,
    Quit,
    ReloadConfig,
    ReloadPlaylist,
    SeekBackward,
    SeekForward,
    SkipNext,
    SpeedDown,
    SpeedUp,
    Tick,
    ToggleGapless,
    TogglePause,
    VolumeDown,
    VolumeUp,
    /// A Error happened in the backend (for example `NotFound`) that makes it unrecoverable to continue to play the current track.
    /// This will basically be treated as a [`Eos`](PlayerCmd::Eos), with some extra handling.
    ///
    /// This should **not** be used if the whole backend is unrecoverable.
    Error(PlayerErrorType),

    PlaylistPlaySpecific(PlaylistPlaySpecific),
    PlaylistAddTrack(PlaylistAddTrack),
    PlaylistRemoveTrack(PlaylistRemoveTrackIndexed),
    PlaylistClear,
    PlaylistSwapTrack(PlaylistSwapTrack),
    PlaylistShuffle,
    PlaylistRemoveDeletedTracks,
}

pub type StreamTX = broadcast::Sender<UpdateEvents>;
pub type SharedPlaylist = Arc<RwLock<Playlist>>;

#[allow(clippy::module_name_repetitions)]
pub struct GeneralPlayer {
    pub backend: Backend,
    pub playlist: SharedPlaylist,
    pub config: SharedServerSettings,
    pub current_track_updated: bool,
    pub mpris: Option<mpris::Mpris>,
    pub discord: Option<discord::Rpc>,
    pub db: Database,
    pub db_podcast: DBPod,
    pub cmd_tx: PlayerCmdSender,
    pub stream_tx: StreamTX,

    /// Keep track of continues backend errors (like `NotFound`) to not keep trying infinitely.
    pub errors_since_last_progress: usize,
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
            playlist,
            config,
            mpris,
            discord,
            db,
            db_podcast,
            cmd_tx,
            stream_tx,
            current_track_updated: false,

            errors_since_last_progress: 0,
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

    /// Create a new [`GeneralPlayer`], with the default Backend ([`BackendSelect::Rusty`])
    ///
    /// # Errors
    ///
    /// - if connecting to the database fails
    /// - if config path creation fails
    pub fn new(
        config: SharedServerSettings,
        cmd_tx: PlayerCmdSender,
        stream_tx: StreamTX,
        playlist: SharedPlaylist,
    ) -> Result<Self> {
        Self::new_backend(
            BackendSelect::default(),
            config,
            cmd_tx,
            stream_tx,
            playlist,
        )
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
            if let Some(track) = self.playlist.read().current_track() {
                mpris.add_and_play(track);
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
            if let Some(track) = self.playlist.read().current_track() {
                discord.update(track);
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
    pub fn start_play(&mut self) {
        let mut playlist = self.playlist.write();
        if playlist.is_stopped() | playlist.is_paused() {
            playlist.set_status(RunningStatus::Running);
        }

        playlist.proceed();

        if let Some(track) = playlist.current_track().cloned() {
            info!("Starting Track {track:#?}");

            if playlist.has_next_track() {
                playlist.set_next_track(None);
                drop(playlist);
                self.current_track_updated = true;
                info!("gapless next track played");
                self.add_and_play_mpris_discord();
                return;
            }
            drop(playlist);

            self.current_track_updated = true;
            let wait = async {
                self.add_and_play(&track).await;
            };
            Handle::current().block_on(wait);

            self.add_and_play_mpris_discord();
            self.player_restore_last_position();

            self.send_stream_ev(UpdateEvents::TrackChanged(TrackChangedInfo {
                current_track_index: u64::try_from(self.playlist.read().get_current_track_index())
                    .unwrap(),
                current_track_updated: self.current_track_updated,
                title: self.media_info().media_title,
                progress: self.get_progress(),
            }));
        }
    }

    fn add_and_play_mpris_discord(&mut self) {
        if let Some(track) = self.playlist.read().current_track() {
            if let Some(ref mut mpris) = self.mpris {
                mpris.add_and_play(track);
            }

            if let Some(ref discord) = self.discord {
                discord.update(track);
            }
        }
    }
    pub fn enqueue_next_from_playlist(&mut self) {
        let mut playlist = self.playlist.write();
        if playlist.has_next_track() {
            return;
        }

        let Some(track) = playlist.fetch_next_track().cloned() else {
            return;
        };
        drop(playlist);

        self.enqueue_next(&track);

        info!("Next track enqueued: {track:#?}");
    }

    /// Skip to the next track, if there is one
    pub fn next(&mut self) {
        if self.playlist.read().current_track().is_some() {
            info!("skip route 1 which is in most cases.");
            self.playlist.write().set_next_track(None);
            self.skip_one();
        } else {
            info!("skip route 2 cause no current track.");
            self.stop();
        }
    }

    /// Switch & Play the previous track in the playlist
    pub fn previous(&mut self) {
        let mut playlist = self.playlist.write();
        playlist.previous();
        playlist.proceed_false();
        drop(playlist);
        self.next();
    }

    /// Resume playback if paused, pause playback if running
    pub fn toggle_pause(&mut self) {
        // NOTE: if this ".read()" call is in a match's statement, it will not be unlocked until the end of the match
        // see https://github.com/rust-lang/rust/issues/93883
        let status = self.playlist.read().status();
        match status {
            RunningStatus::Running => {
                <Self as PlayerTrait>::pause(self);
            }
            RunningStatus::Stopped => {}
            RunningStatus::Paused => {
                <Self as PlayerTrait>::resume(self);
            }
        }
    }

    /// Pause playback if running
    pub fn pause(&mut self) {
        // NOTE: if this ".read()" call is in a match's statement, it will not be unlocked until the end of the match
        // see https://github.com/rust-lang/rust/issues/93883
        let status = self.playlist.read().status();
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
        let status = self.playlist.read().status();
        match status {
            RunningStatus::Running | RunningStatus::Stopped => {}
            RunningStatus::Paused => {
                <Self as PlayerTrait>::resume(self);
            }
        }
    }
    /// # Panics
    ///
    /// if the underlying "seek" returns a error (which current never happens)
    pub fn seek_relative(&mut self, forward: bool) {
        // fallback to 5 instead of not seeking at all
        let track_len = self
            .playlist
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

    #[allow(clippy::cast_sign_loss)]
    pub fn player_save_last_position(&mut self) {
        let playlist = self.playlist.read();
        let Some(track) = playlist.current_track() else {
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

    pub fn player_restore_last_position(&mut self) {
        let playlist = self.playlist.read();
        let Some(track) = playlist.current_track().cloned() else {
            info!("Not restoring Last position as there is no current track");
            return;
        };
        drop(playlist);

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
        if restored {
            if let Err(err) = self.set_last_position(&track, None) {
                error!("Resetting last_position failed. Error: {err:#?}");
            }
        }
    }

    /// Update all the places that should be updated on a new Progress report.
    pub fn update_progress(&mut self, progress: &PlayerProgress) {
        self.mpris_update_progress(progress);

        self.send_stream_ev(UpdateEvents::Progress(*progress));
    }

    /// Send stream events with consistent error handling
    fn send_stream_ev(&self, ev: UpdateEvents) {
        // there is only one error case: no receivers
        if self.stream_tx.send(ev).is_err() {
            debug!("Stream Event not send: No Receivers");
        }
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
        self.send_stream_ev(UpdateEvents::VolumeChanged { volume: vol });

        vol
    }
    fn set_volume(&mut self, volume: Volume) -> Volume {
        let vol = self.get_player_mut().set_volume(volume);
        self.send_stream_ev(UpdateEvents::VolumeChanged { volume: vol });

        vol
    }
    /// This function should not be used directly, use GeneralPlayer::pause
    fn pause(&mut self) {
        self.playlist.write().set_status(RunningStatus::Paused);
        self.get_player_mut().pause();
        if let Some(ref mut mpris) = self.mpris {
            mpris.pause();
        }
        if let Some(ref discord) = self.discord {
            discord.pause();
        }
    }
    /// This function should not be used directly, use GeneralPlayer::play
    fn resume(&mut self) {
        self.playlist.write().set_status(RunningStatus::Running);
        self.get_player_mut().resume();
        if let Some(ref mut mpris) = self.mpris {
            mpris.resume();
        }
        let time_pos = self.get_player().position();
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
        self.playlist.write().stop();
        self.get_player_mut().stop();
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
        self.get_player_mut().skip_one();
    }

    fn position(&self) -> Option<PlayerTimeUnit> {
        self.get_player().position()
    }

    fn enqueue_next(&mut self, track: &Track) {
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
