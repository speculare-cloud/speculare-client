use crate::options::{Plugin, PluginsMap};

use chrono::prelude::Utc;
use serde::Serialize;
use std::cell::RefCell;
use sys_metrics::{cpu::*, disks::*, host::*};
use sys_metrics::{Disks, IoStats, LoadAvg, Memory};

#[derive(Debug, Clone, Serialize)]
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
    pub created_at: chrono::NaiveDateTime,
    pub plugins: RefCell<Vec<Plugin>>,
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
            created_at: Utc::now().naive_local(),
            plugins: RefCell::new(Vec::new()),
        }
    }
}

impl Data {
    /// Get each common metrics and "save" them in the Data struct
    pub fn eat_data(&mut self) {
        let eat_data_time = Utc::now().naive_local();
        trace!("eat_data: {:?}", eat_data_time);

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
        // Set the time at which this has been created
        self.created_at = eat_data_time;
    }

    /// Get each plugins metrics and "save" them in the Data struct
    pub fn eat_plugins(&mut self, plugins: &PluginsMap) {
        trace!("eat_plugins: {:?}", Utc::now().naive_local());
        // For each plugins, get their result and append to the Data
        for (key, val) in plugins {
            // Execute the entrypoint and get the return of it
            let res = match (val.func)() {
                Ok(res_func) => {
                    debug!("PLUGIN {} returned: {:?}", key, res_func);
                    res_func
                }
                Err(err) => {
                    error!("PLUGIN {} failed with: {}", key, err);
                    continue;
                }
            };
            // Add the plugin data to the Data struct
            self.add_plugin(Plugin {
                key: key.to_owned(),
                val: res,
            });
        }
    }

    /// Add Plugin struct (key/val) to the plugins field of Data
    pub fn add_plugin(&mut self, plugin: Plugin) {
        self.plugins.borrow_mut().push(plugin);
    }

    /// Clear previous Plugin struct from the plugins field of Data
    pub fn clear_plugins(&mut self) {
        self.plugins.borrow_mut().clear();
    }
}
