use configr::{Config, Configr};
use serde::{Deserialize, Serialize};

pub const MUSIC_DIR: &str = "~/Music";

#[derive(Clone, Configr, Deserialize, Serialize)]
pub struct TermusicConfig {
    pub music_dir: String,
}
impl Default for TermusicConfig {
    fn default() -> Self {
        TermusicConfig {
            music_dir: MUSIC_DIR.to_string(),
        }
    }
}
