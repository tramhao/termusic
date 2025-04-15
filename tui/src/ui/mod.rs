/*
 * MIT License
 *
 * termusic - Copyright (c) 2021 Larry Hao
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy
 * of this software and associated documentation files (the "Software"), to deal
 * in the Software without restriction, including without limitation the rights
 * to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 * copies of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in all
 * copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 * AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
 * SOFTWARE.
 */
pub mod components;
pub mod model;
mod music_player_client;
mod tui_cmd;
pub mod utils;

use anyhow::Context;
use anyhow::Result;
use futures_util::FutureExt;
use model::{Model, TermusicLayout};
use music_player_client::Playback;
use std::time::Duration;
use sysinfo::System;
use termusiclib::player::music_player_client::MusicPlayerClient;
use termusiclib::player::playlist_helpers::PlaylistRemoveTrackType;
use termusiclib::player::PlayerProgress;
use termusiclib::player::StreamUpdates;
use termusiclib::player::UpdateEvents;
use termusiclib::player::UpdatePlaylistEvents;
pub use termusiclib::types::*;
use termusicplayback::Status;
use tokio::sync::mpsc::{self, UnboundedReceiver};
use tokio_stream::Stream;
use tokio_stream::StreamExt;
use tonic::transport::Channel;
use tui_cmd::PlaylistCmd;
use tui_cmd::TuiCmd;
use tuirealm::application::PollStrategy;
use tuirealm::{Application, Update};

use crate::CombinedSettings;

/// The Interval in which to force a redraw, if no redraw happened in that time.
const FORCED_REDRAW_INTERVAL: Duration = Duration::from_millis(1000);

/// The main TUI struct which handles message passing and the main-loop.
pub struct UI {
    model: Model,
    playback: Playback,
    cmd_rx: UnboundedReceiver<TuiCmd>,
}

impl UI {
    /// Create a new [`UI`] instance
    pub async fn new(config: CombinedSettings, client: MusicPlayerClient<Channel>) -> Result<Self> {
        let (cmd_tx, cmd_rx) = mpsc::unbounded_channel();
        let mut model = Model::new(config, cmd_tx).await;
        model.init_config();
        let playback = Playback::new(client);
        Ok(Self {
            model,
            playback,
            cmd_rx,
        })
    }

    /// Force a redraw if [`FORCED_REDRAW_INTERVAL`] has passed.
    fn exec_interval_redraw(&mut self) {
        if self.model.since_last_redraw() >= FORCED_REDRAW_INTERVAL {
            self.model.force_redraw();
        }
    }

    /// Handle terminal init & finalize and start the UI Loop.
    pub async fn run(&mut self) -> Result<()> {
        self.model.init_terminal();

        let res = self.run_inner().await;

        // reset terminal in any case
        self.model.finalize_terminal();

        res
    }

    /// Main Loop function.
    ///
    /// This function does NOT handle initializing and finializing the terminal.
    async fn run_inner(&mut self) -> Result<()> {
        let mut stream_updates = self.playback.subscribe_to_stream_updates().await?;

        self.load_playlist().await?;

        // Main loop
        let mut progress_interval = 0;
        while !self.model.quit {
            self.model.te_update_lyric_options();
            self.model.update_outside_msg();
            if self.model.layout != TermusicLayout::Podcast {
                self.model.lyric_update();
            }
            if let Err(err) = self.handle_stream_events(&mut stream_updates) {
                self.model.mount_error_popup(err);
            }
            if progress_interval == 0 {
                self.model.run();
            }
            self.run_playback().await?;
            progress_interval += 1;
            if progress_interval >= 80 {
                progress_interval = 0;
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

            // Check whether to force redraw
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
            let mut system = System::new();
            system.refresh_all();
            for proc in system.processes().values() {
                let Some(exe) = proc.exe().map(|v| v.display().to_string()) else {
                    continue;
                };
                if exe.contains("termusic-server") {
                    #[cfg(not(target_os = "windows"))]
                    proc.kill_with(sysinfo::Signal::Term);
                    #[cfg(target_os = "windows")]
                    proc.kill();
                    break;
                }
            }
        }

        Ok(())
    }

    /// Handle running [`Status`] having possibly changed.
    fn handle_status(&mut self, new_status: Status) {
        let old_status = self.model.playlist.status();
        // nothing needs to be done as the status is the same
        if new_status == old_status {
            return;
        }

        self.model.playlist.set_status(new_status);

        match new_status {
            Status::Running => {
                // This is to show the first album photo
                if old_status == Status::Stopped {
                    self.model.player_update_current_track_after();
                }
            }
            Status::Stopped => {
                // This is to clear the photo shown when stopped
                if self.model.playlist.is_empty() {
                    self.model.player_update_current_track_after();
                }
            }
            Status::Paused => {}
        }
    }

    /// Execute a TUI-Server Request from the channel.
    async fn run_playback(&mut self) -> Result<()> {
        if let Ok(cmd) = self.cmd_rx.try_recv() {
            match cmd {
                TuiCmd::TogglePause => {
                    let status = self.playback.toggle_pause().await?;
                    self.model.playlist.set_status(status);
                    self.model.progress_update_title();
                }
                TuiCmd::SkipNext => {
                    self.playback.skip_next().await?;
                    self.model.playlist.clear_current_track();
                }
                TuiCmd::SkipPrevious => self.playback.skip_previous().await?,
                TuiCmd::GetProgress => {
                    let response = self.playback.get_progress().await?;
                    let pprogress: PlayerProgress = response.progress.unwrap_or_default().into();
                    self.model.progress_update(
                        pprogress.position,
                        pprogress.total_duration.unwrap_or_default(),
                    );
                    if response.current_track_updated {
                        self.model.handle_current_track_index(
                            usize::try_from(response.current_track_index).unwrap(),
                            false,
                        );
                    }

                    self.model.lyric_update_for_radio(response.radio_title);

                    self.handle_status(Status::from_u32(response.status));
                }

                TuiCmd::CycleLoop => {
                    let res = self.playback.cycle_loop().await?;
                    self.model.config_server.write().settings.player.loop_mode = res;
                }
                TuiCmd::ReloadConfig => self.playback.reload_config().await?,
                TuiCmd::SeekBackward => {
                    let pprogress = self.playback.seek_backward().await?;
                    self.model.progress_update(
                        pprogress.position,
                        pprogress.total_duration.unwrap_or_default(),
                    );
                    self.model.force_redraw();
                }
                TuiCmd::SeekForward => {
                    let pprogress = self.playback.seek_forward().await?;
                    self.model.progress_update(
                        pprogress.position,
                        pprogress.total_duration.unwrap_or_default(),
                    );
                    self.model.force_redraw();
                }
                TuiCmd::SpeedDown => {
                    self.model.config_server.write().settings.player.speed =
                        self.playback.speed_down().await?;
                    self.model.progress_update_title();
                }
                TuiCmd::SpeedUp => {
                    self.model.config_server.write().settings.player.speed =
                        self.playback.speed_up().await?;
                    self.model.progress_update_title();
                }
                TuiCmd::ToggleGapless => {
                    self.model.config_server.write().settings.player.gapless =
                        self.playback.toggle_gapless().await?;
                    self.model.progress_update_title();
                }
                TuiCmd::VolumeDown => {
                    let volume = self.playback.volume_down().await?;
                    self.model.config_server.write().settings.player.volume = volume;
                    self.model.progress_update_title();
                }
                TuiCmd::VolumeUp => {
                    let volume = self.playback.volume_up().await?;
                    self.model.config_server.write().settings.player.volume = volume;
                    self.model.progress_update_title();
                }
                TuiCmd::Playlist(ev) => self.run_playback_playlist(ev).await?,
            }
        }

        Ok(())
    }

    /// Handle Playlist requests.
    ///
    /// Less nesting.
    async fn run_playback_playlist(&mut self, ev: PlaylistCmd) -> Result<()> {
        match ev {
            PlaylistCmd::AddTrack(tracks) => {
                self.playback.add_to_playlist(tracks).await?;
            }
            PlaylistCmd::RemoveTrack(tracks) => {
                self.playback
                    .remove_from_playlist(PlaylistRemoveTrackType::Indexed(tracks))
                    .await?;
            }
            PlaylistCmd::Clear => {
                self.playback
                    .remove_from_playlist(PlaylistRemoveTrackType::Clear)
                    .await?;
            }
            PlaylistCmd::SwapTrack(info) => {
                self.playback.swap_tracks(info).await?;
            }
            PlaylistCmd::Shuffle => {
                self.playback.shuffle_playlist().await?;
            }
            PlaylistCmd::PlaySpecific(info) => {
                self.playback.play_specific(info).await?;
            }
            PlaylistCmd::RemoveDeletedItems => {
                self.playback.remove_deleted_tracks().await?;
            }
            PlaylistCmd::SelfReloadPlaylist => {
                self.load_playlist().await?;
            }
        }

        Ok(())
    }

    /// Handle Stream updates from the provided stream.
    ///
    /// In case of lag, sends a [`PlayerCmd::GetProgress`] to `self.model`.
    ///
    /// - Does not wait until the next event (non-blocking).
    /// - Processess *all* available events.
    fn handle_stream_events(
        &mut self,
        stream: &mut (impl Stream<Item = Result<StreamUpdates, anyhow::Error>> + std::marker::Unpin),
    ) -> Result<()> {
        while let Some(ev) = stream.next().now_or_never().flatten() {
            let ev = ev
                .map(UpdateEvents::try_from)
                .context("Conversion from StreamUpdates to UpdateEvents failed!")?;

            debug!("Stream Event: {ev:?}");

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
                    self.model.playlist.set_status(Status::from_u32(playing));
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

    /// Load the playlist from the server
    async fn load_playlist(&mut self) -> Result<()> {
        info!("Requesting Playlist from server");
        let tracks = self.playback.get_playlist().await?;
        let current_track_index = tracks.current_track_index;
        self.model
            .playlist
            .load_from_grpc(tracks, &self.model.podcast.db_podcast)?;

        self.model.playlist_sync();

        self.model
            .handle_current_track_index(usize::try_from(current_track_index).unwrap(), true);

        Ok(())
    }
}
