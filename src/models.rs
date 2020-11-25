use sys_metrics::{Disks, IoStats, LoadAvg, Memory};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub api_token: String,
    pub api_url: String,
}

#[derive(Debug, Serialize)]
pub struct Data {
    pub uuid: String,
    pub os: String,
    pub hostname: String,
    pub uptime: i64,
    pub cpu_freq: i64,
    pub load_avg: LoadAvg,
    pub disks: Option<Vec<Disks>>,
    pub iostats: Option<Vec<IoStats>>,
    pub memory: Memory,
    pub users: Option<Vec<String>>,
}
