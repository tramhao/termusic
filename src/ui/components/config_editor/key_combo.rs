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
use crate::config::{Settings, ALT_SHIFT, CONTROL_ALT, CONTROL_ALT_SHIFT, CONTROL_SHIFT};
use crate::ui::{ConfigEditorMsg, IdConfigEditor, Msg};
use tui_realm_stdlib::utils::get_block;
use tuirealm::command::{Cmd, CmdResult, Direction};
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

#[derive(Debug, Clone, PartialEq)]
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
impl From<MyModifiers> for &'static str {
    fn from(modifier: MyModifiers) -> Self {
        match modifier {
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
}

impl From<MyModifiers> for String {
    fn from(modifier: MyModifiers) -> Self {
        <MyModifiers as Into<&'static str>>::into(modifier).to_owned()
    }
}

impl MyModifiers {
    pub const fn modifier(&self) -> KeyModifiers {
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
const MODIFIER_LIST: [MyModifiers; 8] = [
    MyModifiers::None,
    MyModifiers::Shift,
    MyModifiers::Control,
    MyModifiers::Alt,
    MyModifiers::ControlShift,
    MyModifiers::AltShift,
    MyModifiers::ControlAlt,
    MyModifiers::ControlAltShift,
];

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
        // let value = self.states.get_value();
        // self.get_input_type().validate(value.as_str())
        true
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

        // self.attr(Attribute::Value, AttrValue::String(s.as_ref().to_string()));

        self.attr(
            Attribute::Value,
            AttrValue::Payload(PropPayload::One(PropValue::Usize(i))),
        );
        self
    }

    /// ### `render_open_tab`
    ///
    /// Render component when tab is open
    fn render_open_tab(&mut self, render: &mut Frame<'_>, area: Rect) {
        // Make choices
        let choices: Vec<ListItem<'_>> = self
            .states
            .choices
            .iter()
            .map(|x| ListItem::new(Spans::from(x.clone())))
            .collect();
        let foreground = self
            .props
            .get_or(Attribute::Foreground, AttrValue::Color(Color::Reset))
            .unwrap_color();
        let background = self
            .props
            .get_or(Attribute::Background, AttrValue::Color(Color::Reset))
            .unwrap_color();
        let hg: Color = self
            .props
            .get_or(Attribute::HighlightedColor, AttrValue::Color(foreground))
            .unwrap_color();
        // Prepare layout
        let chunks = Layout::default()
            .direction(LayoutDirection::Horizontal)
            .margin(0)
            .constraints([Constraint::Ratio(2, 3), Constraint::Ratio(1, 3)].as_ref())
            .split(area);
        let chunks_left = Layout::default()
            .direction(LayoutDirection::Vertical)
            .margin(0)
            .constraints([Constraint::Length(2), Constraint::Min(1)].as_ref())
            .split(chunks[0]);
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
            .style(Style::default().bg(background));
        let title = self
            .props
            .get(Attribute::Title)
            .map(tuirealm::AttrValue::unwrap_title);
        let block = match title {
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
        let p: Paragraph<'_> = Paragraph::new(selected_text)
            .style(if focus {
                borders.style()
            } else {
                inactive_style.unwrap_or_default()
            })
            .block(block);
        render.render_widget(p, chunks_left[0]);
        // Render the list of elements in chunks [1]
        // Make list
        let mut list = List::new(choices)
            .block(
                Block::default()
                    .borders(BorderSides::LEFT | BorderSides::BOTTOM | BorderSides::RIGHT)
                    .border_style(if focus {
                        borders.style()
                    } else {
                        Style::default()
                    })
                    .style(Style::default().bg(background)),
            )
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
        render.render_stateful_widget(list, chunks_left[1], &mut state);
    }

    /// ### `render_closed_tab`
    ///
    /// Render component when tab is closed
    fn render_closed_tab(&self, render: &mut Frame<'_>, area: Rect) {
        let foreground = self
            .props
            .get_or(Attribute::Foreground, AttrValue::Color(Color::Reset))
            .unwrap_color();
        let background = self
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
        let style = if focus {
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
            .border_style(borders_style)
            .style(style);
        // let title = self.props.get(Attribute::Title).map(|x| x.unwrap_title());
        let title = self
            .props
            .get(Attribute::Title)
            .map(tuirealm::AttrValue::unwrap_title);
        let block = match title {
            Some((text, alignment)) => block.title(text).title_alignment(alignment),
            None => block,
        };
        let selected_text: String = match self.states.choices.get(self.states.selected) {
            None => String::default(),
            Some(s) => s.clone(),
        };
        let p: Paragraph<'_> = Paragraph::new(selected_text).style(style).block(block);

        let chunks = Layout::default()
            .direction(LayoutDirection::Horizontal)
            .margin(0)
            .constraints([Constraint::Ratio(2, 3), Constraint::Ratio(1, 3)].as_ref())
            .split(area);
        render.render_widget(p, chunks[0]);

        let text_to_display = "abc";

        let title = self
            .props
            .get_or(
                Attribute::Title,
                AttrValue::Title((String::default(), Alignment::Center)),
            )
            .unwrap_title();
        let mut block = get_block(borders, Some(title), focus, inactive_style);
        // Create widget
        let p: Paragraph<'_> = Paragraph::new(text_to_display).style(style).block(block);
        render.render_widget(p, chunks[1]);
        // Set cursor, if focus
        if focus {
            let x: u16 = area.x
                + calc_utf8_cursor_position(
                    &self.states_input.render_value_chars()[0..self.states_input.cursor],
                )
                + 1;
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
                self.render_open_tab(render, area);
            } else {
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
                CmdResult::Changed(self.state())
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
        states.set_choices(&choices);
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
        states.set_choices(&choices);
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
            .title("C'est oui ou bien c'est non?", Alignment::Center)
            .choices(&["Oui!", "Non", "Peut-être"])
            .value(1, "abc")
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
    id: IdConfigEditor,
    config: Settings,
    on_key_tab: Msg,
    on_key_backtab: Msg,
}

impl KEModifierSelect {
    pub fn new(
        name: &str,
        id: IdConfigEditor,
        config: &Settings,
        on_key_tab: Msg,
        on_key_backtab: Msg,
    ) -> Self {
        let init_value = Self::init_modifier_select(&id, config);
        let mut choices = vec![];
        for modifier in &MODIFIER_LIST {
            choices.push(String::from(modifier.clone()));
        }
        Self {
            component: KeyCombo::default()
                .borders(
                    Borders::default().modifiers(BorderType::Rounded).color(
                        config
                            .style_color_symbol
                            .library_border()
                            .unwrap_or(Color::Blue),
                    ),
                )
                .foreground(
                    config
                        .style_color_symbol
                        .library_foreground()
                        .unwrap_or(Color::Blue),
                )
                .title(name, Alignment::Left)
                .rewind(false)
                // .inactive(Style::default().bg(Color::Green))
                .highlighted_color(
                    config
                        .style_color_symbol
                        .library_highlight()
                        .unwrap_or(Color::LightGreen),
                )
                .highlighted_str(">> ")
                .choices(&choices)
                .value(init_value, "abc"),
            id,
            config: config.clone(),
            on_key_tab,
            on_key_backtab,
        }
    }

    const fn init_modifier_select(id: &IdConfigEditor, config: &Settings) -> usize {
        match *id {
            IdConfigEditor::GlobalQuit => config.keys.global_quit.modifier(),
            IdConfigEditor::GlobalLeft => config.keys.global_left.modifier(),
            IdConfigEditor::GlobalRight => config.keys.global_right.modifier(),
            IdConfigEditor::GlobalUp => config.keys.global_up.modifier(),
            IdConfigEditor::GlobalDown => config.keys.global_down.modifier(),
            IdConfigEditor::GlobalGotoTop => config.keys.global_goto_top.modifier(),
            IdConfigEditor::GlobalGotoBottom => config.keys.global_goto_bottom.modifier(),
            IdConfigEditor::GlobalPlayerTogglePause => {
                config.keys.global_player_toggle_pause.modifier()
            }
            IdConfigEditor::GlobalPlayerNext => config.keys.global_player_next.modifier(),
            IdConfigEditor::GlobalPlayerPrevious => config.keys.global_player_previous.modifier(),
            IdConfigEditor::GlobalHelp => config.keys.global_help.modifier(),
            IdConfigEditor::GlobalVolumeUp => config.keys.global_player_volume_plus_2.modifier(),
            IdConfigEditor::GlobalVolumeDown => config.keys.global_player_volume_minus_2.modifier(),
            IdConfigEditor::GlobalPlayerSeekForward => {
                config.keys.global_player_seek_forward.modifier()
            }
            IdConfigEditor::GlobalPlayerSeekBackward => {
                config.keys.global_player_seek_backward.modifier()
            }
            IdConfigEditor::GlobalPlayerSpeedUp => config.keys.global_player_speed_up.modifier(),
            IdConfigEditor::GlobalPlayerSpeedDown => {
                config.keys.global_player_speed_down.modifier()
            }
            IdConfigEditor::GlobalLyricAdjustForward => {
                config.keys.global_lyric_adjust_forward.modifier()
            }
            IdConfigEditor::GlobalLyricAdjustBackward => {
                config.keys.global_lyric_adjust_backward.modifier()
            }
            IdConfigEditor::GlobalLyricCycle => config.keys.global_lyric_cycle.modifier(),
            IdConfigEditor::GlobalLayoutTreeview => config.keys.global_layout_treeview.modifier(),
            IdConfigEditor::GlobalLayoutDatabase => config.keys.global_layout_database.modifier(),
            IdConfigEditor::GlobalPlayerToggleGapless => {
                config.keys.global_player_toggle_gapless.modifier()
            }
            IdConfigEditor::LibraryDelete => config.keys.library_delete.modifier(),
            IdConfigEditor::LibraryLoadDir => config.keys.library_load_dir.modifier(),
            IdConfigEditor::LibraryYank => config.keys.library_yank.modifier(),
            IdConfigEditor::LibraryPaste => config.keys.library_paste.modifier(),
            IdConfigEditor::LibrarySearch => config.keys.library_search.modifier(),
            IdConfigEditor::LibrarySearchYoutube => config.keys.library_search_youtube.modifier(),
            IdConfigEditor::LibraryTagEditor => config.keys.library_tag_editor_open.modifier(),
            IdConfigEditor::PlaylistDelete => config.keys.playlist_delete.modifier(),
            IdConfigEditor::PlaylistDeleteAll => config.keys.playlist_delete_all.modifier(),
            IdConfigEditor::PlaylistShuffle => config.keys.playlist_shuffle.modifier(),
            IdConfigEditor::PlaylistSearch => config.keys.playlist_search.modifier(),
            IdConfigEditor::PlaylistAddFront => config.keys.playlist_add_front.modifier(),
            IdConfigEditor::PlaylistPlaySelected => config.keys.playlist_play_selected.modifier(),
            IdConfigEditor::PlaylistModeCycle => config.keys.playlist_mode_cycle.modifier(),
            IdConfigEditor::PlaylistSwapDown => config.keys.playlist_swap_down.modifier(),
            IdConfigEditor::PlaylistSwapUp => config.keys.playlist_swap_up.modifier(),
            IdConfigEditor::DatabaseAddAll => config.keys.database_add_all.modifier(),
            IdConfigEditor::GlobalConfig => config.keys.global_config_open.modifier(),
            _ => 0,
        }
    }
}

impl Component<Msg, NoUserEvent> for KEModifierSelect {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        let cmd_result = match ev {
            // Global Hotkey
            Event::Keyboard(keyevent)
                if keyevent == self.config.keys.global_config_save.key_event() =>
            {
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
            Event::Keyboard(keyevent) if keyevent == self.config.keys.global_esc.key_event() => {
                match self.state() {
                    State::One(_) => return Some(Msg::ConfigEditor(ConfigEditorMsg::CloseCancel)),
                    _ => self.perform(Cmd::Cancel),
                }
            }
            Event::Keyboard(keyevent) if keyevent == self.config.keys.global_quit.key_event() => {
                match self.state() {
                    State::One(_) => return Some(Msg::ConfigEditor(ConfigEditorMsg::CloseCancel)),
                    _ => self.perform(Cmd::Cancel),
                }
            }
            Event::Keyboard(keyevent) if keyevent == self.config.keys.global_down.key_event() => {
                match self.state() {
                    State::One(_) => return Some(self.on_key_tab.clone()),
                    _ => self.perform(Cmd::Move(Direction::Down)),
                }
            }
            Event::Keyboard(keyevent) if keyevent == self.config.keys.global_up.key_event() => {
                match self.state() {
                    State::One(_) => return Some(self.on_key_backtab.clone()),
                    _ => self.perform(Cmd::Move(Direction::Up)),
                }
            }
            Event::Keyboard(KeyEvent {
                code: Key::Enter, ..
            }) => self.perform(Cmd::Submit),
            // input
            Event::Keyboard(KeyEvent {
                code: Key::Char(ch),
                ..
            }) => {
                self.perform(Cmd::Type(ch))
                // let result = self.perform(Cmd::Type(ch));
                // Some(self.update_key(result))
            }
            _ => CmdResult::None,
        };
        match cmd_result {
            CmdResult::Submit(State::One(StateValue::Usize(_index))) => {
                // Some(Msg::None)
                Some(Msg::ConfigEditor(ConfigEditorMsg::KeyChanged(
                    self.id.clone(),
                )))
            }
            _ => Some(Msg::None),
        }
    }
}

#[derive(MockComponent)]
pub struct ConfigPlaylistLqueue {
    component: KEModifierSelect,
}

impl ConfigPlaylistLqueue {
    pub fn new(config: &Settings) -> Self {
        Self {
            component: KEModifierSelect::new(
                " Playlist L Queue ",
                IdConfigEditor::PlaylistLqueue,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::PlaylistLqueueBlurDown),
                Msg::ConfigEditor(ConfigEditorMsg::PlaylistLqueueBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigPlaylistLqueue {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}
