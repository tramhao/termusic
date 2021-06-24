use super::TagEditorActivity;

// use crate::song::Song;
// use std::fs::{self, File};
// use std::io::{BufRead, BufReader, Write};
// use std::path::{Path, PathBuf};
// use std::str::FromStr;
use crate::lyric::SongTag;
use crate::ui::components::scrolltable;
use tuirealm::PropsBuilder;

use tuirealm::props::{TableBuilder, TextSpan};

impl TagEditorActivity {
    pub fn add_lyric_options(&mut self, items: Vec<SongTag>) {
        // let line = String::from_utf8(item.file.into()).expect("utf8 error");

        // self.queue_items.insert(0, item);
        self.lyric_options = items;

        self.sync_items();
    }

    pub fn sync_items(&mut self) {
        let mut table: TableBuilder = TableBuilder::default();

        for (idx, record) in self.lyric_options.iter().enumerate() {
            if idx > 0 {
                table.add_row();
            }

            table.add_col(TextSpan::from(format!("{}", record)));
        }
        let table = table.build();

        match self.view.get_props(super::COMPONENT_TE_SCROLLTABLE_OPTIONS) {
            None => None,
            Some(props) => {
                let props = scrolltable::ScrollTablePropsBuilder::from(props)
                    .with_table(Some(String::from("Queue")), table)
                    .build();
                self.view
                    .update(super::COMPONENT_TE_SCROLLTABLE_OPTIONS, props)
            }
        };
    }

    pub fn empty_queue(&mut self) {
        self.lyric_options.clear();
        self.sync_items();
    }
}
