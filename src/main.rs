use clap::Parser;

mod app;
mod database;
mod editor;
mod markup;
mod pages;
mod utils;

fn main() -> std::io::Result<()> {
    let args = Args::parse();
    let db = database::Database::new(
        std::path::PathBuf::from(args.database),
        args.retention as f32 / 100.0,
    );

    let terminal = ratatui::init();
    let app = app::App::new(db);
    let res = app.run(terminal);
    ratatui::restore();
    res
}

/// A flashcard application for the terminal
#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    /// Where to store your cards [example: ~/lazycard.ron]
    #[clap(value_name = "DATABASE_FILE", value_hint = clap::ValueHint::FilePath)]
    database: String,

    /// Desired retention in percent for your cards
    #[arg(short, long, value_name = "PERCENT", default_value_t = 80, value_parser = clap::value_parser!(u8).range(0..=100))]
    retention: u8,
}
