#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]
// #![warn(clippy::restriction)]
/**
 * MIT License
 *
 * termusic - Copyright (c) 2021 Larry Hao
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
// mod app;
// mod config;
// mod invidious;
// mod player;
mod song;
// mod songtag;
// #[cfg(feature = "mpris")]
// mod souvlaki;
// mod ui;
// mod utils;

// use app::App;
// use config::Termusic;
// use std::path::Path;

// fn main() {
//     let mut config = Termusic::default();
//     config.load().unwrap_or_default();

//     let mut args: Vec<String> = std::env::args().collect();
//     // match args.len() {}

//     args.remove(0);
//     let mut should_exit = false;
//     for i in args {
//         let i = i.as_str();
//         match i {
//             "-v" | "--version" => {
//                 println!("Termusic version is: {}", VERSION);
//                 should_exit = true;
//             }

//             "-h" | "--help" => {
//                 println!(
//                     r"Termusic help:
// Usage: termusic [DIRECTORY] [OPTIONS]
// -v or --version print version and exit.
// -h or --help print this message and exit.
// directory: start termusic with directory.
// no arguments: start termusic with ~/.config/termusic/config.toml"
//                 );
//                 should_exit = true;
//             }

//             _ => {
//                 let p = Path::new(i);
//                 let mut p_string = String::new();
//                 if p.exists() {
//                     if p.has_root() {
//                         if let Ok(p1) = p.canonicalize() {
//                             p_string = p1.as_path().to_string_lossy().to_string();
//                         }
//                     } else if let Ok(p_base) = std::env::current_dir() {
//                         let p2 = p_base.join(&p);
//                         if let Ok(p3) = p2.canonicalize() {
//                             p_string = p3.as_path().to_string_lossy().to_string();
//                         }
//                     }
//                     config.music_dir_from_cli = Some(p_string);
//                 } else {
//                     println!(
//                         r"Unknown arguments
// Termusic help:
// Usage: termusic [DIRECTORY] [OPTIONS]
// -v or --version print version and exit.
// -h or --help print this message and exit.
// directory: start termusic with directory.
// no arguments: start termusic with ~/.config/termusic/config.toml"
//                     );
//                     should_exit = true;
//                 }
//             }
//         }
//     }

//     if should_exit {
//         return;
//     }

//     // glib::set_application_name("termusic");
//     // glib::set_prgname(Some("termusic"));
//     let mut app: App = App::new(config);
//     app.run();
// }
extern crate tuirealm;

use std::time::Duration;
use tuirealm::application::PollStrategy;
use tuirealm::props::{Alignment, Color, TextModifiers};
use tuirealm::{event::NoUserEvent, Application, AttrValue, Attribute, EventListenerCfg};
// -- internal
// mod app;
mod ui;
use std::path::Path;
use ui::app::model::Model;
use ui::components::{Digit, Label, Letter, MusicLibrary, Playlist};

const VERSION: &str = env!("CARGO_PKG_VERSION");

// Let's define the messages handled by our app. NOTE: it must derive `PartialEq`
#[derive(Debug, PartialEq)]
pub enum Msg {
    AppClose,
    DigitCounterChanged(isize),
    DigitCounterBlur,
    LibraryTreeExtendDir(String),
    LibraryTreeGoToUpperDir,
    LetterCounterChanged(isize),
    LetterCounterBlur,
    LibraryTreeBlur,
    PlaylistTableBlur,
    None,
}

// Let's define the component ids for our application
#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub enum Id {
    DigitCounter,
    LetterCounter,
    Label,
    Library,
    Playlist,
    Progress,
    Lyric,
}

fn main() {
    // Setup model
    let mut model = Model::new(Path::new("/home/tramhao/Music"));
    // Setup application
    // NOTE: NoUserEvent is a shorthand to tell tui-realm we're not going to use any custom user event
    // NOTE: the event listener is configured to use the default crossterm input listener and to raise a Tick event each second
    // which we will use to update the clock
    let mut app: Application<Id, Msg, NoUserEvent> = Application::init(
        EventListenerCfg::default()
            .default_input_listener(Duration::from_millis(20))
            .poll_timeout(Duration::from_millis(40))
            .tick_interval(Duration::from_secs(1)),
    );

    // Mount components
    assert!(app
        .mount(
            Id::Library,
            Box::new(MusicLibrary::new(model.tree.clone(), None)),
            vec![]
        )
        .is_ok());
    assert!(app
        .mount(Id::Playlist, Box::new(Playlist::default()), vec![])
        .is_ok());
    assert!(app
        .mount(
            Id::Label,
            Box::new(
                Label::default()
                    .text(format!("Press <CTRL+H> for help. Version: {}", VERSION,))
                    .alignment(Alignment::Left)
                    .background(Color::Reset)
                    .foreground(Color::Cyan)
                    .modifiers(TextModifiers::BOLD),
            ),
            Vec::default(),
        )
        .is_ok());
    // Mount counters
    assert!(app
        .mount(Id::LetterCounter, Box::new(Letter::new(0)), Vec::new())
        .is_ok());
    assert!(app
        .mount(Id::DigitCounter, Box::new(Digit::new(5)), Vec::default())
        .is_ok());
    // Active letter counter
    assert!(app.active(&Id::Library).is_ok());
    // Enter alternate screen
    let _ = model.terminal.enter_alternate_screen();
    let _ = model.terminal.enable_raw_mode();
    // Main loop
    // NOTE: loop until quit; quit is set in update if AppClose is received from counter
    while !model.quit {
        // Tick
        // match app.tick(&mut model, PollStrategy::UpTo(3)) {
        match app.tick(&mut model, PollStrategy::Once) {
            Err(err) => {
                assert!(app
                    .attr(
                        &Id::Label,
                        Attribute::Text,
                        AttrValue::String(format!("Application error: {}", err)),
                    )
                    .is_ok());
            }
            Ok(sz) if sz > 0 => {
                // NOTE: redraw if at least one msg has been processed
                model.redraw = true;
            }
            _ => {}
        }
        // Redraw
        if model.redraw {
            model.view(&mut app);
            model.redraw = false;
        }
    }
    // Terminate terminal
    let _ = model.terminal.leave_alternate_screen();
    let _ = model.terminal.disable_raw_mode();
    let _ = model.terminal.clear_screen();
}
