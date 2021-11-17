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
use std::io::{Stdout, Write};
use tuirealm::terminal::TerminalBridge;
use tuirealm::tui::backend::CrosstermBackend;
use tuirealm::tui::Terminal as TuiTerminal;

pub struct Context {
    pub context: TuiTerminal<CrosstermBackend<Stdout>>,
}

impl Context {
    pub fn new() -> Self {
        Self {
            context: TerminalBridge::new().expect("Could not initialize terminal"),
        }
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
