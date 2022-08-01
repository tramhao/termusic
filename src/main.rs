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
mod cli;
mod config;
#[cfg(feature = "discord")]
mod discord;
mod invidious;
mod player;
mod playlist;
mod songtag;
mod sqlite;
mod track;
#[cfg(feature = "cover")]
mod ueberzug;
mod ui;
mod utils;

use anyhow::Result;
use config::Settings;

use ui::{UI, VERSION};

fn main() -> Result<()> {
    let mut config = Settings::default();
    config.load().unwrap_or_default();

    let args = cli::Args::parse()?;
    config.music_dir_from_cli = args.music_dir_from_cli;
    config.disable_album_art_from_cli = args.disable_album_art_from_cli;
    config.disable_discord_rpc_from_cli = args.disable_discord_rpc_from_cli;
    config.max_depth_cli = args.max_depth_cli;

    let mut ui = UI::new(&config);
    ui.run();
    Ok(())
}
