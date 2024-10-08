mod app;
mod database;
mod pages;
mod typst;

fn main() -> std::io::Result<()> {
    let terminal = ratatui::init();
    let app = app::App::new().run(terminal);
    ratatui::restore();
    app
}
