mod app;
mod pages;
mod settings;
mod symbols;
mod terminal;

const APP_NAME: &str = env!("CARGO_PKG_NAME");
const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
const APP_QUALIFIER: &str = "org";
const APP_ORGANIZATION: &str = "hikikones";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // use unicode_segmentation::UnicodeSegmentation;
    // let s = "The quick (\"brown\")  fox\nNew       line\r\nAnother *one*";
    // let w = s.split_word_bound_indices().collect::<Vec<(usize, &str)>>();
    // dbg!(w);

    // return Ok(());

    let args: Args = clap::Parser::parse();

    let Some(database_file) = args.database.or_else(|| get_database_file()) else {
        return Err("No database file path specified or a \
        default one could not be retrieved from the operating system")?;
    };
    let db = database::Database::new(database_file)?;

    let terminal = terminal::Terminal::init()?;

    let mut app = app::App::new(db, args.settings);
    let res = app.run(terminal);
    app.quit()?;

    terminal::Terminal::restore()?;

    res
}

fn get_database_file() -> Option<std::path::PathBuf> {
    const FILENAME: &str = "database.ron";
    directories::ProjectDirs::from(APP_QUALIFIER, APP_ORGANIZATION, APP_NAME)
        .map(|project_dirs| project_dirs.config_dir().join(FILENAME))
}

#[derive(Debug, clap::Parser)]
#[command(version, about, styles = CLAP_STYLING)]
struct Args {
    /// The path for your database file. If not set,
    /// the location will be determined by the conventions of your operating system.
    #[arg(long, value_name = "DATABASE_FILE.ron", value_hint = clap::ValueHint::FilePath)]
    database: Option<std::path::PathBuf>,

    /// The path for your settings file. If not set,
    /// the location will be determined by the conventions of your operating system.
    #[arg(long, value_name = "SETTINGS_FILE.toml", value_hint = clap::ValueHint::FilePath)]
    settings: Option<std::path::PathBuf>,
}

const CLAP_STYLING: clap::builder::styling::Styles = clap::builder::styling::Styles::styled()
    .header(clap_cargo::style::HEADER)
    .usage(clap_cargo::style::USAGE)
    .literal(clap_cargo::style::LITERAL)
    .placeholder(clap_cargo::style::PLACEHOLDER)
    .error(clap_cargo::style::ERROR)
    .valid(clap_cargo::style::VALID)
    .invalid(clap_cargo::style::INVALID);
