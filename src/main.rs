mod app;
mod utils;

use app::App;
use utils::myinput::InputHandler;
use utils::terminal::Terminal;

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

fn main() {
    let mut terminal: Terminal = Terminal::new();
    let input: InputHandler = InputHandler::new();
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

    let mut model: App = App::new(myview);

    while !model.quit {
        // Listen for input events
        if let Ok(Some(ev)) = input.read_event() {
            // Pass event to view
            let msg = model.view.on(ev);
            model.redraw();
            // Call the elm friend update
            model.update(msg);
        }
        // If redraw, draw interface
        if model.redraw || model.last_redraw.elapsed() > Duration::from_millis(50) {
            // Call the elm friend vie1 function
            view(&mut terminal, &mut model.view);
            model.reset();
        }
        sleep(Duration::from_millis(10));
    }

    drop(terminal);
}

fn view(t: &mut Terminal, view: &View) {
    let _ = t.terminal.draw(|f| {
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
