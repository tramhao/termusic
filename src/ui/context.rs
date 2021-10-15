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
// use super::inputhandler::InputHandler;
use crossterm::event::DisableMouseCapture;
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use std::io::{stdout, Stdout, Write};
use tuirealm::tui::backend::CrosstermBackend;
use tuirealm::tui::Terminal as TuiTerminal;

pub struct Context {
    pub context: TuiTerminal<CrosstermBackend<Stdout>>,
    // pub(crate) input_hnd: InputHandler,
}

impl Context {
    pub fn new() -> Self {
        let _drop = enable_raw_mode();
        // Create terminal
        let mut stdout = stdout();
        assert!(execute!(stdout, EnterAlternateScreen).is_ok());
        let ctx = match TuiTerminal::new(CrosstermBackend::new(stdout)) {
            Ok(c) => c,
            Err(e) => panic!("error when initializing terminal:{}", e.to_string()),
        };

        Self {
            // input_hnd: InputHandler::new(),
            context: ctx,
        }
    }

    pub fn enter_alternate_screen(&mut self) {
        let _drop = execute!(
            self.context.backend_mut(),
            EnterAlternateScreen,
            DisableMouseCapture
        );
    }

    pub fn leave_alternate_screen(&mut self) {
        let _drop = execute!(
            self.context.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        );
    }

    pub fn clear_screen(&mut self) {
        let _drop = self.context.clear();
    }

    pub fn clear_image(&mut self) {
        if write!(self.context.backend_mut(), "\x1b_Ga=d\x1b\\").is_ok()
            && self.context.backend_mut().flush().is_ok()
        {}
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        // Re-enable terminal stuff
        self.leave_alternate_screen();
        let _drop = disable_raw_mode();
    }
}
