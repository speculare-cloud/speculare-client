use metrics_rs::models::{Disks, LoadAvg, Memory, Sensors};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub api_token: String,
    pub api_url: String,
}

#[derive(Debug, Serialize)]
pub struct StaticData {
    pub uuid: String,
    pub os: String,
    pub hostname: String,
    pub mac_address: String,
}

#[derive(Debug, Serialize)]
pub struct DynData {
    pub uuid: String,
    pub uptime: i64,
    pub cpu_freq: i64,
    pub load_avg: LoadAvg,
    pub sensors: Vec<Sensors>,
    pub disks: Vec<Disks>,
    pub memory: Memory,
    pub users: Vec<String>,
}
