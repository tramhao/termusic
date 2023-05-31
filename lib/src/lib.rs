#![forbid(unsafe_code)]
#![recursion_limit = "2048"]
#![warn(clippy::all, clippy::correctness)]
#![warn(rust_2018_idioms)]
// #![warn(clippy::nursery)]
#![warn(clippy::pedantic)]
use include_dir::{include_dir, Dir};

pub mod config;
pub mod invidious;
// pub mod player;
pub mod playlist;
pub mod podcast;
pub mod songtag;
pub mod sqlite;
pub mod track;
pub mod types;
// #[cfg(feature = "cover")]
pub mod ueberzug;
pub mod utils;
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub static THEME_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/themes");
