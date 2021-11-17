//! ## Components
//!
//! demo example components

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
use crate::Msg;

use tuirealm::props::{Alignment, Borders, Color, Style};

use tuirealm::tui::widgets::Block;

// -- modules
// mod clock;
mod counter;
mod label;
mod music_library;

// -- export
// pub use clock::Clock;
pub use counter::{Digit, Letter};
pub use label::Label;
pub use music_library::MusicLibrary;

///
/// Get block
pub fn get_block<'a>(props: &Borders, title: (String, Alignment), focus: bool) -> Block<'a> {
    Block::default()
        .borders(props.sides)
        .border_style(if focus {
            props.style()
        } else {
            Style::default().fg(Color::Reset).bg(Color::Reset)
        })
        .border_type(props.modifiers)
        .title(title.0)
        .title_alignment(title.1)
}
