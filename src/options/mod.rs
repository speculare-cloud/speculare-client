use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub api_token: String,
    pub api_url: String,
    pub harvest_interval: u64,
    pub syncing_interval: u64,
    pub plugins_path: String,
}

pub mod config;
pub use self::config::*;

pub mod config_prompt;
pub use self::config_prompt::*;
