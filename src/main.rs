mod app;
mod config;
mod lyric;
mod player;
mod song;
mod ui;

use anyhow::{anyhow, Result};
use app::App;
use config::TermusicConfig;
use configr::Config;

fn main() -> Result<()> {
    let mut path = dirs_next::home_dir()
        .map(|h| h.join(".config"))
        .ok_or_else(|| anyhow!("failed to find os config dir."))?;

    let config = match TermusicConfig::load("termusic", true) {
        Ok(c) => c,
        Err(_) => match TermusicConfig::load_custom("termusic", &mut path) {
            Ok(c) => c,
            Err(_) => TermusicConfig::default(),
        },
    };

    let mut app: App = App::new(config);
    app.run();
    Ok(())
}
