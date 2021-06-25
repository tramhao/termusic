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
use super::TagEditorActivity;
use crate::lyric;
use crate::ui::keymap::*;
use id3::frame::Lyrics;
// use crate::ui::components::scrolltable;
use super::ExitReason;
use tuirealm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use tuirealm::{Msg, Payload, Value};

impl TagEditorActivity {
    /// ### update
    ///
    /// Update auth activity model based on msg
    /// The function exits when returns None
    pub(super) fn update(&mut self, msg: Option<(String, Msg)>) -> Option<(String, Msg)> {
        let ref_msg: Option<(&str, &Msg)> = msg.as_ref().map(|(s, msg)| (s.as_str(), msg));
        match ref_msg {
            None => None, // Exit after None
            Some(msg) => match msg {
                (super::COMPONENT_TE_RADIO_TAG, &MSG_KEY_TAB) => {
                    self.view.active(super::COMPONENT_TE_INPUT_ARTIST);
                    None
                }
                (super::COMPONENT_TE_INPUT_ARTIST, &MSG_KEY_TAB) => {
                    self.view.active(super::COMPONENT_TE_INPUT_SONGNAME);
                    None
                }
                (super::COMPONENT_TE_INPUT_SONGNAME, &MSG_KEY_TAB) => {
                    self.view.active(super::COMPONENT_TE_SCROLLTABLE_OPTIONS);
                    None
                }

                (super::COMPONENT_TE_SCROLLTABLE_OPTIONS, &MSG_KEY_TAB) => {
                    self.view.active(super::COMPONENT_TE_TEXTAREA_LYRIC);
                    None
                }
                (super::COMPONENT_TE_TEXTAREA_LYRIC, &MSG_KEY_TAB) => {
                    self.view.active(super::COMPONENT_TE_RADIO_TAG);
                    None
                }
                (
                    super::COMPONENT_TE_RADIO_TAG,
                    Msg::OnSubmit(Payload::One(Value::Usize(choice))),
                ) => {
                    match *choice {
                        0 => {
                            // Get Tag
                            let mut song = self.song.clone().unwrap();
                            match self.view.get_state(super::COMPONENT_TE_INPUT_ARTIST) {
                                Some(Payload::One(Value::Str(artist))) => {
                                    song.artist = Some(artist);
                                }
                                _ => {}
                            }
                            match self.view.get_state(super::COMPONENT_TE_INPUT_SONGNAME) {
                                Some(Payload::One(Value::Str(title))) => {
                                    song.title = Some(title);
                                }
                                _ => {}
                            }

                            match lyric::lyric_options(&song) {
                                Ok(l) => self.add_lyric_options(l),
                                Err(e) => self.mount_error(&e.to_string()),
                            };
                        }
                        1 => {
                            // Rename file by Tag
                            let mut song = self.song.clone().unwrap();
                            match self.view.get_state(super::COMPONENT_TE_INPUT_ARTIST) {
                                Some(Payload::One(Value::Str(artist))) => {
                                    song.artist = Some(artist);
                                }
                                _ => {}
                            }
                            match self.view.get_state(super::COMPONENT_TE_INPUT_SONGNAME) {
                                Some(Payload::One(Value::Str(title))) => {
                                    song.title = Some(title);
                                }
                                _ => {}
                            }
                            match song.save() {
                                Ok(()) => {
                                    match song.rename_by_tag() {
                                        Ok(()) => {
                                            self.song = Some(song.clone());
                                            self.exit_reason =
                                                Some(ExitReason::NeedRefreshPlaylist);
                                            self.init_by_song(self.song.clone().unwrap())
                                        }
                                        Err(e) => self.mount_error(&e.to_string()),
                                    };
                                }
                                Err(e) => self.mount_error(&e.to_string()),
                            };
                        }
                        _ => {}
                    }
                    None
                }
                (super::COMPONENT_TE_SCROLLTABLE_OPTIONS, &MSG_KEY_CHAR_L)
                | (super::COMPONENT_TE_SCROLLTABLE_OPTIONS, &MSG_KEY_ENTER) => {
                    match self.view.get_state(super::COMPONENT_TE_SCROLLTABLE_OPTIONS) {
                        Some(Payload::One(Value::Usize(index))) => {
                            if self.lyric_options.len() < 1 {
                                return None;
                            }
                            let mut song = self.song.clone().unwrap();
                            let tag_lyric = self.lyric_options.get(index.clone()).unwrap();
                            let mut artist = String::from("");
                            for a in tag_lyric.artist.iter() {
                                artist += a;
                            }
                            song.artist = Some(artist);
                            song.title = Some(tag_lyric.title.clone().unwrap());
                            song.album = Some(tag_lyric.album.clone().unwrap());

                            match lyric::fetch_lyric(&tag_lyric) {
                                Ok(lyric_string) => {
                                    // println!("{}", lyric_string);
                                    // let lyric_string = lyric::lrc::Lyric::from_str(lyric_string.as_ref())?;
                                    let lyric_frame = Lyrics {
                                        lang: tag_lyric.lang_ext.clone().unwrap(),
                                        description: String::from("added by termusic."),
                                        text: lyric_string,
                                    };
                                    song.lyric_frames.clear();
                                    song.lyric_frames.push(lyric_frame);
                                    match song.save() {
                                        Ok(()) => {
                                            match song.rename_by_tag() {
                                                Ok(()) => {
                                                    self.song = Some(song.clone());
                                                    self.exit_reason =
                                                        Some(ExitReason::NeedRefreshPlaylist);
                                                    self.init_by_song(self.song.clone().unwrap())
                                                }
                                                Err(e) => self.mount_error(&e.to_string()),
                                            };
                                        }
                                        Err(e) => self.mount_error(&e.to_string()),
                                    };
                                }
                                Err(e) => self.mount_error(&e.to_string()),
                            };

                            None
                        }
                        _ => None,
                    }
                }

                (super::COMPONENT_TE_INPUT_ARTIST, &MSG_KEY_ENTER)
                | (super::COMPONENT_TE_INPUT_SONGNAME, &MSG_KEY_ENTER) => {
                    // Get Tag
                    let mut song = self.song.clone().unwrap();
                    match self.view.get_state(super::COMPONENT_TE_INPUT_ARTIST) {
                        Some(Payload::One(Value::Str(artist))) => {
                            song.artist = Some(artist);
                        }
                        _ => {}
                    }
                    match self.view.get_state(super::COMPONENT_TE_INPUT_SONGNAME) {
                        Some(Payload::One(Value::Str(title))) => {
                            song.title = Some(title);
                        }
                        _ => {}
                    }

                    match lyric::lyric_options(&song) {
                        Ok(l) => self.add_lyric_options(l),
                        Err(e) => self.mount_error(&e.to_string()),
                    };
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

                (_, &MSG_KEY_CHAR_L) => {
                    let event: Event = Event::Key(KeyEvent {
                        code: KeyCode::Right,
                        modifiers: KeyModifiers::NONE,
                    });
                    self.view.on(event);
                    None
                }

                // -- error
                (super::COMPONENT_TE_TEXT_ERROR, &MSG_KEY_ESC)
                | (super::COMPONENT_TE_TEXT_ERROR, &MSG_KEY_ENTER) => {
                    self.umount_error();
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
}
