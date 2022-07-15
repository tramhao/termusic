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
use super::Msg;
use crate::config::Settings;
use tui_realm_stdlib::{Label, Span};
use tuirealm::event::NoUserEvent;
use tuirealm::props::{Alignment, Color, TextModifiers, TextSpan};
use tuirealm::{Component, Event, MockComponent};

#[derive(MockComponent)]
pub struct LabelGeneric {
    component: Label,
}

impl LabelGeneric {
    pub fn new(config: &Settings, text: &str) -> Self {
        Self {
            component: Label::default()
                .text(text)
                .alignment(Alignment::Left)
                .background(
                    config
                        .style_color_symbol
                        .library_background()
                        .unwrap_or(Color::Reset),
                )
                .foreground(
                    config
                        .style_color_symbol
                        .library_highlight()
                        .unwrap_or(Color::Cyan),
                )
                .modifiers(TextModifiers::BOLD),
        }
    }
}

impl Component<Msg, NoUserEvent> for LabelGeneric {
    fn on(&mut self, _ev: Event<NoUserEvent>) -> Option<Msg> {
        None
    }
}

#[derive(MockComponent)]
pub struct LabelSpan {
    component: Span,
    // config: Settings,
}

impl LabelSpan {
    pub fn new(config: &Settings, span: &[TextSpan]) -> Self {
        Self {
            component: Span::default()
                .spans(span)
                .alignment(Alignment::Left)
                .background(
                    config
                        .style_color_symbol
                        .library_background()
                        .unwrap_or(Color::Reset),
                )
                .foreground(
                    config
                        .style_color_symbol
                        .library_highlight()
                        .unwrap_or(Color::Cyan),
                )
                .modifiers(TextModifiers::BOLD),
            // config: config.clone(),
        }
    }
    // pub fn set_text(&mut self, text: &str, foreground: Color, background: Color) {
    //     self.component
    //         .spans(&[TextSpan::from(text).bold().fg(foreground).bg(background)]);
    // }
}

impl Component<Msg, NoUserEvent> for LabelSpan {
    fn on(&mut self, _ev: Event<NoUserEvent>) -> Option<Msg> {
        None
    }
}
