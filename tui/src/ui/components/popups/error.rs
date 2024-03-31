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
use termusiclib::types::{Id, Msg};
use tui_realm_stdlib::Paragraph;
use tuirealm::{
    event::{Key, KeyEvent},
    props::{Alignment, BorderType, Borders, Color, TextModifiers, TextSpan},
    Component, Event, MockComponent, NoUserEvent,
};

use crate::ui::model::Model;

#[derive(MockComponent)]
pub struct ErrorPopup {
    component: Paragraph,
}

impl ErrorPopup {
    pub fn new<E: Into<anyhow::Error>>(msg: E) -> Self {
        let msg = msg.into();
        error!("Displaying error popup: {msg:?}");
        // TODO: Consider changing to ":?" to output "Caused By" (and possibly backtrace) OR do a custom printing (copied from anyhow) once more than 4 lines can be displayed in height
        let msg = format!("{msg:#}");
        Self {
            component: Paragraph::default()
                .borders(
                    Borders::default()
                        .color(Color::Red)
                        .modifiers(BorderType::Rounded),
                )
                .title(" Error ", Alignment::Center)
                .foreground(Color::Red)
                // .background(Color::Black)
                .modifiers(TextModifiers::BOLD)
                .alignment(Alignment::Center)
                .text(&[TextSpan::from(msg)]/* &msg.lines().map(|v| TextSpan::from(v)).collect::<Vec<_>>() */),
        }
    }
}

impl Component<Msg, NoUserEvent> for ErrorPopup {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        match ev {
            Event::Keyboard(KeyEvent {
                code: Key::Enter | Key::Esc,
                ..
            }) => Some(Msg::ErrorPopupClose),
            _ => None,
        }
    }
}

impl Model {
    /// Mount error and give focus to it
    // This should likely be refactored to be "std::error::Error", but see https://github.com/dtolnay/anyhow/issues/63 on why it was easier this way
    pub fn mount_error_popup<E: Into<anyhow::Error>>(&mut self, err: E) {
        assert!(self
            .app
            .remount(Id::ErrorPopup, Box::new(ErrorPopup::new(err)), vec![])
            .is_ok());
        assert!(self.app.active(&Id::ErrorPopup).is_ok());
    }

    pub fn umount_error_popup(&mut self) {
        self.app.umount(&Id::ErrorPopup).ok();
    }
}
