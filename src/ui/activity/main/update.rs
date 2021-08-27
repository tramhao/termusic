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
use super::{
    MainActivity, Status, COMPONENT_CONFIRMATION_INPUT, COMPONENT_CONFIRMATION_RADIO,
    COMPONENT_INPUT_URL, COMPONENT_PARAGRAPH_LYRIC, COMPONENT_PROGRESS, COMPONENT_TABLE,
    COMPONENT_TEXT_ERROR, COMPONENT_TEXT_HELP, COMPONENT_TREEVIEW,
};
use crate::song::Song;
use crate::ui::keymap::*;
use std::str::FromStr;
// ext
use super::{StatusLine, TransferState};
use crate::player::AudioPlayer;
use crate::songtag::lrc::Lyric;
use humantime::format_duration;
use std::path::{Path, PathBuf};
use std::thread;
use std::thread::sleep;
use std::time::Duration;
use tui_realm_stdlib::{LabelPropsBuilder, ParagraphPropsBuilder, ProgressBarPropsBuilder};
use tui_realm_treeview::TreeViewPropsBuilder;
use tuirealm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use tuirealm::props::TextSpan;
use tuirealm::tui::layout::Alignment;
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
                (COMPONENT_TREEVIEW,key) if key== &MSG_KEY_TAB => {
                    self.view.active(COMPONENT_TABLE);
                    None
                }
                (COMPONENT_TABLE, key) if key== &MSG_KEY_TAB => {
                    self.view.active(COMPONENT_TREEVIEW);
                    None
                }
                // yank
                (COMPONENT_TREEVIEW, key) if key== &MSG_KEY_CHAR_Y => {
                    self.yank();
                    None
                }
                // paste
                (COMPONENT_TREEVIEW, key) if key== &MSG_KEY_CHAR_P => {
                    if let Err(e) = self.paste(){
                        self.mount_error(e.to_string().as_ref());
                    }
                    None
                }

                (COMPONENT_TREEVIEW, Msg::OnSubmit(Payload::One(Value::Str(node_id)))) => {
                    // Update tree
                    self.scan_dir(PathBuf::from(node_id.as_str()).as_path());
                    // Update
                    if let Some(props) = self.view.get_props(COMPONENT_TREEVIEW) {
                        let props = TreeViewPropsBuilder::from(props)
                            .with_tree(self.tree.root())
                            .with_title(String::from(self.path.to_string_lossy()),Alignment::Left)
                            .build();
                        let msg = self.view.update(COMPONENT_TREEVIEW, props);
                        self.update(msg);
                    }
                    None
                }
                (COMPONENT_TREEVIEW, key) if key== &MSG_KEY_BACKSPACE => {
                    // Update tree
                    match self.upper_dir() {
                        None => None,
                        Some(p) => {
                            let p: PathBuf = p.to_path_buf();
                            self.scan_dir(p.as_path());
                            // Update
                            if let Some(props) = self.view.get_props(COMPONENT_TREEVIEW) {
                                let props = TreeViewPropsBuilder::from(props)
                                    .with_tree(self.tree.root())
                                    .with_title(String::from(self.path.to_string_lossy()),Alignment::Left)
                                    .build();
                                let msg = self.view.update(COMPONENT_TREEVIEW, props);
                                self.update(msg);
                            }
                            None
                        }
                    }
                }
                // seek
                (_, key) if key== &MSG_KEY_CHAR_F => match self.player.seek(5) {
                    Ok(_) =>{
                        self.time_pos += 5;
                        None
                    },
                    Err(_) => {
                        self.status = Some(Status::Stopped);
                        None
                    },
                },
                // seek backward
                (_, key) if key== &MSG_KEY_CHAR_B => match self.player.seek(-5) {
                    Ok(_) => {
                        self.time_pos -= 5;
                        None
                    }
                    Err(_) => {
                        self.status = Some(Status::Stopped);
                        None
                    }
                },
                // adjust lyric delay 
                (_, key) if key== &MSG_KEY_CHAR_CAPITAL_F => {
                   if let Some(song) = self.current_song.as_mut(){
                       if let Some(lyric) = song.parsed_lyric.as_mut() {
                           lyric.adjust_offset(self.time_pos,1000);
                           if let Some(text) = lyric.as_lrc_text() {
                                song.set_lyric(&text,"Adjusted");
                           }
                           if let Err(e) = song.save_tag() {
                               self.mount_error(e.to_string().as_ref());
                           };
                       }
                }
                   None
                }
                ,
                // adjust lyric delay 
                (_, key) if key== &MSG_KEY_CHAR_CAPITAL_B =>  {
                    if let Some(song) = self.current_song.as_mut() {
                        if let Some(lyric) = song.parsed_lyric.as_mut() {
                           lyric.adjust_offset(self.time_pos,-1000);
                           if let Some(text) = lyric.as_lrc_text() {
                                   song.set_lyric(&text,"Adjusted");
                           }
                           if let Err(e) = song.save_tag() {
                               self.mount_error(e.to_string().as_ref());
                           };
                        }
                }
                    None
                },


                (COMPONENT_TREEVIEW, key) if key== &MSG_KEY_CHAR_H => {
                    let event: Event = Event::Key(KeyEvent {
                        code: KeyCode::Left,
                        modifiers: KeyModifiers::NONE,
                    });
                    self.view.on(event);
                    None
                }
                (COMPONENT_TABLE, key) if key== &MSG_KEY_CHAR_G => {
                    let event: Event = Event::Key(KeyEvent {
                        code: KeyCode::Home,
                        modifiers: KeyModifiers::NONE,
                    });
                    self.view.on(event);
                    None
                }

                (COMPONENT_TABLE,key) if key==  &MSG_KEY_CHAR_CAPITAL_G => {
                    let event: Event = Event::Key(KeyEvent {
                        code: KeyCode::End,
                        modifiers: KeyModifiers::NONE,
                    });
                    self.view.on(event);
                    None
                }

                (_,key) if key==  &MSG_KEY_CHAR_J => {
                    let event: Event = Event::Key(KeyEvent {
                        code: KeyCode::Down,
                        modifiers: KeyModifiers::NONE,
                    });
                    self.view.on(event);
                    None
                }
                (_,key) if key==  &MSG_KEY_CHAR_K => {
                    let event: Event = Event::Key(KeyEvent {
                        code: KeyCode::Up,
                        modifiers: KeyModifiers::NONE,
                    });
                    self.view.on(event);
                    None
                }
                (COMPONENT_TREEVIEW,key) if key==  &MSG_KEY_CHAR_L => {
                    // Add selected song to queue
                    if let Some(Payload::One(Value::Str(node_id))) = self.view.get_state(COMPONENT_TREEVIEW) {
                            let p: &Path = Path::new(node_id.as_str());
                            if p.is_dir() {
                                let event: Event = Event::Key(KeyEvent {
                                    code: KeyCode::Right,
                                    modifiers: KeyModifiers::NONE,
                                });
                                self.view.on(event);
                            } else {
                                match Song::from_str(&p.to_string_lossy()) {
                                    Ok(s) => self.add_queue(s),
                                    Err(e) => self.mount_error(&e.to_string()),
                                };
                            }
                    }
                    None
                }
                (COMPONENT_TREEVIEW,key) if key==  &MSG_KEY_CHAR_CAPITAL_L => {
                    // Add all songs in a folder to queue
                    if let Some(Payload::One(Value::Str(node_id))) = self.view.get_state(COMPONENT_TREEVIEW) {
                            let p: &Path = Path::new(node_id.as_str());
                            if p.is_dir() {
                                // let p = p.to_string_lossy();
                                let new_items = Self::dir_children(p);
                                for i in new_items.iter().rev() {
                                    match Song::from_str(i) {
                                        Ok(s) => self.add_queue(s),
                                        Err(e) => self.mount_error(format!("add queue error: {}",e).as_str()),
                                    };
                                }
                            }
                    }
                            None
                }

                (COMPONENT_TABLE,key) if key==  &MSG_KEY_CHAR_L => {
                    if let Some(Payload::One(Value::Usize(index))) = self.view.get_state(COMPONENT_TABLE) {
                        self.time_pos = 0;
                        if let Some(song) = self.queue_items.get(index) {
                            if let Some(file) = &song.file {
                                self.player.queue_and_play(file);
                            }
                            self.current_song = Some(song.to_owned());
                        }
                        self.update_photo();
                    }
                    None
                }

                (COMPONENT_TABLE,key) if key==  &MSG_KEY_CHAR_D => {
                    match self.view.get_state(COMPONENT_TABLE) {
                        Some(Payload::One(Value::Usize(index))) => {
                            self.delete_item(index);
                            None
                        }
                        _ => None,
                    }
                }

                (COMPONENT_TABLE,key) if key==  &MSG_KEY_CHAR_CAPITAL_D => {
                    self.empty_queue();
                    None
                }

                // Toggle pause
                (_,key) if key==  &MSG_KEY_SPACE => {
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
                (_,key) if key==  &MSG_KEY_CHAR_N => {
                    self.status = Some(Status::Stopped);
                    None
                }
                // shuffle
                (COMPONENT_TABLE,key) if key==  &MSG_KEY_CHAR_S => {
                    self.shuffle();
                    None
                }

                // start download
                (COMPONENT_TREEVIEW,key) if key==  &MSG_KEY_CHAR_S => {
                    self.mount_youtube_url();
                    None
                }

                (COMPONENT_TREEVIEW,key) if key==  &MSG_KEY_CHAR_D => {
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

                (COMPONENT_INPUT_URL, Msg::OnSubmit(Payload::One(Value::Str(url)))) => {
                        self.umount_youtube_url();
                        if url.starts_with("http") {
                            match self.youtube_dl(url) {
                                Ok(_) => {}
                                Err(e) => self.mount_error(format!("add queue error: {}",e).as_str()),
                            }
                        } else {
                            self.mount_youtube_options();
                            self.youtube_options_search(url);
                        }
                    None
                }

                (super::COMPONENT_SCROLLTABLE_YOUTUBE,key) if key== &MSG_KEY_TAB => {
                    self.youtube_options_next_page();
                    None
                }

                (super::COMPONENT_SCROLLTABLE_YOUTUBE,key) if key== &MSG_KEY_SHIFT_TAB => {
                    self.youtube_options_prev_page();
                    None
                }

                (super::COMPONENT_SCROLLTABLE_YOUTUBE,key) if (key== &MSG_KEY_ESC) | (key == &MSG_KEY_CHAR_CAPITAL_Q)  => {
                    self.umount_youtube_options();
                    None
                }

                (super::COMPONENT_SCROLLTABLE_YOUTUBE,key) if key== &MSG_KEY_ENTER => {
                    if let Some(Payload::One(Value::Usize(index))) = self.view.get_state(super::COMPONENT_SCROLLTABLE_YOUTUBE) {
                        // download from search result here
                        if let Err(e) = self.youtube_options_download(index) {
                            self.mount_error(format!("download song error: {}",e).as_str());
                        }
                    }
                    self.umount_youtube_options();
                    None
                }
                (COMPONENT_INPUT_URL,key) if key==  &MSG_KEY_ESC => {
                    self.umount_youtube_url();
                    None
                }

                (COMPONENT_CONFIRMATION_INPUT, Msg::OnSubmit(_)) => {
                    if let Some(Payload::One(Value::Str(p))) =
                        self.view.get_state(COMPONENT_CONFIRMATION_INPUT)
                    {
                        self.umount_confirmation_input();
                        if p == "DELETE" {
                            if let Err(e) =self.delete_songs() {
                                self.mount_error(format!("delete song error: {}",e).as_str());
                            };
                        }
                    }
                    None
                }

                (COMPONENT_CONFIRMATION_INPUT,key) if key==  &MSG_KEY_ESC => {
                    self.umount_confirmation_input();
                    None
                }
                (_,key) if key==  &MSG_KEY_CHAR_H => {
                    let event: Event = Event::Key(KeyEvent {
                        code: KeyCode::Left,
                        modifiers: KeyModifiers::NONE,
                    });
                    self.view.on(event);
                    None
                }
                (_,key) if key==  &MSG_KEY_CHAR_L => {
                    let event: Event = Event::Key(KeyEvent {
                        code: KeyCode::Right,
                        modifiers: KeyModifiers::NONE,
                    });
                    self.view.on(event);
                    None
                }

                (COMPONENT_CONFIRMATION_RADIO, Msg::OnSubmit(_)) => {
                    if let Some(Payload::One(Value::Usize(index))) =
                        self.view.get_state(COMPONENT_CONFIRMATION_RADIO)
                    {
                        self.umount_confirmation_radio();

                        if index != 0 {
                            return None;
                        }
                        if let Err(e) =self.delete_song() {
                                self.mount_error(format!("delete song error: {}",e).as_str());
                        };
                    }
                    None
                }

                // switch lyrics
                (_,key) if key == &MSG_KEY_CHAR_CAPITAL_T => {
                    if let Some(mut song) = self.current_song.clone() {
                        if song.lyric_frames.is_empty() {
                            return None
                        }
                        song.lyric_selected +=1;
                        if song.lyric_selected >= song.lyric_frames.len()  {
                            song.lyric_selected = 0;
                        }
                        if let Some(f) = song.lyric_frames.get(song.lyric_selected) {
                            if let Ok(parsed_lyric) = Lyric::from_str(&f.text) {
                                let tx = self.sender_message.clone();
                                song.parsed_lyric = Some(parsed_lyric);
                                let lang_ext = f.description.clone();
                                self.current_song = Some(song);
                                thread::spawn(move || {
                                    let _ = tx.send(super::MessageState::Show(("Lyric switch successful".to_string(),format!("{} lyric is showing",lang_ext))));
                                    sleep(Duration::from_secs(5));
                                    let _ = tx.send(super::MessageState::Hide);
                                });
                            }
                        }
                    }
                    None
                }


                (COMPONENT_CONFIRMATION_RADIO,key) if key==  &MSG_KEY_ESC => {
                    self.umount_confirmation_radio();
                    None
                }

                // increase volume
                (_,key) if (key==  &MSG_KEY_CHAR_PLUS) | (key == &MSG_KEY_CHAR_EQUAL) => {
                    self.player.volume_up();
                    None
                }
                // decrease volume
                (_,key) if (key==  &MSG_KEY_CHAR_MINUS) | (key == &MSG_KEY_CHAR_DASH) => {
                    self.player.volume_down();
                    None
                }

                (_,key) if key==  &MSG_KEY_QUESTION_MARK => {
                    // Show help
                    self.mount_help();
                    None
                }
                // -- help
                (COMPONENT_TEXT_HELP,key) if (key==  &MSG_KEY_ENTER) | (key == &MSG_KEY_ESC) | (key == &MSG_KEY_CHAR_CAPITAL_Q) => {
                    self.umount_help();
                    None
                }
                // -- error
                (COMPONENT_TEXT_ERROR,key) if (key==  &MSG_KEY_ESC) | (key == &MSG_KEY_ENTER) | (key == &MSG_KEY_CHAR_CAPITAL_Q)=> {
                    self.umount_error();
                    None
                }

                (COMPONENT_TREEVIEW,key) if key==  &MSG_KEY_CHAR_T => {
                    self.run_tageditor();
                    None
                }

                // Refresh playlist
                (_,key) if key==  &MSG_KEY_CHAR_R => {
                    self.sync_playlist(None);
                    None
                }

                (_,key) if (key==  &MSG_KEY_ESC) | (key == &MSG_KEY_CHAR_CAPITAL_Q) => {
                    // Quit on esc
                    self.exit_reason = Some(super::ExitReason::Quit);
                    None
                }
                _ => None,
            },
        }
    }

    pub fn update_progress(&mut self) {
        let (new_prog, time_pos, duration) = self.player.get_progress();
        if (new_prog, time_pos, duration) == (0.9, 0, 100) {
            return;
        }

        if time_pos >= duration {
            self.status = Some(Status::Stopped);
            return;
        }

        let song = match self.current_song.clone() {
            Some(song) => song,
            None => return,
        };

        let artist = song.artist().unwrap_or("Unknown Artist");
        let title = song.title().unwrap_or("Unknown Title");

        if time_pos > self.time_pos || time_pos < 2 {
            self.time_pos = time_pos;
            if let Some(props) = self.view.get_props(COMPONENT_PROGRESS) {
                let props = ProgressBarPropsBuilder::from(props)
                    .with_progress(new_prog)
                    .with_title(
                        format!("Playing: {} - {}", artist, title),
                        Alignment::Center,
                    )
                    .with_label(format!(
                        "{}     :     {} ",
                        format_duration(Duration::from_secs(time_pos as u64)),
                        format_duration(Duration::from_secs(duration as u64))
                    ))
                    .build();
                self.view.update(COMPONENT_PROGRESS, props);
                self.redraw = true;
                self.view();
                self.redraw = false;
            }
        }

        // Update lyrics
        if self.queue_items.is_empty() {
            return;
        }

        if song.lyric_frames.is_empty() {
            if let Some(props) = self.view.get_props(COMPONENT_PARAGRAPH_LYRIC) {
                let props = ParagraphPropsBuilder::from(props)
                    .with_texts(vec![TextSpan::new("No lyrics available.")])
                    .build();
                self.view.update(COMPONENT_PARAGRAPH_LYRIC, props);
                return;
            }
        }

        let mut line = String::new();
        if let Some(l) = song.parsed_lyric.as_ref() {
            if l.unsynced_captions.is_empty() {
                return;
            }
            if let Some(l) = l.get_text(time_pos) {
                line = l;
            }
        }

        if let Some(props) = self.view.get_props(COMPONENT_PARAGRAPH_LYRIC) {
            let props = ParagraphPropsBuilder::from(props)
                .with_texts(vec![TextSpan::new(line)])
                .build();
            self.view.update(COMPONENT_PARAGRAPH_LYRIC, props);
        }
    }

    // update picture of album
    pub fn update_photo(&mut self) {
        // if terminal is not kitty, just don't show photo
        if viuer::KittySupport::Local != viuer::get_kitty_support() {
            return;
        };

        let song = match &self.current_song {
            Some(song) => song,
            None => return,
        };

        // clear all previous image
        match self.context.as_mut() {
            Some(c) => c.clear_image(),
            None => return,
        }
        // if no photo, just return
        if song.picture.is_empty() {
            return;
        }

        // just show the first photo
        if let Some(picture) = song.picture.get(0) {
            if let Ok(image) = image::load_from_memory(&picture.data) {
                let (term_width, term_height) = viuer::terminal_size();
                // Set desired image dimensions
                let (orig_width, orig_height) = image::GenericImageView::dimensions(&image);
                let ratio = orig_height as f64 / orig_width as f64;
                let width = 20_u16;
                let height = (width as f64 * ratio) as u16;
                let config = viuer::Config {
                    transparent: true,
                    absolute_offset: true,
                    x: term_width - width - 1,
                    y: (term_height - height / 2 - 8) as i16 - 1,
                    // x: term_width / 3 - width - 1,
                    // y: (term_height - height / 2) as i16 - 2,
                    width: Some(width as u32),
                    height: None,
                    ..Default::default()
                };
                let _ = viuer::print(&image, &config);
            }
        }
    }

    // change status bar text to indicate the downloading state
    pub fn update_download_progress(&mut self) {
        if let Ok(transfer_state) = self.receiver.try_recv() {
            match transfer_state {
                TransferState::Running => {
                    self.update_status_line(StatusLine::Running);
                }
                TransferState::Success => {
                    self.update_status_line(StatusLine::Success);
                }
                TransferState::Completed(Some(file)) => {
                    self.sync_playlist(Some(file.as_str()));
                    self.update_status_line(StatusLine::Default);
                }
                TransferState::Completed(None) => {
                    self.sync_playlist(None);
                    self.update_status_line(StatusLine::Default);
                }
                TransferState::ErrDownload => {
                    self.mount_error("download failed");
                    self.update_status_line(StatusLine::Error);
                }
                TransferState::ErrEmbedData => {
                    // This case will not happen in main activity
                }
            }
        };
    }

    // change status bar text to indicate the downloading state
    pub fn update_status_line(&mut self, s: StatusLine) {
        match s {
            StatusLine::Default => {
                let text = "Press \"?\" for help.".to_string();
                if let Some(props) = self.view.get_props(super::COMPONENT_LABEL_HELP) {
                    let props = LabelPropsBuilder::from(props)
                        .with_text(text)
                        .with_background(tuirealm::tui::style::Color::Reset)
                        .with_foreground(tuirealm::tui::style::Color::Cyan)
                        .build();

                    let msg = self.view.update(super::COMPONENT_LABEL_HELP, props);
                    self.update(msg);
                    self.redraw = true;
                }
            }
            StatusLine::Running => {
                let text = " Downloading...".to_string();

                if let Some(props) = self.view.get_props(super::COMPONENT_LABEL_HELP) {
                    let props = LabelPropsBuilder::from(props)
                        .with_text(text)
                        .with_foreground(tuirealm::tui::style::Color::White)
                        .with_background(tuirealm::tui::style::Color::Red)
                        .build();

                    let msg = self.view.update(super::COMPONENT_LABEL_HELP, props);
                    self.update(msg);
                    self.redraw = true;
                }
            }
            StatusLine::Success => {
                let text = " Download Success!".to_string();

                if let Some(props) = self.view.get_props(super::COMPONENT_LABEL_HELP) {
                    let props = LabelPropsBuilder::from(props)
                        .with_text(text)
                        .with_foreground(tuirealm::tui::style::Color::Black)
                        .with_background(tuirealm::tui::style::Color::Green)
                        .build();

                    let msg = self.view.update(super::COMPONENT_LABEL_HELP, props);
                    self.update(msg);
                    self.redraw = true;
                }
            }
            StatusLine::Error => {
                let text = " Download Error!".to_string();

                if let Some(props) = self.view.get_props(super::COMPONENT_LABEL_HELP) {
                    let props = LabelPropsBuilder::from(props)
                        .with_text(text)
                        .with_foreground(tuirealm::tui::style::Color::White)
                        .with_background(tuirealm::tui::style::Color::Red)
                        .build();

                    let msg = self.view.update(super::COMPONENT_LABEL_HELP, props);
                    self.update(msg);
                    self.redraw = true;
                }
            }
        }
    }

    // update message box
    pub fn update_message(&mut self) {
        if let Ok(message_state) = self.receiver_message.try_recv() {
            match message_state {
                super::MessageState::Show((title, text)) => {
                    self.mount_message(&title, &text);
                }
                super::MessageState::Hide => {
                    self.umount_message();
                }
            }
        }
    }
}
