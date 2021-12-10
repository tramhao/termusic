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
pub mod components;
pub mod model;

use crate::config::Termusic;
use crate::songtag::SongTag;
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
    GeneralSearchPopupShowLibrary,
    GeneralSearchPopupShowPlaylist,
    GeneralSearchPopupCloseCancel,
    GeneralSearchInputBlur,
    GeneralSearchPopupUpdateLibrary(String),
    GeneralSearchPopupUpdatePlaylist(String),
    GeneralSearchTableBlur,
    GeneralSearchPopupCloseLibraryAddPlaylist,
    GeneralSearchPopupCloseOkLibraryLocate,
    GeneralSearchPopupClosePlaylistPlaySelected,
    GeneralSearchPopupCloseOkPlaylistLocate,
    LyricCycle,
    LyricAdjustDelay(i64),
    PlayerTogglePause,
    PlayerVolumeUp,
    PlayerVolumeDown,
    PlayerSeek(isize),
    PlaylistAddFront,
    PlaylistNextSong,
    PlaylistPrevSong,
    PlaylistTableBlur,
    PlaylistAdd(String),
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
    TECounterDeleteBlur,
    TECounterDeleteOk,
    TEDownload(usize),
    TEEmbed(usize),
    TEHelpPopupShow,
    TEHelpPopupClose,
    TEInputArtistBlur,
    TEInputTitleBlur,
    TERadioTagBlur,
    TERadioTagOk,
    TESearch,
    TESelectLyricBlur,
    TESelectLyricOk(usize),
    TETableLyricOptionsBlur,
    TETextareaLyricBlur,
    ThemeSelectShow,
    ThemeSelectCloseCancel,
    ThemeSelectCloseOk,
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
    GeneralSearchInput,
    GeneralSearchTable,
    Lyric,
    MessagePopup,
    Playlist,
    Progress,
    QuitPopup,
    TECounterDelete,
    TEHelpPopup,
    TELabelHint,
    TEInputArtist,
    TEInputTitle,
    TERadioTag,
    TESelectLyric,
    TETableLyricOptions,
    TETextareaLyric,
    ThemeSelect,
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

pub enum SearchLyricState {
    Finish(Vec<SongTag>),
}

#[allow(clippy::non_ascii_literal)]
impl std::fmt::Display for Loop {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let loop_state = match self {
            Self::Single => "🔂",
            Self::Playlist => "🔁",
            Self::Queue => "⬇",
            // Self::Single => "single",
            // Self::Playlist => "playlist",
            // Self::Queue => "consume",
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
        let mut model = Model::new(config);
        model.init_config();
        model.library_reload_tree();
        Self { model }
    }
    /// ### run
    ///
    /// Main loop for Ui thread
    pub fn run(&mut self) {
        self.model.init_terminal();
        self.model.playlist_load().ok();
        // Main loop
        let mut progress_interval = 0;
        while !self.model.quit {
            #[cfg(feature = "mpris")]
            self.model.update_mpris();

            self.model.te_update_lyric_options();
            self.model.update_playlist_items();
            self.model.update_components();
            self.model.progress_update();
            self.model.update_lyric();

            if progress_interval == 0 {
                self.model.run();
            }
            progress_interval += 1;
            if progress_interval >= 8 {
                progress_interval = 0;
            }

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
            // sleep(Duration::from_millis(20));
        }
        assert!(self.model.playlist_save().is_ok());
        assert!(self.model.config.save().is_ok());
        assert!(self.model.clear_photo().is_ok());

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
