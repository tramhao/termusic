//! ## TagEditorActivity
//!
//! `tageditor_activity` is the module which implements the Tageditor activity, which is the activity to
//! edit tag and fetch lyrics

mod lyric_options;
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
// Submodules
mod update;
mod view;

// Locals
use crate::lyric::SongTag;
use crate::song::Song;
// use super::super::super::player::Player;
use super::{Activity, Context, ExitReason};
// Ext
use super::main::TransferState;
use crossterm::terminal::enable_raw_mode;
use log::error;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use tuirealm::View;

// -- components
const COMPONENT_TE_LABEL_HELP: &str = "LABEL_TE_HELP";
const COMPONENT_TE_TEXT_HELP: &str = "TEXT_TE_HELP";
const COMPONENT_TE_TEXT_ERROR: &str = "TEXT_TE_ERROR";
const COMPONENT_TE_INPUT_ARTIST: &str = "INPUT_TE_ARTIST";
const COMPONENT_TE_INPUT_SONGNAME: &str = "INPUT_TE_SONGNAME";
const COMPONENT_TE_RADIO_TAG: &str = "RADIO_TE_TAG";
const COMPONENT_TE_SCROLLTABLE_OPTIONS: &str = "SCROLLTABLE_TE_OPTIONS";
const COMPONENT_TE_TEXTAREA_LYRIC: &str = "TEXTAREA_TE_LYRIC";

/// ### ViewLayout
///

/// ## TagEditorActivity
///
/// TagEditor activity states holder
pub struct TagEditorActivity {
    exit_reason: Option<ExitReason>,
    context: Option<Context>, // Context holder
    view: View,               // View
    redraw: bool,
    song: Option<Song>,
    lyric_options: Vec<SongTag>,
    sender: Sender<TransferState>,
    receiver: Receiver<TransferState>,
}

impl Default for TagEditorActivity {
    fn default() -> Self {
        // Initialize user input
        let mut user_input_buffer: Vec<String> = Vec::with_capacity(16);
        for _ in 0..16 {
            user_input_buffer.push(String::new());
        }
        let (tx, rx): (Sender<TransferState>, Receiver<TransferState>) = mpsc::channel();

        TagEditorActivity {
            exit_reason: None,
            context: None,
            view: View::init(),
            redraw: true, // Draw at first `on_draw`
            song: None,
            lyric_options: vec![],
            sender: tx,
            receiver: rx,
        }
    }
}

impl TagEditorActivity {
    // pub fn run(&mut self) {}
}

impl Activity for TagEditorActivity {
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
        // Disable raw mode
        // if let Err(err) = disable_raw_mode() {
        //     error!("Failed to disable raw mode: {}", err);
        // }
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
