use super::utils::keymap::MSG_KEY_ESC;

use super::utils::myinput::InputHandler;
use super::utils::terminal::Terminal;
use std::time::Instant;

use std::thread::sleep;
use std::time::Duration;

use tuirealm::components::{input, label};
use tuirealm::props::borders::{BorderType, Borders};
use tuirealm::{InputType, PropsBuilder, Update, View};
// tui
use tui::layout::{Constraint, Direction, Layout};
use tui::style::Color;

const COMPONENT_INPUT: &str = "INPUT";
const COMPONENT_LABEL: &str = "LABEL";

use tuirealm::{Msg, Payload, Value};
// tui

pub struct App {
    pub quit: bool,           // Becomes true when the user presses <ESC>
    pub redraw: bool,         // Tells whether to refresh the UI; performance optimization
    pub last_redraw: Instant, // Last time the ui has been redrawed
    pub view: View,
    pub context: Option<Terminal>,
}

impl App {
    pub fn new() -> Self {
        let mut terminal: Terminal = Terminal::new();
        // Enter alternate screen
        terminal.enter_alternate_screen();
        // Clear screen
        terminal.clear_screen();

        let mut myview: View = View::init();
        // Let's mount the component we need
        myview.mount(
            COMPONENT_INPUT,
            Box::new(input::Input::new(
                input::InputPropsBuilder::default()
                    .with_borders(Borders::ALL, BorderType::Rounded, Color::LightYellow)
                    .with_foreground(Color::LightYellow)
                    .with_input(InputType::Text)
                    .with_label(String::from("Type in something nice"))
                    .build(),
            )),
        );
        myview.mount(
            COMPONENT_LABEL,
            Box::new(label::Label::new(
                label::LabelPropsBuilder::default()
                    .with_foreground(Color::Cyan)
                    .with_text(String::from("Your input will appear in after a submit"))
                    .build(),
            )),
        );
        // We need to initialize the focus
        myview.active(COMPONENT_INPUT);

        App {
            quit: false,
            redraw: true,
            last_redraw: Instant::now(),
            view: myview,
            context: Some(terminal),
        }
    }

    pub fn run(&mut self) {
        let input: InputHandler = InputHandler::new();
        while !self.quit {
            // Listen for input events
            if let Ok(Some(ev)) = input.read_event() {
                // Pass event to view
                let msg = self.view.on(ev);
                self.redraw();
                // Call the elm friend update
                self.update(msg);
            }
            // If redraw, draw interface
            if self.redraw || self.last_redraw.elapsed() > Duration::from_millis(50) {
                // Call the elm friend vie1 function
                self.view();
                self.reset();
            }
            sleep(Duration::from_millis(10));
        }

        drop(self.context.take());
    }

    pub fn quit(&mut self) {
        self.quit = true;
    }

    pub fn redraw(&mut self) {
        self.redraw = true;
    }

    fn view(&mut self) {
        let mut ctx: Terminal = self.context.take().unwrap();
        let _ = ctx.terminal.draw(|f| {
            // Prepare chunks
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints([Constraint::Length(3), Constraint::Length(3)].as_ref())
                .split(f.size());
            self.view.render(COMPONENT_INPUT, f, chunks[0]);
            self.view.render(COMPONENT_LABEL, f, chunks[1]);
        });
        self.context = Some(ctx);
    }
    pub fn reset(&mut self) {
        self.redraw = false;
        self.last_redraw = Instant::now();
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
