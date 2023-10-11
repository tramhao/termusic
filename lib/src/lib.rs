#![forbid(unsafe_code)]
#![recursion_limit = "2048"]
#![warn(rust_2018_idioms)]
#![warn(clippy::correctness)]
// TODO: Allow when we fixed all of the pedantic warnings
//#![warn(clippy::all, clippy::correctness)]
//#![warn(clippy::pedantic)]
#[allow(clippy::missing_errors_doc, clippy::must_use_candidate)]
pub mod config;
#[allow(clippy::missing_errors_doc, clippy::must_use_candidate)]
pub mod invidious;
#[allow(clippy::missing_errors_doc, clippy::must_use_candidate)]
pub mod playlist;
#[allow(clippy::missing_errors_doc, clippy::must_use_candidate)]
pub mod podcast;
#[allow(clippy::missing_errors_doc, clippy::must_use_candidate)]
pub mod songtag;
#[allow(clippy::missing_errors_doc, clippy::must_use_candidate)]
pub mod sqlite;
#[allow(clippy::missing_errors_doc, clippy::must_use_candidate)]
pub mod track;
#[allow(clippy::missing_errors_doc, clippy::must_use_candidate)]
pub mod types;
#[allow(clippy::missing_errors_doc, clippy::must_use_candidate)]
pub mod ueberzug;
#[allow(clippy::missing_errors_doc, clippy::must_use_candidate)]
pub mod utils;
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
use include_dir::{include_dir, Dir};
pub static THEME_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/themes");
pub use tokio_stream::{Stream, StreamExt};

#[macro_use]
extern crate log;
