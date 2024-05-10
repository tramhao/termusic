use std::fmt::Display;

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
use crate::config::BindingForEvent;
use anyhow::{bail, Result};
use termusiclib::config::Keys;
use termusiclib::config::SharedSettings;
use termusiclib::types::{ConfigEditorMsg, IdKey, KFMsg, Msg};
use tui_realm_stdlib::utils::get_block;
use tuirealm::command::{Cmd, CmdResult, Direction, Position};
use tuirealm::event::{Key, KeyEvent, KeyModifiers, NoUserEvent};
use tuirealm::{Component, Event, Frame, MockComponent, State, StateValue};
use unicode_width::UnicodeWidthStr;

use tuirealm::props::{
    Alignment, AttrValue, Attribute, BorderSides, BorderType, Borders, Color, PropPayload,
    PropValue, Props, Style, TextModifiers,
};
use tuirealm::tui::{
    layout::{Constraint, Corner, Direction as LayoutDirection, Layout, Rect},
    text::Spans,
    widgets::{Block, List, ListItem, ListState, Paragraph},
};

pub const INPUT_INVALID_STYLE: &str = "invalid-style";
pub const INPUT_PLACEHOLDER: &str = "placeholder";
pub const INPUT_PLACEHOLDER_STYLE: &str = "placeholder-style";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MyModifiers {
    None,
    Shift,
    Control,
    Alt,
    ControlShift,
    AltShift,
    ControlAlt,
    ControlAltShift,
}

impl Display for MyModifiers {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

const CONTROL_SHIFT: KeyModifiers = KeyModifiers::CONTROL.union(KeyModifiers::SHIFT);
const ALT_SHIFT: KeyModifiers = KeyModifiers::ALT.union(KeyModifiers::SHIFT);
const CONTROL_ALT: KeyModifiers = KeyModifiers::CONTROL.union(KeyModifiers::ALT);
const CONTROL_ALT_SHIFT: KeyModifiers = KeyModifiers::CONTROL
    .union(KeyModifiers::ALT)
    .union(KeyModifiers::SHIFT);

impl MyModifiers {
    pub const LIST: &'static [MyModifiers; 8] = &[
        MyModifiers::None,
        MyModifiers::Shift,
        MyModifiers::Control,
        MyModifiers::Alt,
        MyModifiers::ControlShift,
        MyModifiers::AltShift,
        MyModifiers::ControlAlt,
        MyModifiers::ControlAltShift,
    ];

    /// Get the [`MyModifiers::LIST`] index for the given [`KeyModifiers`] value
    pub const fn from_modifier_list_index(val: KeyModifiers) -> usize {
        match val {
            KeyModifiers::SHIFT => 1,
            KeyModifiers::CONTROL => 2,
            KeyModifiers::ALT => 3,
            CONTROL_SHIFT => 4,
            ALT_SHIFT => 5,
            CONTROL_ALT => 6,
            CONTROL_ALT_SHIFT => 7,
            _ => 0,
        }
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            MyModifiers::None => "none",
            MyModifiers::Shift => "shift",
            MyModifiers::Control => "control",
            MyModifiers::Alt => "alt",
            MyModifiers::ControlShift => "ctrl_shift",
            MyModifiers::AltShift => "alt_shift",
            MyModifiers::ControlAlt => "ctrl_alt",
            MyModifiers::ControlAltShift => "ctrl_alt_shift",
        }
    }

    pub const fn as_modifier(self) -> KeyModifiers {
        match self {
            Self::None => KeyModifiers::NONE,
            Self::Shift => KeyModifiers::SHIFT,
            Self::Control => KeyModifiers::CONTROL,
            Self::Alt => KeyModifiers::ALT,
            Self::ControlShift => CONTROL_SHIFT,
            Self::AltShift => ALT_SHIFT,
            Self::ControlAlt => CONTROL_ALT,
            Self::ControlAltShift => CONTROL_ALT_SHIFT,
        }
    }
}

// -- states

/// ## `SelectStates`
///
/// Component states
#[derive(Default)]
pub struct SelectStates {
    /// Available choices
    pub choices: Vec<String>,
    /// Currently selected choice
    pub selected: usize,
    /// Choice selected before opening the tab
    pub previously_selected: usize,
    pub tab_open: bool,
}

#[allow(unused)]
impl SelectStates {
    /// ### `next_choice`
    ///
    /// Move choice index to next choice
    pub fn next_choice(&mut self, rewind: bool) {
        if self.tab_open {
            if rewind && self.selected + 1 >= self.choices.len() {
                self.selected = 0;
            } else if self.selected + 1 < self.choices.len() {
                self.selected += 1;
            }
        }
    }

    /// ### `prev_choice`
    ///
    /// Move choice index to previous choice
    pub fn prev_choice(&mut self, rewind: bool) {
        if self.tab_open {
            if rewind && self.selected == 0 && !self.choices.is_empty() {
                self.selected = self.choices.len() - 1;
            } else if self.selected > 0 {
                self.selected -= 1;
            }
        }
    }

    /// ### `set_choices`
    ///
    /// Set `SelectStates` choices from a vector of str
    /// In addition resets current selection and keep index if possible or set it to the first value
    /// available
    pub fn set_choices(&mut self, choices: &[String]) {
        self.choices = choices.to_vec();
        // Keep index if possible
        if self.selected >= self.choices.len() {
            self.selected = match self.choices.len() {
                0 => 0,
                l => l - 1,
            };
        }
    }

    pub fn select(&mut self, i: usize) {
        if i < self.choices.len() {
            self.selected = i;
        }
    }

    /// ### `close_tab`
    ///
    /// Close tab
    pub fn close_tab(&mut self) {
        self.tab_open = false;
    }

    /// ### `open_tab`
    ///
    /// Open tab
    pub fn open_tab(&mut self) {
        self.previously_selected = self.selected;
        self.tab_open = true;
    }

    /// Cancel tab open
    pub fn cancel_tab(&mut self) {
        self.close_tab();
        self.selected = self.previously_selected;
    }

    /// ### `is_tab_open`
    ///
    /// Returns whether the tab is open
    pub fn is_tab_open(&self) -> bool {
        self.tab_open
    }
}

#[derive(Default)]
pub struct InputStates {
    pub input: Vec<char>, // Current input
    pub cursor: usize,    // Input position
}

#[allow(unused)]
impl InputStates {
    /// ### `append`
    ///
    /// Append, if possible according to input type, the character to the input vec
    pub fn append(&mut self, ch: char, max_len: Option<usize>) {
        // Check if max length has been reached
        if self.input.len() < max_len.unwrap_or(usize::MAX) {
            // Check whether can push
            self.input.insert(self.cursor, ch);
            self.incr_cursor();
        }
    }

    /// ### `backspace`
    ///
    /// Delete element at cursor -1; then decrement cursor by 1
    pub fn backspace(&mut self) {
        if self.cursor > 0 && !self.input.is_empty() {
            self.input.remove(self.cursor - 1);
            // Decrement cursor
            self.cursor -= 1;
        }
    }

    /// ### `delete`
    ///
    /// Delete element at cursor
    pub fn delete(&mut self) {
        if self.cursor < self.input.len() {
            self.input.remove(self.cursor);
        }
    }

    /// ### `incr_cursor`
    ///
    /// Increment cursor value by one if possible
    pub fn incr_cursor(&mut self) {
        if self.cursor < self.input.len() {
            self.cursor += 1;
        }
    }

    /// ### `cursoro_at_begin`
    ///
    /// Place cursor at the begin of the input
    pub fn cursor_at_begin(&mut self) {
        self.cursor = 0;
    }

    /// ### `cursor_at_end`
    ///
    /// Place cursor at the end of the input
    pub fn cursor_at_end(&mut self) {
        self.cursor = self.input.len();
    }

    /// ### `decr_cursor`
    ///
    /// Decrement cursor value by one if possible
    pub fn decr_cursor(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
        }
    }

    /// ### `render_value`
    ///
    /// Get value as string to render
    pub fn render_value(&self) -> String {
        self.render_value_chars().iter().collect::<String>()
    }

    /// ### `render_value_chars`
    ///
    /// Render value as a vec of chars
    pub fn render_value_chars(&self) -> Vec<char> {
        self.input.clone()
    }

    /// ### `get_value`
    ///
    /// Get value as string
    pub fn get_value(&self) -> String {
        self.input.iter().collect()
    }
}

// -- component

#[derive(Default)]
pub struct KeyCombo {
    props: Props,
    pub states: SelectStates,
    hg_str: Option<String>, // CRAP CRAP CRAP
    pub states_input: InputStates,
}

#[allow(unused)]
impl KeyCombo {
    pub fn input_len(mut self, ilen: usize) -> Self {
        self.attr(Attribute::InputLength, AttrValue::Length(ilen));
        self
    }

    pub fn invalid_style(mut self, s: Style) -> Self {
        self.attr(Attribute::Custom(INPUT_INVALID_STYLE), AttrValue::Style(s));
        self
    }

    pub fn placeholder<S: AsRef<str>>(mut self, placeholder: S, style: Style) -> Self {
        self.attr(
            Attribute::Custom(INPUT_PLACEHOLDER),
            AttrValue::String(placeholder.as_ref().to_string()),
        );
        self.attr(
            Attribute::Custom(INPUT_PLACEHOLDER_STYLE),
            AttrValue::Style(style),
        );
        self
    }

    fn get_input_len(&self) -> Option<usize> {
        self.props
            .get(Attribute::InputLength)
            .map(AttrValue::unwrap_length)
    }

    /// ### `is_valid`
    ///
    /// Checks whether current input is valid
    fn is_valid(&self) -> bool {
        let value = self.states_input.get_value();
        BindingForEvent::key_from_str(value.as_str()).is_ok()
    }
    pub fn foreground(mut self, fg: Color) -> Self {
        self.attr(Attribute::Foreground, AttrValue::Color(fg));
        self
    }

    pub fn background(mut self, bg: Color) -> Self {
        self.attr(Attribute::Background, AttrValue::Color(bg));
        self
    }

    pub fn borders(mut self, b: Borders) -> Self {
        self.attr(Attribute::Borders, AttrValue::Borders(b));
        self
    }

    pub fn title<S: AsRef<str>>(mut self, t: S, a: Alignment) -> Self {
        self.attr(
            Attribute::Title,
            AttrValue::Title((t.as_ref().to_string(), a)),
        );
        self
    }

    pub fn highlighted_str<S: AsRef<str>>(mut self, s: S) -> Self {
        self.attr(
            Attribute::HighlightedStr,
            AttrValue::String(s.as_ref().to_string()),
        );
        self
    }

    pub fn highlighted_color(mut self, c: Color) -> Self {
        self.attr(Attribute::HighlightedColor, AttrValue::Color(c));
        self
    }

    pub fn inactive(mut self, s: Style) -> Self {
        self.attr(Attribute::FocusStyle, AttrValue::Style(s));
        self
    }

    pub fn rewind(mut self, r: bool) -> Self {
        self.attr(Attribute::Rewind, AttrValue::Flag(r));
        self
    }

    pub fn choices<S: AsRef<str>>(mut self, choices: &[S]) -> Self {
        self.attr(
            Attribute::Content,
            AttrValue::Payload(PropPayload::Vec(
                choices
                    .iter()
                    .map(|x| PropValue::Str(x.as_ref().to_string()))
                    .collect(),
            )),
        );
        self
    }

    pub fn value<S: AsRef<str>>(mut self, i: usize, s: S) -> Self {
        // Set state

        self.attr_input(Attribute::Value, AttrValue::String(s.as_ref().to_string()));

        self.attr(
            Attribute::Value,
            AttrValue::Payload(PropPayload::One(PropValue::Usize(i))),
        );
        self
    }

    pub fn attr_input(&mut self, attr: Attribute, value: AttrValue) {
        let sanitize_input = matches!(
            attr,
            Attribute::InputLength | Attribute::InputType | Attribute::Value
        );
        // Check if new input
        let new_input = match attr {
            Attribute::Value => Some(value.clone().unwrap_string()),
            _ => None,
        };
        self.props.set(attr, value);
        if sanitize_input {
            let input = match new_input {
                None => self.states_input.input.clone(),
                Some(v) => v.chars().collect(),
            };
            self.states_input.input = Vec::new();
            self.states_input.cursor = 0;
            let max_len = self.get_input_len();
            for ch in input {
                self.states_input.append(ch, max_len);
            }
        }
    }

    /// ### `render_open_tab`
    ///
    /// Render component when tab is open
    #[allow(clippy::too_many_lines)]
    fn render_open_tab(&mut self, render: &mut Frame<'_>, area: Rect) {
        // Make choices
        let choices: Vec<ListItem<'_>> = self
            .states
            .choices
            .iter()
            .map(|x| ListItem::new(Spans::from(x.clone())))
            .collect();
        let mut foreground = self
            .props
            .get_or(Attribute::Foreground, AttrValue::Color(Color::Reset))
            .unwrap_color();
        let mut background = self
            .props
            .get_or(Attribute::Background, AttrValue::Color(Color::Reset))
            .unwrap_color();
        let hg: Color = self
            .props
            .get_or(Attribute::HighlightedColor, AttrValue::Color(foreground))
            .unwrap_color();
        // Prepare layout
        let chunks = Layout::default()
            .direction(LayoutDirection::Vertical)
            .margin(0)
            .constraints([Constraint::Length(2), Constraint::Min(1)].as_ref())
            .split(area);
        // Render like "closed" tab in chunk 0
        let selected_text: String = match self.states.choices.get(self.states.selected) {
            None => String::default(),
            Some(s) => s.clone(),
        };
        let borders = self
            .props
            .get_or(Attribute::Borders, AttrValue::Borders(Borders::default()))
            .unwrap_borders();
        let block: Block<'_> = Block::default()
            .borders(BorderSides::LEFT | BorderSides::TOP | BorderSides::RIGHT)
            .border_style(borders.style())
            .border_type(borders.modifiers)
            .style(Style::default().bg(background));
        let title = self
            .props
            .get(Attribute::Title)
            .map(tuirealm::AttrValue::unwrap_title);
        let mut block = match title {
            Some((text, alignment)) => block.title(text).title_alignment(alignment),
            None => block,
        };
        let focus = self
            .props
            .get_or(Attribute::Focus, AttrValue::Flag(false))
            .unwrap_flag();
        let inactive_style = self
            .props
            .get(Attribute::FocusStyle)
            .map(tuirealm::AttrValue::unwrap_style);

        let mut style = if focus {
            borders.style()
        } else {
            inactive_style.unwrap_or_default()
        };

        // Apply invalid style
        if !self.is_valid() {
            if let Some(style_invalid) = self
                .props
                .get(Attribute::Custom(INPUT_INVALID_STYLE))
                .map(AttrValue::unwrap_style)
            {
                block = block.border_style(style_invalid);
                foreground = style_invalid.fg.unwrap_or(Color::Reset);
                background = style_invalid.bg.unwrap_or(Color::Reset);
                style = style_invalid;
            }
        }
        let p: Paragraph<'_> = Paragraph::new(selected_text).style(style).block(block);

        render.render_widget(p, chunks[0]);
        // Render the list of elements in chunks [1]
        // Make list
        let mut block = Block::default()
            .borders(BorderSides::LEFT | BorderSides::BOTTOM | BorderSides::RIGHT)
            .border_type(borders.modifiers)
            .border_style(if focus {
                borders.style()
            } else {
                Style::default()
            })
            .style(Style::default().bg(background));

        // Apply invalid style
        if !self.is_valid() {
            // if focus && !self.is_valid() {
            if let Some(style) = self
                .props
                .get(Attribute::Custom(INPUT_INVALID_STYLE))
                .map(AttrValue::unwrap_style)
            {
                block = block.border_style(style);
                foreground = style.fg.unwrap_or(Color::Reset);
                background = style.bg.unwrap_or(Color::Reset);
            }
        }
        let mut list = List::new(choices)
            .block(block)
            .start_corner(Corner::TopLeft)
            .style(Style::default().fg(foreground).bg(background))
            .highlight_style(
                Style::default()
                    .fg(hg)
                    .add_modifier(TextModifiers::REVERSED),
            );
        // Highlighted symbol
        self.hg_str = self
            .props
            .get(Attribute::HighlightedStr)
            .map(tuirealm::AttrValue::unwrap_string);
        if let Some(hg_str) = &self.hg_str {
            list = list.highlight_symbol(hg_str);
        }
        let mut state: ListState = ListState::default();
        state.select(Some(self.states.selected));
        render.render_stateful_widget(list, chunks[1], &mut state);
    }

    /// ### `render_closed_tab`
    ///
    /// Render component when tab is closed
    fn render_closed_tab(&self, render: &mut Frame<'_>, area: Rect) {
        // Render select
        let mut foreground = self
            .props
            .get_or(Attribute::Foreground, AttrValue::Color(Color::Reset))
            .unwrap_color();
        let mut background = self
            .props
            .get_or(Attribute::Background, AttrValue::Color(Color::Reset))
            .unwrap_color();
        let inactive_style = self
            .props
            .get(Attribute::FocusStyle)
            .map(tuirealm::AttrValue::unwrap_style);
        let focus = self
            .props
            .get_or(Attribute::Focus, AttrValue::Flag(false))
            .unwrap_flag();
        let mut style = if focus {
            Style::default().bg(background).fg(foreground)
        } else {
            inactive_style.unwrap_or_default()
        };
        let borders = self
            .props
            .get_or(Attribute::Borders, AttrValue::Borders(Borders::default()))
            .unwrap_borders();
        let borders_style = if focus {
            borders.style()
        } else {
            inactive_style.unwrap_or_default()
        };
        let block: Block<'_> = Block::default()
            .borders(BorderSides::ALL)
            .border_type(borders.modifiers)
            .border_style(borders_style)
            .style(style);
        let title = self
            .props
            .get(Attribute::Title)
            .map(AttrValue::unwrap_title);

        let mut block = match title {
            Some((text, alignment)) => block.title(text).title_alignment(alignment),
            None => block,
        };
        // Apply invalid style
        if !self.is_valid() {
            // if focus && !self.is_valid() {
            if let Some(style_invalid) = self
                .props
                .get(Attribute::Custom(INPUT_INVALID_STYLE))
                .map(AttrValue::unwrap_style)
            {
                block = block.border_style(style_invalid);
                foreground = style_invalid.fg.unwrap_or(Color::Reset);
                background = style_invalid.bg.unwrap_or(Color::Reset);
                style = style_invalid;
            }
        }
        let selected_text: String = match self.states.choices.get(self.states.selected) {
            None => String::default(),
            Some(s) => s.clone(),
        };
        let p: Paragraph<'_> = Paragraph::new(selected_text).style(style).block(block);

        render.render_widget(p, area);
    }

    fn render_input(&self, render: &mut Frame<'_>, area: Rect) {
        let chunks = Layout::default()
            .direction(LayoutDirection::Horizontal)
            .margin(0)
            .constraints([Constraint::Ratio(2, 3), Constraint::Ratio(1, 3)].as_ref())
            .split(area);

        let mut foreground = self
            .props
            .get_or(Attribute::Foreground, AttrValue::Color(Color::Reset))
            .unwrap_color();
        let mut background = self
            .props
            .get_or(Attribute::Background, AttrValue::Color(Color::Reset))
            .unwrap_color();
        let modifiers = self
            .props
            .get_or(
                Attribute::TextProps,
                AttrValue::TextModifiers(TextModifiers::empty()),
            )
            .unwrap_text_modifiers();
        let borders = self
            .props
            .get_or(Attribute::Borders, AttrValue::Borders(Borders::default()))
            .unwrap_borders()
            .sides(BorderSides::NONE);

        let focus = self
            .props
            .get_or(Attribute::Focus, AttrValue::Flag(false))
            .unwrap_flag();
        let inactive_style = self
            .props
            .get(Attribute::FocusStyle)
            .map(AttrValue::unwrap_style);
        let mut block = get_block(borders, None, focus, inactive_style);
        // Apply invalid style
        if focus && !self.is_valid() {
            if let Some(style) = self
                .props
                .get(Attribute::Custom(INPUT_INVALID_STYLE))
                .map(AttrValue::unwrap_style)
            {
                block = block.borders(BorderSides::NONE);
                foreground = style.fg.unwrap_or(Color::Reset);
                background = style.bg.unwrap_or(Color::Reset);
            }
        }
        let text_to_display = self.states_input.render_value();
        let show_placeholder = text_to_display.is_empty();
        // Choose whether to show placeholder; if placeholder is unset, show nothing
        let text_to_display = if show_placeholder {
            self.props
                .get_or(
                    Attribute::Custom(INPUT_PLACEHOLDER),
                    AttrValue::String(String::new()),
                )
                .unwrap_string()
        } else {
            text_to_display
        };
        // Choose paragraph style based on whether is valid or not and if has focus and if should show placeholder
        let paragraph_style = if focus {
            Style::default()
                .fg(foreground)
                .bg(background)
                .add_modifier(modifiers)
        } else {
            inactive_style.unwrap_or_default()
        };
        let paragraph_style = if show_placeholder && focus {
            self.props
                .get_or(
                    Attribute::Custom(INPUT_PLACEHOLDER_STYLE),
                    AttrValue::Style(paragraph_style),
                )
                .unwrap_style()
        } else {
            paragraph_style
        };
        // Create widget
        let p: Paragraph<'_> = Paragraph::new(text_to_display)
            .style(paragraph_style)
            .block(block);
        render.render_widget(p, chunks[1]);
        // Set cursor, if focus
        if focus {
            let x: u16 = chunks[1].x
                + calc_utf8_cursor_position(
                    &self.states_input.render_value_chars()[0..self.states_input.cursor],
                );
            render.set_cursor(x, area.y + 1);
        }
    }

    fn rewindable(&self) -> bool {
        self.props
            .get_or(Attribute::Rewind, AttrValue::Flag(false))
            .unwrap_flag()
    }
}

impl MockComponent for KeyCombo {
    fn view(&mut self, render: &mut Frame<'_>, area: Rect) {
        if self.props.get_or(Attribute::Display, AttrValue::Flag(true)) == AttrValue::Flag(true) {
            if self.states.is_tab_open() {
                self.render_input(render, area);
                self.render_open_tab(render, area);
            } else {
                self.render_input(render, area);
                self.render_closed_tab(render, area);
            }
        }
    }

    fn query(&self, attr: Attribute) -> Option<AttrValue> {
        self.props.get(attr)
    }

    fn attr(&mut self, attr: Attribute, value: AttrValue) {
        match attr {
            Attribute::Content => {
                // Reset choices
                let choices: Vec<String> = value
                    .unwrap_payload()
                    .unwrap_vec()
                    .iter()
                    .map(|x| x.clone().unwrap_str())
                    .collect();
                self.states.set_choices(&choices);
            }
            Attribute::Value => {
                self.states
                    .select(value.unwrap_payload().unwrap_one().unwrap_usize());
            }
            Attribute::Focus if self.states.is_tab_open() => {
                if let AttrValue::Flag(false) = value {
                    self.states.cancel_tab();
                }
                self.props.set(attr, value);
            }
            attr => {
                self.props.set(attr, value);
            }
        }
    }

    fn state(&self) -> State {
        if self.states.is_tab_open() {
            State::None
        } else {
            State::One(StateValue::Usize(self.states.selected))
        }
    }

    fn perform(&mut self, cmd: Cmd) -> CmdResult {
        match cmd {
            Cmd::Move(Direction::Down) => {
                // Increment choice
                self.states.next_choice(self.rewindable());
                // Return CmdResult On Change or None if tab is closed
                if self.states.is_tab_open() {
                    CmdResult::Changed(State::One(StateValue::Usize(self.states.selected)))
                } else {
                    CmdResult::None
                }
            }
            Cmd::Move(Direction::Up) => {
                // Increment choice
                self.states.prev_choice(self.rewindable());
                // Return CmdResult On Change or None if tab is closed
                if self.states.is_tab_open() {
                    CmdResult::Changed(State::One(StateValue::Usize(self.states.selected)))
                } else {
                    CmdResult::None
                }
            }
            Cmd::Cancel => {
                self.states.cancel_tab();
                let prev_input = self.states_input.input.clone();
                self.states_input.delete();
                if prev_input == self.states_input.input {
                    CmdResult::None
                } else {
                    CmdResult::Changed(self.state())
                }
            }
            Cmd::Submit => {
                // Open or close tab
                if self.states.is_tab_open() {
                    self.states.close_tab();
                    CmdResult::Submit(self.state())
                } else {
                    self.states.open_tab();
                    CmdResult::None
                }
            }

            Cmd::Delete => {
                // Backspace and None
                let prev_input = self.states_input.input.clone();
                self.states_input.backspace();
                if prev_input == self.states_input.input {
                    CmdResult::None
                } else {
                    CmdResult::Changed(self.state())
                }
            }
            Cmd::Move(Direction::Left) => {
                self.states_input.decr_cursor();
                CmdResult::None
            }
            Cmd::Move(Direction::Right) => {
                self.states_input.incr_cursor();
                CmdResult::None
            }
            Cmd::GoTo(Position::Begin) => {
                self.states_input.cursor_at_begin();
                CmdResult::None
            }
            Cmd::GoTo(Position::End) => {
                self.states_input.cursor_at_end();
                CmdResult::None
            }
            Cmd::Type(ch) => {
                // Push char to input
                let prev_input = self.states_input.input.clone();
                self.states_input.append(ch, self.get_input_len());
                // Message on change
                if prev_input == self.states_input.input {
                    CmdResult::None
                } else {
                    CmdResult::Changed(self.state())
                }
            }

            _ => CmdResult::None,
        }
    }
}

/// ### `calc_utf8_cursor_position`
///
/// Calculate the UTF8 compliant position for the cursor given the characters preceeding the cursor position.
/// Use this function to calculate cursor position whenever you want to handle UTF8 texts with cursors
#[allow(clippy::cast_possible_truncation)]
pub fn calc_utf8_cursor_position(chars: &[char]) -> u16 {
    chars.iter().collect::<String>().width() as u16
}

#[cfg(test)]
mod test {

    use super::*;

    use pretty_assertions::assert_eq;

    use tuirealm::props::{PropPayload, PropValue};

    #[test]
    fn test_components_select_states() {
        let mut states: SelectStates = SelectStates::default();
        assert_eq!(states.selected, 0);
        assert_eq!(states.choices.len(), 0);
        assert_eq!(states.tab_open, false);
        let choices: &[String] = &[
            "lemon".to_string(),
            "strawberry".to_string(),
            "vanilla".to_string(),
            "chocolate".to_string(),
        ];
        states.set_choices(choices);
        assert_eq!(states.selected, 0);
        assert_eq!(states.choices.len(), 4);
        // Move
        states.prev_choice(false);
        assert_eq!(states.selected, 0);
        states.next_choice(false);
        // Tab is closed!!!
        assert_eq!(states.selected, 0);
        states.open_tab();
        assert_eq!(states.is_tab_open(), true);
        // Now we can move
        states.next_choice(false);
        assert_eq!(states.selected, 1);
        states.next_choice(false);
        assert_eq!(states.selected, 2);
        // Forward overflow
        states.next_choice(false);
        states.next_choice(false);
        assert_eq!(states.selected, 3);
        states.prev_choice(false);
        assert_eq!(states.selected, 2);
        // Close tab
        states.close_tab();
        assert_eq!(states.is_tab_open(), false);
        states.prev_choice(false);
        assert_eq!(states.selected, 2);
        // Update
        let choices: &[String] = &["lemon".to_string(), "strawberry".to_string()];
        states.set_choices(choices);
        assert_eq!(states.selected, 1); // Move to first index available
        assert_eq!(states.choices.len(), 2);
        let choices = vec![];
        states.set_choices(&choices);
        assert_eq!(states.selected, 0); // Move to first index available
        assert_eq!(states.choices.len(), 0);
        // Rewind
        let choices: &[String] = &[
            "lemon".to_string(),
            "strawberry".to_string(),
            "vanilla".to_string(),
            "chocolate".to_string(),
        ];
        states.set_choices(choices);
        states.open_tab();
        assert_eq!(states.selected, 0);
        states.prev_choice(true);
        assert_eq!(states.selected, 3);
        states.next_choice(true);
        assert_eq!(states.selected, 0);
        states.next_choice(true);
        assert_eq!(states.selected, 1);
        states.prev_choice(true);
        assert_eq!(states.selected, 0);
        // Cancel tab
        states.close_tab();
        states.select(2);
        states.open_tab();
        states.prev_choice(true);
        states.prev_choice(true);
        assert_eq!(states.selected, 0);
        states.cancel_tab();
        assert_eq!(states.selected, 2);
        assert_eq!(states.is_tab_open(), false);
    }

    #[test]
    fn test_components_select() {
        // Make component
        let mut component = KeyCombo::default()
            .foreground(Color::Red)
            .background(Color::Black)
            .borders(Borders::default())
            .highlighted_color(Color::Red)
            .highlighted_str(">>")
            .title(" C'est oui ou bien c'est non? ", Alignment::Center)
            .choices(&["Oui!", "Non", "Peut-être"])
            .value(1, "")
            .rewind(false);
        assert_eq!(component.states.is_tab_open(), false);
        component.states.open_tab();
        assert_eq!(component.states.is_tab_open(), true);
        component.states.close_tab();
        assert_eq!(component.states.is_tab_open(), false);
        // Update
        component.attr(
            Attribute::Value,
            AttrValue::Payload(PropPayload::One(PropValue::Usize(2))),
        );
        // Get value
        assert_eq!(component.state(), State::One(StateValue::Usize(2)));
        // Open tab
        component.states.open_tab();
        // Events
        // Move cursor
        assert_eq!(
            component.perform(Cmd::Move(Direction::Up)),
            CmdResult::Changed(State::One(StateValue::Usize(1))),
        );
        assert_eq!(
            component.perform(Cmd::Move(Direction::Up)),
            CmdResult::Changed(State::One(StateValue::Usize(0))),
        );
        // Upper boundary
        assert_eq!(
            component.perform(Cmd::Move(Direction::Up)),
            CmdResult::Changed(State::One(StateValue::Usize(0))),
        );
        // Move down
        assert_eq!(
            component.perform(Cmd::Move(Direction::Down)),
            CmdResult::Changed(State::One(StateValue::Usize(1))),
        );
        assert_eq!(
            component.perform(Cmd::Move(Direction::Down)),
            CmdResult::Changed(State::One(StateValue::Usize(2))),
        );
        // Lower boundary
        assert_eq!(
            component.perform(Cmd::Move(Direction::Down)),
            CmdResult::Changed(State::One(StateValue::Usize(2))),
        );
        // Press enter
        assert_eq!(
            component.perform(Cmd::Submit),
            CmdResult::Submit(State::One(StateValue::Usize(2))),
        );
        // Tab should be closed
        assert_eq!(component.states.is_tab_open(), false);
        // Re open
        assert_eq!(component.perform(Cmd::Submit), CmdResult::None);
        assert_eq!(component.states.is_tab_open(), true);
        // Move arrows
        assert_eq!(
            component.perform(Cmd::Submit),
            CmdResult::Submit(State::One(StateValue::Usize(2))),
        );
        assert_eq!(component.states.is_tab_open(), false);
        assert_eq!(
            component.perform(Cmd::Move(Direction::Down)),
            CmdResult::None
        );
        assert_eq!(component.perform(Cmd::Move(Direction::Up)), CmdResult::None);
    }
}

#[derive(MockComponent)]
struct KEModifierSelect {
    component: KeyCombo,
    id: IdKey,
    config: SharedSettings,
    on_key_tab: Msg,
    on_key_backtab: Msg,
}

impl KEModifierSelect {
    pub fn new(
        name: &str,
        id: IdKey,
        config: SharedSettings,
        on_key_tab: Msg,
        on_key_backtab: Msg,
    ) -> Self {
        let config_r = config.read();
        let (init_select, init_key) = Self::init_modifier_select(id, &config_r.keys);
        let mut choices = vec![];
        for modifier in MyModifiers::LIST {
            choices.push(modifier.as_str());
        }
        let component = KeyCombo::default()
            .borders(
                Borders::default()
                    .modifiers(BorderType::Rounded)
                    .color(config_r.style_color_symbol.fallback_border()),
            )
            .foreground(config_r.style_color_symbol.fallback_foreground())
            .title(name, Alignment::Left)
            .rewind(false)
            .highlighted_color(config_r.style_color_symbol.fallback_highlight())
            .highlighted_str(">> ")
            .choices(&choices)
            .placeholder("a/b/c", Style::default().fg(Color::Rgb(128, 128, 128)))
            .invalid_style(Style::default().fg(Color::Red))
            .value(init_select, init_key);

        drop(config_r);
        Self {
            component,
            id,
            config,
            on_key_tab,
            on_key_backtab,
        }
    }

    /// Get the Selected Modifier choice index for the given [`IdKey`] and the key representation as string
    ///
    /// Returns `(mod_list_index, key_str)`
    fn init_modifier_select(id: IdKey, keys: &Keys) -> (usize, String) {
        let mod_key = match id {
            IdKey::DatabaseAddAll => keys.database_add_all.mod_key(),
            IdKey::GlobalConfig => keys.global_config_open.mod_key(),
            IdKey::GlobalDown => keys.global_down.mod_key(),
            IdKey::GlobalGotoBottom => keys.global_goto_bottom.mod_key(),
            IdKey::GlobalGotoTop => keys.global_goto_top.mod_key(),
            IdKey::GlobalHelp => keys.global_help.mod_key(),
            IdKey::GlobalLayoutTreeview => keys.global_layout_treeview.mod_key(),
            IdKey::GlobalLayoutDatabase => keys.global_layout_database.mod_key(),
            IdKey::GlobalLeft => keys.global_left.mod_key(),
            IdKey::GlobalLyricAdjustForward => keys.global_lyric_adjust_forward.mod_key(),
            IdKey::GlobalLyricAdjustBackward => keys.global_lyric_adjust_backward.mod_key(),
            IdKey::GlobalLyricCycle => keys.global_lyric_cycle.mod_key(),
            IdKey::GlobalPlayerToggleGapless => keys.global_player_toggle_gapless.mod_key(),
            IdKey::GlobalPlayerTogglePause => keys.global_player_toggle_pause.mod_key(),
            IdKey::GlobalPlayerNext => keys.global_player_next.mod_key(),
            IdKey::GlobalPlayerPrevious => keys.global_player_previous.mod_key(),
            IdKey::GlobalPlayerSeekForward => keys.global_player_seek_forward.mod_key(),
            IdKey::GlobalPlayerSeekBackward => keys.global_player_seek_backward.mod_key(),
            IdKey::GlobalPlayerSpeedUp => keys.global_player_speed_up.mod_key(),
            IdKey::GlobalPlayerSpeedDown => keys.global_player_speed_down.mod_key(),
            IdKey::GlobalQuit => keys.global_quit.mod_key(),
            IdKey::GlobalRight => keys.global_right.mod_key(),
            IdKey::GlobalUp => keys.global_up.mod_key(),
            IdKey::GlobalVolumeDown => keys.global_player_volume_minus_2.mod_key(),
            IdKey::GlobalVolumeUp => keys.global_player_volume_plus_2.mod_key(),
            IdKey::GlobalSavePlaylist => keys.global_save_playlist.mod_key(),
            IdKey::LibraryDelete => keys.library_delete.mod_key(),
            IdKey::LibraryLoadDir => keys.library_load_dir.mod_key(),
            IdKey::LibraryPaste => keys.library_paste.mod_key(),
            IdKey::LibrarySearch => keys.library_search.mod_key(),
            IdKey::LibrarySearchYoutube => keys.library_search_youtube.mod_key(),
            IdKey::LibraryTagEditor => keys.library_tag_editor_open.mod_key(),
            IdKey::LibraryYank => keys.library_yank.mod_key(),
            IdKey::PlaylistDelete => keys.playlist_delete.mod_key(),
            IdKey::PlaylistDeleteAll => keys.playlist_delete_all.mod_key(),
            IdKey::PlaylistShuffle => keys.playlist_shuffle.mod_key(),
            IdKey::PlaylistModeCycle => keys.playlist_mode_cycle.mod_key(),
            IdKey::PlaylistPlaySelected => keys.playlist_play_selected.mod_key(),
            IdKey::PlaylistSearch => keys.playlist_search.mod_key(),
            IdKey::PlaylistSwapDown => keys.playlist_swap_down.mod_key(),
            IdKey::PlaylistSwapUp => keys.playlist_swap_up.mod_key(),
            IdKey::PlaylistAddRandomAlbum => keys.playlist_add_random_album.mod_key(),
            IdKey::PlaylistAddRandomTracks => keys.playlist_add_random_tracks.mod_key(),
            IdKey::LibrarySwitchRoot => keys.library_switch_root.mod_key(),
            IdKey::LibraryAddRoot => keys.library_add_root.mod_key(),
            IdKey::LibraryRemoveRoot => keys.library_remove_root.mod_key(),
            IdKey::GlobalLayoutPodcast => keys.global_layout_podcast.mod_key(),
            IdKey::GlobalXywhMoveLeft => keys.global_xywh_move_left.mod_key(),
            IdKey::GlobalXywhMoveRight => keys.global_xywh_move_right.mod_key(),
            IdKey::GlobalXywhMoveUp => keys.global_xywh_move_up.mod_key(),
            IdKey::GlobalXywhMoveDown => keys.global_xywh_move_down.mod_key(),
            IdKey::GlobalXywhZoomIn => keys.global_xywh_zoom_in.mod_key(),
            IdKey::GlobalXywhZoomOut => keys.global_xywh_zoom_out.mod_key(),
            IdKey::GlobalXywhHide => keys.global_xywh_hide.mod_key(),
            IdKey::PodcastMarkPlayed => keys.podcast_mark_played.mod_key(),
            IdKey::PodcastMarkAllPlayed => keys.podcast_mark_all_played.mod_key(),
            IdKey::PodcastEpDownload => keys.podcast_episode_download.mod_key(),
            IdKey::PodcastEpDeleteFile => keys.podcast_episode_delete_file.mod_key(),
            IdKey::PodcastDeleteFeed => keys.podcast_delete_feed.mod_key(),
            IdKey::PodcastDeleteAllFeeds => keys.podcast_delete_all_feeds.mod_key(),
            IdKey::PodcastSearchAddFeed => keys.podcast_search_add_feed.mod_key(),
            IdKey::PodcastRefreshFeed => keys.podcast_refresh_feed.mod_key(),
            IdKey::PodcastRefreshAllFeeds => keys.podcast_refresh_all_feeds.mod_key(),
        };

        (MyModifiers::from_modifier_list_index(mod_key.0), mod_key.1)
    }

    /// Try to get a [`BindingForEvent`], if the input is valid
    fn key_event(&mut self) -> Result<BindingForEvent> {
        let mod_list_index = self.component.states.selected;
        let modifier: KeyModifiers = MyModifiers::LIST
            .get(mod_list_index)
            .unwrap_or(&MyModifiers::None)
            .as_modifier();
        self.update_key_input_by_modifier(modifier);
        if let Ok(code) = BindingForEvent::key_from_str(&self.component.states_input.get_value()) {
            return Ok(BindingForEvent { code, modifier });
        }
        bail!("Failed to get the key code!")
    }

    fn update_key_input_by_modifier(&mut self, modifier: KeyModifiers) {
        let codes = self.component.states_input.get_value();
        if BindingForEvent::key_from_str(&codes).is_err() {
            return;
        }

        // For Function keys, no need to change case
        if codes.starts_with('F') {
            return;
        }

        // For other keys, if shift is in modifier, change case accordingly
        if modifier.bits() % 2 == 1 {
            self.component
                .attr_input(Attribute::Value, AttrValue::String(codes.to_uppercase()));
        } else {
            self.component
                .attr_input(Attribute::Value, AttrValue::String(codes.to_lowercase()));
        }
    }
}

impl Component<Msg, NoUserEvent> for KEModifierSelect {
    #[allow(clippy::too_many_lines)]
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        let config = self.config.clone();
        let keys = &config.read().keys;
        let cmd_result = match ev {
            // Global Hotkey
            Event::Keyboard(keyevent) if keyevent == keys.config_save.key_event() => {
                return Some(Msg::ConfigEditor(ConfigEditorMsg::CloseOk));
            }
            Event::Keyboard(KeyEvent {
                code: Key::Down, ..
            }) => match self.state() {
                State::One(_) => return Some(self.on_key_tab.clone()),
                _ => self.perform(Cmd::Move(Direction::Down)),
            },
            Event::Keyboard(KeyEvent { code: Key::Up, .. }) => match self.state() {
                State::One(_) => return Some(self.on_key_backtab.clone()),
                _ => self.perform(Cmd::Move(Direction::Up)),
            },
            Event::Keyboard(KeyEvent { code: Key::Tab, .. }) => {
                return Some(Msg::ConfigEditor(ConfigEditorMsg::ChangeLayout));
            }
            // Local Hotkey
            Event::Keyboard(keyevent) if keyevent == keys.global_esc.key_event() => {
                match self.state() {
                    State::One(_) => return Some(Msg::ConfigEditor(ConfigEditorMsg::CloseCancel)),
                    _ => self.perform(Cmd::Cancel),
                }
            }

            Event::Keyboard(keyevent) if keyevent == keys.global_quit.key_event() => {
                match self.state() {
                    State::One(_) => {
                        if let Key::Char(ch) = keyevent.code {
                            self.perform(Cmd::Type(ch));
                            if let Ok(binding) = self.key_event() {
                                return Some(Msg::ConfigEditor(ConfigEditorMsg::KeyChange(
                                    self.id, binding,
                                )));
                            };
                        }
                        CmdResult::None
                    }

                    _ => self.perform(Cmd::Cancel),
                }
            }

            Event::Keyboard(keyevent) if keyevent == keys.global_up.key_event() => {
                match self.state() {
                    State::One(_) => {
                        if let Key::Char(ch) = keyevent.code {
                            self.perform(Cmd::Type(ch));

                            if let Ok(binding) = self.key_event() {
                                return Some(Msg::ConfigEditor(ConfigEditorMsg::KeyChange(
                                    self.id, binding,
                                )));
                            };
                        }
                        CmdResult::None
                    }

                    _ => self.perform(Cmd::Move(Direction::Up)),
                }
            }
            Event::Keyboard(keyevent) if keyevent == keys.global_down.key_event() => {
                match self.state() {
                    State::One(_) => {
                        if let Key::Char(ch) = keyevent.code {
                            self.perform(Cmd::Type(ch));

                            if let Ok(binding) = self.key_event() {
                                return Some(Msg::ConfigEditor(ConfigEditorMsg::KeyChange(
                                    self.id, binding,
                                )));
                            };
                        }
                        CmdResult::None
                    }
                    _ => self.perform(Cmd::Move(Direction::Down)),
                }
            }
            Event::Keyboard(KeyEvent {
                code: Key::Enter, ..
            }) => self.perform(Cmd::Submit),
            // input

            // Local Hotkeys
            Event::Keyboard(KeyEvent {
                code: Key::Left, ..
            }) => self.perform(Cmd::Move(Direction::Left)),
            Event::Keyboard(KeyEvent {
                code: Key::Right, ..
            }) => self.perform(Cmd::Move(Direction::Right)),
            Event::Keyboard(KeyEvent {
                code: Key::Home, ..
            }) => self.perform(Cmd::GoTo(Position::Begin)),
            Event::Keyboard(KeyEvent { code: Key::End, .. }) => {
                self.perform(Cmd::GoTo(Position::End))
            }
            Event::Keyboard(KeyEvent {
                code: Key::Delete, ..
            }) => self.perform(Cmd::Cancel),
            Event::Keyboard(KeyEvent {
                code: Key::Backspace,
                ..
            }) => self.perform(Cmd::Delete),

            Event::Keyboard(KeyEvent {
                code: Key::Char(ch),
                ..
            }) => {
                self.perform(Cmd::Type(ch));
                match self.state() {
                    State::One(_) => {
                        if let Ok(binding) = self.key_event() {
                            return Some(Msg::ConfigEditor(ConfigEditorMsg::KeyChange(
                                self.id, binding,
                            )));
                        }
                        CmdResult::None
                    }
                    _ => CmdResult::None,
                }
            }
            Event::Keyboard(keyevent) if keyevent == keys.global_esc.key_event() => {
                return Some(Msg::ConfigEditor(ConfigEditorMsg::CloseCancel));
            }
            _ => CmdResult::None,
        };
        match cmd_result {
            CmdResult::Submit(State::One(StateValue::Usize(_index))) => {
                // Some(Msg::None)
                if let Ok(binding) = self.key_event() {
                    return Some(Msg::ConfigEditor(ConfigEditorMsg::KeyChange(
                        self.id, binding,
                    )));
                }
                Some(Msg::None)
            }
            _ => Some(Msg::None),
        }
    }
}

#[derive(MockComponent)]
pub struct ConfigGlobalQuit {
    component: KEModifierSelect,
}

impl ConfigGlobalQuit {
    pub fn new(config: SharedSettings) -> Self {
        Self {
            component: KEModifierSelect::new(
                " Quit ",
                IdKey::GlobalQuit,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::KeyFocus(KFMsg::GlobalQuitBlurDown)),
                Msg::ConfigEditor(ConfigEditorMsg::KeyFocus(KFMsg::GlobalQuitBlurUp)),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigGlobalQuit {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct ConfigGlobalLeft {
    component: KEModifierSelect,
}

impl ConfigGlobalLeft {
    pub fn new(config: SharedSettings) -> Self {
        Self {
            component: KEModifierSelect::new(
                " Left ",
                IdKey::GlobalLeft,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::KeyFocus(KFMsg::GlobalLeftBlurDown)),
                Msg::ConfigEditor(ConfigEditorMsg::KeyFocus(KFMsg::GlobalLeftBlurUp)),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigGlobalLeft {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct ConfigGlobalDown {
    component: KEModifierSelect,
}

impl ConfigGlobalDown {
    pub fn new(config: SharedSettings) -> Self {
        Self {
            component: KEModifierSelect::new(
                " Down ",
                IdKey::GlobalDown,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::KeyFocus(KFMsg::GlobalDownBlurDown)),
                Msg::ConfigEditor(ConfigEditorMsg::KeyFocus(KFMsg::GlobalDownBlurUp)),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigGlobalDown {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}
#[derive(MockComponent)]
pub struct ConfigGlobalRight {
    component: KEModifierSelect,
}

impl ConfigGlobalRight {
    pub fn new(config: SharedSettings) -> Self {
        Self {
            component: KEModifierSelect::new(
                " Right ",
                IdKey::GlobalRight,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::KeyFocus(KFMsg::GlobalRightBlurDown)),
                Msg::ConfigEditor(ConfigEditorMsg::KeyFocus(KFMsg::GlobalRightBlurUp)),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigGlobalRight {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}
#[derive(MockComponent)]
pub struct ConfigGlobalUp {
    component: KEModifierSelect,
}

impl ConfigGlobalUp {
    pub fn new(config: SharedSettings) -> Self {
        Self {
            component: KEModifierSelect::new(
                " Up ",
                IdKey::GlobalUp,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::KeyFocus(KFMsg::GlobalUpBlurDown)),
                Msg::ConfigEditor(ConfigEditorMsg::KeyFocus(KFMsg::GlobalUpBlurUp)),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigGlobalUp {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct ConfigGlobalGotoTop {
    component: KEModifierSelect,
}

impl ConfigGlobalGotoTop {
    pub fn new(config: SharedSettings) -> Self {
        Self {
            component: KEModifierSelect::new(
                " Goto Top ",
                IdKey::GlobalGotoTop,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::KeyFocus(KFMsg::GlobalGotoTopBlurDown)),
                Msg::ConfigEditor(ConfigEditorMsg::KeyFocus(KFMsg::GlobalGotoTopBlurUp)),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigGlobalGotoTop {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct ConfigGlobalGotoBottom {
    component: KEModifierSelect,
}

impl ConfigGlobalGotoBottom {
    pub fn new(config: SharedSettings) -> Self {
        Self {
            component: KEModifierSelect::new(
                " Goto Bottom ",
                IdKey::GlobalGotoBottom,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::KeyFocus(KFMsg::GlobalGotoBottomBlurDown)),
                Msg::ConfigEditor(ConfigEditorMsg::KeyFocus(KFMsg::GlobalGotoBottomBlurUp)),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigGlobalGotoBottom {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct ConfigGlobalPlayerTogglePause {
    component: KEModifierSelect,
}

impl ConfigGlobalPlayerTogglePause {
    pub fn new(config: SharedSettings) -> Self {
        Self {
            component: KEModifierSelect::new(
                " Pause Toggle ",
                IdKey::GlobalPlayerTogglePause,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::KeyFocus(
                    KFMsg::GlobalPlayerTogglePauseBlurDown,
                )),
                Msg::ConfigEditor(ConfigEditorMsg::KeyFocus(
                    KFMsg::GlobalPlayerTogglePauseBlurUp,
                )),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigGlobalPlayerTogglePause {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct ConfigGlobalPlayerNext {
    component: KEModifierSelect,
}

impl ConfigGlobalPlayerNext {
    pub fn new(config: SharedSettings) -> Self {
        Self {
            component: KEModifierSelect::new(
                " Next Song ",
                IdKey::GlobalPlayerNext,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::KeyFocus(KFMsg::GlobalPlayerNextBlurDown)),
                Msg::ConfigEditor(ConfigEditorMsg::KeyFocus(KFMsg::GlobalPlayerNextBlurUp)),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigGlobalPlayerNext {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct ConfigGlobalPlayerPrevious {
    component: KEModifierSelect,
}

impl ConfigGlobalPlayerPrevious {
    pub fn new(config: SharedSettings) -> Self {
        Self {
            component: KEModifierSelect::new(
                " Previous Song ",
                IdKey::GlobalPlayerPrevious,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::KeyFocus(
                    KFMsg::GlobalPlayerPreviousBlurDown,
                )),
                Msg::ConfigEditor(ConfigEditorMsg::KeyFocus(KFMsg::GlobalPlayerPreviousBlurUp)),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigGlobalPlayerPrevious {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct ConfigGlobalHelp {
    component: KEModifierSelect,
}

impl ConfigGlobalHelp {
    pub fn new(config: SharedSettings) -> Self {
        Self {
            component: KEModifierSelect::new(
                " Help ",
                IdKey::GlobalHelp,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::KeyFocus(KFMsg::GlobalHelpBlurDown)),
                Msg::ConfigEditor(ConfigEditorMsg::KeyFocus(KFMsg::GlobalHelpBlurUp)),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigGlobalHelp {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}
#[derive(MockComponent)]
pub struct ConfigGlobalVolumeUp {
    component: KEModifierSelect,
}

impl ConfigGlobalVolumeUp {
    pub fn new(config: SharedSettings) -> Self {
        Self {
            component: KEModifierSelect::new(
                " Volume + ",
                IdKey::GlobalVolumeUp,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::KeyFocus(KFMsg::GlobalVolumeUpBlurDown)),
                Msg::ConfigEditor(ConfigEditorMsg::KeyFocus(KFMsg::GlobalVolumeUpBlurUp)),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigGlobalVolumeUp {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct ConfigGlobalVolumeDown {
    component: KEModifierSelect,
}

impl ConfigGlobalVolumeDown {
    pub fn new(config: SharedSettings) -> Self {
        Self {
            component: KEModifierSelect::new(
                " Volume - ",
                IdKey::GlobalVolumeDown,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::KeyFocus(KFMsg::GlobalVolumeDownBlurDown)),
                Msg::ConfigEditor(ConfigEditorMsg::KeyFocus(KFMsg::GlobalVolumeDownBlurUp)),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigGlobalVolumeDown {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct ConfigGlobalPlayerSeekForward {
    component: KEModifierSelect,
}

impl ConfigGlobalPlayerSeekForward {
    pub fn new(config: SharedSettings) -> Self {
        Self {
            component: KEModifierSelect::new(
                " Seek Forward ",
                IdKey::GlobalPlayerSeekForward,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::KeyFocus(
                    KFMsg::GlobalPlayerSeekForwardBlurDown,
                )),
                Msg::ConfigEditor(ConfigEditorMsg::KeyFocus(
                    KFMsg::GlobalPlayerSeekForwardBlurUp,
                )),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigGlobalPlayerSeekForward {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct ConfigGlobalPlayerSeekBackward {
    component: KEModifierSelect,
}

impl ConfigGlobalPlayerSeekBackward {
    pub fn new(config: SharedSettings) -> Self {
        Self {
            component: KEModifierSelect::new(
                " Seek Backward ",
                IdKey::GlobalPlayerSeekBackward,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::KeyFocus(
                    KFMsg::GlobalPlayerSeekBackwardBlurDown,
                )),
                Msg::ConfigEditor(ConfigEditorMsg::KeyFocus(
                    KFMsg::GlobalPlayerSeekBackwardBlurUp,
                )),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigGlobalPlayerSeekBackward {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct ConfigGlobalPlayerSpeedUp {
    component: KEModifierSelect,
}

impl ConfigGlobalPlayerSpeedUp {
    pub fn new(config: SharedSettings) -> Self {
        Self {
            component: KEModifierSelect::new(
                " Speed Up ",
                IdKey::GlobalPlayerSpeedUp,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::KeyFocus(
                    KFMsg::GlobalPlayerSpeedUpBlurDown,
                )),
                Msg::ConfigEditor(ConfigEditorMsg::KeyFocus(KFMsg::GlobalPlayerSpeedUpBlurUp)),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigGlobalPlayerSpeedUp {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct ConfigGlobalPlayerSpeedDown {
    component: KEModifierSelect,
}

impl ConfigGlobalPlayerSpeedDown {
    pub fn new(config: SharedSettings) -> Self {
        Self {
            component: KEModifierSelect::new(
                " Speed Down ",
                IdKey::GlobalPlayerSpeedDown,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::KeyFocus(
                    KFMsg::GlobalPlayerSpeedDownBlurDown,
                )),
                Msg::ConfigEditor(ConfigEditorMsg::KeyFocus(
                    KFMsg::GlobalPlayerSpeedDownBlurUp,
                )),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigGlobalPlayerSpeedDown {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct ConfigGlobalLyricAdjustForward {
    component: KEModifierSelect,
}

impl ConfigGlobalLyricAdjustForward {
    pub fn new(config: SharedSettings) -> Self {
        Self {
            component: KEModifierSelect::new(
                " Lyric Forward ",
                IdKey::GlobalLyricAdjustForward,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::KeyFocus(
                    KFMsg::GlobalLyricAdjustForwardBlurDown,
                )),
                Msg::ConfigEditor(ConfigEditorMsg::KeyFocus(
                    KFMsg::GlobalLyricAdjustForwardBlurUp,
                )),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigGlobalLyricAdjustForward {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct ConfigGlobalLyricAdjustBackward {
    component: KEModifierSelect,
}

impl ConfigGlobalLyricAdjustBackward {
    pub fn new(config: SharedSettings) -> Self {
        Self {
            component: KEModifierSelect::new(
                " Lyric Backward ",
                IdKey::GlobalLyricAdjustBackward,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::KeyFocus(
                    KFMsg::GlobalLyricAdjustBackwardBlurDown,
                )),
                Msg::ConfigEditor(ConfigEditorMsg::KeyFocus(
                    KFMsg::GlobalLyricAdjustBackwardBlurUp,
                )),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigGlobalLyricAdjustBackward {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct ConfigGlobalLyricCycle {
    component: KEModifierSelect,
}

impl ConfigGlobalLyricCycle {
    pub fn new(config: SharedSettings) -> Self {
        Self {
            component: KEModifierSelect::new(
                " Lyric Cycle ",
                IdKey::GlobalLyricCycle,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::KeyFocus(KFMsg::GlobalLyricCycleBlurDown)),
                Msg::ConfigEditor(ConfigEditorMsg::KeyFocus(KFMsg::GlobalLyricCycleBlurUp)),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigGlobalLyricCycle {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct ConfigGlobalLayoutTreeview {
    component: KEModifierSelect,
}

impl ConfigGlobalLayoutTreeview {
    pub fn new(config: SharedSettings) -> Self {
        Self {
            component: KEModifierSelect::new(
                " Layout Tree ",
                IdKey::GlobalLayoutTreeview,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::KeyFocus(
                    KFMsg::GlobalLayoutTreeviewBlurDown,
                )),
                Msg::ConfigEditor(ConfigEditorMsg::KeyFocus(KFMsg::GlobalLayoutTreeviewBlurUp)),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigGlobalLayoutTreeview {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct ConfigGlobalLayoutDatabase {
    component: KEModifierSelect,
}

impl ConfigGlobalLayoutDatabase {
    pub fn new(config: SharedSettings) -> Self {
        Self {
            component: KEModifierSelect::new(
                " Layout DataBase ",
                IdKey::GlobalLayoutDatabase,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::KeyFocus(
                    KFMsg::GlobalLayoutDatabaseBlurDown,
                )),
                Msg::ConfigEditor(ConfigEditorMsg::KeyFocus(KFMsg::GlobalLayoutDatabaseBlurUp)),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigGlobalLayoutDatabase {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct ConfigGlobalPlayerToggleGapless {
    component: KEModifierSelect,
}

impl ConfigGlobalPlayerToggleGapless {
    pub fn new(config: SharedSettings) -> Self {
        Self {
            component: KEModifierSelect::new(
                " Gapless Toggle ",
                IdKey::GlobalPlayerToggleGapless,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::KeyFocus(
                    KFMsg::GlobalPlayerToggleGaplessBlurDown,
                )),
                Msg::ConfigEditor(ConfigEditorMsg::KeyFocus(
                    KFMsg::GlobalPlayerToggleGaplessBlurUp,
                )),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigGlobalPlayerToggleGapless {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct ConfigLibraryDelete {
    component: KEModifierSelect,
}

impl ConfigLibraryDelete {
    pub fn new(config: SharedSettings) -> Self {
        Self {
            component: KEModifierSelect::new(
                " Library Delete ",
                IdKey::LibraryDelete,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::KeyFocus(KFMsg::LibraryDeleteBlurDown)),
                Msg::ConfigEditor(ConfigEditorMsg::KeyFocus(KFMsg::LibraryDeleteBlurUp)),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigLibraryDelete {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct ConfigLibraryLoadDir {
    component: KEModifierSelect,
}

impl ConfigLibraryLoadDir {
    pub fn new(config: SharedSettings) -> Self {
        Self {
            component: KEModifierSelect::new(
                " Library Load Dir ",
                IdKey::LibraryLoadDir,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::KeyFocus(KFMsg::LibraryLoadDirBlurDown)),
                Msg::ConfigEditor(ConfigEditorMsg::KeyFocus(KFMsg::LibraryLoadDirBlurUp)),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigLibraryLoadDir {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct ConfigLibraryYank {
    component: KEModifierSelect,
}

impl ConfigLibraryYank {
    pub fn new(config: SharedSettings) -> Self {
        Self {
            component: KEModifierSelect::new(
                " Library Yank ",
                IdKey::LibraryYank,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::KeyFocus(KFMsg::LibraryYankBlurDown)),
                Msg::ConfigEditor(ConfigEditorMsg::KeyFocus(KFMsg::LibraryYankBlurUp)),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigLibraryYank {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct ConfigLibraryPaste {
    component: KEModifierSelect,
}

impl ConfigLibraryPaste {
    pub fn new(config: SharedSettings) -> Self {
        Self {
            component: KEModifierSelect::new(
                " Library Paste ",
                IdKey::LibraryPaste,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::KeyFocus(KFMsg::LibraryPasteBlurDown)),
                Msg::ConfigEditor(ConfigEditorMsg::KeyFocus(KFMsg::LibraryPasteBlurUp)),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigLibraryPaste {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct ConfigLibrarySearch {
    component: KEModifierSelect,
}

impl ConfigLibrarySearch {
    pub fn new(config: SharedSettings) -> Self {
        Self {
            component: KEModifierSelect::new(
                " Library Search ",
                IdKey::LibrarySearch,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::KeyFocus(KFMsg::LibrarySearchBlurDown)),
                Msg::ConfigEditor(ConfigEditorMsg::KeyFocus(KFMsg::LibrarySearchBlurUp)),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigLibrarySearch {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct ConfigLibrarySearchYoutube {
    component: KEModifierSelect,
}

impl ConfigLibrarySearchYoutube {
    pub fn new(config: SharedSettings) -> Self {
        Self {
            component: KEModifierSelect::new(
                " Library Search Youtube ",
                IdKey::LibrarySearchYoutube,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::KeyFocus(
                    KFMsg::LibrarySearchYoutubeBlurDown,
                )),
                Msg::ConfigEditor(ConfigEditorMsg::KeyFocus(KFMsg::LibrarySearchYoutubeBlurUp)),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigLibrarySearchYoutube {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct ConfigLibraryTagEditor {
    component: KEModifierSelect,
}

impl ConfigLibraryTagEditor {
    pub fn new(config: SharedSettings) -> Self {
        Self {
            component: KEModifierSelect::new(
                " Library Tag Editor ",
                IdKey::LibraryTagEditor,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::KeyFocus(KFMsg::LibraryTagEditorBlurDown)),
                Msg::ConfigEditor(ConfigEditorMsg::KeyFocus(KFMsg::LibraryTagEditorBlurUp)),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigLibraryTagEditor {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct ConfigPlaylistDelete {
    component: KEModifierSelect,
}

impl ConfigPlaylistDelete {
    pub fn new(config: SharedSettings) -> Self {
        Self {
            component: KEModifierSelect::new(
                " Playlist Delete ",
                IdKey::PlaylistDelete,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::KeyFocus(KFMsg::PlaylistDeleteBlurDown)),
                Msg::ConfigEditor(ConfigEditorMsg::KeyFocus(KFMsg::PlaylistDeleteBlurUp)),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigPlaylistDelete {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct ConfigPlaylistDeleteAll {
    component: KEModifierSelect,
}

impl ConfigPlaylistDeleteAll {
    pub fn new(config: SharedSettings) -> Self {
        Self {
            component: KEModifierSelect::new(
                " Playlist Delete All ",
                IdKey::PlaylistDeleteAll,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::KeyFocus(KFMsg::PlaylistDeleteAllBlurDown)),
                Msg::ConfigEditor(ConfigEditorMsg::KeyFocus(KFMsg::PlaylistDeleteAllBlurUp)),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigPlaylistDeleteAll {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct ConfigPlaylistShuffle {
    component: KEModifierSelect,
}

impl ConfigPlaylistShuffle {
    pub fn new(config: SharedSettings) -> Self {
        Self {
            component: KEModifierSelect::new(
                " Playlist Shuffle ",
                IdKey::PlaylistShuffle,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::KeyFocus(KFMsg::PlaylistShuffleBlurDown)),
                Msg::ConfigEditor(ConfigEditorMsg::KeyFocus(KFMsg::PlaylistShuffleBlurUp)),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigPlaylistShuffle {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct ConfigPlaylistModeCycle {
    component: KEModifierSelect,
}

impl ConfigPlaylistModeCycle {
    pub fn new(config: SharedSettings) -> Self {
        Self {
            component: KEModifierSelect::new(
                " Playlist Mode Cycle ",
                IdKey::PlaylistModeCycle,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::KeyFocus(KFMsg::PlaylistModeCycleBlurDown)),
                Msg::ConfigEditor(ConfigEditorMsg::KeyFocus(KFMsg::PlaylistModeCycleBlurUp)),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigPlaylistModeCycle {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct ConfigPlaylistPlaySelected {
    component: KEModifierSelect,
}

impl ConfigPlaylistPlaySelected {
    pub fn new(config: SharedSettings) -> Self {
        Self {
            component: KEModifierSelect::new(
                " Playlist Play Selected ",
                IdKey::PlaylistPlaySelected,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::KeyFocus(
                    KFMsg::PlaylistPlaySelectedBlurDown,
                )),
                Msg::ConfigEditor(ConfigEditorMsg::KeyFocus(KFMsg::PlaylistPlaySelectedBlurUp)),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigPlaylistPlaySelected {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct ConfigPlaylistSearch {
    component: KEModifierSelect,
}

impl ConfigPlaylistSearch {
    pub fn new(config: SharedSettings) -> Self {
        Self {
            component: KEModifierSelect::new(
                " Playlist Search ",
                IdKey::PlaylistSearch,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::KeyFocus(KFMsg::PlaylistSearchBlurDown)),
                Msg::ConfigEditor(ConfigEditorMsg::KeyFocus(KFMsg::PlaylistSearchBlurUp)),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigPlaylistSearch {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct ConfigPlaylistSwapDown {
    component: KEModifierSelect,
}

impl ConfigPlaylistSwapDown {
    pub fn new(config: SharedSettings) -> Self {
        Self {
            component: KEModifierSelect::new(
                " Playlist Swap Down ",
                IdKey::PlaylistSwapDown,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::KeyFocus(KFMsg::PlaylistSwapDownBlurDown)),
                Msg::ConfigEditor(ConfigEditorMsg::KeyFocus(KFMsg::PlaylistSwapDownBlurUp)),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigPlaylistSwapDown {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct ConfigPlaylistSwapUp {
    component: KEModifierSelect,
}

impl ConfigPlaylistSwapUp {
    pub fn new(config: SharedSettings) -> Self {
        Self {
            component: KEModifierSelect::new(
                " Playlist Swap Up ",
                IdKey::PlaylistSwapUp,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::KeyFocus(KFMsg::PlaylistSwapUpBlurDown)),
                Msg::ConfigEditor(ConfigEditorMsg::KeyFocus(KFMsg::PlaylistSwapUpBlurUp)),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigPlaylistSwapUp {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}
#[derive(MockComponent)]
pub struct ConfigDatabaseAddAll {
    component: KEModifierSelect,
}

impl ConfigDatabaseAddAll {
    pub fn new(config: SharedSettings) -> Self {
        Self {
            component: KEModifierSelect::new(
                " Database Add All ",
                IdKey::DatabaseAddAll,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::KeyFocus(KFMsg::DatabaseAddAllBlurDown)),
                Msg::ConfigEditor(ConfigEditorMsg::KeyFocus(KFMsg::DatabaseAddAllBlurUp)),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigDatabaseAddAll {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct ConfigGlobalConfig {
    component: KEModifierSelect,
}

impl ConfigGlobalConfig {
    pub fn new(config: SharedSettings) -> Self {
        Self {
            component: KEModifierSelect::new(
                " Config Editor ",
                IdKey::GlobalConfig,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::KeyFocus(KFMsg::GlobalConfigBlurDown)),
                Msg::ConfigEditor(ConfigEditorMsg::KeyFocus(KFMsg::GlobalConfigBlurUp)),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigGlobalConfig {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct ConfigPlaylistAddRandomAlbum {
    component: KEModifierSelect,
}

impl ConfigPlaylistAddRandomAlbum {
    pub fn new(config: SharedSettings) -> Self {
        Self {
            component: KEModifierSelect::new(
                " Playlist Select Album ",
                IdKey::PlaylistAddRandomAlbum,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::KeyFocus(
                    KFMsg::PlaylistAddRandomAlbumBlurDown,
                )),
                Msg::ConfigEditor(ConfigEditorMsg::KeyFocus(
                    KFMsg::PlaylistAddRandomAlbumBlurUp,
                )),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigPlaylistAddRandomAlbum {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct ConfigPlaylistAddRandomTracks {
    component: KEModifierSelect,
}

impl ConfigPlaylistAddRandomTracks {
    pub fn new(config: SharedSettings) -> Self {
        Self {
            component: KEModifierSelect::new(
                " Playlist Select Tracks ",
                IdKey::PlaylistAddRandomTracks,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::KeyFocus(
                    KFMsg::PlaylistAddRandomTracksBlurDown,
                )),
                Msg::ConfigEditor(ConfigEditorMsg::KeyFocus(
                    KFMsg::PlaylistAddRandomTracksBlurUp,
                )),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigPlaylistAddRandomTracks {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct ConfigLibrarySwitchRoot {
    component: KEModifierSelect,
}

impl ConfigLibrarySwitchRoot {
    pub fn new(config: SharedSettings) -> Self {
        Self {
            component: KEModifierSelect::new(
                " Library Switch Root ",
                IdKey::LibrarySwitchRoot,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::KeyFocus(KFMsg::LibrarySwitchRootBlurDown)),
                Msg::ConfigEditor(ConfigEditorMsg::KeyFocus(KFMsg::LibrarySwitchRootBlurUp)),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigLibrarySwitchRoot {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct ConfigLibraryAddRoot {
    component: KEModifierSelect,
}

impl ConfigLibraryAddRoot {
    pub fn new(config: SharedSettings) -> Self {
        Self {
            component: KEModifierSelect::new(
                " Library Add Root ",
                IdKey::LibraryAddRoot,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::KeyFocus(KFMsg::LibraryAddRootBlurDown)),
                Msg::ConfigEditor(ConfigEditorMsg::KeyFocus(KFMsg::LibraryAddRootBlurUp)),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigLibraryAddRoot {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct ConfigLibraryRemoveRoot {
    component: KEModifierSelect,
}

impl ConfigLibraryRemoveRoot {
    pub fn new(config: SharedSettings) -> Self {
        Self {
            component: KEModifierSelect::new(
                " Library Remove Root ",
                IdKey::LibraryRemoveRoot,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::KeyFocus(KFMsg::LibraryRemoveRootBlurDown)),
                Msg::ConfigEditor(ConfigEditorMsg::KeyFocus(KFMsg::LibraryRemoveRootBlurUp)),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigLibraryRemoveRoot {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct ConfigGlobalSavePlaylist {
    component: KEModifierSelect,
}

impl ConfigGlobalSavePlaylist {
    pub fn new(config: SharedSettings) -> Self {
        Self {
            component: KEModifierSelect::new(
                " Global Save Playlist ",
                IdKey::GlobalSavePlaylist,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::KeyFocus(KFMsg::GlobalSavePlaylistBlurDown)),
                Msg::ConfigEditor(ConfigEditorMsg::KeyFocus(KFMsg::GlobalSavePlaylistBlurUp)),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigGlobalSavePlaylist {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct ConfigGlobalLayoutPodcast {
    component: KEModifierSelect,
}

impl ConfigGlobalLayoutPodcast {
    pub fn new(config: SharedSettings) -> Self {
        Self {
            component: KEModifierSelect::new(
                " Layout Podcast ",
                IdKey::GlobalLayoutPodcast,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::KeyFocus(
                    KFMsg::GlobalLayoutPodcastBlurDown,
                )),
                Msg::ConfigEditor(ConfigEditorMsg::KeyFocus(KFMsg::GlobalLayoutPodcastBlurUp)),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigGlobalLayoutPodcast {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct ConfigGlobalXywhMoveLeft {
    component: KEModifierSelect,
}

impl ConfigGlobalXywhMoveLeft {
    pub fn new(config: SharedSettings) -> Self {
        Self {
            component: KEModifierSelect::new(
                " Photo move left ",
                IdKey::GlobalXywhMoveLeft,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::KeyFocus(KFMsg::GlobalXywhMoveLeftBlurDown)),
                Msg::ConfigEditor(ConfigEditorMsg::KeyFocus(KFMsg::GlobalXywhMoveLeftBlurUp)),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigGlobalXywhMoveLeft {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct ConfigGlobalXywhMoveRight {
    component: KEModifierSelect,
}

impl ConfigGlobalXywhMoveRight {
    pub fn new(config: SharedSettings) -> Self {
        Self {
            component: KEModifierSelect::new(
                " Photo move right ",
                IdKey::GlobalXywhMoveRight,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::KeyFocus(
                    KFMsg::GlobalXywhMoveRightBlurDown,
                )),
                Msg::ConfigEditor(ConfigEditorMsg::KeyFocus(KFMsg::GlobalXywhMoveRightBlurUp)),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigGlobalXywhMoveRight {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}
#[derive(MockComponent)]
pub struct ConfigGlobalXywhMoveUp {
    component: KEModifierSelect,
}

impl ConfigGlobalXywhMoveUp {
    pub fn new(config: SharedSettings) -> Self {
        Self {
            component: KEModifierSelect::new(
                " Photo move up ",
                IdKey::GlobalXywhMoveUp,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::KeyFocus(KFMsg::GlobalXywhMoveUpBlurDown)),
                Msg::ConfigEditor(ConfigEditorMsg::KeyFocus(KFMsg::GlobalXywhMoveUpBlurUp)),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigGlobalXywhMoveUp {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}
#[derive(MockComponent)]
pub struct ConfigGlobalXywhMoveDown {
    component: KEModifierSelect,
}

impl ConfigGlobalXywhMoveDown {
    pub fn new(config: SharedSettings) -> Self {
        Self {
            component: KEModifierSelect::new(
                " Photo move down ",
                IdKey::GlobalXywhMoveDown,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::KeyFocus(KFMsg::GlobalXywhMoveDownBlurDown)),
                Msg::ConfigEditor(ConfigEditorMsg::KeyFocus(KFMsg::GlobalXywhMoveDownBlurUp)),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigGlobalXywhMoveDown {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct ConfigGlobalXywhZoomIn {
    component: KEModifierSelect,
}

impl ConfigGlobalXywhZoomIn {
    pub fn new(config: SharedSettings) -> Self {
        Self {
            component: KEModifierSelect::new(
                " Photo zoom in ",
                IdKey::GlobalXywhZoomIn,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::KeyFocus(KFMsg::GlobalXywhZoomInBlurDown)),
                Msg::ConfigEditor(ConfigEditorMsg::KeyFocus(KFMsg::GlobalXywhZoomInBlurUp)),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigGlobalXywhZoomIn {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}
#[derive(MockComponent)]
pub struct ConfigGlobalXywhZoomOut {
    component: KEModifierSelect,
}

impl ConfigGlobalXywhZoomOut {
    pub fn new(config: SharedSettings) -> Self {
        Self {
            component: KEModifierSelect::new(
                " Photo zoom out ",
                IdKey::GlobalXywhZoomOut,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::KeyFocus(KFMsg::GlobalXywhZoomOutBlurDown)),
                Msg::ConfigEditor(ConfigEditorMsg::KeyFocus(KFMsg::GlobalXywhZoomOutBlurUp)),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigGlobalXywhZoomOut {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct ConfigGlobalXywhHide {
    component: KEModifierSelect,
}

impl ConfigGlobalXywhHide {
    pub fn new(config: SharedSettings) -> Self {
        Self {
            component: KEModifierSelect::new(
                " Photo hide ",
                IdKey::GlobalXywhHide,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::KeyFocus(KFMsg::GlobalXywhHideBlurDown)),
                Msg::ConfigEditor(ConfigEditorMsg::KeyFocus(KFMsg::GlobalXywhHideBlurUp)),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigGlobalXywhHide {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct ConfigPodcastMarkPlayed {
    component: KEModifierSelect,
}

impl ConfigPodcastMarkPlayed {
    pub fn new(config: SharedSettings) -> Self {
        Self {
            component: KEModifierSelect::new(
                " Episode Mark Played",
                IdKey::PodcastMarkPlayed,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::KeyFocus(KFMsg::PodcastMarkPlayedBlurDown)),
                Msg::ConfigEditor(ConfigEditorMsg::KeyFocus(KFMsg::PodcastMarkPlayedBlurUp)),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigPodcastMarkPlayed {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct ConfigPodcastMarkAllPlayed {
    component: KEModifierSelect,
}

impl ConfigPodcastMarkAllPlayed {
    pub fn new(config: SharedSettings) -> Self {
        Self {
            component: KEModifierSelect::new(
                " Episode mark all played ",
                IdKey::PodcastMarkAllPlayed,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::KeyFocus(
                    KFMsg::PodcastMarkAllPlayedBlurDown,
                )),
                Msg::ConfigEditor(ConfigEditorMsg::KeyFocus(KFMsg::PodcastMarkAllPlayedBlurUp)),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigPodcastMarkAllPlayed {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct ConfigPodcastEpDownload {
    component: KEModifierSelect,
}

impl ConfigPodcastEpDownload {
    pub fn new(config: SharedSettings) -> Self {
        Self {
            component: KEModifierSelect::new(
                " Episode download",
                IdKey::PodcastEpDownload,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::KeyFocus(KFMsg::PodcastEpDownloadBlurDown)),
                Msg::ConfigEditor(ConfigEditorMsg::KeyFocus(KFMsg::PodcastEpDownloadBlurUp)),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigPodcastEpDownload {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct ConfigPodcastEpDeleteFile {
    component: KEModifierSelect,
}

impl ConfigPodcastEpDeleteFile {
    pub fn new(config: SharedSettings) -> Self {
        Self {
            component: KEModifierSelect::new(
                " Episode delete file ",
                IdKey::PodcastEpDeleteFile,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::KeyFocus(
                    KFMsg::PodcastEpDeleteFileBlurDown,
                )),
                Msg::ConfigEditor(ConfigEditorMsg::KeyFocus(KFMsg::PodcastEpDeleteFileBlurUp)),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigPodcastEpDeleteFile {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct ConfigPodcastDeleteFeed {
    component: KEModifierSelect,
}

impl ConfigPodcastDeleteFeed {
    pub fn new(config: SharedSettings) -> Self {
        Self {
            component: KEModifierSelect::new(
                " Podcast delete feed ",
                IdKey::PodcastDeleteFeed,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::KeyFocus(KFMsg::PodcastDeleteFeedBlurDown)),
                Msg::ConfigEditor(ConfigEditorMsg::KeyFocus(KFMsg::PodcastDeleteFeedBlurUp)),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigPodcastDeleteFeed {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct ConfigPodcastDeleteAllFeeds {
    component: KEModifierSelect,
}

impl ConfigPodcastDeleteAllFeeds {
    pub fn new(config: SharedSettings) -> Self {
        Self {
            component: KEModifierSelect::new(
                " Delete all feeds ",
                IdKey::PodcastDeleteAllFeeds,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::KeyFocus(
                    KFMsg::PodcastDeleteAllFeedsBlurDown,
                )),
                Msg::ConfigEditor(ConfigEditorMsg::KeyFocus(
                    KFMsg::PodcastDeleteAllFeedsBlurUp,
                )),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigPodcastDeleteAllFeeds {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct ConfigPodcastSearchAddFeed {
    component: KEModifierSelect,
}

impl ConfigPodcastSearchAddFeed {
    pub fn new(config: SharedSettings) -> Self {
        Self {
            component: KEModifierSelect::new(
                " Podcast search add feed ",
                IdKey::PodcastSearchAddFeed,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::KeyFocus(
                    KFMsg::PodcastSearchAddFeedBlurDown,
                )),
                Msg::ConfigEditor(ConfigEditorMsg::KeyFocus(KFMsg::PodcastSearchAddFeedBlurUp)),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigPodcastSearchAddFeed {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct ConfigPodcastRefreshFeed {
    component: KEModifierSelect,
}

impl ConfigPodcastRefreshFeed {
    pub fn new(config: SharedSettings) -> Self {
        Self {
            component: KEModifierSelect::new(
                " Refresh feed ",
                IdKey::PodcastRefreshFeed,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::KeyFocus(KFMsg::PodcastRefreshFeedBlurDown)),
                Msg::ConfigEditor(ConfigEditorMsg::KeyFocus(KFMsg::PodcastRefreshFeedBlurUp)),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigPodcastRefreshFeed {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct ConfigPodcastRefreshAllFeeds {
    component: KEModifierSelect,
}

impl ConfigPodcastRefreshAllFeeds {
    pub fn new(config: SharedSettings) -> Self {
        Self {
            component: KEModifierSelect::new(
                " Refresh all feeds ",
                IdKey::PodcastRefreshAllFeeds,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::KeyFocus(
                    KFMsg::PodcastRefreshAllFeedsBlurDown,
                )),
                Msg::ConfigEditor(ConfigEditorMsg::KeyFocus(
                    KFMsg::PodcastRefreshAllFeedsBlurUp,
                )),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigPodcastRefreshAllFeeds {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}
