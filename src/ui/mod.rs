//! ## Utils
//!
//! `Utils` implements utilities functions to work with layouts

// pub mod activity;
// mod activity;
/**
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
mod components;
mod model;

use crate::config::Termusic;
use model::Model;
// Let's define the messages handled by our app. NOTE: it must derive `PartialEq`
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tuirealm::application::PollStrategy;
use tuirealm::{Application, Update};
// -- internal

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
const FORCED_REDRAW_INTERVAL: Duration = Duration::from_millis(1000);

#[derive(Debug, PartialEq)]
pub enum Msg {
    // AppClose,
    DeleteConfirmCloseCancel,
    DeleteConfirmCloseOk,
    DeleteConfirmShow,
    ErrorPopupClose,
    HelpPopupShow,
    HelpPopupClose,
    LibraryTreeExtendDir(String),
    LibraryTreeGoToUpperDir,
    LibraryTreeBlur,
    LibraryYank,
    LibraryPaste,
    LibrarySearchPopupShow,
    LibrarySearchPopupCloseCancel,
    LibrarySearchInputBlur,
    LibrarySearchPopupUpdate(String),
    LibrarySearchTableBlur,
    LibrarySearchPopupCloseAddPlaylist,
    LibrarySearchPopupCloseOkLocate,
    LyricCycle,
    LyricAdjustDelay(i64),
    PlayerTogglePause,
    PlayerVolumeUp,
    PlayerVolumeDown,
    PlayerSeek(isize),
    PlaylistNextSong,
    PlaylistPrevSong,
    PlaylistTableBlur,
    PlaylistAdd(String),
    PlaylistAddSongs(String),
    PlaylistDelete(usize),
    PlaylistDeleteAll,
    PlaylistLoopModeCycle,
    PlaylistPlaySelected(usize),
    PlaylistShuffle,
    QuitPopupCloseCancel,
    QuitPopupCloseOk,
    QuitPopupShow,
    TagEditorRun(String),
    TagEditorBlur(Option<String>),
    YoutubeSearchInputPopupShow,
    YoutubeSearchInputPopupCloseCancel,
    YoutubeSearchInputPopupCloseOk(String),
    YoutubeSearchTablePopupNext,
    YoutubeSearchTablePopupPrevious,
    YoutubeSearchTablePopupCloseCancel,
    YoutubeSearchTablePopupCloseOk(usize),
    None,
}

// Let's define the component ids for our application
#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub enum Id {
    DeleteConfirmRadioPopup,
    DeleteConfirmInputPopup,
    ErrorPopup,
    GlobalListener,
    HelpPopup,
    Label,
    Library,
    LibrarySearchInput,
    LibrarySearchTable,
    Lyric,
    MessagePopup,
    Playlist,
    Progress,
    QuitPopup,
    TELabelHelp,
    TELabelHint,
    TEInputArtist,
    TEInputTitle,
    TERadioTag,
    YoutubeSearchInputPopup,
    YoutubeSearchTablePopup,
}

#[derive(Clone, Copy)]
pub enum Status {
    Running,
    Stopped,
    Paused,
}

// StatusLine shows the status of download
#[derive(Copy, Clone)]
pub enum StatusLine {
    Default,
    Success,
    Running,
    Error,
}

#[derive(Clone, Deserialize, Serialize)]
pub enum Loop {
    Single,
    Playlist,
    Queue,
}

impl std::fmt::Display for Loop {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let loop_state = match self {
            Self::Single => "single loop",
            Self::Playlist => "loop",
            Self::Queue => "queue",
        };
        write!(f, "{}", loop_state)
    }
}

pub struct UI {
    model: Model,
}

impl UI {
    /// Instantiates a new Ui
    pub fn new(config: &Termusic) -> Self {
        let model = Model::new(config);
        // let app = Self::init_application(&model, tick);
        Self { model }
    }
    /// ### run
    ///
    /// Main loop for Ui thread
    pub fn run(&mut self) {
        self.model.init_terminal();
        assert!(self.model.playlist_load().is_ok());
        self.model.playlist_sync();
        // Main loop
        while !self.model.quit {
            // if let Err(err) = self.app.tick(&mut self.model, PollStrategy::UpTo(3)) {
            //     self.mount_error_popup(format!("Application error: {}", err));
            // }
            match self.model.app.tick(PollStrategy::Once) {
                Err(err) => {
                    self.model
                        .mount_error_popup(format!("Application error: {}", err).as_str());
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
            self.model.update_playlist_items();
            match self.model.status {
                Some(Status::Stopped) => {
                    if self.model.playlist_items.is_empty() {
                        continue;
                    }
                    self.model.status = Some(Status::Running);
                    self.model.player_next();
                }
                None => self.model.status = Some(Status::Stopped),
                Some(Status::Running | Status::Paused) => {}
            }
            #[cfg(feature = "mpris")]
            self.model.update_mpris();

            self.model.progress_update();
            self.model.update_lyric();
            self.model.update_components();
            // sleep(Duration::from_millis(20));
        }
        assert!(self.model.playlist_save().is_ok());
        assert!(self.model.config.save().is_ok());

        self.model.finalize_terminal();
    }

    fn check_force_redraw(&mut self) {
        // If source are loading and at least 100ms has elapsed since last redraw...
        if let Some(Status::Running) = self.model.status {
            if self.model.since_last_redraw() >= FORCED_REDRAW_INTERVAL {
                self.model.force_redraw();
            }
        }
    }
}
