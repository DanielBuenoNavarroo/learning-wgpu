mod app;
mod game;
mod renderer;

fn main() {
    env_logger::init();
    let mut app = app::App::new();
    app.run();
}