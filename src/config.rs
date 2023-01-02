use std::path::PathBuf;

pub struct Config {
    pub database: Option<PathBuf>,
}

impl Default for Config {
    fn default() -> Self {
        Self { database: None }
    }
}
