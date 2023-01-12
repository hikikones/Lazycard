use std::path::{Path, PathBuf};

use sqlite::*;

pub struct Config(Sqlite);

pub const ASSETS_DIR: &str = "assets";
const CONFIG_FILE: &str = "config.db";

impl Config {
    pub fn new() -> Self {
        let sqlite = Sqlite::open(CONFIG_FILE).unwrap();

        migrate(&sqlite);

        Self(sqlite)
    }

    pub fn get_database_path(&self) -> Option<PathBuf> {
        let path: Option<String> = self
            .0
            .fetch_one(
                "SELECT id, database_path FROM config WHERE id = 1",
                [],
                |row| row.get(1),
            )
            .unwrap();

        path.map(|p| p.into())
    }

    pub fn set_database_path(&self, path: impl AsRef<Path>) {
        self.0
            .execute_one(
                "UPDATE config SET database_path = ? WHERE id = 1",
                [path.as_ref().to_string_lossy()],
            )
            .unwrap();
    }
}

fn migrate(sqlite: &Sqlite) {
    const CURRENT_VERSION: SqliteId = 1;

    match sqlite.version() {
        CURRENT_VERSION => {}
        0 => {
            sqlite.execute_all(include_str!("schema.sql")).unwrap();
            sqlite.set_version(CURRENT_VERSION);
        }
        _ => todo!(),
    }
}
