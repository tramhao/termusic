use std::pin::Pin;
use std::time::Duration;

use anyhow::Context;
use anyhow::Result;
use futures_util::FutureExt;
use futures_util::StreamExt;
use sysinfo::Pid;
use sysinfo::System;
use termusiclib::player::PlayerProgress;
use termusiclib::player::RunningStatus;
use termusiclib::player::StreamUpdates;
use termusiclib::player::UpdateEvents;
use termusiclib::player::UpdatePlaylistEvents;
use termusiclib::player::music_player_client::MusicPlayerClient;
use tokio::sync::mpsc::{self};
use tokio_stream::Stream;
use tonic::transport::Channel;
use tuirealm::application::PollStrategy;
use tuirealm::{Application, Update};

use crate::CombinedSettings;
use crate::ui::server_req_actor::ServerRequestActor;
use model::Model;
use music_player_client::Playback;
use tui_cmd::PlaylistCmd;
use tui_cmd::TuiCmd;

pub mod components;
mod ids;
pub mod model;
mod msg;
mod music_player_client;
mod server_req_actor;
mod tui_cmd;
pub mod utils;

/// The Interval in which to force a redraw, if no redraw happened in that time.
const FORCED_REDRAW_INTERVAL: Duration = Duration::from_millis(1000);

/// The main TUI struct which handles message passing and the main-loop.
pub struct UI {
    model: Model,
    stream_updates: Pin<Box<dyn Stream<Item = Result<StreamUpdates>>>>,
}

impl UI {
    /// Create a new [`UI`] instance
    pub async fn new(config: CombinedSettings, client: MusicPlayerClient<Channel>) -> Result<Self> {
        let (cmd_tx, cmd_rx) = mpsc::unbounded_channel();
        let mut model = Model::new(config, cmd_tx).await;
        model.init_config();
        let mut playback = Playback::new(client);

        let stream_updates = playback.subscribe_to_stream_updates().await?;

        ServerRequestActor::start_actor(playback, cmd_rx, model.tx_to_main.clone());

        Ok(Self {
            model,
            stream_updates: stream_updates.boxed(),
        })
    }

    /// Force a redraw if [`FORCED_REDRAW_INTERVAL`] has passed.
    fn exec_interval_redraw(&mut self) {
        if self.model.since_last_redraw() >= FORCED_REDRAW_INTERVAL {
            self.model.force_redraw();
        }
    }

    /// Handle terminal init & finalize and start the UI Loop.
    pub fn run(&mut self) -> Result<()> {
        self.model.init_terminal();

        let res = self.run_inner();

        // reset terminal in any case
        self.model.finalize_terminal();

        res
    }

    /// Main Loop function.
    ///
    /// This function does NOT handle initializing and finializing the terminal.
    #[allow(clippy::unnecessary_wraps)] // to easily change if it ever becomes required again
    fn run_inner(&mut self) -> Result<()> {
        // load the initial playlist
        let _ = self
            .model
            .cmd_to_server_tx
            .send(TuiCmd::Playlist(PlaylistCmd::SelfReloadPlaylist));
        // initial request for all the progress states / options
        self.model.request_progress();

        // Main loop
        while !self.model.quit {
            if let Err(err) = self.handle_stream_events() {
                self.model.mount_error_popup(err);
            }

            match self.model.app.tick(PollStrategy::UpTo(20)) {
                Err(err) => {
                    self.model
                        .mount_error_popup((anyhow::anyhow!(err)).context("tick poll error"));
                }
                Ok(messages) if !messages.is_empty() => {
                    // NOTE: redraw if at least one msg has been processed
                    self.model.redraw = true;
                    for msg in messages {
                        let mut msg = Some(msg);
                        while msg.is_some() {
                            msg = self.model.update(msg);
                        }
                    }
                }
                _ => {}
            }

            self.model.ensure_quit_popup_top_most_focus();

            // normally a interval-redraw should not be necessary and instead only happen on events,
            // but there might be some bugs that this works around
            self.exec_interval_redraw();
            self.model.view();
        }

        if self
            .model
            .config_tui
            .read()
            .settings
            .behavior
            .quit_server_on_exit
        {
            Self::quit_server();
        }

        Ok(())
    }

    /// Quit the server, if any is found with the proper name.
    // TODO: send the server a message to quit instead of a signal.
    fn quit_server() {
        let mut system = System::new();
        system.refresh_all();
        let mut target = None;
        let mut clients = 0;
        for proc in system.processes().values() {
            if let Some(exe) = proc.name().to_str() {
                if exe == "termusic-server" {
                    if &proc.pid() == crate::SERVER_PID.get().unwrap_or(&Pid::from_u32(0))
                        || target.is_none()
                    {
                        target = Some(proc);
                    }
                    continue;
                }
                let mut parent_is_termusic = false;
                match proc.parent() {
                    Some(s) => {
                        if let Some(parent) = system.processes().get(&s) {
                            if parent.name() == "termusic" {
                                parent_is_termusic = true;
                            }
                        }
                    }
                    None => parent_is_termusic = false,
                }
                if exe == "termusic" && !parent_is_termusic {
                    clients += 1;
                }
            }
        }
        if clients <= 1 && target.is_some() {
            if let Some(s) = target {
                #[cfg(not(target_os = "windows"))]
                s.kill_with(sysinfo::Signal::Term);
                #[cfg(target_os = "windows")]
                s.kill();
            }
        }
    }

    /// Handle Stream updates from the provided stream.
    ///
    /// In case of lag, sends a [`TuiCmd::GetProgress`] to `self.model`.
    ///
    /// - Does not wait until the next event (non-blocking).
    /// - Processess *all* available events.
    fn handle_stream_events(&mut self) -> Result<()> {
        while let Some(ev) = self.stream_updates.next().now_or_never().flatten() {
            let ev = ev
                .map(UpdateEvents::try_from)
                .context("Conversion from StreamUpdates to UpdateEvents failed!")?;

            // dont log progress events, as that spams the log
            if log::log_enabled!(log::Level::Debug) && !is_progress(&ev) {
                debug!("Stream Event: {ev:?}");
            }

            // just exit on first error, but still print it first
            let Ok(ev) = ev else {
                break;
            };

            match ev {
                UpdateEvents::MissedEvents { amount } => {
                    warn!("Stream Lagged, missed events: {amount}");
                    // we know that we missed events, force to get full information from GetProgress endpoint
                    self.model.command(TuiCmd::GetProgress);
                }
                UpdateEvents::VolumeChanged { volume } => {
                    self.model.config_server.write().settings.player.volume = volume;
                }
                UpdateEvents::SpeedChanged { speed } => {
                    self.model.config_server.write().settings.player.speed = speed;
                }
                UpdateEvents::PlayStateChanged { playing } => {
                    self.model
                        .playback
                        .set_status(RunningStatus::from_u32(playing));
                    self.model.progress_update_title();
                }
                UpdateEvents::TrackChanged(track_changed_info) => {
                    if let Some(progress) = track_changed_info.progress {
                        self.model.progress_update(
                            progress.position,
                            progress.total_duration.unwrap_or_default(),
                        );
                    }

                    if track_changed_info.current_track_updated {
                        self.model.handle_current_track_index(
                            usize::try_from(track_changed_info.current_track_index).unwrap(),
                            false,
                        );
                    }

                    if let Some(title) = track_changed_info.title {
                        self.model.lyric_update_for_radio(title);
                    }
                }
                UpdateEvents::GaplessChanged { gapless } => {
                    self.model.config_server.write().settings.player.gapless = gapless;
                }
                UpdateEvents::Progress(progress) => {
                    self.model.progress_update(
                        progress.position,
                        progress.total_duration.unwrap_or_default(),
                    );
                }
                UpdateEvents::PlaylistChanged(ev) => self.handle_playlist_events(ev)?,
            }
        }

        Ok(())
    }

    /// Handle Playlist Update Events, separately from [`Self::handle_stream_events`] to lessen clutter.
    fn handle_playlist_events(&mut self, ev: UpdatePlaylistEvents) -> Result<()> {
        match ev {
            UpdatePlaylistEvents::PlaylistAddTrack(playlist_add_track) => {
                self.model.handle_playlist_add(playlist_add_track)?;
            }
            UpdatePlaylistEvents::PlaylistRemoveTrack(playlist_remove_track) => {
                self.model.handle_playlist_remove(&playlist_remove_track)?;
            }
            UpdatePlaylistEvents::PlaylistCleared => {
                self.model.handle_playlist_clear();
            }
            UpdatePlaylistEvents::PlaylistLoopMode(loop_mode) => {
                self.model.handle_playlist_loopmode(&loop_mode)?;
            }
            UpdatePlaylistEvents::PlaylistSwapTracks(swapped_tracks) => {
                self.model.handle_playlist_swap_tracks(&swapped_tracks)?;
            }
            UpdatePlaylistEvents::PlaylistShuffled(shuffled) => {
                self.model.handle_playlist_shuffled(shuffled)?;
            }
        }

        Ok(())
    }
}

/// Determine if a given event is a [`UpdateEvents::Progress`].
fn is_progress(ev: &Result<UpdateEvents>) -> bool {
    if let Ok(ev) = ev {
        std::mem::discriminant(ev)
            == std::mem::discriminant(&UpdateEvents::Progress(PlayerProgress {
                position: None,
                total_duration: None,
            }))
    } else {
        false
    }
}
