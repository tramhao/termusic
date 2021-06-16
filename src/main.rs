mod app;
mod lrc;
mod player;
mod song;
mod ui;

use app::App;

const MUSIC_DIR: &str = "~/Music";

fn main() {
    let mut app: App = App::new();
    app.run();
}
