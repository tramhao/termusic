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


mod app;
mod config;
pub mod invidious;
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
