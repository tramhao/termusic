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
// use anyhow::Result;
// use clap::Parser;
// use config::Settings;
// use std::path::Path;
// use std::process;

// use ui::UI;
// pub const VERSION: &str = env!("CARGO_PKG_VERSION");
// pub const MAX_DEPTH: usize = 4;
mod daemon;
#[macro_use]
extern crate log;

use std::{
    env, fs,
    io::{BufReader, Read, Write},
    net::Shutdown,
    os::unix::net::UnixStream,
    process,
    sync::RwLock,
};
// use rodio::source::{SineWave, Source};
use rodio::{Decoder, OutputStream, Sink};
use std::fs::File;
// use std::time::Duration;
use anyhow::{anyhow, Result};
use lazy_static::lazy_static;
use termusiclib::config::Settings;
use termusicplayback::{GeneralPlayer, PlayerCmd};
// use termusiclib::player::{GeneralPlayer, PlayerTrait};

use sysinfo::{PidExt, ProcessExt, System, SystemExt};

lazy_static! {
    static ref TMP_DIR: String = format!(
        "/tmp/termusic-{}/",
        env::var("USER").expect("What is your name again?")
    );
    // static ref LOG: Log = Log::get("termusicd", "termusic");
    // static ref PLAYER: RwLock<GeneralPlayer> = RwLock::new(GeneralPlayer::new());
    // static ref CONFIG: MLConfig = MLConfig::load();
}

fn main() -> Result<()> {
    lovely_env_logger::init_default();
    info!("background thread start");
    let mut config = Settings::default();
    config.load()?;
    info!("config loaded");

    let mut system = System::new_all();
    system.refresh_all();

    if audio_cmd::<usize>(PlayerCmd::ProcessID, true).is_ok() {
        info!("termusic daemon is already running");
        std::process::exit(101);
    }

    daemon::spawn();
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
    // loop {
    //     warn!("running");
    //     std::thread::sleep(std::time::Duration::from_secs(5));
    // }

    // let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    // let sink = Sink::try_new(&stream_handle).unwrap();

    // // Add a dummy source of the sake of the example.

    // let file = BufReader::new(
    //     File::open("/home/tramhao/Music/mp3/misc-new/马上又-生生不息的暖流.mp3").unwrap(),
    // );
    // let source = Decoder::new(file).unwrap();

    // // let source = SineWave::new(440.0).take_duration(Duration::from_secs_f32(0.25)).amplify(0.20);
    // sink.append(source);

    // // The sound plays in a separate thread. This call will block the current thread until the sink
    // // has finished playing all its queued sounds.
    // sink.sleep_until_end();

    // info!("background thread ended");
    Ok(())
}

fn audio_cmd<T: for<'de> serde::Deserialize<'de>>(cmd: PlayerCmd, silent: bool) -> Result<T> {
    let socket_file = format!("{}/socket", *TMP_DIR);
    match UnixStream::connect(socket_file) {
        Ok(mut stream) => {
            let encoded = bincode::serialize(&cmd).expect("What went wrong?!");
            stream
                .write_all(&encoded)
                .expect("Unable to write to socket!");
            stream.shutdown(Shutdown::Write).expect("What went wrong?!");
            let buffer = BufReader::new(&stream);
            let encoded: Vec<u8> = buffer.bytes().map(|r| r.unwrap_or(0)).collect();
            Ok(bincode::deserialize(&encoded).expect("What went wrong?!"))
        }

        Err(why) => {
            if !silent {
                error!("unable to connect to socket: {why}");
                // LOG.line(
                //     LogLevel::Error,
                //     format!("Unable to connect to socket: {why}"),
                //     true,
                // );
            }
            Err(anyhow!(why.to_string()))
        }
    }
}
