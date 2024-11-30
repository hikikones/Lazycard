use std::path::PathBuf;

pub struct Settings {
    pub database: PathBuf,
    pub desired_retention: f32,
}
