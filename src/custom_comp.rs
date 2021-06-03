//! ## Demo
//!
//! `Demo` shows how to use tui-realm in a real case

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
mod components;
mod utils;

use components::counter::{Counter, CounterPropsBuilder};

use utils::context::Context;
use utils::keymap::*;

use std::thread::sleep;
use std::time::{Duration, Instant};

use tuirealm::props::borders::{BorderType, Borders};
use tuirealm::{Msg, PropsBuilder, View};
// tui
use tui::layout::{Constraint, Direction, Layout};
use tui::style::Color;

const COMPONENT_COUNTER1: &str = "COUNTER1";
const COMPONENT_COUNTER2: &str = "COUNTER2";

struct Model {
    quit: bool,           // Becomes true when the user presses <ESC>
    redraw: bool,         // Tells whether to refresh the UI; performance optimization
    last_redraw: Instant, // Last time the ui has been redrawed
}

impl Model {
    fn new() -> Self {
        Model {
            quit: false,
            redraw: true,
            last_redraw: Instant::now(),
        }
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
    // let's create a context: the context contains the backend of crossterm and the input handler
    let mut ctx: Context = Context::new();
    // Enter alternate screen
    ctx.enter_alternate_screen();
    // Clear screen
    ctx.clear_screen();
    // let's create a `View`, which will contain the components
    let mut myview: View = View::init();
    // Mount the component you need; we'll use a Label and an Input
    myview.mount(
        COMPONENT_COUNTER1,
        Box::new(Counter::new(
            CounterPropsBuilder::default()
                .with_borders(Borders::ALL, BorderType::Rounded, Color::LightYellow)
                .with_foreground(Color::LightYellow)
                .with_label(String::from("Counter A"))
                .with_value(4)
                .build(),
        )),
    );
    myview.mount(
        COMPONENT_COUNTER2,
        Box::new(Counter::new(
            CounterPropsBuilder::default()
                .with_borders(Borders::ALL, BorderType::Rounded, Color::LightBlue)
                .with_foreground(Color::LightBlue)
                .with_label(String::from("Counter B"))
                .with_value(0)
                .build(),
        )),
    );
    // We need to give focus to counter 1 then
    myview.active(COMPONENT_COUNTER1);
    // Now we use the Model struct to keep track of some states
    let mut model: Model = Model::new();
    // let's loop until quit is true
    while !model.quit {
        // Listen for input events
        if let Ok(Some(ev)) = ctx.input_hnd.read_event() {
            // Pass event to view
            let msg = myview.on(ev);
            model.redraw();
            // Call the elm friend update
            update(&mut model, &mut myview, msg);
        }
        // If redraw, draw interface
        if model.redraw || model.last_redraw.elapsed() > Duration::from_millis(50) {
            // Call the elm friend vie1 function
            view(&mut ctx, &myview);
            model.reset();
        }
        sleep(Duration::from_millis(10));
    }
    // Let's drop the context finally
    drop(ctx);
}

fn view(ctx: &mut Context, view: &View) {
    let _ = ctx.terminal.draw(|f| {
        // Prepare chunks
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints(
                [
                    Constraint::Length(3),
                    Constraint::Length(3),
                    Constraint::Length(1),
                ]
                .as_ref(),
            )
            .split(f.size());
        view.render(COMPONENT_COUNTER1, f, chunks[0]);
        view.render(COMPONENT_COUNTER2, f, chunks[1]);
    });
}

fn update(model: &mut Model, view: &mut View, msg: Option<(String, Msg)>) -> Option<(String, Msg)> {
    let ref_msg: Option<(&str, &Msg)> = msg.as_ref().map(|(s, msg)| (s.as_str(), msg));
    match ref_msg {
        None => None, // Exit after None
        Some(msg) => match msg {
            (COMPONENT_COUNTER1, &MSG_KEY_TAB) => {
                view.active(COMPONENT_COUNTER2);
                None
            }
            (COMPONENT_COUNTER2, &MSG_KEY_TAB) => {
                view.active(COMPONENT_COUNTER1);
                None
            }
            (_, &MSG_KEY_ESC) => {
                // Quit on esc
                model.quit();
                None
            }
            _ => None,
        },
    }
}
