use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub api_token: String,
    pub api_url: String,
}

pub mod config;
pub use self::config::*;

pub mod config_prompt;
pub use self::config_prompt::*;
