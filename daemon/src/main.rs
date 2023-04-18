// #![forbid(unsafe_code)]
#![recursion_limit = "2048"]
#![warn(clippy::all, clippy::correctness)]
#![warn(rust_2018_idioms)]
// #![warn(clippy::nursery)]
#![warn(clippy::pedantic)]
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
// mod cli;
// mod config;
// #[cfg(feature = "discord")]
// mod discord;
// mod invidious;
// mod player;
// mod playlist;
// #[allow(unused)]
// mod podcast;
// mod songtag;
// mod sqlite;
// mod track;
// #[cfg(feature = "cover")]
// mod ueberzug;
// mod ui;
// mod utils;
use anyhow::Result;
// use clap::Parser;
// use config::Settings;
// use std::path::Path;
// use std::process;

// use ui::UI;
// pub const VERSION: &str = env!("CARGO_PKG_VERSION");
// pub const MAX_DEPTH: usize = 4;

fn main() -> Result<()> {
    // let mut config = Settings::default();
    // config.load()?;
    // let args = cli::Args::parse();

    // if let Some(dir) = args.music_directory {
    //     config.music_dir_from_cli = get_path(&dir);
    // }
    // config.disable_album_art_from_cli = args.disable_cover;
    // config.disable_discord_rpc_from_cli = args.disable_discord;
    // if let Some(d) = args.max_depth {
    //     config.max_depth_cli = d;
    // } else {
    //     config.max_depth_cli = MAX_DEPTH;
    // }
    // match args.action {
    //     Some(cli::Action::Import { file }) => {
    //         eprintln!("need to import from file {file}");
    //         if let Some(path_str) = get_path(&file) {
    //             if let Ok(db_path) = utils::get_app_config_path() {
    //                 if let Err(e) = podcast::import_from_opml(db_path.as_path(), &config, &path_str)
    //                 {
    //                     println!("Error when import file {file}: {e}");
    //                 }
    //             }
    //         }
    //         process::exit(0);
    //     }
    //     Some(cli::Action::Export { file }) => {
    //         eprintln!("need to export to file {file}");
    //         let path_string = get_path_export(&file);
    //         if let Ok(db_path) = utils::get_app_config_path() {
    //             eprintln!("export to {path_string}");
    //             if let Err(e) = podcast::export_to_opml(db_path.as_path(), &path_string) {
    //                 println!("Error when export file {file}: {e}");
    //             }
    //         }

    //         process::exit(0);
    //     }
    //     None => {}
    // }

    // let mut ui = UI::new(&config);
    // ui.run();
    loop {
        eprintln!("running");
        std::thread::sleep(std::time::Duration::from_secs(5));
    }
    Ok(())
}
