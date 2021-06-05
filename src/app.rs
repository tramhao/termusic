use super::utils::keymap::{MSG_KEY_ESC, MSG_KEY_TAB};

use super::utils::myinput::InputHandler;
use super::utils::terminal::Terminal;
use std::time::Instant;

use std::thread::sleep;
use std::time::Duration;

use tuirealm::components::{input, label, scrolltable};
use tuirealm::props::borders::{BorderType, Borders};
use tuirealm::props::{TableBuilder, TextSpan};
use tuirealm::{InputType, PropsBuilder, Update, View};
// tui
use tui::layout::{Constraint, Direction, Layout};
use tui::style::Color;

const COMPONENT_INPUT: &str = "INPUT";
const COMPONENT_LABEL: &str = "LABEL";
const COMPONENT_SCROLLTABLE: &str = "SCROLLTABLE";

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
        // Scrolltable
        myview.mount(
            COMPONENT_SCROLLTABLE,
            Box::new(scrolltable::Scrolltable::new(
                scrolltable::ScrollTablePropsBuilder::default()
                    .with_foreground(Color::LightBlue)
                    .with_borders(Borders::ALL, BorderType::Rounded, Color::Blue)
                    .with_table(
                        Some(String::from("My scrollable data")),
                        TableBuilder::default()
                            .add_col(TextSpan::from("0"))
                            .add_col(TextSpan::from(" "))
                            .add_col(TextSpan::from("andreas"))
                            .add_row()
                            .add_col(TextSpan::from("1"))
                            .add_col(TextSpan::from(" "))
                            .add_col(TextSpan::from("bohdan"))
                            .add_row()
                            .add_col(TextSpan::from("2"))
                            .add_col(TextSpan::from(" "))
                            .add_col(TextSpan::from("charlie"))
                            .add_row()
                            .add_col(TextSpan::from("3"))
                            .add_col(TextSpan::from(" "))
                            .add_col(TextSpan::from("denis"))
                            .add_row()
                            .add_col(TextSpan::from("4"))
                            .add_col(TextSpan::from(" "))
                            .add_col(TextSpan::from("ector"))
                            .add_row()
                            .add_col(TextSpan::from("5"))
                            .add_col(TextSpan::from(" "))
                            .add_col(TextSpan::from("frank"))
                            .add_row()
                            .add_col(TextSpan::from("6"))
                            .add_col(TextSpan::from(" "))
                            .add_col(TextSpan::from("giulio"))
                            .add_row()
                            .add_col(TextSpan::from("7"))
                            .add_col(TextSpan::from(" "))
                            .add_col(TextSpan::from("hermes"))
                            .add_row()
                            .add_col(TextSpan::from("8"))
                            .add_col(TextSpan::from(" "))
                            .add_col(TextSpan::from("italo"))
                            .add_row()
                            .add_col(TextSpan::from("9"))
                            .add_col(TextSpan::from(" "))
                            .add_col(TextSpan::from("lamar"))
                            .add_row()
                            .add_col(TextSpan::from("10"))
                            .add_col(TextSpan::from(" "))
                            .add_col(TextSpan::from("mark"))
                            .add_row()
                            .add_col(TextSpan::from("11"))
                            .add_col(TextSpan::from(" "))
                            .add_col(TextSpan::from("napalm"))
                            .build(),
                    )
                    .build(),
            )),
        );

        // We need to initialize the focus
        myview.active(COMPONENT_SCROLLTABLE);

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
                .direction(Direction::Horizontal)
                .margin(1)
                .constraints([Constraint::Ratio(3, 10), Constraint::Ratio(7, 10)].as_ref())
                .split(f.size());
            let chunks_right = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints([Constraint::Min(2), Constraint::Length(9)].as_ref())
                .split(chunks[1]);

            self.view.render(COMPONENT_LABEL, f, chunks[0]);
            self.view.render(COMPONENT_SCROLLTABLE, f, chunks_right[0]);
            self.view.render(COMPONENT_INPUT, f, chunks_right[1]);
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
                (COMPONENT_INPUT, &MSG_KEY_TAB) => {
                    self.view.active(COMPONENT_SCROLLTABLE);
                    let props = label::LabelPropsBuilder::from(
                        self.view.get_props(COMPONENT_LABEL).unwrap(),
                    )
                    .with_text(format!("You typed: '{}'", "TAB"))
                    .build();
                    let msg = self.view.update(COMPONENT_LABEL, props);
                    self.update(msg)
                }
                (COMPONENT_SCROLLTABLE, &MSG_KEY_TAB) => {
                    self.view.active(COMPONENT_INPUT);
                    let props = label::LabelPropsBuilder::from(
                        self.view.get_props(COMPONENT_LABEL).unwrap(),
                    )
                    .with_text(format!("You typed: '{}'", "TAB"))
                    .build();
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
