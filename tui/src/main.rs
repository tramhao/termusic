#![forbid(unsafe_code)]
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
mod cli;
mod ui;

use anyhow::Result;
use clap::Parser;
use config::Settings;
use std::path::Path;
use std::process;
use sysinfo::{PidExt, ProcessExt, System, SystemExt};
use termusiclib::{config, podcast, utils};
use ui::UI;
#[macro_use]
extern crate log;

pub const MAX_DEPTH: usize = 4;

#[tokio::main]
async fn main() -> Result<()> {
    lovely_env_logger::init_default();

    let mut config = Settings::default();
    config.load()?;
    let args = cli::Args::parse();

    if let Some(dir) = args.music_directory {
        config.music_dir_from_cli = get_path(&dir);
    }
    config.disable_album_art_from_cli = args.disable_cover;
    config.disable_discord_rpc_from_cli = args.disable_discord;
    if let Some(d) = args.max_depth {
        config.max_depth_cli = d;
    } else {
        config.max_depth_cli = MAX_DEPTH;
    }
    match args.action {
        Some(cli::Action::Import { file }) => {
            eprintln!("need to import from file {file}");
            if let Some(path_str) = get_path(&file) {
                if let Ok(db_path) = utils::get_app_config_path() {
                    if let Err(e) = podcast::import_from_opml(db_path.as_path(), &config, &path_str)
                    {
                        println!("Error when import file {file}: {e}");
                    }
                }
            }
            process::exit(0);
        }
        Some(cli::Action::Export { file }) => {
            eprintln!("need to export to file {file}");
            let path_string = get_path_export(&file);
            if let Ok(db_path) = utils::get_app_config_path() {
                eprintln!("export to {path_string}");
                if let Err(e) = podcast::export_to_opml(db_path.as_path(), &path_string) {
                    println!("Error when export file {file}: {e}");
                }
            }

            process::exit(0);
        }
        None => {}
    }

    // launch the daemon if it isn't already
    let termusic_server_prog = "termusic-server";

    let mut system = System::new();
    system.refresh_all();
    let mut launch_daemon = true;
    let mut pid = 0;
    for (id, proc) in system.processes() {
        let exe = proc.exe().display().to_string();
        if exe.contains("termusic-server") {
            pid = id.as_u32();
            launch_daemon = false;
            break;
        }
    }

    if launch_daemon {
        let proc = rust_utils::utils::spawn_process(termusic_server_prog, false, false, [""]);
        pid = proc.id();
        std::thread::sleep(std::time::Duration::from_millis(200));
    }

    println!("Server process ID: {pid}");
    std::thread::sleep(std::time::Duration::from_millis(500));
    let mut ui = UI::new(&config).await?;
    ui.run().await?;

    Ok(())
}

fn get_path(dir: &str) -> Option<String> {
    let music_dir: Option<String>;
    let mut path = Path::new(&dir).to_path_buf();

    if path.exists() {
        if !path.has_root() {
            if let Ok(p_base) = std::env::current_dir() {
                path = p_base.join(path);
            }
        }

        if let Ok(p_canonical) = path.canonicalize() {
            path = p_canonical;
        }

        music_dir = Some(path.to_string_lossy().to_string());
    } else {
        eprintln!("Error: unknown directory '{dir}'");
        process::exit(0);
    }
    music_dir
}

fn get_path_export(dir: &str) -> String {
    let mut path = Path::new(&dir).to_path_buf();

    if !path.has_root() {
        if let Ok(p_base) = std::env::current_dir() {
            path = p_base.join(path);
        }
    }

    if let Ok(p_canonical) = path.canonicalize() {
        path = p_canonical;
    }

    path.to_string_lossy().to_string()
}
