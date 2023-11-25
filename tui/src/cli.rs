use std::path::PathBuf;

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
use clap::{builder::ArgPredicate, Parser, Subcommand};

#[derive(Parser, Debug)]
#[clap(name = "Termusic", author, version, about, long_about=None)] // Read from `Cargo.toml`
                                                                    // #[clap(next_line_help = true)]
                                                                    // #[clap(propagate_version = true)]
pub struct Args {
    /// Commands for podcast
    #[command(subcommand)]
    pub action: Option<Action>,
    /// With no MUSIC_DIRECTORY, use config in `~/.config/termusic/config.toml`,
    /// default is ~/Music.
    pub music_directory: Option<String>,
    /// Not showing album cover. default is showing.  
    #[arg(short = 'c', long)]
    pub disable_cover: bool,
    /// Not showing discord representation. default is showing.
    #[arg(short, long)]
    pub disable_discord: bool,
    /// Max depth(NUMBER) of folder, default is 4.
    #[arg(short, long)]
    pub max_depth: Option<usize>,
    #[clap(flatten)]
    pub log_options: LogOptions,
}

#[derive(Subcommand, Debug)]
pub enum Action {
    /// Import feeds from opml file.
    Export {
        #[arg(value_name = "FILE")]
        file: String,
    },
    /// Export feeds to opml file.
    Import {
        #[arg(value_name = "FILE")]
        file: String,
    },
}

const DEFAULT_LOGFILE_FILENAME: &str = "termusic-tui.log";

#[derive(Debug, Parser, Clone, PartialEq)]
pub struct LogOptions {
    /// Enable logging to a file,
    /// automatically enabled if "log-file" is manually set
    #[arg(
        long = "log-to-file",
        default_value_if("log_file", ArgPredicate::IsPresent, "true")
    )]
    pub log_to_file: bool,

    /// Set logging file
    #[arg(long = "log-file", default_value_os_t = default_logfile_path(), env = "TM_LOGFILE")]
    pub log_file: PathBuf,

    /// Use colored logging for files
    /// Example: live tailing via `tail -f /logfile`
    #[arg(long = "log-filecolor", env = "TM_LOGFILE_COLOR")]
    pub file_color_log: bool,
}

fn default_logfile_path() -> PathBuf {
    std::env::temp_dir().join(DEFAULT_LOGFILE_FILENAME)
}
