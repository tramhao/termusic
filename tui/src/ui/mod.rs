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
mod playback;

use anyhow::Result;
use model::{Model, TermusicLayout};
use playback::Playback;
use std::time::Duration;
use sysinfo::{ProcessExt, System, SystemExt};
use termusiclib::config::Settings;
pub use termusiclib::types::*;
use termusicplayback::{PlayerCmd, Status};
use tokio::sync::mpsc::{self, UnboundedReceiver};
use tuirealm::application::PollStrategy;
use tuirealm::{Application, Update};
// -- internal

const FORCED_REDRAW_INTERVAL: Duration = Duration::from_millis(1000);

// Let's define the messages handled by our app. NOTE: it must derive `PartialEq`

// Let's define the component ids for our application
pub struct UI {
    model: Model,
    playback: Playback,
    cmd_rx: UnboundedReceiver<PlayerCmd>,
}

impl UI {
    fn check_force_redraw(&mut self) {
        // If source are loading and at least 100ms has elapsed since last redraw...
        // if self.model.status == Status::Running {
        if self.model.since_last_redraw() >= FORCED_REDRAW_INTERVAL {
            self.model.force_redraw();
        }
        // }
    }
    /// Instantiates a new Ui
    pub async fn new(config: &Settings) -> Result<Self> {
        let (cmd_tx, cmd_rx) = mpsc::unbounded_channel();
        let mut model = Model::new(config, cmd_tx);
        model.init_config();
        let playback = Playback::new(config.player_port).await?;
        Ok(Self {
            model,
            playback,
            cmd_rx,
        })
    }

    /// ### run
    ///
    /// Main loop for Ui thread
    pub async fn run(&mut self) -> Result<()> {
        self.model.init_terminal();
        // Main loop
        let mut progress_interval = 0;
        while !self.model.quit {
            self.model.te_update_lyric_options();
            // self.model.update_player_msg();
            self.model.update_outside_msg();
            if self.model.layout != TermusicLayout::Podcast {
                self.model.lyric_update();
            }
            if progress_interval == 0 {
                self.model.run();
            }
            self.run_playback().await?;
            progress_interval += 1;
            if progress_interval >= 80 {
                progress_interval = 0;
            }

            match self.model.app.tick(PollStrategy::Once) {
                Err(err) => {
                    self.model
                        .mount_error_popup(format!("Application error: {err}"));
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
            // Check whether to force redraw
            self.check_force_redraw();
            self.model.view();
            // sleep(Duration::from_millis(20));
        }

        if let Err(e) = self.model.playlist.save() {
            eprintln!("error when saving playlist: {e}");
        };
        if let Err(e) = self.model.config.save() {
            eprintln!("error when saving config: {e}");
        };
        if self.model.config.kill_daemon_when_quit {
            let mut system = System::new();
            system.refresh_all();
            for proc in system.processes().values() {
                let exe = proc.exe().display().to_string();
                if exe.contains("termusic-server") {
                    proc.kill();
                    break;
                }
            }
        }

        self.model.finalize_terminal();
        Ok(())
    }

    fn handle_current_track_index(&mut self, current_track_index: usize) {
        info!(
            "index from player is:{current_track_index:?}, index in tui is:{:?}",
            self.model.playlist.get_current_track_index()
        );
        self.model.playlist.clear_current_track();
        self.model
            .playlist
            .set_current_track_index(current_track_index);
        self.model.update_layout_for_current_track();
        self.model.player_update_current_track_after();

        self.model.lyric_update_for_podcast_by_current_track();

        if let Err(e) = self.model.podcast_mark_current_track_played() {
            self.model
                .mount_error_popup(format!("Error when mark episode as played: {e}"));
        }
    }

    fn handle_status(&mut self, status: Status) {
        match status {
            Status::Running => match self.model.playlist.status() {
                Status::Running => {}
                Status::Stopped => {
                    self.model.playlist.set_status(status);
                    // This is to show the first album photo
                    self.model.player_update_current_track_after();
                }
                Status::Paused => {
                    self.model.playlist.set_status(status);
                }
            },
            Status::Stopped => match self.model.playlist.status() {
                Status::Running | Status::Paused => {
                    self.model.playlist.set_status(status);
                    // This is to clear the photo shown when stopped
                    if self.model.playlist.is_empty() {
                        self.model.player_update_current_track_after();
                    }
                }
                Status::Stopped => {}
            },
            Status::Paused => match self.model.playlist.status() {
                Status::Running | Status::Stopped => {
                    self.model.playlist.set_status(status);
                }
                Status::Paused => {}
            },
        }
    }

    async fn run_playback(&mut self) -> Result<()> {
        if let Ok(cmd) = self.cmd_rx.try_recv() {
            match cmd {
                PlayerCmd::TogglePause => {
                    let status = self.playback.toggle_pause().await?;
                    self.model.playlist.set_status(status);
                    self.model.progress_update_title();
                }
                PlayerCmd::SkipNext => {
                    self.playback.skip_next().await?;
                    self.model.playlist.clear_current_track();
                }
                PlayerCmd::GetProgress => {
                    let response = self.playback.get_progress().await?;
                    self.model.progress_update(
                        i64::from(response.position),
                        i64::from(response.duration),
                    );
                    if response.current_track_updated {
                        self.handle_current_track_index(response.current_track_index as usize);
                    }

                    self.model.lyric_update_for_radio(&response.radio_title);

                    self.handle_status(Status::from_u32(response.status));
                }

                PlayerCmd::CycleLoop => self.playback.cycle_loop().await?,
                PlayerCmd::PlaySelected => {
                    self.playback.play_selected().await?;
                    // self.model.playlist.clear_current_track();
                    // This line is required to show current playing message
                    // self.model.playlist.set_current_track_index(None);
                }
                PlayerCmd::SkipPrevious => self.playback.skip_previous().await?,
                PlayerCmd::ReloadConfig => self.playback.reload_config().await?,
                PlayerCmd::ReloadPlaylist => self.playback.reload_playlist().await?,
                PlayerCmd::SeekBackward => {
                    let (position, duration) = self.playback.seek_backward().await?;
                    self.model
                        .progress_update(i64::from(position), i64::from(duration));
                    self.model.force_redraw();
                }
                PlayerCmd::SeekForward => {
                    let (position, duration) = self.playback.seek_forward().await?;
                    self.model
                        .progress_update(i64::from(position), i64::from(duration));
                    self.model.force_redraw();
                }
                PlayerCmd::SpeedDown => {
                    self.model.config.player_speed = self.playback.speed_down().await?;
                    self.model.progress_update_title();
                }
                PlayerCmd::SpeedUp => {
                    self.model.config.player_speed = self.playback.speed_up().await?;
                    self.model.progress_update_title();
                }
                PlayerCmd::ToggleGapless => {
                    self.model.config.player_gapless = self.playback.toggle_gapless().await?;
                    self.model.progress_update_title();
                }
                PlayerCmd::VolumeDown => {
                    let volume = self.playback.volume_down().await?;
                    self.model.config.player_volume = volume;
                    self.model.progress_update_title();
                }
                PlayerCmd::VolumeUp => {
                    let volume = self.playback.volume_up().await?;
                    self.model.config.player_volume = volume;
                    self.model.progress_update_title();
                }
                _ => {}
            }
        }
        Ok(())
    }
}
