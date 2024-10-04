mod app;
mod database;
mod pages;

fn main() -> std::io::Result<()> {
    app::App::new().run(ratatui::init())?;
    ratatui::restore();

    Ok(())
}
