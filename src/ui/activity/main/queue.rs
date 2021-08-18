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
use super::MainActivity;
use super::COMPONENT_TABLE;

use crate::song::Song;
use crate::ui::components::table;
use anyhow::{anyhow, bail, Result};
use humantime::format_duration;
use rand::seq::SliceRandom;
use rand::thread_rng;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::time::Duration;
use tuirealm::PropsBuilder;
use unicode_truncate::{Alignment, UnicodeTruncateStr};

use tuirealm::props::{TableBuilder, TextSpan};

impl MainActivity {
    pub fn add_queue(&mut self, item: Song) {
        self.queue_items.insert(0, item);

        self.sync_items();
    }

    pub fn sync_items(&mut self) {
        let mut table: TableBuilder = TableBuilder::default();

        for (idx, record) in self.queue_items.iter().enumerate() {
            if idx > 0 {
                table.add_row();
            }

            let duration = record.duration();
            let duration_string = format!("{}", duration);
            let duration_truncated = duration_string.unicode_pad(6, Alignment::Left, true);

            let name = record.name.clone().unwrap_or_else(|| "No Name".to_string());
            let artist = record.artist().unwrap_or(&name);
            let artist_truncated = artist.unicode_pad(14, Alignment::Left, true);
            let title = record.title().unwrap_or("Unknown Title");
            let title_truncated = title.unicode_pad(20, Alignment::Left, true);

            table
                .add_col(TextSpan::new(
                    format!("[{}] ", duration_truncated,).as_str(),
                ))
                .add_col(
                    TextSpan::new(&artist_truncated).fg(tuirealm::tui::style::Color::LightYellow),
                )
                .add_col(TextSpan::new(title_truncated.as_ref()).bold())
                .add_col(TextSpan::new(record.album().unwrap_or("Unknown Album")));
        }
        if self.queue_items.is_empty() {
            table.add_col(TextSpan::from("empty queue"));
            table.add_col(TextSpan::from(""));
            table.add_col(TextSpan::from(""));
            table.add_col(TextSpan::from(""));
        }

        let table = table.build();
        let title = self.update_title();

        if let Some(props) = self.view.get_props(COMPONENT_TABLE) {
            let props = table::TablePropsBuilder::from(props)
                .with_title(title, tuirealm::tui::layout::Alignment::Left)
                .with_table(table)
                .build();
            self.view.update(COMPONENT_TABLE, props);
        }
    }
    pub fn delete_item(&mut self, index: usize) {
        if self.queue_items.is_empty() {
            return;
        }
        self.queue_items.remove(index);
        self.sync_items();
    }

    pub fn empty_queue(&mut self) {
        self.queue_items.clear();
        self.sync_items();
    }

    pub fn save_queue(&mut self) -> Result<()> {
        let mut path = self.get_app_config_path()?;
        path.push("queue.log");

        let mut file = File::create(path.as_path())?;
        for i in self.queue_items.iter() {
            if let Some(f) = &i.file {
                writeln!(&mut file, "{}", f)?;
            }
        }

        Ok(())
    }

    pub fn get_app_config_path(&mut self) -> Result<PathBuf> {
        let mut path = dirs_next::home_dir()
            .map(|h| h.join(".config"))
            .ok_or_else(|| anyhow!("failed to find os config dir."))?;

        path.push("termusic");
        fs::create_dir_all(&path)?;
        Ok(path)
    }
    pub fn load_queue(&mut self) -> Result<()> {
        let mut path = self.get_app_config_path()?;
        path.push("queue.log");

        let file = match File::open(path.as_path()) {
            Ok(f) => f,
            Err(_) => {
                File::create(path.as_path())?;

                File::open(path)?
            }
        };
        let reader = BufReader::new(file);
        let lines: Vec<_> = reader
            .lines()
            .map(|line| line.unwrap_or_else(|_| "Error".to_string()))
            .collect();
        for line in lines.iter().rev() {
            match Song::from_str(line) {
                Ok(s) => self.add_queue(s),
                Err(e) => bail!("song add to queue error: {}", e),
            };
        }

        Ok(())
    }

    pub fn shuffle(&mut self) {
        let mut rng = thread_rng();
        self.queue_items.shuffle(&mut rng);
        self.sync_items();
    }

    pub fn update_item_delete(&mut self) {
        self.queue_items.retain(|x| {
            if let Some(p) = &x.file {
                let path = Path::new(p);
                path.exists()
            } else {
                false
            }
        });

        self.sync_items();
    }
    pub fn update_title(&self) -> String {
        let mut duration = Duration::from_secs(0);
        for v in self.queue_items.iter() {
            if let Some(d) = v.duration {
                duration += d;
            }
        }
        format!(
            "─ Queue ───┤ Total {} songs | {} ├─",
            self.queue_items.len(),
            format_duration(Duration::new(duration.as_secs(), 0))
        )
    }
}
