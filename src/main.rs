mod app;
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
mod utils;

// use ui::components::file_list::{FileList, FileListPropsBuilder};
use app::App;
use utils::context::Context;
use utils::keymap::*;

use std::thread::sleep;
use std::time::Duration;

use tuirealm::components::{input, label};
use tuirealm::props::borders::{BorderType, Borders};
use tuirealm::{InputType, Msg, Payload, PropsBuilder, Value, View};
// tui
use tui::layout::{Constraint, Direction, Layout};
use tui::style::Color;

const COMPONENT_INPUT: &str = "INPUT";
const COMPONENT_LABEL: &str = "LABEL";

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
    // myview.mount(
    //     COMPONENT_SCROLLTABLE,
    //     Box::new(FileList::new(
    //         FileListPropsBuilder::default()
    //             .with_background(Color::Yellow)
    //             .with_foreground(Color::Yellow)
    //             .with_borders(Borders::ALL, BorderType::Plain, Color::Yellow)
    //             .build(),
    //     )),
    // );
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
    // We need to give focus to input then
    myview.active(COMPONENT_INPUT);
    // Now we use the Model struct to keep track of some states
    let mut model: App = App::new();
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
            .constraints([Constraint::Length(3), Constraint::Length(3)].as_ref())
            .split(f.size());
        view.render(COMPONENT_INPUT, f, chunks[0]);
        view.render(COMPONENT_LABEL, f, chunks[1]);
    });
}

fn update(model: &mut App, view: &mut View, msg: Option<(String, Msg)>) -> Option<(String, Msg)> {
    let ref_msg: Option<(&str, &Msg)> = msg.as_ref().map(|(s, msg)| (s.as_str(), msg));
    match ref_msg {
        None => None, // Exit after None
        Some(msg) => match msg {
            (COMPONENT_INPUT, Msg::OnChange(Payload::One(Value::Str(input)))) => {
                // Update span
                let props =
                    label::LabelPropsBuilder::from(view.get_props(COMPONENT_LABEL).unwrap())
                        .with_text(format!("You typed: '{}'", input))
                        .build();
                // Report submit
                let msg = view.update(COMPONENT_LABEL, props);
                update(model, view, msg)
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
