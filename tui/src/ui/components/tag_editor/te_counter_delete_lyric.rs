use termusiclib::config::SharedTuiSettings;
/*
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
use anyhow::Result;
use termusiclib::config::v2::tui::theme::styles::ColorTermusic;
use tui_realm_stdlib::utils::get_block;
use tuirealm::command::{Cmd, CmdResult};
use tuirealm::event::{Key, KeyEvent, KeyModifiers};
use tuirealm::props::{Alignment, Borders, Color, PropPayload, PropValue, TextModifiers};
use tuirealm::ratatui::layout::Rect;
use tuirealm::ratatui::widgets::{BorderType, Paragraph};
use tuirealm::{
    AttrValue, Attribute, Component, Event, Frame, MockComponent, Props, State, StateValue,
};

use crate::ui::model::{Model, UserEvent};
use crate::ui::msg::{Msg, TEMsg, TFMsg};
use crate::ui::utils::get_style;

/// ## Counter
///
/// Counter which increments its value on Submit
#[derive(Default)]
struct Counter {
    props: Props,
}

impl Counter {
    #[allow(dead_code)]
    pub fn label<S>(mut self, label: S) -> Self
    where
        S: AsRef<str>,
    {
        self.attr(
            Attribute::Title,
            AttrValue::Title((label.as_ref().to_string(), Alignment::Center)),
        );
        self
    }
    #[allow(dead_code)]
    pub fn text<S>(mut self, t: S) -> Self
    where
        S: Into<String>,
    {
        self.attr(Attribute::Text, AttrValue::String(t.into()));
        self
    }

    pub fn value(mut self, n: Option<usize>) -> Self {
        if let Some(n) = n {
            self.attr(
                Attribute::Value,
                AttrValue::Payload(PropPayload::One(PropValue::Usize(n))),
            );
        } else {
            self.attr(Attribute::Value, AttrValue::Payload(PropPayload::None));
        }
        self
    }

    pub fn alignment(mut self, a: Alignment) -> Self {
        self.attr(Attribute::TextAlign, AttrValue::Alignment(a));
        self
    }

    pub fn foreground(mut self, c: Color) -> Self {
        self.attr(Attribute::Foreground, AttrValue::Color(c));
        self
    }

    pub fn background(mut self, c: Color) -> Self {
        self.attr(Attribute::Background, AttrValue::Color(c));
        self
    }

    pub fn modifiers(mut self, m: TextModifiers) -> Self {
        self.attr(Attribute::TextProps, AttrValue::TextModifiers(m));
        self
    }

    pub fn borders(mut self, b: Borders) -> Self {
        self.attr(Attribute::Borders, AttrValue::Borders(b));
        self
    }

    pub fn get_state(&self) -> Option<usize> {
        match self
            .props
            .get_ref(Attribute::Value)
            .and_then(AttrValue::as_payload)?
        {
            PropPayload::One(PropValue::Usize(v)) => Some(*v),
            _ => None,
        }
    }
}

impl MockComponent for Counter {
    fn view(&mut self, frame: &mut Frame<'_>, area: Rect) {
        // Check if visible
        if self
            .props
            .get_ref(Attribute::Display)
            .and_then(AttrValue::as_flag)
            .unwrap_or(true)
        {
            // Get properties
            let value = self.get_state();
            let text_base = self
                .props
                .get_ref(Attribute::Text)
                .and_then(|v| v.as_string())
                .map_or("", |v| v.as_str());
            let text = if let Some(value) = value {
                format!("{text_base} ({value})")
            } else {
                "{text_base} (-)".to_string()
            };

            let alignment = self
                .props
                .get_ref(Attribute::TextAlign)
                .and_then(AttrValue::as_alignment)
                .unwrap_or(Alignment::Left);
            let style = get_style(&self.props);
            let title = self
                .props
                .get_ref(Attribute::Title)
                .and_then(AttrValue::as_title)
                // NOTE: clone should not be necessary anymore with tui-realm-stdlib next version
                .map_or((String::new(), Alignment::Center), Clone::clone);
            let borders = self
                .props
                .get_ref(Attribute::Borders)
                .and_then(AttrValue::as_borders)
                // Note: Borders should be copy-able
                .map_or(Borders::default(), Clone::clone);
            let focus = self
                .props
                .get_ref(Attribute::Focus)
                .and_then(AttrValue::as_flag)
                .unwrap_or(false);

            let inactive_style = self
                .props
                .get_ref(Attribute::FocusStyle)
                .and_then(AttrValue::as_style)
                .unwrap_or(style);
            frame.render_widget(
                Paragraph::new(text)
                    .block(get_block(
                        borders,
                        Some(&title),
                        focus,
                        Some(inactive_style),
                    ))
                    .style(style)
                    .alignment(alignment),
                area,
            );
        }
    }

    fn query(&self, attr: Attribute) -> Option<AttrValue> {
        self.props.get(attr)
    }

    fn attr(&mut self, attr: Attribute, value: AttrValue) {
        self.props.set(attr, value);
    }

    fn state(&self) -> State {
        let Some(state) = self.get_state() else {
            return State::None;
        };

        State::One(StateValue::Usize(state))
    }

    fn perform(&mut self, cmd: Cmd) -> CmdResult {
        match cmd {
            Cmd::Submit => {
                // self.states.incr();
                CmdResult::Changed(self.state())
            }
            _ => CmdResult::None,
        }
    }
}

// -- Counter components

#[derive(MockComponent)]
pub struct TECounterDelete {
    component: Counter,
    config: SharedTuiSettings,
}

impl TECounterDelete {
    pub fn new(initial_value: Option<usize>, config: SharedTuiSettings) -> Self {
        let component = {
            let config = config.read();
            Counter::default()
                .alignment(Alignment::Center)
                .background(config.settings.theme.library_background())
                .borders(
                    Borders::default()
                        .color(config.settings.theme.library_border())
                        .modifiers(BorderType::Rounded),
                )
                .foreground(
                    // config
                    //     .settings.theme
                    //     .library_highlight(),
                    // TODO: make this configurable
                    config
                        .settings
                        .theme
                        .get_color_from_theme(ColorTermusic::Red),
                )
                .modifiers(TextModifiers::BOLD)
                .text("Delete Selected")
                .value(initial_value)
        };

        Self { component, config }
    }
}

impl Component<Msg, UserEvent> for TECounterDelete {
    fn on(&mut self, ev: Event<UserEvent>) -> Option<Msg> {
        let keys = &self.config.read().settings.keys;
        // Get command
        let _cmd = match ev {
            Event::Keyboard(keyevent) if keyevent == keys.config_keys.save.get() => {
                return Some(Msg::TagEditor(TEMsg::Save));
            }
            Event::Keyboard(KeyEvent { code: Key::Tab, .. }) => {
                return Some(Msg::TagEditor(TEMsg::Focus(TFMsg::CounterDeleteBlurDown)));
            }
            Event::Keyboard(KeyEvent {
                code: Key::BackTab,
                modifiers: KeyModifiers::SHIFT,
            }) => return Some(Msg::TagEditor(TEMsg::Focus(TFMsg::CounterDeleteBlurUp))),

            Event::Keyboard(KeyEvent {
                code: Key::Down, ..
            }) => return Some(Msg::TagEditor(TEMsg::Focus(TFMsg::CounterDeleteBlurDown))),
            Event::Keyboard(KeyEvent { code: Key::Up, .. }) => {
                return Some(Msg::TagEditor(TEMsg::Focus(TFMsg::CounterDeleteBlurUp)));
            }
            Event::Keyboard(keyevent) if keyevent == keys.quit.get() => {
                return Some(Msg::TagEditor(TEMsg::Close));
            }
            Event::Keyboard(keyevent) if keyevent == keys.escape.get() => {
                return Some(Msg::TagEditor(TEMsg::Close));
            }
            Event::Keyboard(keyevent) if keyevent == keys.navigation_keys.up.get() => {
                return Some(Msg::TagEditor(TEMsg::Focus(TFMsg::CounterDeleteBlurUp)));
            }

            Event::Keyboard(keyevent) if keyevent == keys.navigation_keys.down.get() => {
                return Some(Msg::TagEditor(TEMsg::Focus(TFMsg::CounterDeleteBlurDown)));
            }

            Event::Keyboard(KeyEvent {
                code: Key::Enter, ..
            }) => return Some(Msg::TagEditor(TEMsg::CounterDeleteOk)),
            _ => Cmd::None,
        };
        None
    }
}

#[derive(MockComponent)]
pub struct TECounterSave {
    component: Counter,
    config: SharedTuiSettings,
}

impl TECounterSave {
    pub fn new(initial_value: Option<usize>, config: SharedTuiSettings) -> Self {
        let component = {
            let config = config.read();
            Counter::default()
                .alignment(Alignment::Center)
                .background(config.settings.theme.library_background())
                .foreground(config.settings.theme.library_foreground())
                .borders(
                    Borders::default()
                        .color(config.settings.theme.library_border())
                        .modifiers(BorderType::Rounded),
                )
                .modifiers(TextModifiers::BOLD)
                .text("Export LRC")
                .value(initial_value)
        };

        Self { component, config }
    }
}

impl Component<Msg, UserEvent> for TECounterSave {
    fn on(&mut self, ev: Event<UserEvent>) -> Option<Msg> {
        let keys = &self.config.read().settings.keys;
        // Get command
        let _cmd = match ev {
            Event::Keyboard(keyevent) if keyevent == keys.config_keys.save.get() => {
                return Some(Msg::TagEditor(TEMsg::Save));
            }
            Event::Keyboard(KeyEvent { code: Key::Tab, .. }) => {
                return Some(Msg::TagEditor(TEMsg::Focus(TFMsg::CounterSaveBlurDown)));
            }
            Event::Keyboard(KeyEvent {
                code: Key::BackTab,
                modifiers: KeyModifiers::SHIFT,
            }) => return Some(Msg::TagEditor(TEMsg::Focus(TFMsg::CounterSaveBlurUp))),

            Event::Keyboard(KeyEvent {
                code: Key::Down, ..
            }) => return Some(Msg::TagEditor(TEMsg::Focus(TFMsg::CounterSaveBlurDown))),
            Event::Keyboard(KeyEvent { code: Key::Up, .. }) => {
                return Some(Msg::TagEditor(TEMsg::Focus(TFMsg::CounterSaveBlurUp)));
            }
            Event::Keyboard(keyevent) if keyevent == keys.quit.get() => {
                return Some(Msg::TagEditor(TEMsg::Close));
            }
            Event::Keyboard(keyevent) if keyevent == keys.escape.get() => {
                return Some(Msg::TagEditor(TEMsg::Close));
            }
            Event::Keyboard(keyevent) if keyevent == keys.navigation_keys.up.get() => {
                return Some(Msg::TagEditor(TEMsg::Focus(TFMsg::CounterSaveBlurUp)));
            }

            Event::Keyboard(keyevent) if keyevent == keys.navigation_keys.down.get() => {
                return Some(Msg::TagEditor(TEMsg::Focus(TFMsg::CounterSaveBlurDown)));
            }

            Event::Keyboard(KeyEvent {
                code: Key::Enter, ..
            }) => return Some(Msg::TagEditor(TEMsg::CounterSaveOk)),
            _ => Cmd::None,
        };
        None
    }
}
impl Model {
    /// Delete the currently selected lyric index.
    ///
    /// This function only modifies the in-memory date and does *not* save the changed data.
    pub fn te_delete_lyric(&mut self) {
        if let Some(song) = self.tageditor_song.as_mut() {
            if song.lyric_frames().is_empty() {
                song.set_parsed_lyrics(None);
                return;
            }
            song.lyric_frames_remove_selected();
            if (song.lyric_selected_index() >= song.lyric_frames().len())
                && (song.lyric_selected_index() > 0)
            {
                song.set_lyric_selected_index(song.lyric_selected_index() - 1);
            }

            // The unwrap should never fail as we literally just had a exclusive reference to it.
            let song = self.tageditor_song.take().unwrap();
            // The unwrap should also never happen as all components should be properly mounted
            match self.init_by_song(song) {
                Ok(()) => {}
                Err(e) => self.mount_error_popup(e),
            }
        }
    }

    /// Save the currently selected Lyric text as a LRC file.
    pub fn te_export_lyric(&mut self) -> Result<()> {
        if let Some(track) = self.tageditor_song.as_mut() {
            track.export_lrc_selected()?;
        }
        Ok(())
    }
}
