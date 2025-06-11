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
use std::time::Instant;

use termusiclib::config::TuiOverlay;
use termusiclib::types::Msg;
use tui_realm_stdlib::{Label, Span, Spinner};
use tuirealm::command::{Cmd, CmdResult};
use tuirealm::props::{
    Alignment, AttrValue, Attribute, PropPayload, PropValue, TextModifiers, TextSpan,
};
use tuirealm::ratatui::layout::Rect;
use tuirealm::{Component, Event, Frame, MockComponent, State};

use crate::ui::model::UserEvent;

#[derive(MockComponent)]
pub struct LabelGeneric {
    component: Label,
}

impl LabelGeneric {
    pub fn new(config: &TuiOverlay, text: &str) -> Self {
        Self {
            component: Label::default()
                .text(text)
                .alignment(Alignment::Left)
                .background(config.settings.theme.library_background())
                .foreground(config.settings.theme.library_highlight())
                .modifiers(TextModifiers::BOLD),
        }
    }
}

impl Component<Msg, UserEvent> for LabelGeneric {
    fn on(&mut self, _ev: Event<UserEvent>) -> Option<Msg> {
        None
    }
}

pub struct LabelSpan {
    component: Span,
    default_span: Vec<TextSpan>,
    active_message_start_time: Option<Instant>,
    /// Timeout in seconds
    time_out: isize,
}

impl LabelSpan {
    pub fn new(_config: &TuiOverlay, span: &[TextSpan]) -> Self {
        let default = span.to_vec();

        Self {
            // dont style the Span itself, style the TextSpan's themself
            component: Span::default()
                .spans(default.clone())
                .alignment(Alignment::Left)
                .modifiers(TextModifiers::BOLD),
            default_span: default,
            active_message_start_time: None,
            time_out: 10,
        }
    }
}

impl Component<Msg, UserEvent> for LabelSpan {
    fn on(&mut self, _ev: Event<UserEvent>) -> Option<Msg> {
        None
    }
}

impl MockComponent for LabelSpan {
    #[allow(clippy::cast_sign_loss)]
    fn view(&mut self, render: &mut Frame<'_>, area: Rect) {
        if let Some(start_time) = self.active_message_start_time {
            if start_time.elapsed().as_secs() > self.time_out as u64 {
                self.attr(
                    Attribute::Text,
                    AttrValue::Payload(PropPayload::Vec(
                        self.default_span
                            .iter()
                            .cloned()
                            .map(PropValue::TextSpan)
                            .collect(),
                    )),
                );
                self.active_message_start_time = None;
                self.time_out = 10;
            }
        }

        self.component.view(render, area);
    }

    fn query(&self, attr: Attribute) -> Option<AttrValue> {
        self.component.query(attr)
    }

    fn attr(&mut self, attr: Attribute, value: AttrValue) {
        self.active_message_start_time = Some(Instant::now());
        match attr {
            Attribute::Value => self.time_out = value.unwrap_number(),
            attr => self.component.attr(attr, value),
        }
    }

    fn state(&self) -> State {
        self.component.state()
    }

    fn perform(&mut self, _cmd: Cmd) -> CmdResult {
        CmdResult::None
    }
}

#[derive(MockComponent)]
pub struct DownloadSpinner {
    component: Spinner,
}

impl DownloadSpinner {
    pub fn new(config: &TuiOverlay) -> Self {
        Self {
            component: Spinner::default()
                .foreground(config.settings.theme.library_highlight())
                .background(config.settings.theme.library_background())
                // .sequence("⣾⣽⣻⢿⡿⣟⣯⣷"),
                // .sequence("▉▊▋▌▍▎▏▎▍▌▋▊▉"),
                .sequence("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏"),
        }
    }
}

impl Component<Msg, UserEvent> for DownloadSpinner {
    fn on(&mut self, _: Event<UserEvent>) -> Option<Msg> {
        None
    }
}
