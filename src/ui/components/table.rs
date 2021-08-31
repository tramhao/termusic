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
use tuirealm::event::KeyCode;
use tuirealm::props::{
    Alignment, BlockTitle, BordersProps, PropPayload, PropValue, Props, PropsBuilder,
    Table as TextTable,
};
use tuirealm::tui::{
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, BorderType, Borders, Cell, Row, Table as TuiTable, TableState},
};
use tuirealm::{event::Event, Component, Frame, Msg, Payload, Value};

// -- Props

const COLOR_HIGHLIGHTED: &str = "highlighted-color";
const PROP_COLUMN_SPACING: &str = "col-spacing";
const PROP_HEADER: &str = "header";
const PROP_HIGHLIGHTED_TXT: &str = "highlighted-txt";
const PROP_MAX_STEP: &str = "max-step";
const PROP_ROW_HEIGHT: &str = "row-height";
const PROP_SCROLLABLE: &str = "scrollable";
const PROP_WIDTHS: &str = "widhts";
const PROP_TABLE: &str = "table";

pub struct TablePropsBuilder {
    props: Option<Props>,
}

impl Default for TablePropsBuilder {
    fn default() -> Self {
        TablePropsBuilder {
            props: Some(Props::default()),
        }
    }
}

impl PropsBuilder for TablePropsBuilder {
    fn build(&mut self) -> Props {
        self.props.take().unwrap()
    }

    fn hidden(&mut self) -> &mut Self {
        if let Some(props) = self.props.as_mut() {
            props.visible = false;
        }
        self
    }

    fn visible(&mut self) -> &mut Self {
        if let Some(props) = self.props.as_mut() {
            props.visible = true;
        }
        self
    }
}

impl From<Props> for TablePropsBuilder {
    fn from(props: Props) -> Self {
        TablePropsBuilder { props: Some(props) }
    }
}

#[allow(unused)]
impl TablePropsBuilder {
    /// ### with_foreground
    ///
    /// Set foreground color for area
    pub fn with_foreground(&mut self, color: Color) -> &mut Self {
        if let Some(props) = self.props.as_mut() {
            props.foreground = color;
        }
        self
    }

    /// ### with_background
    ///
    /// Set background color for area
    pub fn with_background(&mut self, color: Color) -> &mut Self {
        if let Some(props) = self.props.as_mut() {
            props.background = color;
        }
        self
    }

    /// ### with_highlighted_color
    ///
    /// Set color for highlighted entry
    pub fn with_highlighted_color(&mut self, color: Color) -> &mut Self {
        if let Some(props) = self.props.as_mut() {
            props.palette.insert(COLOR_HIGHLIGHTED, color);
        }
        self
    }

    /// ### with_borders
    ///
    /// Set component borders style
    pub fn with_borders(
        &mut self,
        borders: Borders,
        variant: BorderType,
        color: Color,
    ) -> &mut Self {
        if let Some(props) = self.props.as_mut() {
            props.borders = BordersProps {
                borders,
                variant,
                color,
            }
        }
        self
    }

    /// ### bold
    ///
    /// Set bold property for component
    pub fn bold(&mut self) -> &mut Self {
        if let Some(props) = self.props.as_mut() {
            props.modifiers |= Modifier::BOLD;
        }
        self
    }

    /// ### italic
    ///
    /// Set italic property for component
    pub fn italic(&mut self) -> &mut Self {
        if let Some(props) = self.props.as_mut() {
            props.modifiers |= Modifier::ITALIC;
        }
        self
    }

    /// ### underlined
    ///
    /// Set underlined property for component
    pub fn underlined(&mut self) -> &mut Self {
        if let Some(props) = self.props.as_mut() {
            props.modifiers |= Modifier::UNDERLINED;
        }
        self
    }

    /// ### slow_blink
    ///
    /// Set slow_blink property for component
    pub fn slow_blink(&mut self) -> &mut Self {
        if let Some(props) = self.props.as_mut() {
            props.modifiers |= Modifier::SLOW_BLINK;
        }
        self
    }

    /// ### rapid_blink
    ///
    /// Set rapid_blink property for component
    pub fn rapid_blink(&mut self) -> &mut Self {
        if let Some(props) = self.props.as_mut() {
            props.modifiers |= Modifier::RAPID_BLINK;
        }
        self
    }

    /// ### reversed
    ///
    /// Set reversed property for component
    pub fn reversed(&mut self) -> &mut Self {
        if let Some(props) = self.props.as_mut() {
            props.modifiers |= Modifier::REVERSED;
        }
        self
    }

    /// ### strikethrough
    ///
    /// Set strikethrough property for component
    pub fn strikethrough(&mut self) -> &mut Self {
        if let Some(props) = self.props.as_mut() {
            props.modifiers |= Modifier::CROSSED_OUT;
        }
        self
    }

    /// ### with_table
    ///
    /// Set table content
    /// You can define a title if you want. The title will be displayed on the upper border of the box
    pub fn with_table(&mut self, table: TextTable) -> &mut Self {
        if let Some(props) = self.props.as_mut() {
            props
                .own
                .insert(PROP_TABLE, PropPayload::One(PropValue::Table(table)));
        }
        self
    }

    /// ### with_title
    ///
    /// Set title
    pub fn with_title<S: AsRef<str>>(&mut self, title: S, alignment: Alignment) -> &mut Self {
        if let Some(props) = self.props.as_mut() {
            props.title = Some(BlockTitle::new(title, alignment));
        }
        self
    }

    /// ### with_header
    ///
    /// Set header for table
    pub fn with_header(&mut self, header: &[&str]) -> &mut Self {
        if let Some(props) = self.props.as_mut() {
            props.own.insert(
                PROP_HEADER,
                PropPayload::Vec(
                    header
                        .iter()
                        .map(|x| PropValue::Str(x.to_string()))
                        .collect(),
                ),
            );
        }
        self
    }

    /// ### with_widths
    ///
    /// Set widths in percentage for table columns.
    /// Panics if amount of columns doesn't match the amount of columns in table
    pub fn with_widths(&mut self, widths: &[u16]) -> &mut Self {
        if let Some(props) = self.props.as_mut() {
            props.own.insert(
                PROP_WIDTHS,
                PropPayload::Vec(widths.iter().map(|x| PropValue::U16(*x)).collect()),
            );
        }
        self
    }

    /// ### with_row_height
    ///
    /// Set row height
    pub fn with_row_height(&mut self, height: u16) -> &mut Self {
        if let Some(props) = self.props.as_mut() {
            props
                .own
                .insert(PROP_ROW_HEIGHT, PropPayload::One(PropValue::U16(height)));
        }
        self
    }

    /// ### with_col_spacing
    ///
    /// Set column spacing
    pub fn with_col_spacing(&mut self, spacing: u16) -> &mut Self {
        if let Some(props) = self.props.as_mut() {
            props.own.insert(
                PROP_COLUMN_SPACING,
                PropPayload::One(PropValue::U16(spacing)),
            );
        }
        self
    }

    /// ### with_highlighted_str
    ///
    /// Display a symbol to highlighted line in scroll table
    pub fn with_highlighted_str(&mut self, s: Option<&str>) -> &mut Self {
        if let Some(props) = self.props.as_mut() {
            match s {
                None => {
                    props.own.remove(PROP_HIGHLIGHTED_TXT);
                }
                Some(s) => {
                    props.own.insert(
                        PROP_HIGHLIGHTED_TXT,
                        PropPayload::One(PropValue::Str(s.to_string())),
                    );
                }
            }
        }
        self
    }

    /// ### with_max_scroll_step
    ///
    /// Defines the max step for PAGEUP/PAGEDOWN keys
    pub fn with_max_scroll_step(&mut self, step: usize) -> &mut Self {
        if let Some(props) = self.props.as_mut() {
            props
                .own
                .insert(PROP_MAX_STEP, PropPayload::One(PropValue::Usize(step)));
        }
        self
    }

    /// ### scrollable
    ///
    /// Sets whether the list is scrollable
    pub fn scrollable(&mut self, scrollable: bool) -> &mut Self {
        if let Some(props) = self.props.as_mut() {
            props.own.insert(
                PROP_SCROLLABLE,
                PropPayload::One(PropValue::Bool(scrollable)),
            );
        }
        self
    }
}

// -- States

struct OwnStates {
    focus: bool,
    list_index: usize, // Index of selected item in textarea
    list_len: usize,   // Lines in text area
}

impl OwnStates {
    /// ### set_list_len
    ///
    /// Set list length
    pub fn set_list_len(&mut self, len: usize) {
        self.list_len = len;
    }

    /// ### incr_list_index
    ///
    /// Incremenet list index
    pub fn incr_list_index(&mut self) {
        // Check if index is at last element
        if self.list_index + 1 < self.list_len {
            self.list_index += 1;
        }
    }

    /// ### decr_list_index
    ///
    /// Decrement list index
    pub fn decr_list_index(&mut self) {
        // Check if index is bigger than 0
        if self.list_index > 0 {
            self.list_index -= 1;
        }
    }

    /// ### fix_list_index
    ///
    /// Keep index if possible, otherwise set to lenght - 1
    pub fn fix_list_index(&mut self) {
        if self.list_index >= self.list_len && self.list_len > 0 {
            self.list_index = self.list_len - 1;
        } else if self.list_len == 0 {
            self.list_index = 0;
        }
    }

    /// ### list_index_at_first
    ///
    /// Set list index to the first item in the list
    pub fn list_index_at_first(&mut self) {
        self.list_index = 0;
    }

    /// ### list_index_at_last
    ///
    /// Set list index at the last item of the list
    pub fn list_index_at_last(&mut self) {
        if self.list_len > 0 {
            self.list_index = self.list_len - 1;
        } else {
            self.list_index = 0;
        }
    }

    /// ### calc_max_step_ahead
    ///
    /// Calculate the max step ahead to scroll list
    pub fn calc_max_step_ahead(&self, max: usize) -> usize {
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

    /// ### calc_max_step_ahead
    ///
    /// Calculate the max step ahead to scroll list
    pub fn calc_max_step_behind(&self, max: usize) -> usize {
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
pub struct Table {
    props: Props,
    states: OwnStates,
}

impl Table {
    /// ### new
    ///
    /// Instantiates a new `Table` component.
    pub fn new(props: Props) -> Self {
        let len: usize = match props.own.get(PROP_TABLE).as_ref() {
            Some(PropPayload::One(PropValue::Table(t))) => t.len(),
            _ => 0,
        };
        Table {
            props,
            states: OwnStates {
                focus: false,
                list_index: 0,
                list_len: len,
            },
        }
    }

    /// ### scrollable
    ///
    /// returns the value of the scrollable flag; by default is false
    fn scrollable(&self) -> bool {
        match self.props.own.get(PROP_SCROLLABLE) {
            Some(PropPayload::One(PropValue::Bool(scrollable))) => *scrollable,
            _ => false,
        }
    }

    /// ### layout
    ///
    /// Returns layout based on properties.
    /// If layout is not set in properties, they'll be divided by rows number
    fn layout(&self) -> Vec<Constraint> {
        match self.props.own.get(PROP_WIDTHS) {
            Some(PropPayload::Vec(widths)) => widths
                .iter()
                .map(|x| *x.unwrap_u16())
                .map(Constraint::Percentage)
                .collect(),
            _ => {
                // Get amount of columns (maximum len of row elements)
                let columns: usize = match self.props.own.get(PROP_TABLE).as_ref() {
                    Some(PropPayload::One(PropValue::Table(rows))) => {
                        rows.iter().map(|col| col.len()).max().unwrap_or(0)
                    }
                    _ => 0,
                };
                // Calc width in equal way
                let width: u16 = (100 / columns) as u16;
                (0..columns)
                    .map(|_| Constraint::Percentage(width))
                    .collect()
            }
        }
    }
}

impl Component for Table {
    /// ### render
    ///
    /// Based on the current properties and states, renders a widget using the provided render engine in the provided Area
    /// If focused, cursor is also set (if supported by widget)
    fn render(&self, render: &mut Frame, area: Rect) {
        if self.props.visible {
            let active: bool = match self.scrollable() {
                true => self.states.focus,
                false => true,
            };
            let div: Block = tui_realm_stdlib::utils::get_block(
                &self.props.borders,
                self.props.title.as_ref(),
                active,
            );
            // Get row height
            let row_height: u16 = match self.props.own.get(PROP_ROW_HEIGHT) {
                Some(PropPayload::One(PropValue::U16(h))) => *h,
                _ => 1,
            };
            // Make rows
            let rows: Vec<Row> = match self.props.own.get(PROP_TABLE).as_ref() {
                Some(PropPayload::One(PropValue::Table(table))) => table
                    .iter()
                    .map(|row| {
                        let columns: Vec<Cell> = row
                            .iter()
                            .map(|col| {
                                let (fg, bg, modifiers) =
                                    tui_realm_stdlib::utils::use_or_default_styles(
                                        &self.props,
                                        col,
                                    );
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
            let highlighted_color: Color = match self.props.palette.get(COLOR_HIGHLIGHTED) {
                None => match self.states.focus {
                    true => self.props.background,
                    false => self.props.foreground,
                },
                Some(color) => *color,
            };
            let (fg, bg): (Color, Color) = match active {
                true => (self.props.background, highlighted_color),
                false => (highlighted_color, self.props.background),
            };
            // Make list
            let widths: Vec<Constraint> = self.layout();
            let mut table = TuiTable::new(rows)
                .block(div)
                .highlight_style(
                    Style::default()
                        .fg(fg)
                        .bg(bg)
                        .add_modifier(self.props.modifiers),
                )
                .widths(&widths);
            // Highlighted symbol
            if let Some(PropPayload::One(PropValue::Str(highlight))) =
                self.props.own.get(PROP_HIGHLIGHTED_TXT)
            {
                table = table.highlight_symbol(highlight);
            }
            // Col spacing
            if let Some(PropPayload::One(PropValue::U16(spacing))) =
                self.props.own.get(PROP_COLUMN_SPACING)
            {
                table = table.column_spacing(*spacing);
            }
            // Header
            if let Some(PropPayload::Vec(headers)) = self.props.own.get(PROP_HEADER) {
                let headers: Vec<&str> = headers
                    .iter()
                    .map(|x| match x {
                        PropValue::Str(s) => s,
                        _ => "",
                    })
                    .collect();
                table = table.header(
                    Row::new(headers)
                        .style(
                            Style::default()
                                .fg(self.props.foreground)
                                .bg(self.props.background)
                                .add_modifier(self.props.modifiers),
                        )
                        .height(row_height),
                );
            }
            if self.scrollable() {
                let mut state: TableState = TableState::default();
                state.select(Some(self.states.list_index));
                render.render_stateful_widget(table, area, &mut state);
            } else {
                render.render_widget(table, area);
            }
        }
    }

    /// ### update
    ///
    /// Update component properties
    /// Properties should first be retrieved through `get_props` which creates a builder from
    /// existing properties and then edited before calling update.
    /// Returns a Msg to the view
    fn update(&mut self, props: Props) -> Msg {
        self.props = props;
        // re-Set list length
        self.states
            .set_list_len(match self.props.own.get(PROP_TABLE).as_ref() {
                Some(PropPayload::One(PropValue::Table(t))) => t.len(),
                _ => 0,
            });
        // Fix list index
        self.states.fix_list_index();
        // disable if scrollable
        if self.scrollable() {
            self.blur();
        }
        // Return None
        Msg::None
    }

    /// ### get_props
    ///
    /// Returns a copy of the component properties.
    fn get_props(&self) -> Props {
        self.props.clone()
    }

    /// ### on
    ///
    /// Handle input event and update internal states.
    /// Returns a Msg to the view.
    fn on(&mut self, ev: Event) -> Msg {
        // Return key
        if let Event::Key(key) = ev {
            if self.scrollable() {
                match key.code {
                    KeyCode::Down => {
                        // Go down
                        self.states.incr_list_index();
                        Msg::OnKey(key)
                    }
                    KeyCode::Up => {
                        // Go up
                        self.states.decr_list_index();
                        Msg::OnKey(key)
                    }
                    KeyCode::PageDown => {
                        // Scroll by step
                        let step: usize = self.states.calc_max_step_ahead(
                            match self.props.own.get(PROP_MAX_STEP) {
                                Some(PropPayload::One(PropValue::Usize(step))) => *step,
                                _ => 8,
                            },
                        );
                        (0..step).for_each(|_| self.states.incr_list_index());
                        Msg::OnKey(key)
                    }
                    KeyCode::PageUp => {
                        // Scroll by step
                        let step: usize = self.states.calc_max_step_behind(
                            match self.props.own.get(PROP_MAX_STEP) {
                                Some(PropPayload::One(PropValue::Usize(step))) => *step,
                                _ => 8,
                            },
                        );
                        (0..step).for_each(|_| self.states.decr_list_index());
                        Msg::OnKey(key)
                    }
                    KeyCode::End => {
                        self.states.list_index_at_last();
                        Msg::OnKey(key)
                    }
                    KeyCode::Home => {
                        self.states.list_index_at_first();
                        Msg::OnKey(key)
                    }
                    _ => Msg::OnKey(key),
                }
            } else {
                Msg::OnKey(key)
            }
        } else {
            Msg::None
        }
    }

    /// ### get_state
    ///
    /// Get current state from component
    /// For this component returns None if not scrollable, otherwise returns the index of the list
    fn get_state(&self) -> Payload {
        match self.scrollable() {
            true => Payload::One(Value::Usize(self.states.list_index)),
            false => Payload::None,
        }
    }

    // -- events

    /// ### blur
    ///
    /// Blur component
    fn blur(&mut self) {
        self.states.focus = false;
    }

    /// ### active
    ///
    /// Active component
    fn active(&mut self) {
        if self.scrollable() {
            self.states.focus = true;
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use tuirealm::props::{TableBuilder, TextSpan};

    use crossterm::event::KeyEvent;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_component_table_scrolling() {
        // Make component
        let mut component: Table = Table::new(
            TablePropsBuilder::default()
                .with_foreground(Color::Red)
                .with_background(Color::Blue)
                .with_highlighted_color(Color::Yellow)
                .hidden()
                .visible()
                .bold()
                .italic()
                .rapid_blink()
                .reversed()
                .slow_blink()
                .strikethrough()
                .underlined()
                .with_borders(Borders::ALL, BorderType::Double, Color::Red)
                .with_highlighted_str(Some("🚀"))
                .with_max_scroll_step(4)
                .scrollable(true)
                .with_title("events", Alignment::Center)
                .with_table(
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
                )
                .with_header(&["Event", "Message", "Behaviour", "???"])
                .with_col_spacing(2)
                .with_row_height(3)
                .with_widths(&[25, 25, 25, 25])
                .with_borders(Borders::ALL, BorderType::Double, Color::Red)
                .build(),
        );
        assert_eq!(component.props.foreground, Color::Red);
        assert_eq!(component.props.background, Color::Blue);
        assert_eq!(component.props.visible, true);
        assert!(component.props.modifiers.intersects(Modifier::BOLD));
        assert!(component.props.modifiers.intersects(Modifier::ITALIC));
        assert!(component.props.modifiers.intersects(Modifier::UNDERLINED));
        assert!(component.props.modifiers.intersects(Modifier::SLOW_BLINK));
        assert!(component.props.modifiers.intersects(Modifier::RAPID_BLINK));
        assert!(component.props.modifiers.intersects(Modifier::REVERSED));
        assert!(component.props.modifiers.intersects(Modifier::CROSSED_OUT));
        assert_eq!(component.props.borders.borders, Borders::ALL);
        assert_eq!(component.props.borders.variant, BorderType::Double);
        assert_eq!(component.props.borders.color, Color::Red);
        assert_eq!(
            component.props.palette.get(COLOR_HIGHLIGHTED).unwrap(),
            &Color::Yellow
        );
        assert_eq!(
            component.props.own.get(PROP_HIGHLIGHTED_TXT).unwrap(),
            &PropPayload::One(PropValue::Str(String::from("🚀")))
        );
        assert_eq!(
            component.props.own.get(PROP_MAX_STEP).unwrap(),
            &PropPayload::One(PropValue::Usize(4))
        );
        assert_eq!(component.props.title.as_ref().unwrap().text(), "events");
        assert_eq!(
            component.props.title.as_ref().unwrap().alignment(),
            Alignment::Center
        );
        assert_eq!(component.states.list_len, 7);
        assert_eq!(component.states.list_index, 0);
        component.active();
        assert_eq!(component.states.focus, true);
        component.blur();
        assert_eq!(component.states.focus, false);
        // Own funcs
        assert_eq!(component.layout().len(), 4);
        // Increment list index
        component.states.list_index += 1;
        assert_eq!(component.states.list_index, 1);
        // Check messages
        // Handle inputs
        assert_eq!(
            component.on(Event::Key(KeyEvent::from(KeyCode::Down))),
            Msg::OnKey(KeyEvent::from(KeyCode::Down))
        );
        // Index should be incremented
        assert_eq!(component.states.list_index, 2);
        // Index should be decremented
        assert_eq!(
            component.on(Event::Key(KeyEvent::from(KeyCode::Up))),
            Msg::OnKey(KeyEvent::from(KeyCode::Up))
        );
        // Index should be incremented
        assert_eq!(component.states.list_index, 1);
        // Index should be 2
        assert_eq!(
            component.on(Event::Key(KeyEvent::from(KeyCode::PageDown))),
            Msg::OnKey(KeyEvent::from(KeyCode::PageDown))
        );
        // Index should be incremented
        assert_eq!(component.states.list_index, 5);
        assert_eq!(
            component.on(Event::Key(KeyEvent::from(KeyCode::PageDown))),
            Msg::OnKey(KeyEvent::from(KeyCode::PageDown))
        );
        // Index should be incremented
        assert_eq!(component.states.list_index, 6);
        // Index should be 0
        assert_eq!(
            component.on(Event::Key(KeyEvent::from(KeyCode::PageUp))),
            Msg::OnKey(KeyEvent::from(KeyCode::PageUp))
        );
        assert_eq!(component.states.list_index, 2);
        assert_eq!(
            component.on(Event::Key(KeyEvent::from(KeyCode::PageUp))),
            Msg::OnKey(KeyEvent::from(KeyCode::PageUp))
        );
        assert_eq!(component.states.list_index, 0);
        // End
        assert_eq!(
            component.on(Event::Key(KeyEvent::from(KeyCode::End))),
            Msg::OnKey(KeyEvent::from(KeyCode::End))
        );
        assert_eq!(component.states.list_index, 6);
        // Home
        assert_eq!(
            component.on(Event::Key(KeyEvent::from(KeyCode::Home))),
            Msg::OnKey(KeyEvent::from(KeyCode::Home))
        );
        assert_eq!(component.states.list_index, 0);
        // Update
        let props = TablePropsBuilder::from(component.get_props())
            .with_foreground(Color::Red)
            .hidden()
            .with_table(
                TableBuilder::default()
                    .add_col(TextSpan::from("name"))
                    .add_col(TextSpan::from("age"))
                    .add_col(TextSpan::from("birthdate"))
                    .build(),
            )
            .build();
        assert_eq!(component.update(props), Msg::None);
        assert_eq!(component.props.foreground, Color::Red);
        assert_eq!(component.props.visible, false);
        assert_eq!(component.states.list_len, 1);
        assert_eq!(component.states.list_index, 0);
        // Get value
        assert_eq!(component.get_state(), Payload::One(Value::Usize(0)));
        // Event
        assert_eq!(
            component.on(Event::Key(KeyEvent::from(KeyCode::Delete))),
            Msg::OnKey(KeyEvent::from(KeyCode::Delete))
        );
        assert_eq!(component.on(Event::Resize(0, 0)), Msg::None);
    }

    #[test]
    fn test_components_table() {
        // Make component
        let mut component: Table = Table::new(
            TablePropsBuilder::default()
                .with_foreground(Color::Red)
                .with_background(Color::Blue)
                .with_highlighted_color(Color::Yellow)
                .hidden()
                .visible()
                .bold()
                .italic()
                .rapid_blink()
                .reversed()
                .slow_blink()
                .strikethrough()
                .underlined()
                .with_borders(Borders::ALL, BorderType::Double, Color::Red)
                .with_highlighted_str(Some("🚀"))
                .with_max_scroll_step(4)
                .with_table(
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
                )
                .with_header(&["Event", "Message", "Behaviour"])
                .with_col_spacing(2)
                .with_row_height(3)
                .with_widths(&[33, 33, 33])
                .with_borders(Borders::ALL, BorderType::Double, Color::Red)
                .build(),
        );
        assert_eq!(component.props.foreground, Color::Red);
        assert_eq!(component.props.background, Color::Blue);
        assert_eq!(component.props.visible, true);
        assert!(component.props.modifiers.intersects(Modifier::BOLD));
        assert!(component.props.modifiers.intersects(Modifier::ITALIC));
        assert!(component.props.modifiers.intersects(Modifier::UNDERLINED));
        assert!(component.props.modifiers.intersects(Modifier::SLOW_BLINK));
        assert!(component.props.modifiers.intersects(Modifier::RAPID_BLINK));
        assert!(component.props.modifiers.intersects(Modifier::REVERSED));
        assert!(component.props.modifiers.intersects(Modifier::CROSSED_OUT));
        assert_eq!(component.props.borders.borders, Borders::ALL);
        assert_eq!(component.props.borders.variant, BorderType::Double);
        assert_eq!(component.props.borders.color, Color::Red);
        component.active();
        component.blur();
        // Update
        let props = TablePropsBuilder::from(component.get_props())
            .with_foreground(Color::Red)
            .hidden()
            .build();
        assert_eq!(component.update(props), Msg::None);
        assert_eq!(component.props.foreground, Color::Red);
        assert_eq!(component.props.visible, false);
        // Get value (not scrollable)
        assert_eq!(component.get_state(), Payload::None);
        // Event
        assert_eq!(
            component.on(Event::Key(KeyEvent::from(KeyCode::Delete))),
            Msg::OnKey(KeyEvent::from(KeyCode::Delete))
        );
        assert_eq!(component.on(Event::Resize(0, 0)), Msg::None);
    }
}
