use super::MainActivity;
use super::COMPONENT_SCROLLTABLE;

use crate::song::Song;
use crate::ui::components::scrolltable;
use anyhow::{anyhow, bail, Result};
use rand::seq::SliceRandom;
use rand::thread_rng;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use tuirealm::PropsBuilder;
use unicode_truncate::{Alignment, UnicodeTruncateStr};

use tuirealm::props::{TableBuilder, TextSpan, TextSpanBuilder};

impl MainActivity {
    pub fn add_queue(&mut self, item: Song) {
        self.queue_items.insert(0, item);

        self.sync_items();
    }

    pub fn sync_items(&mut self) {
        if self.queue_items.is_empty() {
            return;
        }
        let mut table: TableBuilder = TableBuilder::default();

        for (idx, record) in self.queue_items.iter().enumerate() {
            if idx > 0 {
                table.add_row();
            }
            let duration = record.duration();
            let duration_string = format!("{}", duration);
            let duration_truncated = duration_string.unicode_pad(6, Alignment::Left, true);

            let artist = record
                .artist()
                .unwrap_or_else(|| record.name.as_ref().unwrap());
            let artist_truncated = artist.unicode_pad(14, Alignment::Left, true);
            let title = record.title().unwrap_or("Unknown Title");
            let title_truncated = title.unicode_pad(20, Alignment::Left, true);

            table
                .add_col(
                    TextSpanBuilder::new(format!("[{}] ", duration_truncated,).as_str()).build(),
                )
                .add_col(
                    TextSpanBuilder::new(&artist_truncated)
                        .with_foreground(tui::style::Color::LightYellow)
                        .build(),
                )
                .add_col(TextSpan::from(" "))
                .add_col(
                    TextSpanBuilder::new(title_truncated.as_ref())
                        .bold()
                        .build(),
                )
                .add_col(
                    TextSpanBuilder::new(
                        format!(" {}", record.album().unwrap_or("Unknown Album")).as_str(),
                    )
                    .build(),
                );
            // table.add_col(TextSpan::from(format!("{}", record)));
        }
        let table = table.build();

        match self.view.get_props(COMPONENT_SCROLLTABLE) {
            None => None,
            Some(props) => {
                let props = scrolltable::ScrollTablePropsBuilder::from(props.clone())
                    .with_table(Some(props.texts.title.unwrap()), table)
                    .build();
                self.view.update(COMPONENT_SCROLLTABLE, props)
            }
        };
    }
    pub fn delete_item(&mut self, index: usize) {
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
            writeln!(&mut file, "{}", i.file.as_ref().unwrap()).unwrap();
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
        let lines: Vec<_> = reader.lines().map(|line| line.unwrap()).collect();
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
            let p: &Path = Path::new(x.file.as_ref().unwrap());
            p.exists()
        });

        self.sync_items();
    }
}
