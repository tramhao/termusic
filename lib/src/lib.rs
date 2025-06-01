#![forbid(unsafe_code)]
#![recursion_limit = "2048"]
#![warn(clippy::all, clippy::correctness)]
#![warn(rust_2018_idioms)]
#![warn(clippy::pedantic)]
// TODO: work to remove the following lints
#![allow(clippy::missing_errors_doc)]

pub mod config;
pub mod ids;
pub mod invidious;
pub mod library_db;
pub mod new_database;
pub mod player;
pub mod playlist;
pub mod podcast;
pub mod songtag;
pub mod taskpool;
pub mod track;
pub mod types;
pub mod ueberzug;
pub mod utils;
pub mod xywh;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

use include_dir::{Dir, include_dir};
pub static THEME_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/themes");

#[macro_use]
extern crate log;

#[cfg(test)]
mod tests {
    use std::{ffi::OsStr, path::PathBuf};

    use crate::config::v2::tui::theme::ThemeColors;

    /// Test that all themes in /lib/themes/ can be loaded
    #[test]
    fn should_parse_all_themes() {
        let cargo_manifest_dir = env!("CARGO_MANIFEST_DIR");
        let path = PathBuf::from(format!("{cargo_manifest_dir}/themes/"));
        for entry in path.read_dir().unwrap() {
            let entry = entry.unwrap();
            let entry_path = entry.path();

            if entry_path.extension() != Some(OsStr::new("yml")) {
                continue;
            }

            println!(
                "Theme: {}",
                entry_path.file_name().unwrap().to_string_lossy()
            );

            let actual_theme = ThemeColors::from_yaml_file(&entry_path).unwrap();

            assert!(actual_theme.file_name.is_some_and(|v| !v.is_empty()));
        }
    }
}
