//! ## Model
//!
//! app model
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
use crate::{song::Song, Application, Id, Msg};

use std::collections::VecDeque;
use std::path::{Path, PathBuf};
use tui_realm_treeview::Tree;
use tuirealm::terminal::TerminalBridge;
use tuirealm::tui::layout::{Constraint, Direction, Layout};
use tuirealm::{AttrValue, Attribute, NoUserEvent, Update, View};

pub const MAX_DEPTH: usize = 3;

pub struct Model {
    /// Indicates that the application must quit
    pub quit: bool,
    /// Tells whether to redraw interface
    pub redraw: bool,
    /// Used to draw to terminal
    pub terminal: TerminalBridge,
    pub path: PathBuf,
    pub tree: Tree,
    pub playlist_items: VecDeque<Song>,
}

impl Model {
    pub fn new(p: &Path) -> Self {
        Self {
            quit: false,
            redraw: true,
            tree: Tree::new(Self::dir_tree(p, MAX_DEPTH)),
            path: p.to_path_buf(),
            terminal: TerminalBridge::new().expect("Could not initialize terminal"),
            playlist_items: VecDeque::with_capacity(100),
        }
    }
    pub fn view(&mut self, app: &mut Application<Id, Msg, NoUserEvent>) {
        assert!(self
            .terminal
            .raw_mut()
            .draw(|f| {
                let chunks_main = Layout::default()
                    .direction(Direction::Vertical)
                    .margin(0)
                    .constraints([Constraint::Min(2), Constraint::Length(1)].as_ref())
                    .split(f.size());
                let chunks_left = Layout::default()
                    .direction(Direction::Horizontal)
                    .margin(0)
                    .constraints([Constraint::Ratio(1, 3), Constraint::Ratio(2, 3)].as_ref())
                    .split(chunks_main[0]);
                let chunks_right = Layout::default()
                    .direction(Direction::Vertical)
                    .margin(0)
                    .constraints(
                        [
                            Constraint::Min(2),
                            Constraint::Length(3),
                            Constraint::Length(4),
                        ]
                        .as_ref(),
                    )
                    .split(chunks_left[1]);

                // app.view(&Id::Progress, f, chunks_right[1]);
                // app.view(&Id::Lyric, f, chunks_right[2]);

                app.view(&Id::Library, f, chunks_left[0]);
                app.view(&Id::Playlist, f, chunks_right[0]);
                app.view(&Id::DigitCounter, f, chunks_right[1]);
                app.view(&Id::Label, f, chunks_main[1]);
            })
            .is_ok());
    }
}

// Let's implement Update for model

impl Update<Id, Msg, NoUserEvent> for Model {
    fn update(&mut self, view: &mut View<Id, Msg, NoUserEvent>, msg: Option<Msg>) -> Option<Msg> {
        if let Some(msg) = msg {
            // Set redraw
            self.redraw = true;
            // Match message
            match msg {
                Msg::AppClose => {
                    self.quit = true; // Terminate
                    None
                }
                Msg::DigitCounterBlur => {
                    // Give focus to letter counter
                    assert!(view.active(&Id::LetterCounter).is_ok());
                    None
                }
                Msg::LibraryTreeBlur => {
                    // Give focus to letter counter
                    assert!(view.active(&Id::Playlist).is_ok());
                    None
                }
                Msg::PlaylistTableBlur => {
                    assert!(view.active(&Id::Library).is_ok());
                    None
                }
                Msg::DigitCounterChanged(v) => {
                    // Update label
                    assert!(view
                        .attr(
                            &Id::Label,
                            Attribute::Text,
                            AttrValue::String(format!("DigitCounter has now value: {}", v))
                        )
                        .is_ok());
                    None
                }
                Msg::LetterCounterBlur => {
                    // Give focus to digit counter
                    assert!(view.active(&Id::DigitCounter).is_ok());
                    None
                }
                Msg::LetterCounterChanged(v) => {
                    // Update label
                    assert!(view
                        .attr(
                            &Id::Label,
                            Attribute::Text,
                            AttrValue::String(format!("LetterCounter has now value: {}", v))
                        )
                        .is_ok());
                    None
                }
                Msg::LibraryTreeExtendDir(path) => {
                    self.extend_dir(&path, PathBuf::from(path.as_str()).as_path(), MAX_DEPTH);
                    self.reload_tree(view);
                    None
                }
                Msg::LibraryTreeGoToUpperDir => {
                    if let Some(parent) = self.upper_dir() {
                        self.scan_dir(parent.as_path());
                        self.reload_tree(view);
                    }
                    None
                }
                Msg::None => None,
                Msg::PlaylistAdd(current_node) => {
                    self.add_playlist(&current_node, view);
                    None
                } // _ => None,
            }
        } else {
            None
        }
    }
}
