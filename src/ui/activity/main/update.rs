//! ## SetupActivity
//!
//! `setup_activity` is the module which implements the Setup activity, which is the activity to
//! work on termscp configuration

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
// locals
use super::{
    MainActivity, Status, COMPONENT_LABEL_HELP, COMPONENT_PROGRESS, COMPONENT_SCROLLTABLE,
    COMPONENT_TREEVIEW,
};
use crate::ui::keymap::*;
// ext
use humantime::format_duration;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tuirealm::components::{label, progress_bar};
use tuirealm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use tuirealm::PropsBuilder;
use tuirealm::{Msg, Payload, Value};

use tui_realm_treeview::TreeViewPropsBuilder;

impl MainActivity {
    /// ### update
    ///
    /// Update auth activity model based on msg
    /// The function exits when returns None
    pub(super) fn update(&mut self, msg: Option<(String, Msg)>) -> Option<(String, Msg)> {
        let ref_msg: Option<(&str, &Msg)> = msg.as_ref().map(|(s, msg)| (s.as_str(), msg));
        match ref_msg {
            None => None, // Exit after None
            Some(msg) => match msg {
                // (COMPONENT_INPUT, Msg::OnChange(Payload::One(Value::Str(input)))) => {
                //     // Update span
                //     let props = label::LabelPropsBuilder::from(
                //         self.view.get_props(COMPONENT_LABEL).unwrap(),
                //     )
                //     .with_text(format!("You typed: '{}'", input))
                //     .build();
                //     // Report submit
                //     let msg = self.view.update(COMPONENT_LABEL, props);
                //     self.update(msg)
                // }
                (COMPONENT_TREEVIEW, &MSG_KEY_TAB) => {
                    self.view.active(COMPONENT_SCROLLTABLE);
                    None
                }
                (COMPONENT_SCROLLTABLE, &MSG_KEY_TAB) => {
                    self.view.active(COMPONENT_TREEVIEW);
                    None
                }
                (COMPONENT_TREEVIEW, Msg::OnChange(Payload::One(Value::Str(node_id)))) => {
                    // Update span
                    let props = label::LabelPropsBuilder::from(
                        self.view.get_props(COMPONENT_LABEL_HELP).unwrap(),
                    )
                    .with_text(format!("Selected: '{}'", node_id))
                    .build();
                    // Report submit
                    let msg = self.view.update(COMPONENT_LABEL_HELP, props);
                    self.update(msg)
                }
                (COMPONENT_TREEVIEW, Msg::OnSubmit(Payload::One(Value::Str(node_id)))) => {
                    // Update tree
                    self.scan_dir(PathBuf::from(node_id.as_str()).as_path());
                    // Update
                    let props = TreeViewPropsBuilder::from(
                        self.view.get_props(COMPONENT_TREEVIEW).unwrap(),
                    )
                    .with_tree(self.tree.root())
                    .with_title(Some(String::from(self.path.to_string_lossy())))
                    .build();
                    let msg = self.view.update(COMPONENT_TREEVIEW, props);
                    self.update(msg)
                }
                (COMPONENT_TREEVIEW, &MSG_KEY_BACKSPACE) => {
                    // Update tree
                    match self.upper_dir() {
                        None => None,
                        Some(p) => {
                            let p: PathBuf = p.to_path_buf();
                            self.scan_dir(p.as_path());
                            // Update
                            let props = TreeViewPropsBuilder::from(
                                self.view.get_props(COMPONENT_TREEVIEW).unwrap(),
                            )
                            .with_tree(self.tree.root())
                            .with_title(Some(String::from(self.path.to_string_lossy())))
                            .build();
                            let msg = self.view.update(COMPONENT_TREEVIEW, props);
                            self.update(msg)
                        }
                    }
                }
                (COMPONENT_TREEVIEW, &MSG_KEY_CHAR_H) => {
                    let event: Event = Event::Key(KeyEvent {
                        code: KeyCode::Left,
                        modifiers: KeyModifiers::NONE,
                    });
                    self.view.on(event);
                    None
                }
                (_, &MSG_KEY_CHAR_J) => {
                    let event: Event = Event::Key(KeyEvent {
                        code: KeyCode::Down,
                        modifiers: KeyModifiers::NONE,
                    });
                    self.view.on(event);
                    None
                }
                (_, &MSG_KEY_CHAR_K) => {
                    let event: Event = Event::Key(KeyEvent {
                        code: KeyCode::Up,
                        modifiers: KeyModifiers::NONE,
                    });
                    self.view.on(event);
                    None
                }
                (COMPONENT_TREEVIEW, &MSG_KEY_CHAR_L) => {
                    // Add selected song to queue
                    match self.view.get_state(COMPONENT_TREEVIEW) {
                        Some(Payload::One(Value::Str(node_id))) => {
                            let p: &Path = Path::new(node_id.as_str());
                            if p.is_dir() {
                                let event: Event = Event::Key(KeyEvent {
                                    code: KeyCode::Right,
                                    modifiers: KeyModifiers::NONE,
                                });
                                self.view.on(event);
                                None
                            } else {
                                let p = p.to_string_lossy();
                                self.add_queue(String::from(p));
                                None
                            }
                        }
                        _ => None,
                    }
                }
                (COMPONENT_TREEVIEW, &MSG_KEY_CHAR_CAPITAL_L) => {
                    // Add all songs in a folder to queue
                    match self.view.get_state(COMPONENT_TREEVIEW) {
                        Some(Payload::One(Value::Str(node_id))) => {
                            let p: &Path = Path::new(node_id.as_str());
                            if p.is_dir() {
                                // let p = p.to_string_lossy();
                                let new_items = Self::dir_children(p);
                                for i in new_items.iter().rev() {
                                    self.add_queue(i.to_owned());
                                }
                            }
                            None
                        }
                        _ => None,
                    }
                }

                (COMPONENT_SCROLLTABLE, &MSG_KEY_CHAR_L) => {
                    match self.view.get_state(COMPONENT_SCROLLTABLE) {
                        Some(Payload::One(Value::Usize(index))) => {
                            self.time_pos = 0;
                            self.player.queue_and_play(self.queue_items[index].clone());
                            None
                        }
                        _ => None,
                    }
                }

                (COMPONENT_SCROLLTABLE, &MSG_KEY_CHAR_D) => {
                    match self.view.get_state(COMPONENT_SCROLLTABLE) {
                        Some(Payload::One(Value::Usize(index))) => {
                            self.delete_item(index);
                            None
                        }
                        _ => None,
                    }
                }

                (COMPONENT_SCROLLTABLE, &MSG_KEY_CHAR_CPAITAL_D) => {
                    self.empty_queue();
                    None
                }

                // Toggle pause
                (_, &MSG_KEY_CHAR_P) => {
                    if self.player.is_paused() {
                        self.status = Some(Status::Running);
                        self.player.resume();
                    } else {
                        self.status = Some(Status::Paused);
                        self.player.pause();
                    }
                    None
                }
                // Toggle skip
                (_, &MSG_KEY_CHAR_N) => {
                    self.status = Some(Status::Stopped);
                    None
                }

                // increase volume
                (_, &MSG_KEY_CHAR_PLUS) | (_, &MSG_KEY_CHAR_EQUAL) => {
                    self.player.volume_up();
                    None
                }
                // decrease volume
                (_, &MSG_KEY_CHAR_MINUS) | (_, &MSG_KEY_CHAR_DASH) => {
                    self.player.volume_down();
                    None
                }
                (_, &MSG_KEY_ESC) | (_, &MSG_KEY_CHAR_Q) => {
                    // Quit on esc
                    self.exit_reason = Some(super::ExitReason::Quit);
                    None
                }
                _ => None,
            },
        }
    }

    pub fn update_progress(&mut self) {
        let (new_prog, time_pos, duration, song_title) = self.player.get_progress();
        // let old_prog: f64 = match props.own.get("progress") {
        //     Some(PropPayload::One(PropValue::F64(val))) => val.to_owned(),
        //     _ => 0.0,
        // };

        if time_pos >= self.time_pos + 1 || time_pos < 2 {
            self.time_pos = time_pos;
            let props = self.view.get_props(COMPONENT_PROGRESS).unwrap();
            let props = progress_bar::ProgressBarPropsBuilder::from(props)
                .with_progress(new_prog)
                .with_texts(
                    Some(format!("Playing: {}", song_title)),
                    format!(
                        "{} : {} ",
                        format_duration(Duration::from_secs(time_pos as u64)),
                        format_duration(Duration::from_secs(duration as u64))
                    ),
                )
                .build();

            self.view.update(COMPONENT_PROGRESS, props);
            self.redraw = true;
            // self.update(msg);
        }
    }
}
