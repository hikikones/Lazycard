mod app;
mod pages;
mod terminal;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Args = clap::Parser::parse();
    let db = database::Database::new(args.database, args.desired_retention as f32 / 100.0)?;

    let terminal = terminal::Terminal::init()?;

    let mut app = app::App::new(db, args.external_editor);
    let res = app.run(terminal);
    app.quit()?;

    terminal::Terminal::restore()?;

    res
}

#[derive(Debug, clap::Parser)]
#[command(version, about, styles = CLAP_STYLING)]
struct Args {
    /// Where to store your cards [example: ~/lazycard.ron]
    #[arg(value_name = "DATABASE_FILE.ron", value_hint = clap::ValueHint::FilePath)]
    database: std::path::PathBuf,

    /// Desired retention in percent for your cards
    #[arg(long, value_name = "PERCENT", default_value_t = 80, value_parser = clap::value_parser!(u8).range(0..=100))]
    desired_retention: u8,

    /// Write cards in your default text editor
    #[clap(long, action)]
    external_editor: bool,
}

const CLAP_STYLING: clap::builder::styling::Styles = clap::builder::styling::Styles::styled()
    .header(clap_cargo::style::HEADER)
    .usage(clap_cargo::style::USAGE)
    .literal(clap_cargo::style::LITERAL)
    .placeholder(clap_cargo::style::PLACEHOLDER)
    .error(clap_cargo::style::ERROR)
    .valid(clap_cargo::style::VALID)
    .invalid(clap_cargo::style::INVALID);
