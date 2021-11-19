//! ## Utils
//!
//! `Utils` implements utilities functions to work with layouts

// pub mod activity;
/**
 * MIT License
 *
 * termscp - Copyright (c) 2021 Christian Visintin
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
pub mod app;
pub mod components;

use crate::config::Termusic;
use app::model::Model;
// Let's define the messages handled by our app. NOTE: it must derive `PartialEq`
use std::time::Duration;
use tuirealm::application::PollStrategy;
use tuirealm::props::{Alignment, Color, TextModifiers};
use tuirealm::{event::NoUserEvent, Application, AttrValue, Attribute, EventListenerCfg};
// -- internal

use components::{Digit, Label, Letter, MusicLibrary, Playlist};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
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
    PlaylistAdd(String),
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

pub struct UI {
    config: Termusic,
    model: Model,
    app: Application<Id, Msg, NoUserEvent>,
}

impl UI {
    /// Instantiates a new Ui
    pub fn new(config: &Termusic) -> Self {
        let model = Model::new(config);
        // let app = Self::init_application(&model, tick);
        let mut app: Application<Id, Msg, NoUserEvent> = Application::init(
            EventListenerCfg::default()
                .default_input_listener(Duration::from_millis(20))
                .poll_timeout(Duration::from_millis(40))
                .tick_interval(Duration::from_secs(1)),
        );
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

        Self {
            config: config.clone(),
            // client: FeedClient::default(),
            model,
            app,
        }
    }

    /// ### run
    ///
    /// Main loop for Ui thread
    pub fn run(&mut self) {
        self.model.init_terminal();
        // Fetch sources once
        // self.fetch_all_sources();
        // Main loop
        while !self.model.quit {
            // if let Err(err) = self.app.tick(&mut self.model, PollStrategy::UpTo(3)) {
            //     self.mount_error_popup(format!("Application error: {}", err));
            // }
            // // Poll fetched sources
            // self.poll_fetched_sources();
            // // Run tasks
            // self.run_tasks();
            // // Check whether to force redraw
            // self.check_force_redraw();
            // // View
            // self.model.view(&mut self.app);
            match self.app.tick(&mut self.model, PollStrategy::Once) {
                Err(err) => {
                    assert!(self
                        .app
                        .attr(
                            &Id::Label,
                            Attribute::Text,
                            AttrValue::String(format!("Application error: {}", err)),
                        )
                        .is_ok());
                }
                Ok(sz) if sz > 0 => {
                    // NOTE: redraw if at least one msg has been processed
                    self.model.redraw = true;
                }
                _ => {}
            }
            // Redraw
            if self.model.redraw {
                self.model.view(&mut self.app);
                self.model.redraw = false;
            }
        }
        self.model.finalize_terminal();
    }
}
