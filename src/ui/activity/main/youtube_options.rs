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
use crate::invidious::{InvidiousInstance, YoutubeVideo};
use crate::ui::components::table;
use anyhow::{anyhow, Result};
use humantime::format_duration;
use std::time::Duration;
use tuirealm::props::{TableBuilder, TextSpan};
use tuirealm::PropsBuilder;
use unicode_truncate::{Alignment, UnicodeTruncateStr};

pub struct YoutubeOptions {
    items: Vec<YoutubeVideo>,
    page: u32,
    search_word: String,
    invidious_instance: InvidiousInstance,
}

impl YoutubeOptions {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            page: 1,
            search_word: "".to_string(),
            invidious_instance: crate::invidious::InvidiousInstance::default(),
        }
    }
    pub fn get_by_index(&self, index: usize) -> Result<&YoutubeVideo> {
        if let Some(item) = self.items.get(index) {
            return Ok(item);
        }
        Err(anyhow!("index not found"))
    }

    pub fn search(&mut self, keyword: &str) -> Result<()> {
        self.search_word = keyword.to_string();
        match crate::invidious::InvidiousInstance::new(keyword) {
            Ok((instance, result)) => {
                self.invidious_instance = instance;
                self.items = result;
                self.page = 1;
                Ok(())
            }
            Err(e) => Err(e),
        }
    }
    pub fn prev_page(&mut self) -> Result<()> {
        if self.page > 1 {
            self.page -= 1;
            match self
                .invidious_instance
                .get_search_query(self.search_word.as_str(), self.page)
            {
                Ok(y) => {
                    self.items = y;
                    Ok(())
                }
                Err(e) => Err(e),
            }
        } else {
            Ok(())
        }
    }
    pub fn next_page(&mut self) -> Result<()> {
        self.page += 1;
        match self
            .invidious_instance
            .get_search_query(self.search_word.as_str(), self.page)
        {
            Ok(y) => {
                self.items = y;
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    pub fn page(&self) -> u32 {
        self.page
    }
}

impl MainActivity {
    pub fn youtube_options_download(&mut self, index: usize) {
        // download from search result here
        let mut url = "https://www.youtube.com/watch?v=".to_string();
        if let Ok(item) = self.youtube_options.get_by_index(index) {
            url.push_str(&item.video_id);
            self.youtube_dl(url.as_ref());
        }
    }

    pub fn youtube_options_search(&mut self, keyword: &str) {
        match self.youtube_options.search(keyword) {
            Ok(()) => self.sync_youtube_options(),
            Err(e) => self.mount_error(format!("search error: {}", e).as_str()),
        }
    }

    pub fn youtube_options_prev_page(&mut self) {
        match self.youtube_options.prev_page() {
            Ok(_) => self.sync_youtube_options(),
            Err(e) => self.mount_error(format!("search error: {}", e).as_str()),
        }
    }
    pub fn youtube_options_next_page(&mut self) {
        match self.youtube_options.next_page() {
            Ok(_) => self.sync_youtube_options(),
            Err(e) => self.mount_error(format!("search error: {}", e).as_str()),
        }
    }
    pub fn sync_youtube_options(&mut self) {
        if self.youtube_options.items.is_empty() {
            if let Some(props) = self.view.get_props(super::COMPONENT_SCROLLTABLE_YOUTUBE) {
                if let Some(domain) = &self.youtube_options.invidious_instance.domain {
                    let props = table::TablePropsBuilder::from(props)
                        .with_table(
                            TableBuilder::default()
                                .add_col(TextSpan::from(format!(
                                    "Empty result.Probably {} is down.",
                                    domain
                                )))
                                .build(),
                        )
                        .build();
                    let msg = self
                        .view
                        .update(super::COMPONENT_SCROLLTABLE_YOUTUBE, props);
                    self.update(msg);
                }
            }
            return;
        }

        let mut table: TableBuilder = TableBuilder::default();
        for (idx, record) in self.youtube_options.items.iter().enumerate() {
            if idx > 0 {
                table.add_row();
            }
            let duration = record.length_seconds;
            let duration_string = format!("{}", format_duration(Duration::from_secs(duration)));
            let duration_truncated = duration_string.unicode_pad(6, Alignment::Left, true);

            let title = record.title.as_str();

            table
                .add_col(TextSpan::new(
                    format!("[{}] ", duration_truncated,).as_str(),
                ))
                .add_col(TextSpan::new(title).bold());
        }
        let table = table.build();

        if let Some(props) = self.view.get_props(super::COMPONENT_SCROLLTABLE_YOUTUBE) {
            if let Some(domain) = &self.youtube_options.invidious_instance.domain {
                let title = format!(
                    "── Page {} ──┼─ {} ─┼─ {} ─────",
                    self.youtube_options.page(),
                    "Tab/Shift+Tab switch pages",
                    domain,
                );
                let props = table::TablePropsBuilder::from(props)
                    .with_title(title, tuirealm::tui::layout::Alignment::Left)
                    .with_header(&["Duration", "Name"])
                    .with_widths(&[15, 85])
                    .with_table(table)
                    .build();
                self.view
                    .update(super::COMPONENT_SCROLLTABLE_YOUTUBE, props);
            }
        }
    }
}
