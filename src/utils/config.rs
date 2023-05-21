use crate::Args;

use super::cget_uuid;

use clap::Parser;
use config::ConfigError;
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]

pub struct Config {
    #[serde(skip)]
    pub uuid: String,
    // ENDPOINT SETTINGS
    pub api_token: String,
    pub api_url: String,

    // HARVEST INTERVALS
    #[serde(default = "default_harvest")]
    pub harvest_interval: u64,
    #[serde(default = "default_syncing")]
    pub syncing_interval: u64,
    #[serde(default = "default_loadavg")]
    pub loadavg_interval: u64,

    // GLOBAL SETTINGS
    #[serde(default = "default_cache_size")]
    pub cache_size: i64,
    #[cfg(feature = "auth")]
    pub sso_url: String,
}

impl Config {
    pub fn new() -> Result<Self, ConfigError> {
        let args = Args::parse();

        let config_builder = config::Config::builder().add_source(config::File::new(
            &args
                .config_path
                .unwrap_or_else(|| "/etc/speculare/client.config".to_owned()),
            config::FileFormat::Toml,
        ));

        let mut config: Self = config_builder.build()?.try_deserialize()?;
        config.uuid = cget_uuid();

        assert!(config.harvest_interval > 0);
        assert!(config.syncing_interval > 0);
        assert!(config.loadavg_interval > 0);

        Ok(config)
    }
}

fn default_harvest() -> u64 {
    1
}

fn default_syncing() -> u64 {
    1
}

fn default_loadavg() -> u64 {
    5
}

fn default_cache_size() -> i64 {
    16
}
