use super::inputhandler::InputHandler;
use crossterm::event::DisableMouseCapture;
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use std::io::{stdout, Stdout};
use tui::backend::CrosstermBackend;
use tui::Terminal as TuiTerminal;

pub struct Context {
    pub context: TuiTerminal<CrosstermBackend<Stdout>>,
    pub(crate) input_hnd: InputHandler,
}

impl Context {
    pub fn new() -> Self {
        let _ = enable_raw_mode();
        // Create terminal
        let mut stdout = stdout();
        assert!(execute!(stdout, EnterAlternateScreen).is_ok());
        Self {
            input_hnd: InputHandler::new(),
            context: TuiTerminal::new(CrosstermBackend::new(stdout)).unwrap(),
        }
    }

    pub fn enter_alternate_screen(&mut self) {
        let _ = execute!(
            self.context.backend_mut(),
            EnterAlternateScreen,
            DisableMouseCapture
        );
    }

    pub fn leave_alternate_screen(&mut self) {
        let _ = execute!(
            self.context.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        );
    }

    pub fn clear_screen(&mut self) {
        let _ = self.context.clear();
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        // Re-enable terminal stuff
        self.leave_alternate_screen();
        let _ = disable_raw_mode();
    }
}
