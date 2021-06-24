mod app;
mod lyric;
mod player;
mod song;
mod ui;

use app::App;

const MUSIC_DIR: &str = "~/Music";

fn main() {
    let mut app: App = App::new();
    app.run();
}
