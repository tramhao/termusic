mod app;
mod utils;

use app::App;

fn main() {
    let mut app: App = App::new();
    app.run();
}
