use super::ui::context::Context;
use super::ui::inputhandler::InputHandler;
use std::time::Instant;

use std::thread::sleep;
use std::time::Duration;

// tui

pub struct App {
    pub quit: bool,           // Becomes true when the user presses <ESC>
    pub redraw: bool,         // Tells whether to refresh the UI; performance optimization
    pub last_redraw: Instant, // Last time the ui has been redrawed
    pub context: Option<Context>,
}

impl App {
    pub fn new() -> Self {
        let mut ctx: Context = Context::new();
        // Enter alternate screen
        ctx.enter_alternate_screen();
        // Clear screen
        ctx.clear_screen();

        App {
            quit: false,
            redraw: true,
            last_redraw: Instant::now(),
            context: Some(ctx),
        }
    }

    pub fn run(&mut self) {
        let input: InputHandler = InputHandler::new();
        while !self.quit {
            // Listen for input events
            if let Ok(Some(ev)) = input.read_event() {
                // Pass event to view
                // let msg = self.view.on(ev);
                self.redraw();
                // Call the elm friend update
                // self.update(msg);
            }
            // If redraw, draw interface
            if self.redraw || self.last_redraw.elapsed() > Duration::from_millis(50) {
                // Call the elm friend vie1 function
                // self.view();
                self.reset();
            }
            sleep(Duration::from_millis(10));
        }

        drop(self.context.take());
    }

    fn quit(&mut self) {
        self.quit = true;
    }

    fn redraw(&mut self) {
        self.redraw = true;
    }

    fn reset(&mut self) {
        self.redraw = false;
        self.last_redraw = Instant::now();
    }
}
