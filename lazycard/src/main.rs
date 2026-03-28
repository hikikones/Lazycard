use clap::Parser;

mod app;
mod pages;
mod terminal;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let db = database::Database::new(
        std::path::PathBuf::from(args.database),
        args.desired_retention as f32 / 100.0,
    )?;

    let terminal = terminal::Terminal::init()?;

    let mut app = app::App::new(db, args.external_editor);
    let res = app.run(terminal);
    app.quit()?;

    terminal::Terminal::restore()?;

    res
}

/// A flashcard application for the terminal
#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
struct Args {
    /// Where to store your cards [example: ~/lazycard.ron]
    #[arg(value_name = "DATABASE_FILE", value_hint = clap::ValueHint::FilePath)]
    database: String,

    /// Desired retention in percent for your cards
    #[arg(long, value_name = "PERCENT", default_value_t = 80, value_parser = clap::value_parser!(u8).range(0..=100))]
    desired_retention: u8,

    /// Write cards in your default text editor
    #[clap(long, action)]
    external_editor: bool,
}
