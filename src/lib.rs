use include_dir::{include_dir, Dir};

pub mod cli;
pub mod config;
#[cfg(feature = "discord")]
pub mod discord;
pub mod invidious;
pub mod playlist;
pub mod podcast;
pub mod songtag;
pub mod sqlite;
pub mod track;
pub mod types;
#[cfg(feature = "cover")]
pub mod ueberzug;
pub mod utils;
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub static THEME_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/themes");
