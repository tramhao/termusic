pub mod config;
pub mod invidious;
pub mod playlist;
#[allow(unused)]
pub mod podcast;
pub mod songtag;
pub mod track;
pub mod types;
pub mod utils;
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
