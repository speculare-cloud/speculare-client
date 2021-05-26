use serde::{Deserialize, Serialize};
use std::{collections::HashMap, io::Error};

pub type PluginsMap = HashMap<String, PluginInfo>;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub api_token: String,
    pub api_url: String,
    pub harvest_interval: u64,
    pub syncing_interval: u64,
    pub loadavg_interval: u64,
    pub plugins_path: String,
}

#[derive(Debug)]
pub struct PluginInfo {
    pub lib: libloading::Library,
    pub func: fn() -> Result<String, Error>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Plugin {
    pub key: String,
    pub val: String,
}

pub mod config;
pub use self::config::*;

pub mod config_prompt;
pub use self::config_prompt::*;

pub mod plugins_init;
pub use self::plugins_init::*;
