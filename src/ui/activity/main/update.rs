/*
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
use super::{ExitReason, Id, Model, Msg, Status, StatusLine, TermusicActivity, UpdateComponents};
use crate::ui::activity::Loop;
use crate::{
    song::Song,
    ui::keymap::{
        MSG_KEY_BACKSPACE, MSG_KEY_CHAR_B, MSG_KEY_CHAR_CAPITAL_B, MSG_KEY_CHAR_CAPITAL_D,
        MSG_KEY_CHAR_CAPITAL_F, MSG_KEY_CHAR_CAPITAL_G, MSG_KEY_CHAR_CAPITAL_L,
        MSG_KEY_CHAR_CAPITAL_N, MSG_KEY_CHAR_CAPITAL_Q, MSG_KEY_CHAR_CAPITAL_T, MSG_KEY_CHAR_D,
        MSG_KEY_CHAR_DASH, MSG_KEY_CHAR_EQUAL, MSG_KEY_CHAR_F, MSG_KEY_CHAR_G, MSG_KEY_CHAR_H,
        MSG_KEY_CHAR_J, MSG_KEY_CHAR_K, MSG_KEY_CHAR_L, MSG_KEY_CHAR_M, MSG_KEY_CHAR_MINUS,
        MSG_KEY_CHAR_N, MSG_KEY_CHAR_P, MSG_KEY_CHAR_PLUS, MSG_KEY_CHAR_R, MSG_KEY_CHAR_S,
        MSG_KEY_CHAR_T, MSG_KEY_CHAR_Y, MSG_KEY_CTRL_H, MSG_KEY_ENTER, MSG_KEY_ESC,
        MSG_KEY_SHIFT_TAB, MSG_KEY_SLASH, MSG_KEY_SPACE, MSG_KEY_TAB,
    },
};
use humantime::format_duration;
use if_chain::if_chain;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::thread::{self, sleep};
use std::time::Duration;
use tuirealm::props::Alignment;
use tuirealm::{
    event::{Event, KeyEvent, KeyModifiers},
    props::TextSpan,
};
use tuirealm::{NoUserEvent, Update};

impl Update<Id, Msg, NoUserEvent> for Model {
    /// ### update
    ///
    /// Update auth activity model based on msg
    /// The function exits when returns None
    fn update(&mut self, view: &mut View<Id, Msg, NoUserEvent>, msg: Option<Msg>) -> Option<Msg> {
        match msg.unwrap_or(Msg::None) {
            Msg::AppClose => {
                self.quit = true;
                None
            }
            Msg::None => None,
            // (COMPONENT_TREEVIEW_LIBRARY, key) => {
            //     self.update_on_library(key);
            //     None
            // }
            // (COMPONENT_TABLE_PLAYLIST, key) => {
            //     self.update_on_playlist(key);
            //     None
            // }

            // (COMPONENT_INPUT_URL, Msg::OnSubmit(Payload::One(Value::Str(url)))) => {
            //     self.update_search_or_download(url);
            //     None
            // }

            // (COMPONENT_TABLE_YOUTUBE, key) => {
            //     self.update_table_youtube(key);
            //     None
            // }
            // (COMPONENT_INPUT_URL, key) if key == &MSG_KEY_ESC => {
            //     self.umount_youtube_url();
            //     None
            // }

            // (COMPONENT_CONFIRMATION_INPUT, Msg::OnSubmit(_)) => {
            //     self.update_delete_songs();
            //     None
            // }

            // (COMPONENT_CONFIRMATION_INPUT, key) if key == &MSG_KEY_ESC => {
            //     self.umount_confirmation_input();
            //     None
            // }

            // (COMPONENT_CONFIRMATION_RADIO, Msg::OnSubmit(_)) => {
            //     self.update_delete_song();
            //     None
            // }

            // (COMPONENT_CONFIRMATION_RADIO, key) if key == &MSG_KEY_ESC => {
            //     self.umount_confirmation_radio();
            //     None
            // }

            // // -- help
            // (COMPONENT_TEXT_HELP, key)
            //     if (key == &MSG_KEY_ENTER)
            //         | (key == &MSG_KEY_ESC)
            //         | (key == &MSG_KEY_CHAR_CAPITAL_Q) =>
            // {
            //     self.umount_help();
            //     None
            // }
            // // -- error
            // (COMPONENT_TEXT_ERROR, key)
            //     if (key == &MSG_KEY_ESC)
            //         | (key == &MSG_KEY_ENTER)
            //         | (key == &MSG_KEY_CHAR_CAPITAL_Q) =>
            // {
            //     self.umount_error();
            //     None
            // }

            // (COMPONENT_INPUT_SEARCH_LIBRARY, key)
            //     if (key == &MSG_KEY_ESC) | (key == &MSG_KEY_CHAR_CAPITAL_Q) =>
            // {
            //     self.umount_search_library();
            //     None
            // }

            // (COMPONENT_INPUT_SEARCH_LIBRARY, Msg::OnChange(Payload::One(Value::Str(input)))) => {
            //     // Update span
            //     self.update_search_library(input);
            //     None
            // }

            // (COMPONENT_INPUT_SEARCH_LIBRARY, Msg::OnSubmit(Payload::One(Value::Str(_)))) => {
            //     self.view.active(COMPONENT_TABLE_SEARCH_LIBRARY);
            //     None
            // }

            // (COMPONENT_INPUT_SEARCH_LIBRARY, key) if (key == &MSG_KEY_TAB) => {
            //     self.view.active(COMPONENT_TABLE_SEARCH_LIBRARY);
            //     None
            // }

            // (COMPONENT_TABLE_SEARCH_LIBRARY, key) => {
            //     self.update_on_table_search_library(key);
            //     None
            // }

            // (_, key) => {
            //     self.update_on_global_key(key);
            //     None
            // }
        }
    }
}

impl TermusicActivity {
    pub fn update_progress_title(&mut self) {
        if_chain! {
            if let Some(song) = &self.current_song;
            let artist = song.artist().unwrap_or("Unknown Artist");
            let title = song.title().unwrap_or("Unknown Title");
            if let Some(props) = self.view.get_props(COMPONENT_PROGRESS);
            then {
                let props = ProgressBarPropsBuilder::from(props)
                    // .with_progress(new_prog)
                    .with_title(
                        format!(
                            "Playing: {:^.20} - {:^.20} | Volume: {}",
                            artist, title, self.config.volume
                        ),
                        Alignment::Center,
                    )
                    .build();
                let msg = self.view.update(COMPONENT_PROGRESS, props);
                // self.redraw = true;
                self.update(&msg);
            }
        }
    }

    pub fn update_duration(&mut self) {
        let (_new_prog, _time_pos, duration) = self.player.get_progress();
        if let Some(song) = &mut self.current_song {
            let diff = song.duration().as_secs().checked_sub(duration as u64);
            if let Some(d) = diff {
                if d > 1 {
                    let _drop = song.update_duration();
                }
            } else {
                let _drop = song.update_duration();
            }
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
            Some(s) => s,
            None => return,
        };

        // if time_pos > self.time_pos || time_pos < 2 {
        self.time_pos = time_pos;
        if let Some(props) = self.view.get_props(COMPONENT_PROGRESS) {
            let props = ProgressBarPropsBuilder::from(props)
                .with_progress(new_prog)
                .with_label(format!(
                    "{}     :     {} ",
                    format_duration(Duration::from_secs(time_pos as u64)),
                    format_duration(Duration::from_secs(duration as u64))
                ))
                .build();
            let msg = self.view.update(COMPONENT_PROGRESS, props);
            self.redraw = true;
            self.update(&msg);
        }
        // }

        // Update lyrics
        if self.playlist_items.is_empty() {
            return;
        }

        if song.lyric_frames_is_empty() {
            if let Some(props) = self.view.get_props(COMPONENT_PARAGRAPH_LYRIC) {
                let props = ParagraphPropsBuilder::from(props)
                    .with_texts(vec![TextSpan::new("No lyrics available.")])
                    .build();
                self.view.update(COMPONENT_PARAGRAPH_LYRIC, props);
                return;
            }
        }

        let mut line = String::new();
        if let Some(l) = song.parsed_lyric() {
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

    // change status bar text to indicate the downloading state
    pub fn update_components(&mut self) {
        if let Ok(update_components_state) = self.receiver.try_recv() {
            match update_components_state {
                UpdateComponents::DownloadRunning => {
                    self.update_status_line(StatusLine::Running);
                }
                UpdateComponents::DownloadSuccess => {
                    self.update_status_line(StatusLine::Success);
                }
                UpdateComponents::DownloadCompleted(Some(file)) => {
                    self.sync_library(Some(file.as_str()));
                    self.update_status_line(StatusLine::Default);
                }
                UpdateComponents::DownloadCompleted(None) => {
                    self.sync_library(None);
                    self.update_status_line(StatusLine::Default);
                }
                UpdateComponents::DownloadErrDownload => {
                    self.mount_error("download failed");
                    self.update_status_line(StatusLine::Error);
                }
                UpdateComponents::DownloadErrEmbedData => {
                    // This case will not happen in main activity
                }
                UpdateComponents::YoutubeSearchSuccess(y) => {
                    self.youtube_options = y;
                    self.sync_youtube_options();
                    self.redraw = true;
                }
                UpdateComponents::YoutubeSearchFail(e) => {
                    self.mount_error(&e);
                }
                UpdateComponents::MessageShow((title, text)) => {
                    self.mount_message(&title, &text);
                }
                UpdateComponents::MessageHide((title, text)) => {
                    self.umount_message(&title, &text);
                }
            }
        };
    }

    // change status bar text to indicate the downloading state
    pub fn update_status_line(&mut self, s: StatusLine) {
        match s {
            StatusLine::Default => {
                let text = format!("Press <CTRL+H> for help. Version: {}", crate::VERSION);
                if let Some(props) = self.view.get_props(COMPONENT_LABEL_HELP) {
                    let props = LabelPropsBuilder::from(props)
                        .with_text(text)
                        .with_background(tuirealm::tui::style::Color::Reset)
                        .with_foreground(tuirealm::tui::style::Color::Cyan)
                        .build();

                    let msg = self.view.update(COMPONENT_LABEL_HELP, props);
                    self.update(&msg);
                    self.redraw = true;
                }
            }
            StatusLine::Running => {
                let text = " Downloading...".to_string();

                if let Some(props) = self.view.get_props(COMPONENT_LABEL_HELP) {
                    let props = LabelPropsBuilder::from(props)
                        .with_text(text)
                        .with_foreground(tuirealm::tui::style::Color::White)
                        .with_background(tuirealm::tui::style::Color::Red)
                        .build();

                    let msg = self.view.update(COMPONENT_LABEL_HELP, props);
                    self.update(&msg);
                    self.redraw = true;
                }
            }
            StatusLine::Success => {
                let text = " Download Success!".to_string();

                if let Some(props) = self.view.get_props(COMPONENT_LABEL_HELP) {
                    let props = LabelPropsBuilder::from(props)
                        .with_text(text)
                        .with_foreground(tuirealm::tui::style::Color::Black)
                        .with_background(tuirealm::tui::style::Color::Green)
                        .build();

                    let msg = self.view.update(COMPONENT_LABEL_HELP, props);
                    self.update(&msg);
                    self.redraw = true;
                }
            }
            StatusLine::Error => {
                let text = " Download Error!".to_string();

                if let Some(props) = self.view.get_props(COMPONENT_LABEL_HELP) {
                    let props = LabelPropsBuilder::from(props)
                        .with_text(text)
                        .with_foreground(tuirealm::tui::style::Color::White)
                        .with_background(tuirealm::tui::style::Color::Red)
                        .build();

                    let msg = self.view.update(COMPONENT_LABEL_HELP, props);
                    self.update(&msg);
                    self.redraw = true;
                }
            }
        }
    }

    pub fn update_playing_song(&self) {
        if let Some(song) = &self.current_song {
            let name = song.name().unwrap_or("Unknown Song");
            self.show_message_timeout("Current Playing", name, None);
        }
    }

    pub fn next_song(&mut self) {
        if self.playlist_items.is_empty() {
            return;
        }
        if let Some(song) = self.playlist_items.pop_front() {
            if let Some(file) = song.file() {
                self.player.add_and_play(file);
            }
            match self.config.loop_mode {
                Loop::Playlist => self.playlist_items.push_back(song.clone()),
                Loop::Single => self.playlist_items.push_front(song.clone()),
                Loop::Queue => {}
            }
            self.current_song = Some(song);
            self.sync_playlist();
            self.update_photo();
            self.update_progress_title();
            self.update_duration();
            self.update_playing_song();
        }
    }

    pub fn previous_song(&mut self) {
        if let Loop::Single | Loop::Queue = self.config.loop_mode {
            return;
        }

        if self.playlist_items.is_empty() {
            return;
        }

        if let Some(song) = self.playlist_items.pop_back() {
            self.playlist_items.push_front(song);
        }
        if let Some(song) = self.playlist_items.pop_back() {
            self.playlist_items.push_front(song);
        }
        self.next_song();
    }

    pub fn cycle_loop_mode(&mut self) {
        match self.config.loop_mode {
            Loop::Queue => {
                self.config.loop_mode = Loop::Playlist;
            }
            Loop::Playlist => {
                self.config.loop_mode = Loop::Single;
                if let Some(song) = self.playlist_items.pop_back() {
                    self.playlist_items.push_front(song);
                }
            }
            Loop::Single => {
                self.config.loop_mode = Loop::Queue;
                if let Some(song) = self.playlist_items.pop_front() {
                    self.playlist_items.push_back(song);
                }
            }
        };
        self.sync_playlist();
        self.update_title_playlist();
    }

    pub fn cycle_lyrics(&mut self) {
        if let Some(mut song) = self.current_song.clone() {
            if let Ok(f) = song.cycle_lyrics() {
                let lang_ext = f.description.clone();
                self.current_song = Some(song);
                self.show_message_timeout(
                    "Lyric switch successful",
                    format!("{} lyric is showing", lang_ext).as_str(),
                    None,
                );
            }
        }
    }

    pub fn show_message_timeout(&self, title: &str, text: &str, time_out: Option<u64>) {
        let tx = self.sender.clone();
        let title_string = title.to_string();
        let text_string = text.to_string();
        let time_out = time_out.unwrap_or(5);

        thread::spawn(move || {
            tx.send(UpdateComponents::MessageShow((
                title_string.clone(),
                text_string.clone(),
            )))
            .ok();
            sleep(Duration::from_secs(time_out));
            tx.send(UpdateComponents::MessageHide((title_string, text_string)))
                .ok();
        });
    }

    pub fn adjust_lyric_delay(&mut self, offset: i64) {
        if let Some(song) = self.current_song.as_mut() {
            if let Err(e) = song.adjust_lyric_delay(self.time_pos, offset) {
                self.mount_error(e.to_string().as_ref());
            };
        }
    }

    fn update_library_stepinto(&mut self, node_id: &str) {
        self.scan_dir(PathBuf::from(node_id).as_path());
        self.config.music_dir = node_id.to_string();
        // Update
        if let Some(props) = self.view.get_props(COMPONENT_TREEVIEW_LIBRARY) {
            let props = TreeViewPropsBuilder::from(props)
                .with_tree(self.tree.root())
                .with_title(String::from(self.path.to_string_lossy()), Alignment::Left)
                .build();
            let msg = self.view.update(COMPONENT_TREEVIEW_LIBRARY, props);
            self.update(&msg);
        }
    }

    fn update_library_stepout(&mut self) {
        if let Some(p) = self.upper_dir() {
            let p: PathBuf = p.to_path_buf();
            self.scan_dir(p.as_path());
            // Update
            if let Some(props) = self.view.get_props(COMPONENT_TREEVIEW_LIBRARY) {
                let props = TreeViewPropsBuilder::from(props)
                    .with_tree(self.tree.root())
                    .with_title(String::from(self.path.to_string_lossy()), Alignment::Left)
                    .build();
                let msg = self.view.update(COMPONENT_TREEVIEW_LIBRARY, props);
                self.update(&msg);
            }
        }
    }

    fn seek(&mut self, offset: i64) {
        self.player.seek(offset).ok();
        self.update_progress();
    }

    fn update_add_song_playlist(&mut self) {
        if let Some(Payload::One(Value::Str(node_id))) =
            self.view.get_state(COMPONENT_TREEVIEW_LIBRARY)
        {
            let p: &Path = Path::new(node_id.as_str());
            if p.is_dir() {
                let event: Event = Event::Key(KeyEvent {
                    code: KeyCode::Right,
                    modifiers: KeyModifiers::NONE,
                });
                self.view.on(event);
            } else {
                match Song::from_str(&p.to_string_lossy()) {
                    Ok(s) => self.add_playlist(s),
                    Err(e) => self.mount_error(&e.to_string()),
                };
            }
        }
    }

    fn update_add_songs_playlist(&mut self) {
        if let Some(Payload::One(Value::Str(node_id))) =
            self.view.get_state(COMPONENT_TREEVIEW_LIBRARY)
        {
            let p: &Path = Path::new(node_id.as_str());
            if p.is_dir() {
                let new_items = Self::dir_children(p);
                for i in &new_items {
                    match Song::from_str(i) {
                        Ok(s) => self.add_playlist(s),
                        Err(e) => self.mount_error(format!("add playlist error: {}", e).as_str()),
                    };
                }
            }
        }
    }

    fn update_play_selected(&mut self) {
        if let Some(Payload::One(Value::Usize(index))) =
            self.view.get_state(COMPONENT_TABLE_PLAYLIST)
        {
            self.time_pos = 0;
            if let Some(song) = self.playlist_items.remove(index) {
                self.playlist_items.push_front(song);
                self.sync_playlist();
                self.status = Some(Status::Stopped);
            }
        }
    }

    fn play_pause(&mut self) {
        if self.player.is_paused() {
            self.status = Some(Status::Running);
            self.player.resume();
        } else {
            self.status = Some(Status::Paused);
            self.player.pause();
        }
    }

    fn update_on_global_key(&mut self, key: &Msg) {
        match key {
            // seek
            key if key == &MSG_KEY_CHAR_F => self.seek(5),

            // seek backward
            key if key == &MSG_KEY_CHAR_B => self.seek(-5),

            // adjust lyric delay
            key if key == &MSG_KEY_CHAR_CAPITAL_F => self.adjust_lyric_delay(1000),

            // adjust lyric delay
            key if key == &MSG_KEY_CHAR_CAPITAL_B => self.adjust_lyric_delay(-1000),

            key if key == &MSG_KEY_CHAR_J => {
                let event: Event = Event::Key(KeyEvent {
                    code: KeyCode::Down,
                    modifiers: KeyModifiers::NONE,
                });
                self.view.on(event);
            }
            key if key == &MSG_KEY_CHAR_K => {
                let event: Event = Event::Key(KeyEvent {
                    code: KeyCode::Up,
                    modifiers: KeyModifiers::NONE,
                });
                self.view.on(event);
            }
            // Toggle pause
            key if key == &MSG_KEY_SPACE => self.play_pause(),

            // Toggle skip
            key if key == &MSG_KEY_CHAR_N => self.next_song(),

            key if key == &MSG_KEY_CHAR_CAPITAL_N => self.previous_song(),
            // switch lyrics
            key if key == &MSG_KEY_CHAR_CAPITAL_T => self.cycle_lyrics(),
            // increase volume
            key if (key == &MSG_KEY_CHAR_PLUS) | (key == &MSG_KEY_CHAR_EQUAL) => {
                self.player.volume_up();
                self.config.volume = self.player.volume();
                self.update_progress_title();
            }
            // decrease volume
            key if (key == &MSG_KEY_CHAR_MINUS) | (key == &MSG_KEY_CHAR_DASH) => {
                self.player.volume_down();
                self.config.volume = self.player.volume();
                self.update_progress_title();
            }

            key if key == &MSG_KEY_CTRL_H => self.mount_help(),

            key if key == &MSG_KEY_CHAR_H => {
                let event: Event = Event::Key(KeyEvent {
                    code: KeyCode::Left,
                    modifiers: KeyModifiers::NONE,
                });
                self.view.on(event);
            }

            key if key == &MSG_KEY_CHAR_L => {
                let event: Event = Event::Key(KeyEvent {
                    code: KeyCode::Right,
                    modifiers: KeyModifiers::NONE,
                });
                self.view.on(event);
            }

            // Refresh playlist
            key if key == &MSG_KEY_CHAR_R => self.sync_library(None),

            key if (key == &MSG_KEY_ESC) | (key == &MSG_KEY_CHAR_CAPITAL_Q) => {
                self.exit_reason = Some(ExitReason::Quit);
            }

            &_ => {}
        }
    }

    fn update_on_library(&mut self, key: &Msg) {
        match key {
            key if key == &MSG_KEY_TAB => self.view.active(COMPONENT_TABLE_PLAYLIST),
            key if key == &MSG_KEY_CHAR_Y => self.yank(),

            key if key == &MSG_KEY_CHAR_P => {
                if let Err(e) = self.paste() {
                    self.mount_error(&e.to_string());
                }
            }
            key if key == &MSG_KEY_BACKSPACE => self.update_library_stepout(),
            key if key == &MSG_KEY_CHAR_H => {
                let event: Event = Event::Key(KeyEvent {
                    code: KeyCode::Left,
                    modifiers: KeyModifiers::NONE,
                });
                self.view.on(event);
            }

            Msg::OnSubmit(Payload::One(Value::Str(node_id))) => {
                self.update_library_stepinto(node_id);
            }
            // Add selected song to playlist
            key if key == &MSG_KEY_CHAR_L => self.update_add_song_playlist(),
            // Add all songs in a folder to playlist
            key if key == &MSG_KEY_CHAR_CAPITAL_L => self.update_add_songs_playlist(),
            // start download
            key if key == &MSG_KEY_CHAR_S => self.mount_youtube_url(),

            key if key == &MSG_KEY_CHAR_D => self.update_delete(),
            key if key == &MSG_KEY_CHAR_T => self.run_tageditor(),

            key if key == &MSG_KEY_SLASH => {
                self.mount_search_library();
                self.update_search_library("*");
            }
            &_ => self.update_on_global_key(key),
        }
    }

    fn update_on_playlist(&mut self, key: &Msg) {
        match key {
            key if key == &MSG_KEY_TAB => self.view.active(COMPONENT_TREEVIEW_LIBRARY),
            key if key == &MSG_KEY_CHAR_G => {
                let event: Event = Event::Key(KeyEvent {
                    code: KeyCode::Home,
                    modifiers: KeyModifiers::NONE,
                });
                self.view.on(event);
            }

            key if key == &MSG_KEY_CHAR_CAPITAL_G => {
                let event: Event = Event::Key(KeyEvent {
                    code: KeyCode::End,
                    modifiers: KeyModifiers::NONE,
                });
                self.view.on(event);
            }

            key if key == &MSG_KEY_CHAR_L => self.update_play_selected(),

            key if key == &MSG_KEY_CHAR_D => {
                if let Some(Payload::One(Value::Usize(index))) =
                    self.view.get_state(COMPONENT_TABLE_PLAYLIST)
                {
                    self.delete_item_playlist(index);
                }
            }

            key if key == &MSG_KEY_CHAR_CAPITAL_D => self.empty_playlist(),

            key if key == &MSG_KEY_CHAR_M => self.cycle_loop_mode(),

            // shuffle
            key if key == &MSG_KEY_CHAR_S => self.shuffle(),

            &_ => self.update_on_global_key(key),
        }
    }

    fn update_search_or_download(&mut self, url: &str) {
        self.umount_youtube_url();
        if url.starts_with("http") {
            match self.youtube_dl(url) {
                Ok(_) => {}
                Err(e) => {
                    self.mount_error(format!("add playlist error: {}", e).as_str());
                }
            }
        } else {
            self.mount_youtube_options();
            self.youtube_options_search(url);
        }
    }

    fn update_download_youtube(&mut self) {
        if let Some(Payload::One(Value::Usize(index))) =
            self.view.get_state(COMPONENT_TABLE_YOUTUBE)
        {
            // download from search result here
            if let Err(e) = self.youtube_options_download(index) {
                self.mount_error(format!("download song error: {}", e).as_str());
            }
        }
        self.umount_youtube_options();
    }

    fn update_table_youtube(&mut self, key: &Msg) {
        match key {
            key if key == &MSG_KEY_TAB => self.youtube_options_next_page(),

            key if key == &MSG_KEY_SHIFT_TAB => self.youtube_options_prev_page(),

            key if (key == &MSG_KEY_ESC) | (key == &MSG_KEY_CHAR_CAPITAL_Q) => {
                self.umount_youtube_options();
            }

            key if key == &MSG_KEY_ENTER => self.update_download_youtube(),

            &_ => self.update_on_global_key(key),
        }
    }

    fn update_on_table_search_library(&mut self, key: &Msg) {
        match key {
            key if (key == &MSG_KEY_TAB) => self.view.active(COMPONENT_INPUT_SEARCH_LIBRARY),

            key if (key == &MSG_KEY_ENTER) => {
                if let Some(Payload::One(Value::Usize(index))) =
                    self.view.get_state(COMPONENT_TABLE_SEARCH_LIBRARY)
                {
                    self.select_after_search_library(index);
                }
                self.umount_search_library();
            }

            key if (key == &MSG_KEY_CHAR_L) => {
                if let Some(Payload::One(Value::Usize(index))) =
                    self.view.get_state(COMPONENT_TABLE_SEARCH_LIBRARY)
                {
                    self.add_playlist_after_search_library(index);
                }
            }

            key if (key == &MSG_KEY_ESC) | (key == &MSG_KEY_CHAR_CAPITAL_Q) => {
                self.umount_search_library();
            }

            &_ => self.update_on_global_key(key),
        }
    }
}
