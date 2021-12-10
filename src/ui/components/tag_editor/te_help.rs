//! # Popups
//!
//! Popups components

/**
 * MIT License
 *
 * tuifeed - Copyright (c) 2021 Christian Visintin
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
use crate::ui::Msg;

use tui_realm_stdlib::Table;
use tuirealm::event::{Key, KeyEvent};
use tuirealm::props::{Alignment, BorderType, Borders, Color, TableBuilder, TextSpan};
use tuirealm::{Component, Event, MockComponent, NoUserEvent};

#[derive(MockComponent)]
pub struct TEHelpPopup {
    component: Table,
}

impl Default for TEHelpPopup {
    fn default() -> Self {
        Self {
            component: Table::default()
                .borders(
                    Borders::default()
                        .modifiers(BorderType::Rounded)
                        .color(Color::Green),
                )
                // .foreground(Color::Yellow)
                // .background(Color::Black)
                .title("Help: Esc or Enter to exit.", Alignment::Center)
                .scroll(false)
                // .highlighted_color(Color::LightBlue)
                // .highlighted_str("\u{1f680}")
                // .highlighted_str("🚀")
                // .rewind(true)
                .step(4)
                .row_height(1)
                .headers(&["Key", "Function"])
                .column_spacing(3)
                .widths(&[30, 70])
                .table(
                    TableBuilder::default()
                        .add_col(TextSpan::new("<ENTER>").bold().fg(Color::Cyan))
                        .add_col(TextSpan::from("Search when focus Artist or Song name."))
                        .add_row()
                        .add_col(TextSpan::new("<ESC> or <Q>").bold().fg(Color::Cyan))
                        .add_col(TextSpan::from("Exit"))
                        .add_row()
                        .add_col(TextSpan::new("<TAB>").bold().fg(Color::Cyan))
                        .add_col(TextSpan::from("Switch focus"))
                        .add_row()
                        .add_col(TextSpan::new("<h,j,k,l,g,G>").bold().fg(Color::Cyan))
                        .add_col(TextSpan::from("Move cursor(vim style)"))
                        .add_row()
                        .add_col(TextSpan::new("<ENTER/l>").bold().fg(Color::Cyan))
                        .add_col(TextSpan::from("Embed selected songtag."))
                        .add_row()
                        .add_col(TextSpan::new("<s>").bold().fg(Color::Cyan))
                        .add_col(TextSpan::from("Download selected song"))
                        .add_col(TextSpan::new("<ESC> or <q>").bold().fg(Color::Cyan))
                        .add_col(TextSpan::from("Exit"))
                        .add_row()
                        .add_col(TextSpan::new("<TAB>").bold().fg(Color::Cyan))
                        .add_col(TextSpan::from("Switch focus"))
                        .add_row()
                        .add_col(TextSpan::new("<h,j,k,l,g,G>").bold().fg(Color::Cyan))
                        .add_col(TextSpan::from("Move cursor(vim style)"))
                        .build(),
                ),
        }
    }
}

impl Component<Msg, NoUserEvent> for TEHelpPopup {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        match ev {
            Event::Keyboard(KeyEvent {
                code: Key::Enter | Key::Esc,
                ..
            }) => Some(Msg::TEHelpPopupClose),
            _ => None,
        }
    }
}
