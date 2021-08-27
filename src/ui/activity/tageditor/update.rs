use super::SearchLyricState;
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
// locals
use super::{TagEditorActivity, COMPONENT_TE_LABEL_HELP};
use crate::ui::activity::main::{StatusLine, TransferState};
use crate::ui::keymap::*;
// use crate::ui::components::scrolltable;
use super::ExitReason;
use crate::song::Song;
use crate::songtag;
use std::path::Path;
use std::str::FromStr;
use tui_realm_stdlib::label;
use tuirealm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use tuirealm::tui::style::Color;
use tuirealm::PropsBuilder;
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
                (super::COMPONENT_TE_RADIO_TAG, key) if key == &MSG_KEY_TAB => {
                    self.view.active(super::COMPONENT_TE_INPUT_ARTIST);
                    None
                }
                (super::COMPONENT_TE_INPUT_ARTIST, key) if key == &MSG_KEY_TAB => {
                    self.view.active(super::COMPONENT_TE_INPUT_SONGNAME);
                    None
                }
                (super::COMPONENT_TE_INPUT_SONGNAME, key) if key == &MSG_KEY_TAB => {
                    self.view.active(super::COMPONENT_TE_SCROLLTABLE_OPTIONS);
                    None
                }

                (super::COMPONENT_TE_SCROLLTABLE_OPTIONS, key) if key == &MSG_KEY_TAB => {
                    self.view.active(super::COMPONENT_TE_SELECT_LYRIC);
                    None
                }

                (super::COMPONENT_TE_SELECT_LYRIC, key) if key == &MSG_KEY_TAB => {
                    self.view.active(super::COMPONENT_TE_DELETE_LYRIC);
                    None
                }

                (super::COMPONENT_TE_DELETE_LYRIC, key) if key == &MSG_KEY_TAB => {
                    self.view.active(super::COMPONENT_TE_TEXTAREA_LYRIC);
                    None
                }

                (super::COMPONENT_TE_TEXTAREA_LYRIC, key) if key == &MSG_KEY_TAB => {
                    self.view.active(super::COMPONENT_TE_RADIO_TAG);
                    None
                }
                (
                    super::COMPONENT_TE_RADIO_TAG,
                    Msg::OnSubmit(Payload::One(Value::Usize(choice))),
                ) => {
                    if *choice == 0 {
                        // Rename file by Tag
                        if let Some(mut song) = self.song.take() {
                            if let Some(Payload::One(Value::Str(artist))) =
                                self.view.get_state(super::COMPONENT_TE_INPUT_ARTIST)
                            {
                                song.artist = Some(artist);
                            }
                            if let Some(Payload::One(Value::Str(title))) =
                                self.view.get_state(super::COMPONENT_TE_INPUT_SONGNAME)
                            {
                                song.title = Some(title);
                            }
                            match song.save_tag() {
                                Ok(()) => {
                                    match song.rename_by_tag() {
                                        Ok(()) => {
                                            if let Some(file) = song.file() {
                                                self.exit_reason =
                                                    Some(ExitReason::NeedRefreshPlaylist(
                                                        file.to_string(),
                                                    ));
                                            }
                                            self.init_by_song(&song)
                                        }
                                        Err(e) => self.mount_error(&e.to_string()),
                                    };
                                }
                                Err(e) => self.mount_error(&e.to_string()),
                            };
                        }
                    }
                    None
                }
                (super::COMPONENT_TE_SCROLLTABLE_OPTIONS, key)
                    if (key == &MSG_KEY_CHAR_L) | (key == &MSG_KEY_ENTER) =>
                {
                    match self.view.get_state(super::COMPONENT_TE_SCROLLTABLE_OPTIONS) {
                        Some(Payload::One(Value::Usize(index))) => {
                            if self.lyric_options.is_empty() {
                                return None;
                            }
                            if let Some(mut song) = self.song.clone() {
                                let song_tag = self.lyric_options.get(index)?;
                                let lang_ext = song_tag
                                    .lang_ext
                                    .to_owned()
                                    .unwrap_or_else(|| String::from("eng"));
                                let mut artist = String::from("");
                                for a in song_tag.artist.iter() {
                                    artist += a;
                                }
                                song.artist = Some(artist);
                                if let Some(title) = &song_tag.title {
                                    song.title = Some(title.to_owned());
                                }
                                if let Some(album) = &song_tag.album {
                                    song.album = Some(album.to_owned());
                                }

                                if let Ok(lyric_string) = song_tag.fetch_lyric() {
                                    song.set_lyric(&lyric_string, &lang_ext);
                                    if let Ok(artwork) = song_tag.fetch_photo() {
                                        song.set_photo(artwork);
                                    }
                                    match song.save_tag() {
                                        Ok(()) => {
                                            match song.rename_by_tag() {
                                                Ok(()) => {
                                                    if let Some(file) = song.file() {
                                                        self.exit_reason =
                                                            Some(ExitReason::NeedRefreshPlaylist(
                                                                file.to_string(),
                                                            ));
                                                    }
                                                    self.init_by_song(&song)
                                                }
                                                Err(e) => self.mount_error(&e.to_string()),
                                            };
                                        }
                                        Err(e) => self.mount_error(&e.to_string()),
                                    }
                                }
                            }

                            None
                        }
                        _ => None,
                    }
                }

                // download
                (super::COMPONENT_TE_SCROLLTABLE_OPTIONS, key) if key == &MSG_KEY_CHAR_S => {
                    if let Some(Payload::One(Value::Usize(index))) =
                        self.view.get_state(super::COMPONENT_TE_SCROLLTABLE_OPTIONS)
                    {
                        if let Some(song_tag) = self.lyric_options.get(index) {
                            if let Some(song) = &self.song {
                                if let Some(file) = &song.file {
                                    if let Err(e) = song_tag.download(file, self.sender.clone()) {
                                        self.mount_error(&e.to_string());
                                    }
                                }
                            }
                        }
                    }
                    None
                }

                // select lyric
                (
                    super::COMPONENT_TE_SELECT_LYRIC,
                    Msg::OnSubmit(Payload::One(Value::Usize(index))),
                ) => {
                    if let Some(mut song) = self.song.clone() {
                        song.lyric_selected = *index;
                        self.init_by_song(&song);
                        self.song = Some(song);
                    }
                    None
                }

                // delete lyric
                (super::COMPONENT_TE_DELETE_LYRIC, key) if key == &MSG_KEY_ENTER => {
                    if let Some(mut song) = self.song.take() {
                        if song.lyric_frames.is_empty() {
                            song.parsed_lyric = None;
                            return None;
                        }
                        song.lyric_frames.remove(song.lyric_selected as usize);
                        if (song.lyric_selected as usize >= song.lyric_frames.len())
                            && (song.lyric_selected > 0)
                        {
                            song.lyric_selected -= 1;
                        }
                        match song.save_tag() {
                            Ok(_) => {
                                self.init_by_song(&song);
                                self.song = Some(song);
                            }
                            Err(e) => self.mount_error(&e.to_string()),
                        }
                    }
                    None
                }

                (super::COMPONENT_TE_INPUT_ARTIST, Msg::OnSubmit(Payload::One(Value::Str(_))))
                | (
                    super::COMPONENT_TE_INPUT_SONGNAME,
                    Msg::OnSubmit(Payload::One(Value::Str(_))),
                ) => {
                    // Get Tag
                    let mut search_str = String::new();
                    if let Some(Payload::One(Value::Str(artist))) =
                        self.view.get_state(super::COMPONENT_TE_INPUT_ARTIST)
                    {
                        search_str.push_str(&artist);
                    }

                    search_str.push(' ');
                    if let Some(Payload::One(Value::Str(title))) =
                        self.view.get_state(super::COMPONENT_TE_INPUT_SONGNAME)
                    {
                        search_str.push_str(&title);
                    }

                    if search_str.len() < 4 {
                        if let Some(song) = &self.song {
                            if let Some(file) = &song.file {
                                let p: &Path = Path::new(file.as_str());
                                if let Some(stem) = p.file_stem() {
                                    search_str = stem.to_string_lossy().to_string();
                                }
                            }
                        }
                    }

                    songtag::lyric_options(&search_str, self.sender_songtag.clone());
                    None
                }

                (super::COMPONENT_TE_SCROLLTABLE_OPTIONS, key) if key == &MSG_KEY_CHAR_G => {
                    let event: Event = Event::Key(KeyEvent {
                        code: KeyCode::Home,
                        modifiers: KeyModifiers::NONE,
                    });
                    self.view.on(event);
                    None
                }

                (super::COMPONENT_TE_SCROLLTABLE_OPTIONS, key)
                    if key == &MSG_KEY_CHAR_CAPITAL_G =>
                {
                    let event: Event = Event::Key(KeyEvent {
                        code: KeyCode::End,
                        modifiers: KeyModifiers::NONE,
                    });
                    self.view.on(event);
                    None
                }

                (super::COMPONENT_TE_TEXTAREA_LYRIC, key) if key == &MSG_KEY_CHAR_G => {
                    let event: Event = Event::Key(KeyEvent {
                        code: KeyCode::Home,
                        modifiers: KeyModifiers::NONE,
                    });
                    self.view.on(event);
                    None
                }

                (super::COMPONENT_TE_TEXTAREA_LYRIC, key) if key == &MSG_KEY_CHAR_CAPITAL_G => {
                    let event: Event = Event::Key(KeyEvent {
                        code: KeyCode::End,
                        modifiers: KeyModifiers::NONE,
                    });
                    self.view.on(event);
                    None
                }

                (_, key) if key == &MSG_KEY_QUESTION_MARK => {
                    // Show help
                    self.mount_help();
                    None
                }
                // -- help
                (super::COMPONENT_TE_TEXT_HELP, key)
                    if (key == &MSG_KEY_ENTER) | (key == &MSG_KEY_ESC) =>
                {
                    self.umount_help();
                    None
                }

                (_, key) if key == &MSG_KEY_CHAR_H => {
                    let event: Event = Event::Key(KeyEvent {
                        code: KeyCode::Left,
                        modifiers: KeyModifiers::NONE,
                    });
                    self.view.on(event);
                    None
                }

                (_, key) if key == &MSG_KEY_CHAR_J => {
                    let event: Event = Event::Key(KeyEvent {
                        code: KeyCode::Down,
                        modifiers: KeyModifiers::NONE,
                    });
                    self.view.on(event);
                    None
                }
                (_, key) if key == &MSG_KEY_CHAR_K => {
                    let event: Event = Event::Key(KeyEvent {
                        code: KeyCode::Up,
                        modifiers: KeyModifiers::NONE,
                    });
                    self.view.on(event);
                    None
                }

                (_, key) if key == &MSG_KEY_CHAR_L => {
                    let event: Event = Event::Key(KeyEvent {
                        code: KeyCode::Right,
                        modifiers: KeyModifiers::NONE,
                    });
                    self.view.on(event);
                    None
                }

                // -- error
                (super::COMPONENT_TE_TEXT_ERROR, key)
                    if (key == &MSG_KEY_ESC) | (key == &MSG_KEY_ENTER) =>
                {
                    self.umount_error();
                    None
                }

                (_, key) if (key == &MSG_KEY_ESC) | (key == &MSG_KEY_CHAR_CAPITAL_Q) => {
                    // Quit on esc
                    self.exit_reason = Some(super::ExitReason::Quit);
                    None
                }
                _ => None,
            },
        }
    }

    pub fn update_download_progress(&mut self) {
        if let Ok(transfer_state) = self.receiver.try_recv() {
            match transfer_state {
                TransferState::Running => {
                    self.update_status_line(StatusLine::Running);
                }
                TransferState::Success => {
                    self.update_status_line(StatusLine::Success);
                }
                TransferState::Completed(file) => {
                    if let Some(f) = file {
                        if let Ok(song) = Song::from_str(&f) {
                            let song1 = song.to_owned();
                            self.song = Some(song);
                            self.exit_reason = Some(ExitReason::NeedRefreshPlaylist(f));
                            self.init_by_song(&song1);
                        }
                    }
                    self.update_status_line(StatusLine::Default);
                }
                TransferState::ErrDownload => {
                    self.mount_error("download failed");
                    self.update_status_line(StatusLine::Error);
                }
                TransferState::ErrEmbedData => {
                    self.mount_error("download ok but tag info is not complete.");
                    self.update_status_line(StatusLine::Error);
                }
            }
        };
    }

    pub fn update_lyric_options(&mut self) {
        if let Ok(SearchLyricState::Finish(l)) = self.receiver_songtag.try_recv() {
            self.add_lyric_options(l);
            self.redraw = true;
        }
    }
    pub fn update_status_line(&mut self, s: StatusLine) {
        match s {
            StatusLine::Default => {
                let text = "Press \"?\" for help.".to_string();

                if let Some(props) = self.view.get_props(COMPONENT_TE_LABEL_HELP) {
                    let props = label::LabelPropsBuilder::from(props)
                        .with_background(Color::Reset)
                        .with_foreground(Color::Cyan)
                        .with_text(text)
                        .build();

                    let msg = self.view.update(COMPONENT_TE_LABEL_HELP, props);
                    self.update(msg);
                    if let Some(song) = &self.song {
                        if let Some(file) = song.file() {
                            self.exit_reason =
                                Some(ExitReason::NeedRefreshPlaylist(file.to_string()));
                        }
                    }
                    self.redraw = true;
                }
            }
            StatusLine::Running => {
                let text = " Downloading...".to_string();

                if let Some(props) = self.view.get_props(COMPONENT_TE_LABEL_HELP) {
                    let props = label::LabelPropsBuilder::from(props)
                        .with_text(text)
                        .with_foreground(Color::White)
                        .with_background(Color::Red)
                        .build();

                    let msg = self.view.update(COMPONENT_TE_LABEL_HELP, props);
                    self.update(msg);
                    self.redraw = true;
                }
            }
            StatusLine::Success => {
                let text = " Download Success!".to_string();

                if let Some(props) = self.view.get_props(COMPONENT_TE_LABEL_HELP) {
                    let props = label::LabelPropsBuilder::from(props)
                        .with_text(text)
                        .with_foreground(Color::Black)
                        .with_background(Color::Green)
                        .build();

                    let msg = self.view.update(COMPONENT_TE_LABEL_HELP, props);
                    self.update(msg);
                    self.redraw = true;
                }
            }
            StatusLine::Error => {
                let text = " Download Error!".to_string();

                if let Some(props) = self.view.get_props(COMPONENT_TE_LABEL_HELP) {
                    let props = label::LabelPropsBuilder::from(props)
                        .with_text(text)
                        .with_foreground(Color::White)
                        .with_background(Color::Red)
                        .build();

                    let msg = self.view.update(COMPONENT_TE_LABEL_HELP, props);
                    self.update(msg);
                    self.redraw = true;
                }
            }
        }
    }
}
