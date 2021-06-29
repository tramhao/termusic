use super::MainActivity;
use super::COMPONENT_SCROLLTABLE;

use crate::song::Song;
use crate::ui::components::scrolltable;
use anyhow::{anyhow, Result};
use humantime::format_duration;
use rand::seq::SliceRandom;
use rand::thread_rng;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::time::Duration;
use tuirealm::PropsBuilder;

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

            table.add_col(TextSpan::from(format!("{}", record)));
        }
        let table = table.build();

        let title = self.update_title();
        match self.view.get_props(COMPONENT_SCROLLTABLE) {
            None => None,
            Some(props) => {
                let props = scrolltable::ScrollTablePropsBuilder::from(props)
                    .with_table(Some(title), table)
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
                Err(e) => return Err(anyhow!("song add to queue error: {}", e)),
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
