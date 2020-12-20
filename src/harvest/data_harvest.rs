use serde::Serialize;
use std::time::Instant;
use sys_metrics::{cpu::*, disks::*, host::*};
use sys_metrics::{Disks, IoStats, LoadAvg, Memory};

#[derive(Debug, Serialize)]
pub struct Data {
    pub uuid: String,
    pub os: String,
    pub hostname: String,
    pub uptime: i64,
    pub cpu_freq: i64,
    pub load_avg: Option<LoadAvg>,
    pub disks: Option<Vec<Disks>>,
    pub iostats: Option<Vec<IoStats>>,
    pub memory: Option<Memory>,
    pub users: Option<Vec<String>>,
}

impl Default for Data {
    fn default() -> Self {
        trace!("Init the default Data");
        let host_info = get_host_info()
            .unwrap_or_else(|err| panic!("Cannot get host_info of the host:{}", err));

        Data {
            uuid: get_uuid().unwrap_or_else(|err| panic!("Cannot get UUID of the host:{}", err)),
            os: host_info.os_version,
            hostname: host_info.hostname,
            uptime: 0,
            cpu_freq: -1,
            memory: None,
            load_avg: None,
            disks: None,
            iostats: None,
            users: None,
        }
    }
}

impl Data {
    pub fn eat_data(&mut self) {
        trace!("Eating data now...");
        let eating_time = Instant::now();
        trace!("Eating_time: {:?}", eating_time);

        // Get the main host information (os, hostname, ...)
        let host_info = get_host_info()
            .unwrap_or_else(|err| panic!("Cannot get host_info of the host:{}", err));
        // Assign self value to the value from host_info
        // Convert to i64, cause as of now the server doesn't handle u64
        self.uptime = host_info.uptime as i64;
        self.load_avg = Some(host_info.loadavg);
        self.memory = Some(host_info.memory);
        // Get the cpu_freq info (maybe more info in the future) or set as -1
        self.cpu_freq = get_cpufreq().unwrap_or(-1.0) as i64;
        // Get the disks info (mountpath, used, ...) for physical disks
        self.disks = match get_partitions_physical() {
            Ok(partitions_phy) => Some(partitions_phy),
            Err(err) => {
                error!("[NF] Disks fetching error: {}", err);
                None
            }
        };
        // Get the iostats (read/wrtn) for physical disks
        self.iostats = match get_iostats_physical() {
            Ok(iostats_phy) => Some(iostats_phy),
            Err(err) => {
                error!("[NF] Iostats fetching error: {}", err);
                None
            }
        };
        // Get the users currently connected
        self.users = match get_users() {
            Ok(curr_users) => Some(curr_users),
            Err(err) => {
                error!("[NF] Users fetching error: {}", err);
                None
            }
        }
    }
}
