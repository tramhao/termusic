//! ## Gallery
//!
//! `Gallery` is a demo which is used to test all the available components

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
extern crate tui;
extern crate tuirealm;

mod utils;

use utils::context::Context;
use utils::keymap::*;

use std::thread::sleep;
use std::time::{Duration, Instant};

// realm
use tuirealm::components::{
    checkbox, input, label, paragraph, progress_bar, radio, scrolltable, span, table, textarea,
};
use tuirealm::props::borders::{BorderType, Borders};
use tuirealm::props::{TableBuilder, TextSpan, TextSpanBuilder};
use tuirealm::{InputType, Msg, PropPayload, PropValue, PropsBuilder, View};
// tui
use tui::layout::{Constraint, Direction, Layout};
use tui::style::Color;

// -- components

const COMPONENT_CHECKBOX: &str = "CHECKBOX";
const COMPONENT_INPUT: &str = "INPUT";
const COMPONENT_LABEL: &str = "LABEL";
const COMPONENT_PARAGRAPH: &str = "PARAGRAPH";
const COMPONENT_PROGBAR: &str = "PROGBAR";
const COMPONENT_RADIO: &str = "RADIO";
const COMPONENT_SPAN: &str = "SPAN";
const COMPONENT_SCROLLTABLE: &str = "SCROLLTABLE";
const COMPONENT_TABLE: &str = "TABLE";
const COMPONENT_TEXTAREA: &str = "TEXTAREA";

// -- application model

struct Model {
    quit: bool,
    redraw: bool,
    last_redraw: Instant,
}

impl Model {
    fn new() -> (Model, View) {
        let view: View = init_view();
        (
            Model {
                quit: false,
                redraw: true,
                last_redraw: Instant::now(),
            },
            view,
        )
    }

    fn quit(&mut self) {
        self.quit = true;
    }

    fn redraw(&mut self) {
        self.redraw = true;
    }

    fn reset(&mut self) {
        self.redraw = false;
        self.last_redraw = Instant::now();
    }
}

fn main() {
    // Create context
    let mut ctx: Context = Context::new();
    // Enter alternate screen
    ctx.enter_alternate_screen();
    // Clear screen
    ctx.clear_screen();
    // Initialize view
    let (mut model, mut viewptr): (Model, View) = Model::new();
    // Poll input events
    while !model.quit {
        // read events
        if let Ok(Some(ev)) = ctx.input_hnd.read_event() {
            let msg = viewptr.on(ev);
            model.redraw();
            update(&mut model, &mut viewptr, msg);
        }
        // If redraw, draw interface
        if model.redraw || model.last_redraw.elapsed() > Duration::from_millis(50) {
            view(&mut ctx, &viewptr);
            model.reset();
        }
        sleep(Duration::from_millis(10));
    }
    drop(ctx);
}

// -- View

fn init_view() -> View {
    let mut view: View = View::init();
    // Init components
    // Checkbox
    view.mount(
        COMPONENT_CHECKBOX,
        Box::new(checkbox::Checkbox::new(
            checkbox::CheckboxPropsBuilder::default()
                .with_color(Color::Cyan)
                .with_borders(Borders::ALL, BorderType::Rounded, Color::Magenta)
                .with_value(vec![1])
                .with_options(
                    Some(String::from("Select ice-cream flavours")),
                    vec![
                        String::from("Vanilla"),
                        String::from("Chocolate"),
                        String::from("Coconut"),
                        String::from("Strawberry"),
                        String::from("Lemon"),
                        String::from("Unicorn 🦄"),
                    ],
                )
                .build(),
        )),
    );
    // Input
    view.mount(
        COMPONENT_INPUT,
        Box::new(input::Input::new(
            input::InputPropsBuilder::default()
                .with_borders(Borders::ALL, BorderType::Rounded, Color::LightYellow)
                .with_foreground(Color::LightYellow)
                .with_input(InputType::Password)
                .with_label(String::from("Type in your password"))
                .build(),
        )),
    );
    // Label
    view.mount(
        COMPONENT_LABEL,
        Box::new(label::Label::new(
            label::LabelPropsBuilder::default()
                .bold()
                .italic()
                .rapid_blink()
                .reversed() // CAUSE COLORS TO BE INVERTED !!!
                .underlined()
                .with_foreground(Color::Red)
                .with_background(Color::Black)
                .with_text(String::from(
                    "Press <ESC> to QUIT!!! Change focus with <TAB>",
                ))
                .build(),
        )),
    );
    // Paragraph
    view.mount(
        COMPONENT_PARAGRAPH,
        Box::new(paragraph::Paragraph::new(
            paragraph::ParagraphPropsBuilder::default()
            .italic()
            .with_background(Color::White)
            .with_foreground(Color::Black)
            .with_borders(Borders::ALL, BorderType::Rounded, Color::Gray)
            .with_texts(Some(String::from("A poem for you")), vec![
                TextSpanBuilder::new("Lorem ipsum dolor sit amet").underlined().with_foreground(Color::Green).build(),
                TextSpan::from(", consectetur adipiscing elit. Praesent mauris est, vehicula et imperdiet sed, tincidunt sed est. Sed sed dui odio. Etiam nunc neque, sodales ut ex nec, tincidunt malesuada eros. Sed quis eros non felis sodales accumsan in ac risus"),
                TextSpan::from("Duis augue diam, tempor vitae posuere et, tempus mattis ligula.")
            ])
            .build()
        ))
    );
    // Progres bar
    view.mount(
        COMPONENT_PROGBAR,
        Box::new(progress_bar::ProgressBar::new(
            progress_bar::ProgressBarPropsBuilder::default()
                .with_background(Color::Black)
                .with_progbar_color(Color::Yellow)
                .with_borders(Borders::ALL, BorderType::Thick, Color::Yellow)
                .with_progress(0.64)
                .with_texts(
                    Some(String::from("Downloading termscp 0.4.2")),
                    String::from("64.2% - ETA 00:48"),
                )
                .build(),
        )),
    );
    // Radio
    view.mount(
        COMPONENT_RADIO,
        Box::new(radio::Radio::new(
            radio::RadioPropsBuilder::default()
                .with_color(Color::Magenta)
                .with_borders(
                    Borders::BOTTOM | Borders::TOP,
                    BorderType::Double,
                    Color::Magenta,
                )
                .with_inverted_color(Color::Black)
                .with_value(1)
                .with_options(
                    Some(String::from("Will you use tui-realm in your next project?")),
                    vec![
                        String::from("Yes!"),
                        String::from("No"),
                        String::from("Maybe"),
                    ],
                )
                .build(),
        )),
    );
    // Span
    view.mount(
        COMPONENT_SPAN,
        Box::new(span::Span::new(
            span::SpanPropsBuilder::default()
                .bold()
                .with_foreground(Color::Green)
                .with_background(Color::Black)
                .with_spans(vec![
                    TextSpan::from("THIS IS A SPAN: "),
                    TextSpanBuilder::new("Hello ")
                        .italic()
                        .slow_blink()
                        .with_foreground(Color::Black)
                        .with_background(Color::White)
                        .build(),
                    TextSpanBuilder::new("World!")
                        .bold()
                        .underlined()
                        .rapid_blink()
                        .with_foreground(Color::Red)
                        .build(),
                ])
                .build(),
        )),
    );
    // Scrolltable
    view.mount(
        COMPONENT_SCROLLTABLE,
        Box::new(scrolltable::Scrolltable::new(
            scrolltable::ScrollTablePropsBuilder::default()
                .with_foreground(Color::LightBlue)
                .with_borders(Borders::ALL, BorderType::Thick, Color::Blue)
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
    // Table
    view.mount(
        COMPONENT_TABLE,
        Box::new(table::Table::new(
            table::TablePropsBuilder::default()
                .with_foreground(Color::Green)
                .with_borders(Borders::ALL, BorderType::Thick, Color::LightGreen)
                .with_table(
                    Some(String::from("My data")),
                    TableBuilder::default()
                        .add_col(TextSpan::from("2021-04-17T18:32:00"))
                        .add_col(TextSpan::from(" "))
                        .add_col(TextSpan::from("The cat has just left the house"))
                        .add_row()
                        .add_col(TextSpan::from("2021-04-17T18:36:00"))
                        .add_col(TextSpan::from(" "))
                        .add_col(TextSpan::from("The cat approached a duck"))
                        .add_row()
                        .add_col(TextSpan::from("2021-04-17T18:36:03"))
                        .add_col(TextSpan::from(" "))
                        .add_col(TextSpan::from("The duck has flown away"))
                        .add_row()
                        .add_col(TextSpan::from("2021-04-17T18:37:00"))
                        .add_col(TextSpan::from(" "))
                        .add_col(TextSpan::from("The cat has met his fiance mimì"))
                        .add_row()
                        .add_col(TextSpan::from("2021-04-17T18:37:10"))
                        .add_col(TextSpan::from(" "))
                        .add_col(TextSpan::from("Mimì thinks the cat is harassing her"))
                        .add_row()
                        .add_col(TextSpan::from("2021-04-17T18:38:52"))
                        .add_col(TextSpan::from(" "))
                        .add_col(TextSpan::from("The cat is very sad and has come back home"))
                        .build(),
                )
                .build(),
        )),
    );
    // Textarea
    view.mount(
        COMPONENT_TEXTAREA,
        Box::new(textarea::Textarea::new(
            textarea::TextareaPropsBuilder::default()
                .with_foreground(Color::White)
                .italic()
                .with_borders(Borders::ALL, BorderType::Rounded, Color::LightRed)
                .with_texts(Some(String::from("termscp")),
                    vec![
                        TextSpanBuilder::new("About TermSCP").bold().underlined().with_foreground(Color::Yellow).build(),
                        TextSpan::from("TermSCP is basically a porting of WinSCP to terminal. So basically is a terminal utility with an TUI to connect to a remote server to retrieve and upload files and to interact with the local file system. It works both on Linux, MacOS, BSD and Windows and supports SFTP, SCP, FTP and FTPS."),
                        TextSpanBuilder::new("Why TermSCP 🤔").bold().underlined().with_foreground(Color::Cyan).build(),
                        TextSpan::from("It happens quite often to me, when using SCP at work to forget the path of a file on a remote machine, which forces me to connect through SSH, gather the file path and finally download it through SCP. I could use WinSCP, but I use Linux and I pratically use the terminal for everything, so I wanted something like WinSCP on my terminal. Yeah, I know there is midnight commander too, but actually I don't like it very much tbh (and hasn't a decent support for scp)."),
                    ]
                )
                .build(),
        )),
    );
    // Focus
    view.active(COMPONENT_CHECKBOX);
    view
}

fn view(ctx: &mut Context, view: &View) {
    let _ = ctx.terminal.draw(|f| {
        // Prepare chunks
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .margin(1)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
            .split(f.size());
        // Make columns
        let lcol = Layout::default()
            .constraints(
                [
                    Constraint::Length(3), // Checkbox
                    Constraint::Length(3), // Input
                    Constraint::Length(1), // Label
                    Constraint::Length(3), // Progress bar
                    Constraint::Length(3), // Radio
                    Constraint::Length(6), // Paragraph
                ]
                .as_ref(),
            )
            .direction(Direction::Vertical)
            .split(chunks[0]);
        let rcol = Layout::default()
            .constraints(
                [
                    Constraint::Length(6), // Scrolltable
                    Constraint::Length(1), // Span
                    Constraint::Length(8), // Table
                    Constraint::Length(8), // Textarea
                ]
                .as_ref(),
            )
            .direction(Direction::Vertical)
            .split(chunks[1]);
        // Render
        // left
        view.render(COMPONENT_CHECKBOX, f, lcol[0]);
        view.render(COMPONENT_INPUT, f, lcol[1]);
        view.render(COMPONENT_LABEL, f, lcol[2]);
        view.render(COMPONENT_PROGBAR, f, lcol[3]);
        view.render(COMPONENT_RADIO, f, lcol[4]);
        view.render(COMPONENT_PARAGRAPH, f, lcol[5]);
        // right
        view.render(COMPONENT_SCROLLTABLE, f, rcol[0]);
        view.render(COMPONENT_SPAN, f, rcol[1]);
        view.render(COMPONENT_TABLE, f, rcol[2]);
        view.render(COMPONENT_TEXTAREA, f, rcol[3]);
    });
}

fn update(model: &mut Model, view: &mut View, msg: Option<(String, Msg)>) -> Option<(String, Msg)> {
    let ref_msg: Option<(&str, &Msg)> = msg.as_ref().map(|(s, msg)| (s.as_str(), msg));
    match ref_msg {
        None => None, // Exit after None
        Some(msg) => match msg {
            (COMPONENT_CHECKBOX, &MSG_KEY_TAB) => {
                view.active(COMPONENT_INPUT);
                // Update progress
                let msg = update_progress(view);
                update(model, view, msg)
            }
            (COMPONENT_INPUT, &MSG_KEY_TAB) => {
                view.active(COMPONENT_RADIO);
                // Update progress
                let msg = update_progress(view);
                update(model, view, msg)
            }
            (COMPONENT_RADIO, &MSG_KEY_TAB) => {
                view.active(COMPONENT_SCROLLTABLE);
                // Update progress
                let msg = update_progress(view);
                update(model, view, msg)
            }
            (COMPONENT_SCROLLTABLE, &MSG_KEY_TAB) => {
                view.active(COMPONENT_TEXTAREA);
                // Update progress
                let msg = update_progress(view);
                update(model, view, msg)
            }
            (COMPONENT_TEXTAREA, &MSG_KEY_TAB) => {
                view.active(COMPONENT_CHECKBOX);
                // Update progress
                let msg = update_progress(view);
                update(model, view, msg)
            }
            (comp, Msg::OnSubmit(payload)) => {
                let props =
                    label::LabelPropsBuilder::from(view.get_props(COMPONENT_LABEL).unwrap())
                        .with_text(format!("GOT SUBMIT EVENT FROM '{}': {:?}", comp, payload))
                        .build();
                // Report submit
                view.update(COMPONENT_LABEL, props);
                // Update progress
                let msg = update_progress(view);
                update(model, view, msg)
            }
            (_, &MSG_KEY_ESC) => {
                // Quit
                model.quit();
                None
            }
            _ => None,
        },
    }
}

// -- misc

fn update_progress(view: &mut View) -> Option<(String, Msg)> {
    let props = view.get_props(COMPONENT_PROGBAR).unwrap();
    let new_prog: f64 = match props.value {
        PropPayload::One(PropValue::F64(val)) => match val + 0.05 > 1.0 {
            true => 0.0,
            false => val + 0.05,
        },
        _ => 0.0,
    };
    view.update(
        COMPONENT_PROGBAR,
        progress_bar::ProgressBarPropsBuilder::from(props)
            .with_progress(new_prog)
            .with_texts(
                Some(String::from("Downloading termscp 0.4.2")),
                format!("{:.2}% - ETA 00:30", new_prog * 100.0),
            )
            .build(),
    )
}
