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
use tuirealm::components::utils::get_block;
use tuirealm::event::{Event, KeyCode};
use tuirealm::props::{BordersProps, PropPayload, PropValue, Props, PropsBuilder, TextParts};
use tuirealm::tui::{
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, Paragraph},
};
use tuirealm::{Canvas, Component, Msg, Payload, Value};

// -- states

struct OwnStates {
    counter: usize,
    focus: bool,
}

impl Default for OwnStates {
    fn default() -> Self {
        OwnStates {
            counter: 0,
            focus: false,
        }
    }
}

impl OwnStates {
    pub fn incr(&mut self) {
        self.counter += 1;
    }
}

// -- Props

pub struct CounterPropsBuilder {
    props: Option<Props>,
}

impl Default for CounterPropsBuilder {
    fn default() -> Self {
        CounterPropsBuilder {
            props: Some(Props::default()),
        }
    }
}

impl PropsBuilder for CounterPropsBuilder {
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

impl From<Props> for CounterPropsBuilder {
    fn from(props: Props) -> Self {
        CounterPropsBuilder { props: Some(props) }
    }
}

impl CounterPropsBuilder {
    pub fn with_foreground(&mut self, color: Color) -> &mut Self {
        if let Some(props) = self.props.as_mut() {
            props.foreground = color;
        }
        self
    }

    #[allow(dead_code)]
    pub fn with_background(&mut self, color: Color) -> &mut Self {
        if let Some(props) = self.props.as_mut() {
            props.background = color;
        }
        self
    }

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

    pub fn with_label(&mut self, label: String) -> &mut Self {
        if let Some(props) = self.props.as_mut() {
            props.texts = TextParts::new(Some(label), None);
        }
        self
    }

    pub fn with_value(&mut self, counter: usize) -> &mut Self {
        if let Some(props) = self.props.as_mut() {
            props.value = PropPayload::One(PropValue::Usize(counter));
        }
        self
    }
}

// -- Component

pub struct Counter {
    props: Props,
    states: OwnStates,
}

impl Counter {
    pub fn new(props: Props) -> Self {
        let mut states: OwnStates = OwnStates::default();
        // Init counter
        if let PropPayload::One(PropValue::Usize(val)) = &props.value {
            states.counter = *val;
        }
        Counter { props, states }
    }
}

impl Component for Counter {
    fn render(&self, render: &mut Canvas, area: Rect) {
        // Make a Span - THIS IS VERY IMPORTANT!!!
        if self.props.visible {
            // Make text
            let prefix: String = match self.props.texts.title.as_ref() {
                None => String::new(),
                Some(t) => t.clone(),
            };
            let text: String = format!("{} ({})", prefix, self.states.counter);
            let block: Block = get_block(&self.props.borders, &None, self.states.focus);
            let (fg, bg) = match self.states.focus {
                true => (self.props.foreground, self.props.background),
                false => (Color::Reset, Color::Reset),
            };
            render.render_widget(
                Paragraph::new(text).block(block).style(
                    Style::default()
                        .fg(fg)
                        .bg(bg)
                        .add_modifier(self.props.modifiers),
                ),
                area,
            );
        }
    }

    fn update(&mut self, props: Props) -> Msg {
        let prev_value = self.states.counter;
        // Get value
        if let PropPayload::One(PropValue::Usize(val)) = &props.value {
            self.states.counter = *val;
        }
        self.props = props;
        // Msg none
        if prev_value != self.states.counter {
            Msg::OnChange(self.get_state())
        } else {
            Msg::None
        }
    }

    fn get_props(&self) -> Props {
        self.props.clone()
    }

    fn on(&mut self, ev: Event) -> Msg {
        // Match event
        if let Event::Key(key) = ev {
            match key.code {
                KeyCode::Enter => {
                    // Increment first
                    self.states.incr();
                    // Return OnChange
                    Msg::OnChange(self.get_state())
                }
                _ => {
                    // Return key event to activity
                    Msg::OnKey(key)
                }
            }
        } else {
            // Ignore event
            Msg::None
        }
    }

    fn get_state(&self) -> Payload {
        Payload::One(Value::Usize(self.states.counter))
    }

    fn blur(&mut self) {
        self.states.focus = false;
    }

    fn active(&mut self) {
        self.states.focus = true;
    }
}
