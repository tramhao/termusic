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
use super::{TagEditorActivity, COMPONENT_TE_SCROLLTABLE_OPTIONS};
use crate::songtag::{SongTag, SongtagProvider};
use tui_realm_stdlib::TablePropsBuilder;
use tuirealm::{
    props::{TableBuilder, TextSpan},
    PropsBuilder,
};

impl TagEditorActivity {
    pub fn add_songtag_options(&mut self, items: Vec<SongTag>) {
        self.songtag_options = items;
        self.sync_songtag_options();
        self.view.active(COMPONENT_TE_SCROLLTABLE_OPTIONS);
    }

    pub fn sync_songtag_options(&mut self) {
        let mut table: TableBuilder = TableBuilder::default();

        for (idx, record) in self.songtag_options.iter().enumerate() {
            if idx > 0 {
                table.add_row();
            }
            let artist = record.artist().unwrap_or("Nobody");
            let title = record.title().unwrap_or("Unknown Title");
            let album = record.album().unwrap_or("Unknown Album");
            let api = match record.service_provider.as_ref() {
                Some(SongtagProvider::Netease) => "netease",
                Some(SongtagProvider::Kugou) => "kugou",
                Some(SongtagProvider::Migu) => "migu",
                None => "N/A",
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

        if let Some(props) = self.view.get_props(COMPONENT_TE_SCROLLTABLE_OPTIONS) {
            let props = TablePropsBuilder::from(props).with_table(table).build();
            self.view.update(COMPONENT_TE_SCROLLTABLE_OPTIONS, props);
        }
    }
}
