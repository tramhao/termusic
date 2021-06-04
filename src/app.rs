use super::utils::keymap::MSG_KEY_ESC;

use std::time::Instant;

use tuirealm::components::label;
use tuirealm::{Msg, Payload, PropsBuilder, Update, Value, View};
// tui

const COMPONENT_INPUT: &str = "INPUT";
const COMPONENT_LABEL: &str = "LABEL";

pub struct App {
    pub quit: bool,           // Becomes true when the user presses <ESC>
    pub redraw: bool,         // Tells whether to refresh the UI; performance optimization
    pub last_redraw: Instant, // Last time the ui has been redrawed
    pub view: View,
}

impl App {
    pub fn new(view: View) -> Self {
        App {
            quit: false,
            redraw: true,
            last_redraw: Instant::now(),
            view,
        }
    }

    pub fn quit(&mut self) {
        self.quit = true;
    }

    pub fn redraw(&mut self) {
        self.redraw = true;
    }

    pub fn reset(&mut self) {
        self.redraw = false;
        self.last_redraw = Instant::now();
    }

    pub fn public_update(&mut self, msg: Option<(String, Msg)>) -> Option<(String, Msg)> {
        let msg2: Option<(String, Msg)> = self.update(msg);
        msg2
    }
}

impl Update for App {
    fn update(&mut self, msg: Option<(String, Msg)>) -> Option<(String, Msg)> {
        let ref_msg: Option<(&str, &Msg)> = msg.as_ref().map(|(s, msg)| (s.as_str(), msg));
        match ref_msg {
            None => None, // Exit after None
            Some(msg) => match msg {
                (COMPONENT_INPUT, Msg::OnChange(Payload::One(Value::Str(input)))) => {
                    // Update span
                    let props = label::LabelPropsBuilder::from(
                        self.view.get_props(COMPONENT_LABEL).unwrap(),
                    )
                    .with_text(format!("You typed: '{}'", input))
                    .build();
                    // Report submit
                    let msg = self.view.update(COMPONENT_LABEL, props);
                    self.update(msg)
                }
                (_, &MSG_KEY_ESC) => {
                    // Quit on esc
                    self.quit();
                    None
                }
                _ => None,
            },
        }
    }
}
