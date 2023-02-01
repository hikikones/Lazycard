use std::{
    path::{Path, PathBuf},
    rc::Rc,
};

use dioxus::prelude::ScopeState;

use sqlite::*;

pub type ConfigContext = Rc<Config>;

pub fn provide_config(cx: &ScopeState) -> &ConfigContext {
    let cfg = ConfigContext::new(Config::new());
    &*cx.use_hook(|| cx.provide_context(cfg))
}

pub fn use_config(cx: &ScopeState) -> &ConfigContext {
    cx.use_hook(|| cx.consume_context::<ConfigContext>().unwrap())
}

pub struct Config(Sqlite);

pub const ASSETS_DIR: &str = "assets";
const CONFIG_FILE: &str = "config.db";

impl Config {
    pub fn new() -> Self {
        let assets_dir = Path::new(ASSETS_DIR);
        if !assets_dir.exists() {
            std::fs::create_dir(assets_dir).unwrap();
        }

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
