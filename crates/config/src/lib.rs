use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Config {
    database: Option<PathBuf>,
}

pub const ASSETS_DIR: &str = "assets";
const CONFIG_FILE: &str = "config.toml";

impl Config {
    pub fn new() -> Self {
        let assets_path = Path::new(ASSETS_DIR);
        if !assets_path.exists() {
            std::fs::create_dir(assets_path).unwrap();
        }

        if let Ok(file) = std::fs::read(CONFIG_FILE) {
            if let Ok(config) = toml::from_slice::<Config>(file.as_ref()) {
                return config;
            }
        }

        Self { database: None }
    }

    pub fn database(&self) -> Option<&PathBuf> {
        self.database.as_ref()
    }

    pub fn set_database(&mut self, path: PathBuf) {
        self.database = Some(path);
        self.save();
    }

    fn save(&self) {
        let bytes = toml::to_vec(self).unwrap();
        let file = Path::new(CONFIG_FILE);
        std::fs::write(file, bytes).unwrap();
    }
}
