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
use super::TermusicActivity;
use super::{COMPONENT_TABLE_PLAYLIST, COMPONENT_TREEVIEW_LIBRARY};

use crate::config::get_app_config_path;
use crate::song::Song;
use anyhow::Result;
use humantime::format_duration;
use rand::seq::SliceRandom;
use rand::thread_rng;
use std::collections::VecDeque;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::str::FromStr;
use std::thread;
use std::time::Duration;
use tui_realm_stdlib::TablePropsBuilder;
use tuirealm::PropsBuilder;

use tuirealm::props::{TableBuilder, TextSpan};

impl TermusicActivity {
    pub fn add_playlist(&mut self, item: Song) {
        self.playlist_items.push_front(item);
        self.sync_playlist();
    }

    pub fn sync_playlist(&mut self) {
        let mut table: TableBuilder = TableBuilder::default();

        for (idx, record) in self.playlist_items.iter().enumerate() {
            if idx > 0 {
                table.add_row();
            }

            let duration = record.duration_formatted().to_string();
            let duration_string = format!("[{:^6.6}]", duration);

            let noname_string = "No Name".to_string();
            let name = record.name().unwrap_or(&noname_string);
            let artist = record.artist().unwrap_or(name);
            let title = record.title().unwrap_or("Unknown Title");

            table
                .add_col(TextSpan::new(duration_string.as_str()))
                .add_col(TextSpan::new(artist).fg(tuirealm::tui::style::Color::LightYellow))
                .add_col(TextSpan::new(title).bold())
                .add_col(TextSpan::new(record.album().unwrap_or("Unknown Album")));
        }
        if self.playlist_items.is_empty() {
            table.add_col(TextSpan::from("0"));
            table.add_col(TextSpan::from("empty playlist"));
            table.add_col(TextSpan::from(""));
            table.add_col(TextSpan::from(""));
        }

        let table = table.build();

        if let Some(props) = self.view.get_props(COMPONENT_TABLE_PLAYLIST) {
            let props = TablePropsBuilder::from(props).with_table(table).build();
            let msg = self.view.update(COMPONENT_TABLE_PLAYLIST, props);
            self.update(msg);
        }
        self.update_title_playlist();
    }
    pub fn delete_item_playlist(&mut self, index: usize) {
        if self.playlist_items.is_empty() {
            return;
        }
        self.playlist_items.remove(index);
        self.sync_playlist();
    }

    pub fn empty_playlist(&mut self) {
        self.playlist_items.clear();
        self.sync_playlist();
        self.view.active(COMPONENT_TREEVIEW_LIBRARY);
    }

    pub fn save_playlist(&mut self) -> Result<()> {
        let mut path = get_app_config_path()?;
        path.push("playlist.log");

        let mut file = File::create(path.as_path())?;
        for i in &self.playlist_items {
            if let Some(f) = i.file() {
                writeln!(&mut file, "{}", f)?;
            }
        }

        Ok(())
    }

    pub fn load_playlist(&mut self) -> Result<()> {
        let mut path = get_app_config_path()?;
        path.push("playlist.log");

        let file = if let Ok(f) = File::open(path.as_path()) {
            f
        } else {
            File::create(path.as_path())?;
            File::open(path)?
        };
        let reader = BufReader::new(file);
        let lines: Vec<_> = reader
            .lines()
            .map(|line| line.unwrap_or_else(|_| "Error".to_string()))
            .collect();

        let tx = self.sender_playlist_items.clone();

        thread::spawn(move || {
            let mut playlist_items = VecDeque::new();
            for line in &lines {
                if let Ok(s) = Song::from_str(line) {
                    playlist_items.push_back(s);
                };
            }
            tx.send(playlist_items).ok();
        });

        Ok(())
    }

    pub fn shuffle(&mut self) {
        let mut rng = thread_rng();
        self.playlist_items.make_contiguous().shuffle(&mut rng);
        self.sync_playlist();
    }

    pub fn update_item_delete(&mut self) {
        self.playlist_items.retain(|x| {
            x.file().map_or(false, |p| {
                let path = Path::new(p);
                path.exists()
            })
        });

        self.sync_playlist();
        self.view.active(COMPONENT_TREEVIEW_LIBRARY);
    }
    pub fn update_title_playlist(&mut self) {
        let mut duration = Duration::from_secs(0);
        for v in &self.playlist_items {
            duration += v.duration();
        }
        let loop_mode_display = if self.config.loop_mode {
            "loop"
        } else {
            "consume"
        };
        let title = format!(
            "\u{2500} Playlist \u{2500}\u{2500}\u{2500}\u{2524} Total {} songs | {} |  Loop mode: {}  \u{251c}\u{2500}",
            self.playlist_items.len(),
            format_duration(Duration::new(duration.as_secs(), 0)),
            loop_mode_display,
        );
        if let Some(props) = self.view.get_props(COMPONENT_TABLE_PLAYLIST) {
            let props = TablePropsBuilder::from(props)
                .with_title(title, tuirealm::tui::layout::Alignment::Left)
                .build();
            let msg = self.view.update(COMPONENT_TABLE_PLAYLIST, props);
            self.update(msg);
        }
    }
}
