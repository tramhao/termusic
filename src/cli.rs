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
// use std::ffi::OsStr;
// use crate::VERSION;
// use anyhow::{anyhow, bail, Result};
// use std::path::Path;

// use lexopt::prelude::*;
// use std::process;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "Termusic", author, version, about, long_about=None)] // Read from `Cargo.toml`
#[command(next_line_help = true)]
// #[command(propagate_version = true)]
pub struct Args {
    /// With no MUSIC_DIRECTORY, use config in `~/.config/termusic/config.toml`,
    /// default is ~/Music.
    pub music_directory: Option<String>,
    /// Not showing album cover. default is showing.  
    #[arg(short = 'c', long)]
    pub disable_cover: bool,
    /// Not showing discord representation. default is showing.
    #[clap(short, long)]
    pub disable_discord: bool,
    /// Max depth(NUMBER) of folder, default is 4.
    #[clap(short, long)]
    pub max_depth: Option<usize>,
}

// impl Args {
//     pub fn parse() -> Result<Self> {
//         let mut music_dir_from_cli: Option<String> = None;
//         let mut disable_album_art_from_cli = false;
//         let mut disable_discord_rpc_from_cli = false;
//         let mut max_depth_cli = 4;

//         let mut parser = lexopt::Parser::from_env();
//         while let Some(arg) = parser.next()? {
//             match arg {
//                 Short('h') | Long("help") => {
//                     display_help();
//                 }
//                 Short('v') | Long("version") => {
//                     println!("Termusic version is: {VERSION}");
//                     process::exit(0);
//                 }

//                 Short('c') | Long("disable-cover") => {
//                     disable_album_art_from_cli = true;
//                 }
//                 Short('d') | Long("disable-discord") => {
//                     disable_discord_rpc_from_cli = true;
//                 }
//                 Short('m') | Long("max-depth") => {
//                     max_depth_cli = parser.value()?.parse()?;
//                 }
//                 Value(val) if music_dir_from_cli.is_none() => {
//                     let dir = val
//                         .into_string()
//                         .map_err(|e| anyhow!("string convert error: {:?}", e))?;

//                     let mut path = Path::new(&dir).to_path_buf();

//                     if path.exists() {
//                         if !path.has_root() {
//                             if let Ok(p_base) = std::env::current_dir() {
//                                 path = p_base.join(path);
//                             }
//                         }

//                         if let Ok(p_canonical) = path.canonicalize() {
//                             path = p_canonical;
//                         }

//                         music_dir_from_cli = Some(path.to_string_lossy().to_string());
//                     } else {
//                         eprintln!("Error: unknown option '{dir}'");
//                         process::exit(0);
//                     }
//                 }
//                 _ => bail!("{}", arg.unexpected()),
//             }
//         }

//         Ok(Args {
//             music_dir_from_cli,
//             disable_album_art_from_cli,
//             disable_discord_rpc_from_cli,
//             max_depth_cli,
//         })
//     }
// }

// fn display_help() {
//     println!(
//         "\
// Termusic help:
// Usage: termusic [OPTIONS] [MUSIC_DIRECTORY]

// With no MUSIC_DIRECTORY, use config in `~/.config/termusic/config.toml`,
// defaults to ~/Music.

// Options:
//     -h, --help                        Print this message and exit.
//     -v, --version                     Print version and exit.
//     -c, --disable-cover               Not showing album cover.
//     -d, --disable-discord             Not showing discord representation.
//     -m NUMBER or -m=NUMBER
//         --max-depth=NUMBER            Max depth(NUMBER) of folder, default to 4.
// "
//     );

//     process::exit(0);
// }
