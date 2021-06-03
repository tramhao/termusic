//! ## Context
//!
//! `Context` is the module which provides all the functionalities related to the UI data holder, called Context

/**
 * MIT License
 *
 * tui-realm - Copyright (C) 2021 Christian Visintin
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
// Dependencies
use super::input::InputHandler;

// Includes
use crossterm::event::DisableMouseCapture;
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use std::io::{stdout, Stdout};
use tui::backend::CrosstermBackend;
use tui::Terminal;

/// ## Context
///
/// Context holds data structures used by the ui
pub struct Context {
    pub(crate) input_hnd: InputHandler,
    pub(crate) terminal: Terminal<CrosstermBackend<Stdout>>,
}

impl Context {
    /// ### new
    ///
    /// Instantiates a new Context
    pub fn new() -> Context {
        let _ = enable_raw_mode();
        // Create terminal
        let mut stdout = stdout();
        assert!(execute!(stdout, EnterAlternateScreen).is_ok());
        Context {
            input_hnd: InputHandler::new(),
            terminal: Terminal::new(CrosstermBackend::new(stdout)).unwrap(),
        }
    }

    /// ### enter_alternate_screen
    ///
    /// Enter alternate screen (gui window)
    pub fn enter_alternate_screen(&mut self) {
        let _ = execute!(
            self.terminal.backend_mut(),
            EnterAlternateScreen,
            DisableMouseCapture
        );
    }

    /// ### leave_alternate_screen
    ///
    /// Go back to normal screen (gui window)
    pub fn leave_alternate_screen(&mut self) {
        let _ = execute!(
            self.terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        );
    }

    /// ### clear_screen
    ///
    /// Clear terminal screen
    pub fn clear_screen(&mut self) {
        let _ = self.terminal.clear();
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        // Re-enable terminal stuff
        self.leave_alternate_screen();
        let _ = disable_raw_mode();
    }
}
