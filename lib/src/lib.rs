#![forbid(unsafe_code)]
#![recursion_limit = "2048"]
#![warn(clippy::all, clippy::correctness)]
#![warn(rust_2018_idioms)]
#![warn(clippy::pedantic)]
#![allow(clippy::missing_errors_doc, clippy::must_use_candidate)]

pub mod config;
pub mod invidious;
pub mod playlist;
pub mod podcast;
pub mod songtag;
pub mod sqlite;
pub mod track;
pub mod types;
pub mod ueberzug;
pub mod utils;
pub mod xywh;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

use include_dir::{include_dir, Dir};
pub static THEME_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/themes");

#[macro_use]
extern crate log;
