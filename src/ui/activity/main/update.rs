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
use crate::invidious::InvidiousInstance;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tui_realm_treeview::TreeViewPropsBuilder;
use tuirealm::components::{label, paragraph, progress_bar};
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
                // yank
                (COMPONENT_TREEVIEW, &MSG_KEY_CHAR_Y) => {
                    self.yank();
                    None
                }
                // paste
                (COMPONENT_TREEVIEW, &MSG_KEY_CHAR_P) => {
                    if let Err(e) = self.paste(){
                        self.mount_error(e.to_string().as_ref());
                    }
                    None
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
                // seek
                (_, &MSG_KEY_CHAR_F) => match self.player.seek(5) {
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
                (_, &MSG_KEY_CHAR_B) => match self.player.seek(-5) {
                    Ok(_) => {
                        self.time_pos -= 5;
                        None
                    }
                    Err(_) => None,
                },
                // adjust lyric delay 
                (_, &MSG_KEY_CHAR_CAPITAL_F) => {
                   if let Some(song) = self.current_song.as_mut(){
                   if let Some(lyric) = song.parsed_lyric.as_mut() {
                       lyric.adjust_offset(self.time_pos,1000);
                       if let Some(song) = self.current_song.as_ref(){
                           if let Err(e) = song.save() {
                               self.mount_error(e.to_string().as_ref());
                           };
                       };
                   }
                }
                   None
                }
                ,
                // adjust lyric delay 
                (_, &MSG_KEY_CHAR_CAPITAL_B) =>  {
                    if let Some(song) = self.current_song.as_mut() {
                   if let Some(lyric) = song.parsed_lyric.as_mut() {
                       lyric.adjust_offset(self.time_pos,-1000);
                       if let Some(song) = self.current_song.as_ref(){
                           if let Err(e) = song.save() {
                               self.mount_error(e.to_string().as_ref());
                           };
                       };

                   }
                }

                    None
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
                (_, &MSG_KEY_SPACE) => {
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

                (COMPONENT_INPUT_URL, Msg::OnSubmit(Payload::One(Value::Str(url)))) => {
                        self.umount_youtube_url();
                        if url.starts_with("http") {
                            self.youtube_dl(&url);
                        } else {
                            self.mount_youtube_options();
                            self.youtube_options_url = url.clone();
                            let domain = self.config.invidious_instance.clone();
                            let mut inv = InvidiousInstance::new(domain);
                            match inv.get_search_query(&url,1) {
                                Ok(y) => {
                                    self.youtube_options = y;
                                    self.sync_youtube_options();
                                                },
                                Err(e) => self.mount_error(format!("search error: {}",e).as_str()),
                            }
                        }
                    None
                }

                (super::COMPONENT_SCROLLTABLE_YOUTUBE,&MSG_KEY_TAB) => {
                    let domain = self.config.invidious_instance.clone();
                    let mut inv = InvidiousInstance::new(domain);
                    self.youtube_options_index +=1;
                    match inv.get_search_query(self.youtube_options_url.as_str(),self.youtube_options_index) {
                        Ok(y) => {
                            self.youtube_options = y;
                            self.sync_youtube_options();
                                        },
                        Err(e) => self.mount_error(format!("search error: {}",e).as_str()),
                    }
                    None
                }

                (super::COMPONENT_SCROLLTABLE_YOUTUBE,&MSG_KEY_SHIFT_TAB) => {
                    let domain = self.config.invidious_instance.clone();
                    let mut inv = InvidiousInstance::new(domain);
                    if self.youtube_options_index >1 {
                    self.youtube_options_index -=1;
                    match inv.get_search_query(self.youtube_options_url.as_str(),self.youtube_options_index) {
                        Ok(y) => {
                            self.youtube_options = y;
                            self.sync_youtube_options();
                                        },
                        Err(e) => self.mount_error(format!("search error: {}",e).as_str()),
                    }}
                    None
                }

                (super::COMPONENT_SCROLLTABLE_YOUTUBE,&MSG_KEY_ESC) | (super::COMPONENT_SCROLLTABLE_YOUTUBE,&MSG_KEY_CHAR_CAPITAL_Q)  => {
                    self.umount_youtube_options();
                    None
                }

                (super::COMPONENT_SCROLLTABLE_YOUTUBE,&MSG_KEY_ENTER) => {
                    if let Some(Payload::One(Value::Usize(index))) = self.view.get_state(super::COMPONENT_SCROLLTABLE_YOUTUBE) {
                        // download from search result here
                        let mut url = "https://www.youtube.com/watch?v=".to_string();
                        url.push_str(self.youtube_options[index].video_id.as_str());
                        self.youtube_dl(url.as_ref());
                    }
                    self.umount_youtube_options();
                    None
                }
                (COMPONENT_INPUT_URL, &MSG_KEY_ESC) => {
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

        let song = match self.current_song.as_ref() {
            Some(song) => song,
            None => return,
        };

        let artist = song.artist().unwrap_or("Unknown Artist");

        if time_pos > self.time_pos || time_pos < 2 {
            self.time_pos = time_pos;
            let props = self.view.get_props(COMPONENT_PROGRESS).unwrap();
            let props = progress_bar::ProgressBarPropsBuilder::from(props)
                .with_progress(new_prog)
                .with_texts(
                    Some(format!("Playing: {} - {}", artist, song_title)),
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
        if self.queue_items.is_empty() {
            return;
        }

        if song.lyric_frames.is_empty() {
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
            Some(l) => {
                if l.unsynced_captions.is_empty() {
                    return;
                }
                l.get_text(time_pos).unwrap()
            }
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
        if song.picture.is_empty() {
            return;
        }

        // just show the first photo
        if let Ok(image) = image::load_from_memory(&song.picture[0].data) {
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
            viuer::print(&image, &config).expect("image printing failed.");
        };
    }

    pub fn update_download_progress(&mut self) {
        if let Ok(transfer_state) = self.receiver.try_recv() {
            match transfer_state {
                TransferState::Running => {
                    self.update_playlist_title();
                    let text = " Downloading...".to_string();

                    let props = label::LabelPropsBuilder::from(
                        self.view.get_props(super::COMPONENT_LABEL_HELP).unwrap(),
                    )
                    .with_text(text)
                    .with_foreground(tui::style::Color::White)
                    .with_background(tui::style::Color::Red)
                    .build();

                    let msg = self.view.update(super::COMPONENT_LABEL_HELP, props);
                    self.update(msg);
                }
                TransferState::Completed => {
                    self.refresh_playlist();
                    let props = label::LabelPropsBuilder::from(
                        self.view.get_props(super::COMPONENT_LABEL_HELP).unwrap(),
                    )
                    .with_background(tui::style::Color::Reset)
                    .with_foreground(tui::style::Color::Cyan)
                    .with_text(String::from("Press \"?\" for help."))
                    .build();

                    let msg = self.view.update(super::COMPONENT_LABEL_HELP, props);
                    self.update(msg);
                }
                TransferState::ErrDownload => {
                    self.mount_error("download failed");
                }
            }
        };
    }
}
