/**
 * MIT License
 *
 * tuifeed - Copyright (c) 2021 Christian Visintin
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
use crate::songtag::{search, SongTag};
use crate::ui::{Id, IdTagEditor, Model, Msg, SearchLyricState, TEMsg};

use anyhow::{anyhow, Context, Result};
use std::path::Path;
use tui_realm_stdlib::Table;
use tuirealm::command::{Cmd, CmdResult, Direction, Position};
use tuirealm::event::{Key, KeyEvent, KeyModifiers, NoUserEvent};
use tuirealm::props::{Alignment, BorderType, Borders, Color, TableBuilder, TextSpan};
use tuirealm::{Component, Event, MockComponent, State, StateValue};

#[derive(MockComponent)]
pub struct TETableLyricOptions {
    component: Table,
}

impl Default for TETableLyricOptions {
    fn default() -> Self {
        Self {
            component: Table::default()
                .borders(
                    Borders::default()
                        .modifiers(BorderType::Thick)
                        .color(Color::Blue),
                )
                // .foreground(Color::Yellow)
                // .background(Color::Reset)
                .title("Search Results", Alignment::Left)
                .scroll(true)
                .highlighted_color(Color::LightBlue)
                .highlighted_str("\u{1f680}")
                // .highlighted_str("🚀")
                .rewind(false)
                .step(4)
                .row_height(1)
                .headers(&["Artist", "Title", "Album", "api", "Copyright Info"])
                .column_spacing(1)
                .widths(&[20, 20, 20, 10, 30])
                .table(
                    TableBuilder::default()
                        .add_col(TextSpan::from("0"))
                        .add_col(TextSpan::from(" "))
                        .add_col(TextSpan::from("No Results."))
                        .build(),
                ),
        }
    }
}

impl Component<Msg, NoUserEvent> for TETableLyricOptions {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        let _cmd_result = match ev {
            Event::Keyboard(KeyEvent { code: Key::Tab, .. }) => {
                return Some(Msg::TagEditor(TEMsg::TETableLyricOptionsBlurDown))
            }
            Event::Keyboard(KeyEvent {
                code: Key::BackTab,
                modifiers: KeyModifiers::SHIFT,
            }) => return Some(Msg::TagEditor(TEMsg::TETableLyricOptionsBlurUp)),

            Event::Keyboard(KeyEvent {
                code: Key::Esc | Key::Char('q'),
                ..
            }) => return Some(Msg::TagEditor(TEMsg::TagEditorClose(None))),
            Event::Keyboard(KeyEvent {
                code: Key::Char('h'),
                modifiers: KeyModifiers::CONTROL,
            }) => return Some(Msg::TagEditor(TEMsg::TEHelpPopupShow)),

            Event::Keyboard(KeyEvent {
                code: Key::Down | Key::Char('j'),
                ..
            }) => self.perform(Cmd::Move(Direction::Down)),
            Event::Keyboard(KeyEvent {
                code: Key::Up | Key::Char('k'),
                ..
            }) => self.perform(Cmd::Move(Direction::Up)),
            Event::Keyboard(KeyEvent {
                code: Key::PageDown,
                ..
            }) => self.perform(Cmd::Scroll(Direction::Down)),
            Event::Keyboard(KeyEvent {
                code: Key::PageUp, ..
            }) => self.perform(Cmd::Scroll(Direction::Up)),
            Event::Keyboard(KeyEvent {
                code: Key::Home | Key::Char('g'),
                ..
            }) => self.perform(Cmd::GoTo(Position::Begin)),
            Event::Keyboard(
                KeyEvent { code: Key::End, .. }
                | KeyEvent {
                    code: Key::Char('G'),
                    modifiers: KeyModifiers::SHIFT,
                },
            ) => self.perform(Cmd::GoTo(Position::End)),
            Event::Keyboard(KeyEvent {
                code: Key::Char('s'),
                ..
            }) => {
                if let State::One(StateValue::Usize(index)) = self.state() {
                    return Some(Msg::TagEditor(TEMsg::TEDownload(index)));
                }
                CmdResult::None
            }
            Event::Keyboard(KeyEvent {
                code: Key::Char('l'),
                ..
            }) => {
                if let State::One(StateValue::Usize(index)) = self.state() {
                    return Some(Msg::TagEditor(TEMsg::TEEmbed(index)));
                }
                CmdResult::None
            }

            _ => CmdResult::None,
        };
        // match cmd_result {
        // CmdResult::Submit(State::One(StateValue::Usize(_index))) => {
        //     return Some(Msg::PlaylistPlaySelected);
        // }
        //_ =>
        Some(Msg::None)
        // }
    }
}

impl Model {
    pub fn te_add_songtag_options(&mut self, items: Vec<SongTag>) {
        self.songtag_options = items;
        self.te_sync_songtag_options();
        assert!(self
            .app
            .active(&Id::TagEditor(IdTagEditor::TableLyricOptions))
            .is_ok());
    }

    fn te_sync_songtag_options(&mut self) {
        let mut table: TableBuilder = TableBuilder::default();

        for (idx, record) in self.songtag_options.iter().enumerate() {
            if idx > 0 {
                table.add_row();
            }
            let artist = record.artist().unwrap_or("Nobody");
            let title = record.title().unwrap_or("Unknown Title");
            let album = record.album().unwrap_or("Unknown Album");
            let mut api = "N/A".to_string();
            if let Some(a) = record.service_provider() {
                api = a.to_string();
            }

            let mut url = record.url().unwrap_or_else(|| "No url".to_string());
            if url.starts_with("http") {
                url = "Downloadable".to_string();
            }

            table
                .add_col(TextSpan::new(artist).fg(tuirealm::tui::style::Color::LightYellow))
                .add_col(TextSpan::new(title).bold())
                .add_col(TextSpan::new(album))
                .add_col(TextSpan::new(api))
                .add_col(TextSpan::new(url));
        }
        let table = table.build();
        assert!(self
            .app
            .attr(
                &Id::TagEditor(IdTagEditor::TableLyricOptions),
                tuirealm::Attribute::Content,
                tuirealm::AttrValue::Table(table),
            )
            .is_ok());
    }

    pub fn te_songtag_search(&mut self) {
        let mut search_str = String::new();
        if let Ok(State::One(StateValue::String(artist))) =
            self.app.state(&Id::TagEditor(IdTagEditor::InputArtist))
        {
            search_str.push_str(&artist);
        }
        search_str.push(' ');
        if let Ok(State::One(StateValue::String(title))) =
            self.app.state(&Id::TagEditor(IdTagEditor::InputTitle))
        {
            search_str.push_str(&title);
        }

        if search_str.len() < 4 {
            if let Some(song) = &self.tageditor_song {
                if let Some(file) = song.file() {
                    let p: &Path = Path::new(file);
                    if let Some(stem) = p.file_stem() {
                        search_str = stem.to_string_lossy().to_string();
                    }
                }
            }
        }
        search(&search_str, self.sender_songtag.clone());
    }
    pub fn te_update_lyric_options(&mut self) {
        if self
            .app
            .mounted(&Id::TagEditor(IdTagEditor::TableLyricOptions))
        {
            if let Ok(SearchLyricState::Finish(l)) = self.receiver_songtag.try_recv() {
                self.te_add_songtag_options(l);
                self.redraw = true;
            }
        }
    }

    pub fn te_songtag_download(&mut self, index: usize) -> Result<()> {
        let song_tag = self
            .songtag_options
            .get(index)
            .context(format!("no song_tag with index {} found", index))?;
        if let Some(song) = &self.tageditor_song {
            let file = song.file().context("no file path found")?;
            song_tag.download(file, &self.sender)?;
        }
        Ok(())
    }
    pub fn te_rename_song_by_tag(&mut self) -> Result<()> {
        if let Some(mut song) = self.tageditor_song.clone() {
            if let Ok(State::One(StateValue::String(artist))) =
                self.app.state(&Id::TagEditor(IdTagEditor::InputArtist))
            {
                song.set_artist(&artist);
            }
            if let Ok(State::One(StateValue::String(title))) =
                self.app.state(&Id::TagEditor(IdTagEditor::InputTitle))
            {
                song.set_title(&title);
            }
            song.save_tag()?;
            self.init_by_song(&song);
            self.playlist_update_library_delete();
        }
        Ok(())
    }

    pub fn te_load_lyric_and_photo(&mut self, index: usize) -> Result<()> {
        if self.songtag_options.is_empty() {
            return Ok(());
        }
        if let Some(mut song) = self.tageditor_song.clone() {
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

            song.save_tag()?;
            self.init_by_song(&song);
            self.playlist_update_library_delete();
            // self.library_sync(song.file());
        }
        Ok(())
    }
}
