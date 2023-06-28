use config::{Config, ConfigError};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub database: Database,
    pub discord: Discord,
}

impl Settings {
    pub fn try_load() -> Result<Self, ConfigError> {
        let cfg = Config::builder()
            .add_source(config::File::with_name("config").required(false))
            .add_source(config::Environment::with_prefix("DEMOCRACY").separator("_"))
            .build()?;

        cfg.try_deserialize()
    }
}

#[derive(Debug, Deserialize)]
pub struct Database {
    pub url: String,
    pub schema: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Discord {
    pub token: String,
}
