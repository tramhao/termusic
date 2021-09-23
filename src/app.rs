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
use super::ui::{
    activity::{main::TermusicActivity, Activity, ExitReason},
    context::Context,
};
use crate::config::Termusic;
#[cfg(feature = "mpris")]
use crate::souvlaki::mpris_handler;
use log::error;
use std::thread::sleep;
use std::time::Duration;
use std::time::Instant;

pub struct App {
    pub config: Termusic,
    pub quit: bool,           // Becomes true when the user presses <ESC>
    pub redraw: bool,         // Tells whether to refresh the UI; performance optimization
    pub last_redraw: Instant, // Last time the ui has been redrawed
    pub context: Option<Context>,
}

impl App {
    pub fn new(config: Termusic) -> Self {
        let mut ctx: Context = Context::new();
        // Enter alternate screen
        ctx.enter_alternate_screen();
        // Clear screen
        ctx.clear_screen();

        Self {
            config,
            quit: false,
            redraw: true,
            last_redraw: Instant::now(),
            context: Some(ctx),
        }
    }

    pub fn run(&mut self) {
        let mut main_activity: TermusicActivity = TermusicActivity::default();
        // Get context
        let ctx: Context = if let Some(ctx) = self.context.take() {
            ctx
        } else {
            error!("Failed to start MainActivity: context is None");
            return;
        };
        // Create activity
        main_activity.init_config(&self.config);
        main_activity.on_create(ctx);
        let mut progress_interval = 0;
        loop {
            main_activity.update_message();
            main_activity.update_queue_items();
            if progress_interval == 0 {
                main_activity.update_progress();
                main_activity.run();
                main_activity.update_components();
                // if let Ok(m) = main_activity.player.dbus_mpris.next() {
                //     mpris_handler(m, &mut main_activity);
                // }
                #[cfg(feature = "mpris")]
                if let Ok(m) = main_activity.player.rx.try_recv() {
                    mpris_handler(m, &mut main_activity);
                }
            }
            progress_interval += 1;
            if progress_interval >= 8 {
                progress_interval = 0;
            }

            // Draw activity
            main_activity.on_draw();
            // Check if activity has terminated
            if let Some(ExitReason::Quit) = main_activity.will_umount() {
                // info!("SetupActivity terminated due to 'Quit'");
                break;
            }
            // Sleep for ticks
            sleep(Duration::from_millis(20));
        }
        // Destroy activity
        self.context = main_activity.on_destroy();

        drop(self.context.take());
    }
}
