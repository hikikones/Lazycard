mod app;
mod database;
mod markup;
mod pages;
mod utils;

fn main() -> std::io::Result<()> {
    let terminal = ratatui::init();
    let app = app::App::new().run(terminal);
    ratatui::restore();
    app
}
