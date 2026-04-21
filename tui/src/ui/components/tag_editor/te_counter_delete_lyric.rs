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
use tui_realm_stdlib::prop_ext::CommonProps;
use tuirealm::command::{Cmd, CmdResult};
use tuirealm::component::{AppComponent, Component};
use tuirealm::event::{Event, Key, KeyEvent, KeyModifiers};
use tuirealm::props::{
    AttrValue, Attribute, Borders, Color, HorizontalAlignment, PropPayload, PropValue, Props,
    QueryResult, Style, TextModifiers, Title,
};
use tuirealm::ratatui::Frame;
use tuirealm::ratatui::layout::Rect;
use tuirealm::ratatui::widgets::{BorderType, Paragraph};
use tuirealm::state::{State, StateValue};

use crate::ui::model::{Model, UserEvent};
use crate::ui::msg::{Msg, TEMsg, TFMsg};

/// ## Counter
///
/// Counter which increments its value on Submit
#[derive(Default)]
struct Counter {
    props: Props,
    common: CommonProps,
}

impl Counter {
    /// Set the main foreground color. This may get overwritten by individual text styles.
    pub fn foreground(mut self, fg: Color) -> Self {
        self.attr(Attribute::Foreground, AttrValue::Color(fg));
        self
    }

    /// Set the main background color. This may get overwritten by individual text styles.
    pub fn background(mut self, bg: Color) -> Self {
        self.attr(Attribute::Background, AttrValue::Color(bg));
        self
    }

    /// Set the main text modifiers. This may get overwritten by individual text styles.
    pub fn modifiers(mut self, m: TextModifiers) -> Self {
        self.attr(Attribute::TextProps, AttrValue::TextModifiers(m));
        self
    }

    /// Set the main style. This may get overwritten by individual text styles.
    ///
    /// This option will overwrite any previous [`foreground`](Self::foreground), [`background`](Self::background) and [`modifiers`](Self::modifiers)!
    #[expect(dead_code)]
    pub fn style(mut self, style: Style) -> Self {
        self.attr(Attribute::Style, AttrValue::Style(style));
        self
    }

    /// Set a custom style for the border when the component is unfocused.
    #[expect(dead_code)]
    pub fn inactive(mut self, s: Style) -> Self {
        self.attr(Attribute::UnfocusedBorderStyle, AttrValue::Style(s));
        self
    }

    /// Add a border to the component.
    pub fn borders(mut self, b: Borders) -> Self {
        self.attr(Attribute::Borders, AttrValue::Borders(b));
        self
    }

    /// Add a title to the component.
    #[expect(dead_code)]
    pub fn title<T: Into<Title>>(mut self, title: T) -> Self {
        self.attr(Attribute::Title, AttrValue::Title(title.into()));
        self
    }

    /// Set the text alignment.
    pub fn alignment(mut self, a: HorizontalAlignment) -> Self {
        self.attr(Attribute::TextAlign, AttrValue::AlignmentHorizontal(a));
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
                AttrValue::Payload(PropPayload::Single(PropValue::Usize(n))),
            );
        } else {
            self.attr(Attribute::Value, AttrValue::Payload(PropPayload::None));
        }
        self
    }

    pub fn get_state(&self) -> Option<usize> {
        match self
            .props
            .get(Attribute::Value)
            .and_then(AttrValue::as_payload)?
        {
            PropPayload::Single(PropValue::Usize(v)) => Some(*v),
            _ => None,
        }
    }
}

impl Component for Counter {
    fn view(&mut self, frame: &mut Frame<'_>, area: Rect) {
        if !self.common.display {
            return;
        }

        // Get properties
        let value = self.get_state();
        let text_base = self
            .props
            .get(Attribute::Text)
            .and_then(|v| v.as_string())
            .map_or("", |v| v.as_str());
        let text = if let Some(value) = value {
            format!("{text_base} ({value})")
        } else {
            "None selected (-)".to_string()
        };

        let alignment = self
            .props
            .get(Attribute::TextAlign)
            .and_then(AttrValue::as_alignment_horizontal)
            .unwrap_or(HorizontalAlignment::Left);

        let block = self.common.get_block();

        let mut widget = Paragraph::new(text)
            .style(self.common.style)
            .alignment(alignment);

        if let Some(block) = block {
            widget = widget.block(block);
        }

        frame.render_widget(widget, area);
    }

    fn query(&self, attr: Attribute) -> Option<QueryResult<'_>> {
        if let Some(value) = self.common.get_for_query(attr) {
            return Some(value);
        }

        self.props.get_for_query(attr)
    }

    fn attr(&mut self, attr: Attribute, value: AttrValue) {
        if let Some(value) = self.common.set(attr, value) {
            self.props.set(attr, value);
        }
    }

    fn state(&self) -> State {
        let Some(state) = self.get_state() else {
            return State::None;
        };

        State::Single(StateValue::Usize(state))
    }

    fn perform(&mut self, cmd: Cmd) -> CmdResult {
        match cmd {
            Cmd::Submit => {
                // self.states.incr();
                CmdResult::Changed(self.state())
            }
            _ => CmdResult::NoChange,
        }
    }
}

// -- Counter components

#[derive(Component)]
pub struct TECounterDelete {
    component: Counter,
    config: SharedTuiSettings,
}

impl TECounterDelete {
    pub fn new(initial_value: Option<usize>, text: &str, config: SharedTuiSettings) -> Self {
        let component = {
            let config = config.read();
            Counter::default()
                .alignment(HorizontalAlignment::Center)
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
                .text(text)
                .value(initial_value)
        };

        Self { component, config }
    }
}

impl AppComponent<Msg, UserEvent> for TECounterDelete {
    fn on(&mut self, ev: &Event<UserEvent>) -> Option<Msg> {
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

#[derive(Component)]
pub struct TECounterSave {
    component: Counter,
    config: SharedTuiSettings,
}

impl TECounterSave {
    pub fn new(initial_value: Option<usize>, text: &str, config: SharedTuiSettings) -> Self {
        let component = {
            let config = config.read();
            Counter::default()
                .alignment(HorizontalAlignment::Center)
                .background(config.settings.theme.library_background())
                .foreground(config.settings.theme.library_foreground())
                .borders(
                    Borders::default()
                        .color(config.settings.theme.library_border())
                        .modifiers(BorderType::Rounded),
                )
                .modifiers(TextModifiers::BOLD)
                .text(text)
                .value(initial_value)
        };

        Self { component, config }
    }
}

impl AppComponent<Msg, UserEvent> for TECounterSave {
    fn on(&mut self, ev: &Event<UserEvent>) -> Option<Msg> {
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
            match song.save_tag() {
                Ok(()) => {
                    // the unwrap should never happen as we are in a branch where we had a reference to it
                    let song = self.tageditor_song.take().unwrap();
                    // the unwrap should also never happen as all components should be properly mounted
                    match self.init_by_song(song) {
                        Ok(()) => {}
                        Err(e) => self.mount_error_popup(e),
                    }
                }
                Err(e) => {
                    self.mount_error_popup(e);
                }
            }
        }
    }

    pub fn te_save_lyric(&mut self) -> Result<()> {
        if let Some(track) = self.tageditor_song.as_mut() {
            track.save_lrc_selected()?;
        }
        Ok(())
    }
}
