use lazycard::{app::App, database::Database, terminal::Terminal};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Args = clap::Parser::parse();

    let cell_size = match widgets::CellSize::query() {
        Ok(cell_size) => cell_size,
        Err(err) => return Err(format!("Failed to get cell size due to {}", err))?,
    };

    let Some(database_file) = args.database.or_else(|| get_database_file()) else {
        return Err("No database file specified or a \
        default one could not be retrieved from the operating system")?;
    };
    let Some(assets_dir) = args.assets.or_else(|| get_assets_dir()) else {
        return Err("No assets directory specified or a \
        default one could not be retrieved from the operating system")?;
    };

    // Make sure assets dir is created
    if !assets_dir.exists() {
        std::fs::create_dir_all(&assets_dir).map_err(|err| {
            format!(
                "Failed to create assets directory at \"{}\" due to {}",
                assets_dir.display(),
                err
            )
        })?;
    }
    // If already exists, make sure it is actually a dir
    else if !assets_dir.is_dir() {
        return Err(format!(
            "Specified assets directory \"{}\" is not a directory",
            assets_dir.display()
        ))?;
    }

    let db = match Database::open(&database_file) {
        Ok(db) => db,
        Err(err) => {
            return Err(format!(
                "Failed to open database \"{}\" due to {}",
                database_file.display(),
                err,
            ))?;
        }
    };

    let terminal = Terminal::init()?;

    let mut app = App::new(db, cell_size, assets_dir, args.settings);
    let res = app.run(terminal);
    app.quit()?;

    Terminal::restore()?;

    res
}

fn get_database_file() -> Option<std::path::PathBuf> {
    const FILENAME: &str = "database.db";
    directories::ProjectDirs::from(
        lazycard::APP_QUALIFIER,
        lazycard::APP_ORGANIZATION,
        lazycard::APP_NAME,
    )
    .map(|project_dirs| project_dirs.config_dir().join(FILENAME))
}

fn get_assets_dir() -> Option<std::path::PathBuf> {
    const DIRNAME: &str = "assets";
    directories::ProjectDirs::from(
        lazycard::APP_QUALIFIER,
        lazycard::APP_ORGANIZATION,
        lazycard::APP_NAME,
    )
    .map(|project_dirs| project_dirs.config_dir().join(DIRNAME))
}

#[derive(Debug, clap::Parser)]
#[command(version, about, styles = CLAP_STYLING)]
struct Args {
    /// Optional path for your database file. By default,
    /// the location will be determined by the conventions of your operating system.
    #[arg(long, value_name = "FILE.db", value_hint = clap::ValueHint::FilePath)]
    database: Option<std::path::PathBuf>,

    /// Optional path for your assets directory. By default,
    /// the location will be determined by the conventions of your operating system.
    #[arg(long, value_name = "DIR", value_hint = clap::ValueHint::DirPath)]
    assets: Option<std::path::PathBuf>,

    /// Optional path for your settings file. By default,
    /// the location will be determined by the conventions of your operating system.
    #[arg(long, value_name = "FILE.toml", value_hint = clap::ValueHint::FilePath)]
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
