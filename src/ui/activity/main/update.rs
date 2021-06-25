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
    MainActivity, Status, COMPONENT_CONFIRMATION_INPUT, COMPONENT_CONFIRMATION_RADIO,
    COMPONENT_INPUT_URL, COMPONENT_PARAGRAPH_LYRIC, COMPONENT_PROGRESS, COMPONENT_SCROLLTABLE,
    COMPONENT_TEXT_ERROR, COMPONENT_TEXT_HELP, COMPONENT_TREEVIEW,
};
use crate::song::Song;
use crate::ui::keymap::*;
use std::str::FromStr;
// ext
use humantime::format_duration;
// use lrc::{Lyrics, TimeTag};
use super::TransferState;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tui_realm_treeview::TreeViewPropsBuilder;
use tuirealm::components::{paragraph, progress_bar};
use tuirealm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use tuirealm::props::TextSpanBuilder;
use tuirealm::PropsBuilder;
use tuirealm::{Msg, Payload, Value};

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
                (COMPONENT_TREEVIEW, &MSG_KEY_TAB) => {
                    self.view.active(COMPONENT_SCROLLTABLE);
                    None
                }
                (COMPONENT_SCROLLTABLE, &MSG_KEY_TAB) => {
                    self.view.active(COMPONENT_TREEVIEW);
                    None
                }
                // (COMPONENT_TREEVIEW, Msg::OnChange(Payload::One(Value::Str(node_id)))) => {
                //     // Update span
                //     let props = label::LabelPropsBuilder::from(
                //         self.view.get_props(COMPONENT_LABEL_HELP).unwrap(),
                //     )
                //     .with_text(format!("Selected: '{}'", node_id))
                //     .build();
                //     // Report submit
                //     let msg = self.view.update(COMPONENT_LABEL_HELP, props);
                //     self.update(msg)
                // }
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
                // seek
                (_, &MSG_KEY_CHAR_F) => match self.player.seek(5) {
                    Ok(_) => None,
                    Err(_) => {
                        self.status = Some(Status::Stopped);
                        None
                    }
                },
                // seek backward
                (_, &MSG_KEY_CHAR_B) => match self.player.seek(-5) {
                    Ok(_) => {
                        self.time_pos -= 5;
                        None
                    }
                    Err(_) => None,
                },

                (COMPONENT_TREEVIEW, &MSG_KEY_CHAR_H) => {
                    let event: Event = Event::Key(KeyEvent {
                        code: KeyCode::Left,
                        modifiers: KeyModifiers::NONE,
                    });
                    self.view.on(event);
                    None
                }
                (COMPONENT_SCROLLTABLE, &MSG_KEY_CHAR_G) => {
                    let event: Event = Event::Key(KeyEvent {
                        code: KeyCode::Home,
                        modifiers: KeyModifiers::NONE,
                    });
                    self.view.on(event);
                    None
                }

                (COMPONENT_SCROLLTABLE, &MSG_KEY_CHAR_CAPITAL_G) => {
                    let event: Event = Event::Key(KeyEvent {
                        code: KeyCode::End,
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
                                match Song::from_str(&p) {
                                    Ok(s) => self.add_queue(s),
                                    Err(e) => self.mount_error(e.to_string().as_ref()),
                                };
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
                                    match Song::from_str(i) {
                                        Ok(s) => self.add_queue(s),
                                        Err(e) => println!("{}", e),
                                    };
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
                            self.current_song = Some(self.queue_items[index].clone());
                            self.player.queue_and_play(self.queue_items[index].clone());
                            self.update_photo();
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
                (_, &MSG_KEY_CHAR_P) | (_, &MSG_KEY_SPACE) => {
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
                // shuffle
                (COMPONENT_SCROLLTABLE, &MSG_KEY_CHAR_S) => {
                    self.shuffle();
                    None
                }

                // start download
                (COMPONENT_TREEVIEW, &MSG_KEY_CHAR_S) => {
                    self.mount_youtube_url();
                    None
                }

                (COMPONENT_TREEVIEW, &MSG_KEY_CHAR_D) => {
                    match self.view.get_state(COMPONENT_TREEVIEW) {
                        Some(Payload::One(Value::Str(node_id))) => {
                            let p: &Path = Path::new(node_id.as_str());
                            if p.is_file() {
                                self.mount_confirmation_radio();
                            } else {
                                self.mount_confirmation_input();
                            }
                            None
                        }
                        _ => None,
                    }
                }

                (COMPONENT_INPUT_URL, Msg::OnSubmit(_)) => {
                    match self.view.get_state(COMPONENT_INPUT_URL) {
                        Some(Payload::One(Value::Str(url))) => self.youtube_dl(url),
                        _ => {}
                    }
                    self.umount_youtube_url();
                    None
                }

                (COMPONENT_INPUT_URL, &MSG_KEY_ESC) => {
                    self.umount_youtube_url();
                    None
                }

                (COMPONENT_CONFIRMATION_INPUT, Msg::OnSubmit(_)) => {
                    match self.view.get_state(COMPONENT_CONFIRMATION_INPUT) {
                        Some(Payload::One(Value::Str(p))) => {
                            if p == "DELETE" {
                                self.delete_songs();
                            }
                        }
                        _ => {}
                    }
                    self.umount_confirmation_input();
                    None
                }

                (COMPONENT_CONFIRMATION_INPUT, &MSG_KEY_ESC) => {
                    self.umount_confirmation_input();
                    None
                }
                (_, &MSG_KEY_CHAR_H) => {
                    let event: Event = Event::Key(KeyEvent {
                        code: KeyCode::Left,
                        modifiers: KeyModifiers::NONE,
                    });
                    self.view.on(event);
                    None
                }
                (_, &MSG_KEY_CHAR_L) => {
                    let event: Event = Event::Key(KeyEvent {
                        code: KeyCode::Right,
                        modifiers: KeyModifiers::NONE,
                    });
                    self.view.on(event);
                    None
                }

                (COMPONENT_CONFIRMATION_RADIO, Msg::OnSubmit(_)) => {
                    match self.view.get_state(COMPONENT_CONFIRMATION_RADIO) {
                        Some(Payload::One(Value::Usize(index))) => {
                            if index != 0 {
                                self.umount_confirmation_radio();
                                return None;
                            }
                            self.delete_song();
                        }
                        _ => {}
                    }
                    self.umount_confirmation_radio();
                    None
                }

                (COMPONENT_CONFIRMATION_RADIO, &MSG_KEY_ESC) => {
                    self.umount_confirmation_radio();
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

                (_, &MSG_KEY_QUESTION_MARK) => {
                    // Show help
                    self.mount_help();
                    None
                }
                // -- help
                (COMPONENT_TEXT_HELP, &MSG_KEY_ENTER) | (COMPONENT_TEXT_HELP, &MSG_KEY_ESC) => {
                    self.umount_help();
                    None
                }
                // -- error
                (COMPONENT_TEXT_ERROR, &MSG_KEY_ESC) | (COMPONENT_TEXT_ERROR, &MSG_KEY_ENTER) => {
                    self.umount_error();
                    None
                }

                (COMPONENT_TREEVIEW, &MSG_KEY_CHAR_T) => {
                    self.run_tageditor();
                    None
                }

                // Refresh playlist
                (_, &MSG_KEY_CHAR_R) => {
                    self.refresh_playlist();
                    None
                }

                (_, &MSG_KEY_ESC) | (_, &MSG_KEY_CHAR_CAPITAL_Q) => {
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

        if time_pos >= duration - 1 {
            self.status = Some(Status::Stopped);
            return;
        }

        if time_pos >= self.time_pos + 1 || time_pos < 2 {
            self.time_pos = time_pos;
            let props = self.view.get_props(COMPONENT_PROGRESS).unwrap();
            let props = progress_bar::ProgressBarPropsBuilder::from(props)
                .with_progress(new_prog)
                .with_texts(
                    Some(format!("Playing: {}", song_title)),
                    format!(
                        "{}     :     {} ",
                        format_duration(Duration::from_secs(time_pos as u64)),
                        format_duration(Duration::from_secs(duration as u64))
                    ),
                )
                .build();
            self.view.update(COMPONENT_PROGRESS, props);
            self.redraw = true;
        }

        // Update lyrics
        if self.queue_items.len() <= 0 {
            return;
        }

        let song = match self.current_song.as_ref() {
            Some(song) => song,
            None => return,
        };

        if song.lyric_frames.len() <= 0 {
            let props = self.view.get_props(COMPONENT_PARAGRAPH_LYRIC).unwrap();
            let props = paragraph::ParagraphPropsBuilder::from(props)
                .with_texts(
                    Some(String::from("Lyrics")),
                    vec![TextSpanBuilder::new("No lyrics available.").build()],
                )
                .build();
            self.view.update(COMPONENT_PARAGRAPH_LYRIC, props);
            return;
        }

        let line = match song.parsed_lyric.as_ref() {
            Some(l) => l.get_text(time_pos).unwrap(),
            None => String::from(""),
        };

        let props = self.view.get_props(COMPONENT_PARAGRAPH_LYRIC).unwrap();
        let props = paragraph::ParagraphPropsBuilder::from(props)
            .with_texts(
                Some(String::from("Lyrics")),
                vec![TextSpanBuilder::new(line.as_ref()).build()],
            )
            .build();
        self.view.update(COMPONENT_PARAGRAPH_LYRIC, props);
    }

    // update picture of album
    pub fn update_photo(&mut self) {
        // if terminal is not kitty, just don't show photo
        if viuer::KittySupport::Local != viuer::get_kitty_support() {
            return;
        };

        let song = match self.current_song.clone() {
            Some(song) => song,
            None => return,
        };

        // clear all previous image
        self.context.as_mut().unwrap().clear_image();

        // if no photo, just return
        if song.picture.len() <= 0 {
            return;
        }

        // just show the first photo
        match image::load_from_memory(&song.picture[0].data) {
            Ok(image) => {
                let (term_width, term_height) = viuer::terminal_size();
                // Set desired image dimensions
                let (orig_width, orig_height) = image::GenericImageView::dimensions(&image);
                let ratio = orig_height as f64 / orig_width as f64;
                let width = 20 as u16;
                let height = (width as f64 * ratio) as u16;
                let config = viuer::Config {
                    transparent: true,
                    absolute_offset: true,
                    x: term_width - width - 1,
                    y: (term_height - height / 2 - 8) as i16 - 1,
                    width: Some(width as u32),
                    height: None,
                    ..Default::default()
                };
                viuer::print(&image, &config).expect("image printing failed.");
            }
            Err(_) => return,
        };
    }

    pub fn update_playlist(&mut self) {
        match self.receiver.try_recv() {
            Ok(transfer_state) => match transfer_state {
                TransferState::Running => {}
                TransferState::Completed => self.refresh_playlist(),
                TransferState::ErrDownload => {
                    self.mount_error("download failed");
                }
            },
            Err(_) => return,
        };
    }
}
