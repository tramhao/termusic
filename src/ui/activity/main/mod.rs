//! ## MainActivity
//!
//! `setup_activity` is the module which implements the Setup activity, which is the activity to
//! work on termscp configuration

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
use super::{Activity, Context, ExitReason};
// Ext
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use log::error;
use tuirealm::View;
// tui

// -- components
const COMPONENT_INPUT: &str = "INPUT";
const COMPONENT_LABEL: &str = "LABEL";
const COMPONENT_SCROLLTABLE: &str = "SCROLLTABLE";

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
}

impl Default for MainActivity {
    fn default() -> Self {
        // Initialize user input
        let mut user_input_buffer: Vec<String> = Vec::with_capacity(16);
        for _ in 0..16 {
            user_input_buffer.push(String::new());
        }
        MainActivity {
            exit_reason: None,
            context: None,
            view: View::init(),
            redraw: true, // Draw at first `on_draw`
        }
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
