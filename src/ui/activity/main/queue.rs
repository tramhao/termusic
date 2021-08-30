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
use super::{COMPONENT_TABLE, COMPONENT_TREEVIEW};

use crate::song::Song;
use anyhow::{anyhow, bail, Result};
use humantime::format_duration;
use rand::seq::SliceRandom;
use rand::thread_rng;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::time::Duration;
use tui_realm_stdlib::TablePropsBuilder;
use tuirealm::PropsBuilder;

use tuirealm::props::{TableBuilder, TextSpan};

impl MainActivity {
    pub fn add_queue(&mut self, item: Song) {
        self.queue_items.insert(0, item);

        self.sync_queue();
    }

    pub fn sync_queue(&mut self) {
        let mut table: TableBuilder = TableBuilder::default();

        for (idx, record) in self.queue_items.iter().enumerate() {
            if idx > 0 {
                table.add_row();
            }

            let mut duration = record.duration().to_string();
            duration.truncate(6);
            let duration_string = format!("[{:^6}]", duration);

            let name = record
                .name
                .to_owned()
                .unwrap_or_else(|| "No Name".to_string());
            let artist = record.artist().unwrap_or(&name);
            let title = record.title().unwrap_or("Unknown Title");

            table
                .add_col(TextSpan::new(duration_string.as_str()))
                .add_col(TextSpan::new(artist).fg(tuirealm::tui::style::Color::LightYellow))
                .add_col(TextSpan::new(title).bold())
                .add_col(TextSpan::new(record.album().unwrap_or("Unknown Album")));
        }
        if self.queue_items.is_empty() {
            table.add_col(TextSpan::from("0"));
            table.add_col(TextSpan::from("empty queue"));
            table.add_col(TextSpan::from(""));
            table.add_col(TextSpan::from(""));
        }

        let table = table.build();
        let title = self.update_title();

        if let Some(props) = self.view.get_props(COMPONENT_TABLE) {
            let props = TablePropsBuilder::from(props)
                .with_title(title, tuirealm::tui::layout::Alignment::Left)
                .with_table(table)
                .build();
            self.view.update(COMPONENT_TABLE, props);
            self.view.active(COMPONENT_TABLE);
        }
    }
    pub fn delete_item(&mut self, index: usize) {
        if self.queue_items.is_empty() {
            return;
        }
        self.queue_items.remove(index);
        self.sync_queue();
    }

    pub fn empty_queue(&mut self) {
        self.queue_items.clear();
        self.sync_queue();
        self.view.active(COMPONENT_TREEVIEW);
    }

    pub fn save_queue(&mut self) -> Result<()> {
        let mut path = self.get_app_config_path()?;
        path.push("queue.log");

        let mut file = File::create(path.as_path())?;
        for i in self.queue_items.iter().rev() {
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

        for line in lines.iter() {
            match Song::from_str(line) {
                Ok(s) => self.queue_items.insert(0, s),
                Err(e) => bail!("song add to queue error: {}", e),
            };
        }

        self.sync_queue();
        self.view.active(super::COMPONENT_TREEVIEW);

        Ok(())
    }

    pub fn shuffle(&mut self) {
        let mut rng = thread_rng();
        self.queue_items.shuffle(&mut rng);
        self.sync_queue();
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

        self.sync_queue();
        self.view.active(COMPONENT_TREEVIEW);
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
