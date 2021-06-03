use std::time::Instant;

pub struct App {
    pub quit: bool,           // Becomes true when the user presses <ESC>
    pub redraw: bool,         // Tells whether to refresh the UI; performance optimization
    pub last_redraw: Instant, // Last time the ui has been redrawed
}

impl App {
    pub fn new() -> Self {
        App {
            quit: false,
            redraw: true,
            last_redraw: Instant::now(),
        }
    }

    pub fn quit(&mut self) {
        self.quit = true;
    }

    pub fn redraw(&mut self) {
        self.redraw = true;
    }

    pub fn reset(&mut self) {
        self.redraw = false;
        self.last_redraw = Instant::now();
    }
}
