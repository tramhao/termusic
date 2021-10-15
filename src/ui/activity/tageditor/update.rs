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
    ExitReason, SearchLyricState, TagEditorActivity, COMPONENT_TE_DELETE_LYRIC,
    COMPONENT_TE_INPUT_ARTIST, COMPONENT_TE_INPUT_SONGNAME, COMPONENT_TE_LABEL_HELP,
    COMPONENT_TE_RADIO_TAG, COMPONENT_TE_SCROLLTABLE_OPTIONS, COMPONENT_TE_SELECT_LYRIC,
    COMPONENT_TE_TEXTAREA_LYRIC, COMPONENT_TE_TEXT_ERROR, COMPONENT_TE_TEXT_HELP,
};
use crate::ui::keymap::{
    MSG_KEY_CHAR_CAPITAL_G, MSG_KEY_CHAR_CAPITAL_Q, MSG_KEY_CHAR_G, MSG_KEY_CHAR_H, MSG_KEY_CHAR_J,
    MSG_KEY_CHAR_K, MSG_KEY_CHAR_L, MSG_KEY_CHAR_S, MSG_KEY_CTRL_H, MSG_KEY_ENTER, MSG_KEY_ESC,
    MSG_KEY_TAB,
};
use crate::{
    song::Song,
    songtag::search,
    ui::activity::main::{StatusLine, UpdateComponents},
};
use anyhow::{anyhow, Result};
use if_chain::if_chain;
use std::path::Path;
use std::str::FromStr;
use tui_realm_stdlib::LabelPropsBuilder;
use tuirealm::{
    event::{Event, KeyCode, KeyEvent, KeyModifiers},
    tui::style::Color,
    Msg, Payload, PropsBuilder, Value,
};

impl TagEditorActivity {
    /// ### update
    ///
    /// Update auth activity model based on msg
    /// The function exits when returns None
    pub fn update(&mut self, msg: &Option<(String, Msg)>) -> Option<(String, Msg)> {
        let ref_msg: Option<(&str, &Msg)> = msg.as_ref().map(|(s, msg)| (s.as_str(), msg));
        ref_msg.and_then(|msg| match msg {
            (component, key) if key == &MSG_KEY_TAB => {
                self.update_on_tab(component);
                None
            }
            (COMPONENT_TE_RADIO_TAG, Msg::OnSubmit(Payload::One(Value::Usize(choice)))) => {
                if *choice == 0 {
                    self.rename_song_by_tag();
                }
                None
            }
            (COMPONENT_TE_SCROLLTABLE_OPTIONS, key)
                if (key == &MSG_KEY_CHAR_L) | (key == &MSG_KEY_ENTER) =>
            {
                if let Err(e) = self.load_lyric_and_photo() {
                    self.mount_error(&e.to_string());
                }
                None
            }

            // download
            (COMPONENT_TE_SCROLLTABLE_OPTIONS, key) if key == &MSG_KEY_CHAR_S => {
                self.songtag_download();
                None
            }

            // select lyric
            (COMPONENT_TE_SELECT_LYRIC, Msg::OnSubmit(Payload::One(Value::Usize(index)))) => {
                if let Some(mut song) = self.song.clone() {
                    song.set_lyric_selected_index(*index);
                    self.init_by_song(&song);
                }
                None
            }

            // delete lyric
            (COMPONENT_TE_DELETE_LYRIC, key) if key == &MSG_KEY_ENTER => {
                self.delete_lyric();
                None
            }

            (
                COMPONENT_TE_INPUT_ARTIST | COMPONENT_TE_INPUT_SONGNAME,
                Msg::OnSubmit(Payload::One(Value::Str(_))),
            ) => {
                // Get Tag
                self.songtag_search();
                None
            }

            (COMPONENT_TE_SCROLLTABLE_OPTIONS, key) if key == &MSG_KEY_CHAR_G => {
                let event = Event::Key(KeyEvent {
                    code: KeyCode::Home,
                    modifiers: KeyModifiers::NONE,
                });
                self.view.on(event);
                None
            }

            (COMPONENT_TE_SCROLLTABLE_OPTIONS, key) if key == &MSG_KEY_CHAR_CAPITAL_G => {
                let event = Event::Key(KeyEvent {
                    code: KeyCode::End,
                    modifiers: KeyModifiers::NONE,
                });
                self.view.on(event);
                None
            }

            (COMPONENT_TE_TEXTAREA_LYRIC, key) if key == &MSG_KEY_CHAR_G => {
                let event: Event = Event::Key(KeyEvent {
                    code: KeyCode::Home,
                    modifiers: KeyModifiers::NONE,
                });
                self.view.on(event);
                None
            }

            (COMPONENT_TE_TEXTAREA_LYRIC, key) if key == &MSG_KEY_CHAR_CAPITAL_G => {
                let event: Event = Event::Key(KeyEvent {
                    code: KeyCode::End,
                    modifiers: KeyModifiers::NONE,
                });
                self.view.on(event);
                None
            }

            // -- help
            (COMPONENT_TE_TEXT_HELP, key) if (key == &MSG_KEY_ENTER) | (key == &MSG_KEY_ESC) => {
                self.umount_help();
                None
            }

            // -- error
            (COMPONENT_TE_TEXT_ERROR, key) if (key == &MSG_KEY_ESC) | (key == &MSG_KEY_ENTER) => {
                self.umount_error();
                None
            }

            (_, key) => {
                self.update_on_global_key(key);
                None
            }
        })
    }

    pub fn update_download_progress(&mut self) {
        if let Ok(transfer_state) = self.receiver.try_recv() {
            match transfer_state {
                UpdateComponents::DownloadRunning => {
                    self.update_status_line(StatusLine::Running);
                }
                UpdateComponents::DownloadSuccess => {
                    self.update_status_line(StatusLine::Success);
                }
                UpdateComponents::DownloadCompleted(file) => {
                    if let Some(f) = file {
                        if let Ok(song) = Song::from_str(&f) {
                            self.exit_reason = Some(ExitReason::NeedRefreshPlaylist(f));
                            self.init_by_song(&song);
                        }
                    }
                    self.update_status_line(StatusLine::Default);
                }
                UpdateComponents::DownloadErrDownload => {
                    self.mount_error("download failed");
                    self.update_status_line(StatusLine::Error);
                }
                UpdateComponents::DownloadErrEmbedData => {
                    self.mount_error("download ok but tag info is not complete.");
                    self.update_status_line(StatusLine::Error);
                }
                _ => {}
            }
        };
    }

    pub fn update_lyric_options(&mut self) {
        if let Ok(SearchLyricState::Finish(l)) = self.receiver_songtag.try_recv() {
            self.add_songtag_options(l);
            self.redraw = true;
        }
    }
    pub fn update_status_line(&mut self, s: StatusLine) {
        match s {
            StatusLine::Default => {
                let text = "Press <CTRL+H> for help.".to_string();

                if let Some(props) = self.view.get_props(COMPONENT_TE_LABEL_HELP) {
                    let props = LabelPropsBuilder::from(props)
                        .with_background(Color::Reset)
                        .with_foreground(Color::Cyan)
                        .with_text(text)
                        .build();

                    let msg = self.view.update(COMPONENT_TE_LABEL_HELP, props);
                    self.update(&msg);
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
                    let props = LabelPropsBuilder::from(props)
                        .with_text(text)
                        .with_foreground(Color::White)
                        .with_background(Color::Red)
                        .build();

                    let msg = self.view.update(COMPONENT_TE_LABEL_HELP, props);
                    self.update(&msg);
                    self.redraw = true;
                }
            }
            StatusLine::Success => {
                let text = " Download Success!".to_string();

                if let Some(props) = self.view.get_props(COMPONENT_TE_LABEL_HELP) {
                    let props = LabelPropsBuilder::from(props)
                        .with_text(text)
                        .with_foreground(Color::Black)
                        .with_background(Color::Green)
                        .build();

                    let msg = self.view.update(COMPONENT_TE_LABEL_HELP, props);
                    self.update(&msg);
                    self.redraw = true;
                }
            }
            StatusLine::Error => {
                let text = " Download Error!".to_string();

                if let Some(props) = self.view.get_props(COMPONENT_TE_LABEL_HELP) {
                    let props = LabelPropsBuilder::from(props)
                        .with_text(text)
                        .with_foreground(Color::White)
                        .with_background(Color::Red)
                        .build();

                    let msg = self.view.update(COMPONENT_TE_LABEL_HELP, props);
                    self.update(&msg);
                    self.redraw = true;
                }
            }
        }
    }

    fn rename_song_by_tag(&mut self) {
        if let Some(mut song) = self.song.clone() {
            if let Some(Payload::One(Value::Str(artist))) =
                self.view.get_state(COMPONENT_TE_INPUT_ARTIST)
            {
                song.set_artist(&artist);
            }
            if let Some(Payload::One(Value::Str(title))) =
                self.view.get_state(COMPONENT_TE_INPUT_SONGNAME)
            {
                song.set_title(&title);
            }
            match song.save_tag() {
                Ok(()) => {
                    if let Some(file) = song.file() {
                        self.exit_reason = Some(ExitReason::NeedRefreshPlaylist(file.to_string()));
                    }
                    self.init_by_song(&song);
                }
                Err(e) => self.mount_error(&e.to_string()),
            };
        }
    }

    fn load_lyric_and_photo(&mut self) -> Result<()> {
        if let Some(Payload::One(Value::Usize(index))) =
            self.view.get_state(COMPONENT_TE_SCROLLTABLE_OPTIONS)
        {
            if self.songtag_options.is_empty() {
                return Ok(());
            }
            if let Some(mut song) = self.song.clone() {
                let song_tag = self
                    .songtag_options
                    .get(index)
                    .ok_or_else(|| anyhow!("cannot get songtag"))?;
                let lang_ext = song_tag.lang_ext().unwrap_or("eng");
                if let Some(artist) = song_tag.artist() {
                    song.set_artist(artist);
                }
                if let Some(title) = song_tag.title() {
                    song.set_title(title);
                }
                if let Some(album) = song_tag.album() {
                    song.set_album(album);
                }

                if let Ok(lyric_string) = song_tag.fetch_lyric() {
                    song.set_lyric(&lyric_string, lang_ext);
                }
                if let Ok(artwork) = song_tag.fetch_photo() {
                    song.set_photo(artwork);
                }

                match song.save_tag() {
                    Ok(()) => {
                        if let Some(file) = song.file() {
                            self.exit_reason =
                                Some(ExitReason::NeedRefreshPlaylist(file.to_string()));
                        }
                        self.init_by_song(&song);
                    }
                    Err(e) => self.mount_error(&e.to_string()),
                }
            }
        }
        Ok(())
    }

    fn songtag_download(&mut self) {
        if_chain! {
            if let Some(Payload::One(Value::Usize(index))) =
                self.view.get_state(COMPONENT_TE_SCROLLTABLE_OPTIONS);
            if let Some(song_tag) = self.songtag_options.get(index);
            if let Some(song) = &self.song;
            if let Some(file) = song.file();
            if let Err(e) = song_tag.download(file, self.sender.clone());

            then {
                self.mount_error(&e.to_string());
            }
        }
    }

    fn delete_lyric(&mut self) {
        if let Some(mut song) = self.song.clone() {
            if song.lyric_frames_is_empty() {
                song.set_parsed_lyric(None);
                return;
            }
            song.lyric_frames_remove_selected();
            if (song.lyric_selected_index() >= song.lyric_frames_len())
                && (song.lyric_selected_index() > 0)
            {
                song.set_lyric_selected_index(song.lyric_selected_index() - 1);
            }
            match song.save_tag() {
                Ok(_) => self.init_by_song(&song),
                Err(e) => self.mount_error(&e.to_string()),
            }
        }
    }

    fn songtag_search(&mut self) {
        let mut search_str = String::new();
        if let Some(Payload::One(Value::Str(artist))) =
            self.view.get_state(COMPONENT_TE_INPUT_ARTIST)
        {
            search_str.push_str(&artist);
        }

        search_str.push(' ');
        if let Some(Payload::One(Value::Str(title))) =
            self.view.get_state(COMPONENT_TE_INPUT_SONGNAME)
        {
            search_str.push_str(&title);
        }

        if_chain! {
            if search_str.len() < 4;
            if let Some(song) = &self.song;
            if let Some(file) = song.file();
            let p: &Path = Path::new(file);
            if let Some(stem) = p.file_stem();

            then {
                search_str = stem.to_string_lossy().to_string();
            }

        }

        search(&search_str, self.sender_songtag.clone());
    }

    fn update_on_tab(&mut self, component: &str) {
        match component {
            COMPONENT_TE_INPUT_ARTIST => self.view.active(COMPONENT_TE_INPUT_SONGNAME),
            COMPONENT_TE_INPUT_SONGNAME => self.view.active(COMPONENT_TE_RADIO_TAG),
            COMPONENT_TE_RADIO_TAG => self.view.active(COMPONENT_TE_SCROLLTABLE_OPTIONS),
            COMPONENT_TE_SCROLLTABLE_OPTIONS => self.view.active(COMPONENT_TE_SELECT_LYRIC),
            COMPONENT_TE_SELECT_LYRIC => self.view.active(COMPONENT_TE_DELETE_LYRIC),
            COMPONENT_TE_DELETE_LYRIC => self.view.active(COMPONENT_TE_TEXTAREA_LYRIC),
            COMPONENT_TE_TEXTAREA_LYRIC => self.view.active(COMPONENT_TE_INPUT_ARTIST),
            &_ => {}
        }
    }

    fn update_on_global_key(&mut self, key: &Msg) {
        match key {
            key if key == &MSG_KEY_CTRL_H => self.mount_help(),
            key if key == &MSG_KEY_CHAR_H => {
                let event: Event = Event::Key(KeyEvent {
                    code: KeyCode::Left,
                    modifiers: KeyModifiers::NONE,
                });
                self.view.on(event);
            }
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
            key if key == &MSG_KEY_CHAR_L => {
                let event: Event = Event::Key(KeyEvent {
                    code: KeyCode::Right,
                    modifiers: KeyModifiers::NONE,
                });
                self.view.on(event);
            }
            key if (key == &MSG_KEY_ESC) | (key == &MSG_KEY_CHAR_CAPITAL_Q) => {
                self.exit_reason = Some(ExitReason::Quit);
            }
            &_ => {}
        }
    }
}
