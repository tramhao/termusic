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
use super::ui::activity::main::MainActivity;
use super::ui::activity::{Activity, ExitReason};
use super::ui::context::Context;
use crate::config::TermusicConfig;
use std::time::Instant;

use log::error;
use std::thread::sleep;
use std::time::Duration;

pub struct App {
    pub config: TermusicConfig,
    pub quit: bool,           // Becomes true when the user presses <ESC>
    pub redraw: bool,         // Tells whether to refresh the UI; performance optimization
    pub last_redraw: Instant, // Last time the ui has been redrawed
    pub context: Option<Context>,
}

impl App {
    pub fn new(config: TermusicConfig) -> Self {
        let mut ctx: Context = Context::new();
        // Enter alternate screen
        ctx.enter_alternate_screen();
        // Clear screen
        ctx.clear_screen();

        App {
            config,
            quit: false,
            redraw: true,
            last_redraw: Instant::now(),
            context: Some(ctx),
        }
    }

    pub fn run(&mut self) {
        let mut activity: MainActivity = MainActivity::default();
        // Get context
        let ctx: Context = match self.context.take() {
            Some(ctx) => ctx,
            None => {
                error!("Failed to start MainActivity: context is None");
                return;
            }
        };
        // Create activity
        activity.init_config(self.config.to_owned());
        activity.on_create(ctx);
        let mut progress_interval = 0;
        loop {
            if progress_interval == 0 {
                activity.update_progress();
                activity.run();
                activity.update_download_progress();
            }
            progress_interval += 1;
            if progress_interval >= 8 {
                progress_interval = 0
            }

            // Draw activity
            activity.on_draw();
            // Check if activity has terminated
            if let Some(ExitReason::Quit) = activity.will_umount() {
                // info!("SetupActivity terminated due to 'Quit'");
                break;
            }
            // Sleep for ticks
            sleep(Duration::from_millis(20));
        }
        // Destroy activity
        self.context = activity.on_destroy();

        drop(self.context.take());
    }
}
