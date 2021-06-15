use super::MainActivity;
use super::COMPONENT_SCROLLTABLE;

use crate::song::Song;
use anyhow::{anyhow, Result};
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use tuirealm::components::scrolltable;
use tuirealm::PropsBuilder;

use tuirealm::props::{TableBuilder, TextSpan};

impl MainActivity {
    pub fn add_queue(&mut self, item: Song) {
        // let line = String::from_utf8(item.file.into()).expect("utf8 error");

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

        match self.view.get_props(COMPONENT_SCROLLTABLE) {
            None => None,
            Some(props) => {
                let props = scrolltable::ScrollTablePropsBuilder::from(props)
                    .with_table(Some(String::from("Queue")), table)
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

        let mut file = File::create(path.as_path()).ok().unwrap();
        for i in self.queue_items.iter() {
            writeln!(&mut file, "{}", i.file).unwrap();
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
                File::create(path.as_path()).ok().unwrap();
                let f = File::open(path).ok().unwrap();
                f
            }
        };
        let reader = BufReader::new(file);

        for (_, line) in reader.lines().enumerate() {
            let file = line.unwrap();
            match Song::load(file.clone()) {
                Ok(s) => self.add_queue(s),
                Err(e) => println!("{}", e),
            };
        }

        // self.sync_items();
        Ok(())
    }
}
