use configr::{Config, Configr};
use serde::{Deserialize, Serialize};

pub const MUSIC_DIR: &str = "~/Music";

// Domain of invidious instance which you get from this list:
// https://github.com/iv-org/documentation/blob/master/Invidious-Instances.md
pub const INVIDIOUS_INSTANCE: &str = "https://vid.puffyan.us";

#[derive(Clone, Configr, Deserialize, Serialize)]
pub struct TermusicConfig {
    pub music_dir: String,
    pub invidious_instance: String,
}
impl Default for TermusicConfig {
    fn default() -> Self {
        TermusicConfig {
            music_dir: MUSIC_DIR.to_string(),
            invidious_instance: INVIDIOUS_INSTANCE.to_string(),
        }
    }
}
