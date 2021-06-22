//! ## MainActivity
//!
//! `main_activity` is the module which implements the Main activity, which is the activity to
//! work on termusic app

mod playlist;
mod queue;
/**
 * MIT License
 *
 * termscp - Copyright (c) 2021 Christian Visintin
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
// Submodules
// mod actions;
// mod config;
mod update;
mod view;

// Locals
// use super::super::super::player::Player;
use super::{Activity, Context, ExitReason, Status};
use crate::player::AudioPlayer;
use crate::song::Song;
use crate::MUSIC_DIR;
// Ext
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use log::error;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use tui_realm_treeview::Tree;
use tuirealm::View;
// tui

// -- components
const COMPONENT_LABEL_HELP: &str = "LABEL_HELP";
const COMPONENT_PARAGRAPH_LYRIC: &str = "PARAGRAPH_LYRIC";
const COMPONENT_SCROLLTABLE: &str = "SCROLLTABLE";
const COMPONENT_TREEVIEW: &str = "TREEVIEW";
const COMPONENT_PROGRESS: &str = "PROGRESS";
const COMPONENT_TEXT_HELP: &str = "TEXT_HELP";
const COMPONENT_INPUT_URL: &str = "INPUT_URL";
const COMPONENT_TEXT_ERROR: &str = "TEXT_ERROR";

/// ### ViewLayout
///

/// ## MainActivity
///
/// Main activity states holder
pub struct MainActivity {
    exit_reason: Option<ExitReason>,
    context: Option<Context>, // Context holder
    view: View,               // View
    redraw: bool,
    path: PathBuf,
    tree: Tree,
    player: AudioPlayer,
    queue_items: Vec<Song>,
    time_pos: i64,
    status: Option<Status>,
    current_song: Option<Song>,
    sender: Sender<TransferState>,
    receiver: Receiver<TransferState>,
}

// TransferState is used to describe the status of download
pub enum TransferState {
    Running, // indicates progress
    Completed,
    ErrDownload,
}

impl Default for MainActivity {
    fn default() -> Self {
        // Initialize user input
        let mut user_input_buffer: Vec<String> = Vec::with_capacity(16);
        for _ in 0..16 {
            user_input_buffer.push(String::new());
        }

        let full_path = shellexpand::tilde(MUSIC_DIR);
        let p: &Path = Path::new(full_path.as_ref());
        let (tx, rx): (Sender<TransferState>, Receiver<TransferState>) = mpsc::channel();
        MainActivity {
            exit_reason: None,
            context: None,
            view: View::init(),
            redraw: true, // Draw at first `on_draw`
            tree: Tree::new(Self::dir_tree(p, 3)),
            path: p.to_path_buf(),
            player: AudioPlayer::new(),
            queue_items: vec![],
            time_pos: 0,
            status: None,
            current_song: None,
            sender: tx,
            receiver: rx,
        }
    }
}

impl MainActivity {
    pub fn run(&mut self) {
        match self.status {
            Some(Status::Stopped) => {
                if self.queue_items.len() < 1 {
                    return;
                }
                self.status = Some(Status::Running);
                let song = self.queue_items.remove(0);
                self.player.queue_and_play(song.clone());
                self.current_song = Some(song.clone());
                self.queue_items.push(song);
                self.sync_items();
                self.update_photo();
            }
            Some(Status::Running) => {}
            Some(Status::Paused) => {}
            None => return,
        };
    }
}

impl Activity for MainActivity {
    /// ### on_create
    ///
    /// `on_create` is the function which must be called to initialize the activity.
    /// `on_create` must initialize all the data structures used by the activity
    /// Context is taken from activity manager and will be released only when activity is destroyed
    fn on_create(&mut self, context: Context) {
        // Set context
        self.context = Some(context);
        // // Clear terminal
        self.context.as_mut().unwrap().clear_screen();
        // // Put raw mode on enabled
        if let Err(err) = enable_raw_mode() {
            error!("Failed to enter raw mode: {}", err);
        }
        // // Init view
        self.init_setup();

        if let Err(err) = self.load_queue() {
            error!("Failed to save queue: {}", err);
        }
        self.status = Some(Status::Stopped);
        // // Verify error state from context
        // if let Some(err) = self.context.as_mut().unwrap().get_error() {
        //     self.mount_error(err.as_str());
        // }
    }

    /// ### on_draw
    ///
    /// `on_draw` is the function which draws the graphical interface.
    /// This function must be called at each tick to refresh the interface
    fn on_draw(&mut self) {
        // Context must be something
        if self.context.is_none() {
            return;
        }
        // Read one event
        if let Ok(Some(event)) = self.context.as_ref().unwrap().input_hnd.read_event() {
            // Set redraw to true
            self.redraw = true;
            // Handle event
            let msg = self.view.on(event);
            self.update(msg);
        }
        // Redraw if necessary
        if self.redraw {
            // View
            self.view();
            // Redraw back to false
            self.redraw = false;
        }
    }

    /// ### will_umount
    ///
    /// `will_umount` is the method which must be able to report to the activity manager, whether
    /// the activity should be terminated or not.
    /// If not, the call will return `None`, otherwise return`Some(ExitReason)`
    fn will_umount(&self) -> Option<&ExitReason> {
        self.exit_reason.as_ref()
    }

    /// ### on_destroy
    ///
    /// `on_destroy` is the function which cleans up runtime variables and data before terminating the activity.
    /// This function must be called once before terminating the activity.
    /// This function finally releases the context
    fn on_destroy(&mut self) -> Option<Context> {
        if let Err(err) = self.save_queue() {
            error!("Failed to save queue: {}", err);
        }
        // Disable raw mode
        if let Err(err) = disable_raw_mode() {
            error!("Failed to disable raw mode: {}", err);
        }
        self.context.as_ref()?;
        // Clear terminal and return
        match self.context.take() {
            Some(mut ctx) => {
                ctx.clear_screen();
                Some(ctx)
            }
            None => None,
        }
    }
}
