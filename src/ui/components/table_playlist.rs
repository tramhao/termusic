//! ## Table
//!
//! `Table` represents a read-only textual table component which can be scrollable through arrows or inactive

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
use tui_realm_stdlib::{
    props::TABLE_COLUMN_SPACING,
    utils::{get_block, use_or_default_styles},
};

use tuirealm::command::{Cmd, CmdResult, Direction, Position};
use tuirealm::props::{
    Alignment, AttrValue, Attribute, Borders, Color, PropPayload, PropValue, Props, Style,
    Table as PropTable, TextModifiers,
};
use tuirealm::tui::{
    layout::{Constraint, Rect},
    text::Span,
    widgets::{Cell, Row, Table as TuiTable, TableState},
};
use tuirealm::{Frame, MockComponent, State, StateValue};

pub const TABLE_INITIAL_INDEX: &str = "initial-index";
// -- States
#[derive(Default)]
pub struct TableStates {
    pub list_index: usize, // Index of selected item in textarea
    pub list_len: usize,   // Lines in text area
}

impl TableStates {
    /// ### `set_list_len`
    ///
    /// Set list length
    pub fn set_list_len(&mut self, len: usize) {
        self.list_len = len;
    }

    /// ### `incr_list_index`
    ///
    /// Incremenet list index
    pub fn incr_list_index(&mut self, rewind: bool) {
        // Check if index is at last element
        if self.list_index + 1 < self.list_len {
            self.list_index += 1;
        } else if rewind {
            self.list_index = 0;
        }
    }

    /// ### `decr_list_index`
    ///
    /// Decrement list index
    pub fn decr_list_index(&mut self, rewind: bool) {
        // Check if index is bigger than 0
        if self.list_index > 0 {
            self.list_index -= 1;
        } else if rewind && self.list_len > 0 {
            self.list_index = self.list_len - 1;
        }
    }

    /// ### `fix_list_index`
    ///
    /// Keep index if possible, otherwise set to lenght - 1
    pub fn fix_list_index(&mut self) {
        if self.list_index >= self.list_len && self.list_len > 0 {
            self.list_index = self.list_len - 1;
        } else if self.list_len == 0 {
            self.list_index = 0;
        }
    }

    /// ### `list_index_at_first`
    ///
    /// Set list index to the first item in the list
    pub fn list_index_at_first(&mut self) {
        self.list_index = 0;
    }

    /// ### `list_index_at_last`
    ///
    /// Set list index at the last item of the list
    pub fn list_index_at_last(&mut self) {
        if self.list_len > 0 {
            self.list_index = self.list_len - 1;
        } else {
            self.list_index = 0;
        }
    }

    pub fn list_index_at_index(&mut self, index: usize) {
        if index > self.list_len - 1 {
            self.list_index = self.list_len - 1;
        } else {
            self.list_index = index;
        }
    }

    /// ### `calc_max_step_ahead`
    ///
    /// Calculate the max step ahead to scroll list
    pub const fn calc_max_step_ahead(&self, max: usize) -> usize {
        let remaining: usize = match self.list_len {
            0 => 0,
            len => len - 1 - self.list_index,
        };
        if remaining > max {
            max
        } else {
            remaining
        }
    }

    /// ### `calc_max_step_ahead`
    ///
    /// Calculate the max step ahead to scroll list
    pub const fn calc_max_step_behind(&self, max: usize) -> usize {
        if self.list_index > max {
            max
        } else {
            self.list_index
        }
    }
}

// -- Component

/// ## Table
///
/// represents a read-only text component without any container.
#[derive(Default)]
pub struct Table {
    props: Props,
    pub states: TableStates,
    hg_str: Option<String>, // CRAP CRAP CRAP
    headers: Vec<String>,   // CRAP CRAP CRAP
}

#[allow(unused)]
impl Table {
    pub fn foreground(mut self, fg: Color) -> Self {
        self.attr(Attribute::Foreground, AttrValue::Color(fg));
        self
    }

    pub fn background(mut self, bg: Color) -> Self {
        self.attr(Attribute::Background, AttrValue::Color(bg));
        self
    }

    pub fn inactive(mut self, s: Style) -> Self {
        self.attr(Attribute::FocusStyle, AttrValue::Style(s));
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

    pub fn title<S: AsRef<str>>(mut self, t: S, a: Alignment) -> Self {
        self.attr(
            Attribute::Title,
            AttrValue::Title((t.as_ref().to_string(), a)),
        );
        self
    }

    pub fn step(mut self, step: usize) -> Self {
        self.attr(Attribute::ScrollStep, AttrValue::Length(step));
        self
    }

    pub fn scroll(mut self, scrollable: bool) -> Self {
        self.attr(Attribute::Scroll, AttrValue::Flag(scrollable));
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

    pub fn column_spacing(mut self, w: u16) -> Self {
        self.attr(Attribute::Custom(TABLE_COLUMN_SPACING), AttrValue::Size(w));
        self
    }

    pub fn row_height(mut self, h: u16) -> Self {
        self.attr(Attribute::Height, AttrValue::Size(h));
        self
    }

    pub fn widths(mut self, w: &[u16]) -> Self {
        self.attr(
            Attribute::Width,
            AttrValue::Payload(PropPayload::Vec(
                w.iter().map(|x| PropValue::U16(*x)).collect(),
            )),
        );
        self
    }

    pub fn headers<S: AsRef<str>>(mut self, headers: &[S]) -> Self {
        self.attr(
            Attribute::Text,
            AttrValue::Payload(PropPayload::Vec(
                headers
                    .iter()
                    .map(|x| PropValue::Str(x.as_ref().to_string()))
                    .collect(),
            )),
        );
        self
    }

    #[allow(clippy::cast_possible_wrap)]
    pub fn initial_node<S: AsRef<str>>(mut self, index: usize) -> Self {
        self.attr(
            Attribute::Custom(TABLE_INITIAL_INDEX),
            AttrValue::Number(index as isize),
        );
        self
    }

    pub fn table(mut self, t: PropTable) -> Self {
        self.attr(Attribute::Content, AttrValue::Table(t));
        self
    }

    pub fn rewind(mut self, r: bool) -> Self {
        self.attr(Attribute::Rewind, AttrValue::Flag(r));
        self
    }

    /// ### scrollable
    ///
    /// returns the value of the scrollable flag; by default is false
    fn is_scrollable(&self) -> bool {
        self.props
            .get_or(Attribute::Scroll, AttrValue::Flag(false))
            .unwrap_flag()
    }

    fn rewindable(&self) -> bool {
        self.props
            .get_or(Attribute::Rewind, AttrValue::Flag(false))
            .unwrap_flag()
    }

    /// ### layout
    ///
    /// Returns layout based on properties.
    /// If layout is not set in properties, they'll be divided by rows number
    fn layout(&self) -> Vec<Constraint> {
        if let Some(PropPayload::Vec(widths)) = self
            .props
            .get(Attribute::Width)
            .map(AttrValue::unwrap_payload)
        {
            widths
                .iter()
                .cloned()
                .map(PropValue::unwrap_u16)
                .map(Constraint::Percentage)
                .collect()
        } else {
            // Get amount of columns (maximum len of row elements)
            let columns: usize = match self
                .props
                .get(Attribute::Content)
                .map(AttrValue::unwrap_table)
            {
                Some(rows) => rows.iter().map(Vec::len).max().unwrap_or(0),
                _ => 0,
            };
            // Calc width in equal way
            #[allow(clippy::cast_possible_truncation)]
            let width: u16 = (100 / columns) as u16;
            (0..columns)
                .map(|_| Constraint::Percentage(width))
                .collect()
        }
    }
}

impl MockComponent for Table {
    #[allow(clippy::too_many_lines)]
    fn view(&mut self, render: &mut Frame, area: Rect) {
        if self.props.get_or(Attribute::Display, AttrValue::Flag(true)) == AttrValue::Flag(true) {
            let foreground = self
                .props
                .get_or(Attribute::Foreground, AttrValue::Color(Color::Reset))
                .unwrap_color();
            let background = self
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
            let title = self
                .props
                .get_or(
                    Attribute::Title,
                    AttrValue::Title((String::default(), Alignment::Center)),
                )
                .unwrap_title();
            let borders = self
                .props
                .get_or(Attribute::Borders, AttrValue::Borders(Borders::default()))
                .unwrap_borders();
            let focus = self
                .props
                .get_or(Attribute::Focus, AttrValue::Flag(false))
                .unwrap_flag();
            let inactive_style = self
                .props
                .get(Attribute::FocusStyle)
                .map(AttrValue::unwrap_style);
            let row_height = self
                .props
                .get_or(Attribute::Height, AttrValue::Size(1))
                .unwrap_size();
            // Make rows
            let rows: Vec<Row> = match self
                .props
                .get(Attribute::Content)
                .map(AttrValue::unwrap_table)
            {
                Some(table) => table
                    .iter()
                    .map(|row| {
                        let columns: Vec<Cell> = row
                            .iter()
                            .map(|col| {
                                let (fg, bg, modifiers) = use_or_default_styles(&self.props, col);
                                Cell::from(Span::styled(
                                    col.content.clone(),
                                    Style::default().add_modifier(modifiers).fg(fg).bg(bg),
                                ))
                            })
                            .collect();
                        Row::new(columns).height(row_height)
                    })
                    .collect(), // Make List item from TextSpan
                _ => Vec::new(),
            };
            let highlighted_color = self
                .props
                .get(Attribute::HighlightedColor)
                .map(AttrValue::unwrap_color);
            let widths: Vec<Constraint> = self.layout();
            let mut table = TuiTable::new(rows)
                .block(get_block(borders, Some(title), focus, inactive_style))
                .widths(&widths);
            if let Some(highlighted_color) = highlighted_color {
                table = table.highlight_style(Style::default().fg(highlighted_color).add_modifier(
                    if focus {
                        modifiers | TextModifiers::REVERSED
                    } else {
                        modifiers
                    },
                ));
            }
            // Highlighted symbol
            self.hg_str = self
                .props
                .get(Attribute::HighlightedStr)
                .map(AttrValue::unwrap_string);
            if let Some(hg_str) = &self.hg_str {
                table = table.highlight_symbol(hg_str);
            }
            // Col spacing
            if let Some(spacing) = self
                .props
                .get(Attribute::Custom(TABLE_COLUMN_SPACING))
                .map(AttrValue::unwrap_size)
            {
                table = table.column_spacing(spacing);
            }
            // Header
            self.headers = self
                .props
                .get(Attribute::Text)
                .map(|x| {
                    x.unwrap_payload()
                        .unwrap_vec()
                        .into_iter()
                        .map(PropValue::unwrap_str)
                        .collect()
                })
                .unwrap_or_default();
            if !self.headers.is_empty() {
                let headers: Vec<&str> = self.headers.iter().map(String::as_str).collect();
                table = table.header(
                    Row::new(headers)
                        .style(
                            Style::default()
                                .fg(foreground)
                                .bg(background)
                                .add_modifier(modifiers),
                        )
                        .height(row_height),
                );
            }
            if self.is_scrollable() {
                let mut state: TableState = TableState::default();
                state.select(Some(self.states.list_index));
                render.render_stateful_widget(table, area, &mut state);
            } else {
                render.render_widget(table, area);
            }
        }
    }

    fn query(&self, attr: Attribute) -> Option<AttrValue> {
        self.props.get(attr)
    }

    #[allow(clippy::cast_sign_loss)]
    fn attr(&mut self, attr: Attribute, value: AttrValue) {
        // Initial index
        if matches!(attr, Attribute::Custom(TABLE_INITIAL_INDEX)) {
            // Select index if exists
            if let AttrValue::Number(index) = value {
                self.states.list_index_at_index(index as usize);
            }
        } else {
            self.props.set(attr, value);
        }

        // Update list len and fix index
        self.states.set_list_len(
            match self
                .props
                .get(Attribute::Content)
                .map(AttrValue::unwrap_table)
            {
                Some(spans) => spans.len(),
                _ => 0,
            },
        );
        self.states.fix_list_index();
    }

    fn state(&self) -> State {
        if self.is_scrollable() {
            State::One(StateValue::Usize(self.states.list_index))
        } else {
            State::None
        }
    }

    fn perform(&mut self, cmd: Cmd) -> CmdResult {
        match cmd {
            Cmd::Move(Direction::Down) => {
                let prev = self.states.list_index;
                self.states.incr_list_index(self.rewindable());
                if prev == self.states.list_index {
                    CmdResult::None
                } else {
                    CmdResult::Changed(self.state())
                }
            }
            Cmd::Move(Direction::Up) => {
                let prev = self.states.list_index;
                self.states.decr_list_index(self.rewindable());
                if prev == self.states.list_index {
                    CmdResult::None
                } else {
                    CmdResult::Changed(self.state())
                }
            }
            Cmd::Scroll(Direction::Down) => {
                let prev = self.states.list_index;
                let step = self
                    .props
                    .get_or(Attribute::ScrollStep, AttrValue::Length(8))
                    .unwrap_length();
                let step: usize = self.states.calc_max_step_ahead(step);
                (0..step).for_each(|_| self.states.incr_list_index(false));
                if prev == self.states.list_index {
                    CmdResult::None
                } else {
                    CmdResult::Changed(self.state())
                }
            }
            Cmd::Scroll(Direction::Up) => {
                let prev = self.states.list_index;
                let step = self
                    .props
                    .get_or(Attribute::ScrollStep, AttrValue::Length(8))
                    .unwrap_length();
                let step: usize = self.states.calc_max_step_behind(step);
                (0..step).for_each(|_| self.states.decr_list_index(false));
                if prev == self.states.list_index {
                    CmdResult::None
                } else {
                    CmdResult::Changed(self.state())
                }
            }
            Cmd::GoTo(Position::Begin) => {
                let prev = self.states.list_index;
                self.states.list_index_at_first();
                if prev == self.states.list_index {
                    CmdResult::None
                } else {
                    CmdResult::Changed(self.state())
                }
            }
            Cmd::GoTo(Position::End) => {
                let prev = self.states.list_index;
                self.states.list_index_at_last();
                if prev == self.states.list_index {
                    CmdResult::None
                } else {
                    CmdResult::Changed(self.state())
                }
            }
            Cmd::GoTo(Position::At(index)) => {
                let prev = self.states.list_index;
                self.states.list_index_at_index(index);
                if prev == self.states.list_index {
                    CmdResult::None
                } else {
                    CmdResult::Changed(self.state())
                }
            }
            _ => CmdResult::None,
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use pretty_assertions::assert_eq;
    use tuirealm::props::{TableBuilder, TextSpan};

    #[test]
    fn table_states() {
        let mut states = TableStates::default();
        assert_eq!(states.list_index, 0);
        assert_eq!(states.list_len, 0);
        states.set_list_len(5);
        assert_eq!(states.list_index, 0);
        assert_eq!(states.list_len, 5);
        // Incr
        states.incr_list_index(true);
        assert_eq!(states.list_index, 1);
        states.list_index = 4;
        states.incr_list_index(false);
        assert_eq!(states.list_index, 4);
        states.incr_list_index(true);
        assert_eq!(states.list_index, 0);
        // Decr
        states.decr_list_index(false);
        assert_eq!(states.list_index, 0);
        states.decr_list_index(true);
        assert_eq!(states.list_index, 4);
        states.decr_list_index(true);
        assert_eq!(states.list_index, 3);
        // Begin
        states.list_index_at_first();
        assert_eq!(states.list_index, 0);
        states.list_index_at_last();
        assert_eq!(states.list_index, 4);
        // Fix
        states.set_list_len(3);
        states.fix_list_index();
        assert_eq!(states.list_index, 2);
    }

    #[test]
    #[allow(clippy::too_many_lines)]
    fn test_component_table_scrolling() {
        // Make component
        let mut component = Table::default()
            .foreground(Color::Red)
            .background(Color::Blue)
            .highlighted_color(Color::Yellow)
            .highlighted_str("\u{1f680}")
            .modifiers(TextModifiers::BOLD)
            .scroll(true)
            .step(4)
            .borders(Borders::default())
            .title("events", Alignment::Center)
            .column_spacing(4)
            .widths(&[25, 25, 25, 25])
            .row_height(3)
            .headers(&["Event", "Message", "Behaviour", "???"])
            .table(
                TableBuilder::default()
                    .add_col(TextSpan::from("KeyCode::Down"))
                    .add_col(TextSpan::from("OnKey"))
                    .add_col(TextSpan::from("Move cursor down"))
                    .add_row()
                    .add_col(TextSpan::from("KeyCode::Up"))
                    .add_col(TextSpan::from("OnKey"))
                    .add_col(TextSpan::from("Move cursor up"))
                    .add_row()
                    .add_col(TextSpan::from("KeyCode::PageDown"))
                    .add_col(TextSpan::from("OnKey"))
                    .add_col(TextSpan::from("Move cursor down by 8"))
                    .add_row()
                    .add_col(TextSpan::from("KeyCode::PageUp"))
                    .add_col(TextSpan::from("OnKey"))
                    .add_col(TextSpan::from("ove cursor up by 8"))
                    .add_row()
                    .add_col(TextSpan::from("KeyCode::End"))
                    .add_col(TextSpan::from("OnKey"))
                    .add_col(TextSpan::from("Move cursor to last item"))
                    .add_row()
                    .add_col(TextSpan::from("KeyCode::Home"))
                    .add_col(TextSpan::from("OnKey"))
                    .add_col(TextSpan::from("Move cursor to first item"))
                    .add_row()
                    .add_col(TextSpan::from("KeyCode::Char(_)"))
                    .add_col(TextSpan::from("OnKey"))
                    .add_col(TextSpan::from("Return pressed key"))
                    .add_col(TextSpan::from("4th mysterious columns"))
                    .build(),
            );
        assert_eq!(component.states.list_len, 7);
        assert_eq!(component.states.list_index, 0);
        // Own funcs
        assert_eq!(component.layout().len(), 4);
        // Increment list index
        component.states.list_index += 1;
        assert_eq!(component.states.list_index, 1);
        // Check messages
        // Handle inputs
        assert_eq!(
            component.perform(Cmd::Move(Direction::Down)),
            CmdResult::Changed(State::One(StateValue::Usize(2)))
        );
        // Index should be incremented
        assert_eq!(component.states.list_index, 2);
        // Index should be decremented
        assert_eq!(
            component.perform(Cmd::Move(Direction::Up)),
            CmdResult::Changed(State::One(StateValue::Usize(1)))
        );
        // Index should be incremented
        assert_eq!(component.states.list_index, 1);
        // Index should be 2
        assert_eq!(
            component.perform(Cmd::Scroll(Direction::Down)),
            CmdResult::Changed(State::One(StateValue::Usize(5)))
        );
        // Index should be incremented
        assert_eq!(component.states.list_index, 5);
        assert_eq!(
            component.perform(Cmd::Scroll(Direction::Down)),
            CmdResult::Changed(State::One(StateValue::Usize(6)))
        );
        // Index should be incremented
        assert_eq!(component.states.list_index, 6);
        // Index should be 0
        assert_eq!(
            component.perform(Cmd::Scroll(Direction::Up)),
            CmdResult::Changed(State::One(StateValue::Usize(2)))
        );
        assert_eq!(component.states.list_index, 2);
        assert_eq!(
            component.perform(Cmd::Scroll(Direction::Up)),
            CmdResult::Changed(State::One(StateValue::Usize(0)))
        );
        assert_eq!(component.states.list_index, 0);
        // End
        assert_eq!(
            component.perform(Cmd::GoTo(Position::End)),
            CmdResult::Changed(State::One(StateValue::Usize(6)))
        );
        assert_eq!(component.states.list_index, 6);
        // Home
        assert_eq!(
            component.perform(Cmd::GoTo(Position::Begin)),
            CmdResult::Changed(State::One(StateValue::Usize(0)))
        );
        assert_eq!(component.states.list_index, 0);
        // Update
        component.attr(
            Attribute::Content,
            AttrValue::Table(
                TableBuilder::default()
                    .add_col(TextSpan::from("name"))
                    .add_col(TextSpan::from("age"))
                    .add_col(TextSpan::from("birthdate"))
                    .build(),
            ),
        );
        assert_eq!(component.states.list_len, 1);
        assert_eq!(component.states.list_index, 0);
        // Get value
        assert_eq!(component.state(), State::One(StateValue::Usize(0)));
    }

    #[test]
    fn test_components_table() {
        // Make component
        let component = Table::default()
            .foreground(Color::Red)
            .background(Color::Blue)
            .highlighted_color(Color::Yellow)
            .highlighted_str("\u{1f680}")
            .modifiers(TextModifiers::BOLD)
            .borders(Borders::default())
            .title("events", Alignment::Center)
            .column_spacing(4)
            .widths(&[33, 33, 33])
            .row_height(3)
            .headers(&["Event", "Message", "Behaviour"])
            .table(
                TableBuilder::default()
                    .add_col(TextSpan::from("KeyCode::Down"))
                    .add_col(TextSpan::from("OnKey"))
                    .add_col(TextSpan::from("Move cursor down"))
                    .add_row()
                    .add_col(TextSpan::from("KeyCode::Up"))
                    .add_col(TextSpan::from("OnKey"))
                    .add_col(TextSpan::from("Move cursor up"))
                    .add_row()
                    .add_col(TextSpan::from("KeyCode::PageDown"))
                    .add_col(TextSpan::from("OnKey"))
                    .add_col(TextSpan::from("Move cursor down by 8"))
                    .add_row()
                    .add_col(TextSpan::from("KeyCode::PageUp"))
                    .add_col(TextSpan::from("OnKey"))
                    .add_col(TextSpan::from("ove cursor up by 8"))
                    .add_row()
                    .add_col(TextSpan::from("KeyCode::End"))
                    .add_col(TextSpan::from("OnKey"))
                    .add_col(TextSpan::from("Move cursor to last item"))
                    .add_row()
                    .add_col(TextSpan::from("KeyCode::Home"))
                    .add_col(TextSpan::from("OnKey"))
                    .add_col(TextSpan::from("Move cursor to first item"))
                    .add_row()
                    .add_col(TextSpan::from("KeyCode::Char(_)"))
                    .add_col(TextSpan::from("OnKey"))
                    .add_col(TextSpan::from("Return pressed key"))
                    .build(),
            );
        // Get value (not scrollable)
        assert_eq!(component.state(), State::None);
    }
}
