//! ## Scrolltable
//!
//! `Scrolltable` represents a read-only textual table component which is scrollable through arrows

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
use tuirealm::props::{BordersProps, Props, PropsBuilder, Table as TextTable, TextParts};
use tuirealm::tui::{
    layout::{Corner, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, BorderType, Borders, List, ListItem, ListState},
};
use tuirealm::{Canvas, Component, Msg, Payload};

use tuirealm::event::{Event, KeyCode};
// -- Props

pub struct QueuePropsBuilder {
    props: Option<Props>,
}

impl Default for QueuePropsBuilder {
    fn default() -> Self {
        QueuePropsBuilder {
            props: Some(Props::default()),
        }
    }
}

impl PropsBuilder for QueuePropsBuilder {
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

impl From<Props> for QueuePropsBuilder {
    fn from(props: Props) -> Self {
        QueuePropsBuilder { props: Some(props) }
    }
}

impl QueuePropsBuilder {
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
    #[allow(dead_code)]
    pub fn bold(&mut self) -> &mut Self {
        if let Some(props) = self.props.as_mut() {
            props.modifiers |= Modifier::BOLD;
        }
        self
    }

    /// ### italic
    ///
    /// Set italic property for component
    #[allow(dead_code)]
    pub fn italic(&mut self) -> &mut Self {
        if let Some(props) = self.props.as_mut() {
            props.modifiers |= Modifier::ITALIC;
        }
        self
    }

    /// ### underlined
    ///
    /// Set underlined property for component
    #[allow(dead_code)]
    pub fn underlined(&mut self) -> &mut Self {
        if let Some(props) = self.props.as_mut() {
            props.modifiers |= Modifier::UNDERLINED;
        }
        self
    }

    /// ### slow_blink
    ///
    /// Set slow_blink property for component
    #[allow(dead_code)]
    pub fn slow_blink(&mut self) -> &mut Self {
        if let Some(props) = self.props.as_mut() {
            props.modifiers |= Modifier::SLOW_BLINK;
        }
        self
    }

    /// ### rapid_blink
    ///
    /// Set rapid_blink property for component
    #[allow(dead_code)]
    pub fn rapid_blink(&mut self) -> &mut Self {
        if let Some(props) = self.props.as_mut() {
            props.modifiers |= Modifier::RAPID_BLINK;
        }
        self
    }

    /// ### reversed
    ///
    /// Set reversed property for component
    #[allow(dead_code)]
    pub fn reversed(&mut self) -> &mut Self {
        if let Some(props) = self.props.as_mut() {
            props.modifiers |= Modifier::REVERSED;
        }
        self
    }

    /// ### strikethrough
    ///
    /// Set strikethrough property for component
    #[allow(dead_code)]
    pub fn strikethrough(&mut self) -> &mut Self {
        if let Some(props) = self.props.as_mut() {
            props.modifiers |= Modifier::CROSSED_OUT;
        }
        self
    }

    /// ### with_table
    ///
    /// Set table
    /// You can define a title if you want. The title will be displayed on the upper border of the box
    pub fn with_table(&mut self, title: Option<String>, table: TextTable) -> &mut Self {
        if let Some(props) = self.props.as_mut() {
            props.texts = TextParts::table(title, table);
        }
        self
    }
}

// -- States

struct OwnStates {
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
}

// -- Component

/// ## Scrolltable
///
/// represents a read-only text component without any container.
pub struct Queue {
    props: Props,
    states: OwnStates,
}

impl Queue {
    /// ### new
    ///
    /// Instantiates a new `Table` component.
    pub fn new(props: Props) -> Self {
        let len: usize = match props.texts.table.as_ref() {
            Some(t) => t.len(),
            None => 0,
        };
        Queue {
            props,
            states: OwnStates {
                list_index: 0,
                list_len: len,
            },
        }
    }
}

impl Component for Queue {
    /// ### render
    ///
    /// Based on the current properties and states, renders a widget using the provided render engine in the provided Area
    /// If focused, cursor is also set (if supported by widget)
    #[cfg(not(tarpaulin_include))]
    fn render(&self, render: &mut Canvas, area: Rect) {
        if self.props.visible {
            let div: Block = tuirealm::components::utils::get_block(
                &self.props.borders,
                &self.props.texts.title,
                true,
            );
            // Make list entries
            let list_items: Vec<ListItem> = match self.props.texts.table.as_ref() {
                None => Vec::new(),
                Some(table) => table
                    .iter()
                    .map(|row| {
                        let columns: Vec<Span> = row
                            .iter()
                            .map(|col| {
                                let (fg, bg, modifiers) =
                                    tuirealm::components::utils::use_or_default_styles(
                                        &self.props,
                                        col,
                                    );
                                Span::styled(
                                    col.content.clone(),
                                    Style::default().add_modifier(modifiers).fg(fg).bg(bg),
                                )
                            })
                            .collect();
                        ListItem::new(Spans::from(columns))
                    })
                    .collect(), // Make List item from TextSpan
            };
            let mut state: ListState = ListState::default();
            state.select(Some(self.states.list_index));
            let (fg, bg) = (self.props.foreground, self.props.background);
            // Make list
            render.render_stateful_widget(
                List::new(list_items)
                    .block(div)
                    .start_corner(Corner::TopLeft)
                    .highlight_style(
                        Style::default()
                            .fg(bg) // inverted
                            .bg(fg),
                    ),
                area,
                &mut state,
            );
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
        self.states.set_list_len(match &self.props.texts.table {
            Some(table) => table.len(),
            None => 0,
        });
        // Fix list index
        self.states.fix_list_index();
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
            match key.code {
                KeyCode::Down => {
                    // Update states
                    self.states.incr_list_index();
                    Msg::OnKey(key)
                }
                KeyCode::Up => {
                    // Update states
                    self.states.decr_list_index();
                    Msg::OnKey(key)
                }
                KeyCode::PageDown => {
                    // Update states
                    for _ in 0..8 {
                        self.states.incr_list_index();
                    }
                    Msg::OnKey(key)
                }
                KeyCode::PageUp => {
                    // Update states
                    for _ in 0..8 {
                        self.states.decr_list_index();
                    }
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
            Msg::None
        }
    }

    /// ### get_state
    ///
    /// Get current state from component
    /// For this component returns always None
    fn get_state(&self) -> Payload {
        Payload::None
    }

    // -- events

    /// ### blur
    ///
    /// Blur component
    fn blur(&mut self) {}

    /// ### active
    ///
    /// Active component
    fn active(&mut self) {}
}

#[cfg(test)]
mod tests {

    use super::*;
    use tuirealm::props::{TableBuilder, TextSpan};

    use pretty_assertions::assert_eq;
    use tuirealm::event::{Event, KeyCode, KeyEvent};

    #[test]
    fn test_components_scrolltable() {
        // Make component
        let mut component: Queue = Queue::new(
            QueuePropsBuilder::default()
                .with_foreground(Color::Red)
                .with_background(Color::Blue)
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
                .with_table(
                    Some(String::from("My data")),
                    TableBuilder::default()
                        .add_col(TextSpan::from("name"))
                        .add_col(TextSpan::from("age"))
                        .add_row()
                        .add_col(TextSpan::from("omar"))
                        .add_col(TextSpan::from("1"))
                        .add_row()
                        .add_col(TextSpan::from("mark"))
                        .add_col(TextSpan::from("2"))
                        .add_row()
                        .add_col(TextSpan::from("tom"))
                        .add_col(TextSpan::from("3"))
                        .add_row()
                        .add_col(TextSpan::from("pippo"))
                        .add_col(TextSpan::from("5"))
                        .add_row()
                        .add_col(TextSpan::from("carl"))
                        .add_col(TextSpan::from("8"))
                        .add_row()
                        .add_col(TextSpan::from("charlie"))
                        .add_col(TextSpan::from("13"))
                        .add_row()
                        .add_col(TextSpan::from("thomas"))
                        .add_col(TextSpan::from("21"))
                        .add_row()
                        .add_col(TextSpan::from("cammello"))
                        .add_col(TextSpan::from("34"))
                        .build(),
                )
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
            component.props.texts.title.as_ref().unwrap().as_str(),
            "My data"
        );
        assert_eq!(component.props.texts.table.as_ref().unwrap().len(), 9);
        assert_eq!(component.states.list_len, 9);
        assert_eq!(component.states.list_index, 0);
        component.active();
        component.blur();
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
        assert_eq!(component.states.list_index, 8);
        // Index should be 0
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
        assert_eq!(component.states.list_index, 8);
        // Home
        assert_eq!(
            component.on(Event::Key(KeyEvent::from(KeyCode::Home))),
            Msg::OnKey(KeyEvent::from(KeyCode::Home))
        );
        assert_eq!(component.states.list_index, 0);
        // Update
        let props = QueuePropsBuilder::from(component.get_props())
            .with_foreground(Color::Red)
            .hidden()
            .with_table(
                Some(String::from("My data")),
                TableBuilder::default()
                    .add_col(TextSpan::from("name"))
                    .add_col(TextSpan::from("age"))
                    .build(),
            )
            .build();
        assert_eq!(component.update(props), Msg::None);
        assert_eq!(component.props.foreground, Color::Red);
        assert_eq!(component.props.visible, false);
        assert_eq!(component.states.list_len, 1);
        assert_eq!(component.states.list_index, 0);
        // Get value
        assert_eq!(component.get_state(), Payload::None);
        // Event
        assert_eq!(
            component.on(Event::Key(KeyEvent::from(KeyCode::Delete))),
            Msg::OnKey(KeyEvent::from(KeyCode::Delete))
        );
        assert_eq!(component.on(Event::Resize(0, 0)), Msg::None);
    }
}
