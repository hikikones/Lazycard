pub mod app;
pub mod database;
pub mod pages;
pub mod settings;
pub mod symbols;
pub mod terminal;

pub const APP_NAME: &str = env!("CARGO_PKG_NAME");
pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const APP_QUALIFIER: &str = "org";
pub const APP_ORGANIZATION: &str = "hikikones";
