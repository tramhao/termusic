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
use super::TagEditorActivity;
use crate::songtag::{SongTag, SongtagProvider};
use tui_realm_stdlib::TablePropsBuilder;
use tuirealm::PropsBuilder;
// use unicode_truncate::{Alignment, UnicodeTruncateStr};

use tuirealm::props::{TableBuilder, TextSpan};

impl TagEditorActivity {
    pub fn add_lyric_options(&mut self, items: Vec<SongTag>) {
        self.lyric_options = items;
        self.sync_items();
        self.view.active(super::COMPONENT_TE_SCROLLTABLE_OPTIONS);
    }

    pub fn sync_items(&mut self) {
        let mut table: TableBuilder = TableBuilder::default();

        for (idx, record) in self.lyric_options.iter().enumerate() {
            if idx > 0 {
                table.add_row();
            }
            let artist = record
                .artist
                .clone()
                .unwrap_or_else(|| "Nobody".to_string());
            // let artist_truncated = artist.unicode_pad(10, Alignment::Left, true);
            let title = record
                .title
                .clone()
                .unwrap_or_else(|| "Unknown Title".to_string());
            // let title_truncated = title.unicode_pad(16, Alignment::Left, true);
            let album = record
                .album
                .clone()
                .unwrap_or_else(|| "Unknown Album".to_string());
            // let album_truncated = album.unicode_pad(16, Alignment::Left, true);
            let api = match record.service_provider {
                Some(SongtagProvider::Netease) => "netease".to_string(),
                Some(SongtagProvider::Kugou) => "kugou".to_string(),
                Some(SongtagProvider::Migu) => "migu".to_string(),
                None => "N/A".to_string(),
            };

            let mut url = record.url.clone().unwrap_or_else(|| "No url".to_string());
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

        if let Some(props) = self.view.get_props(super::COMPONENT_TE_SCROLLTABLE_OPTIONS) {
            let props = TablePropsBuilder::from(props).with_table(table).build();
            self.view
                .update(super::COMPONENT_TE_SCROLLTABLE_OPTIONS, props);
        }
    }
}
