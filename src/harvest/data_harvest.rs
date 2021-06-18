use crate::options::{Plugin, PluginsMap};

use chrono::prelude::Utc;
use serde::Serialize;
use sys_metrics::{cpu::*, disks::*, host::*, memory::*, network::*};

#[derive(Debug, Clone, Serialize)]
pub struct Data {
    pub uuid: String,
    pub system: String,
    pub os_version: String,
    pub hostname: String,
    pub uptime: i64,
    pub cpu_stats: Option<CpuStats>,
    pub cpu_times: Option<CpuTimes>,
    pub load_avg: Option<LoadAvg>,
    pub disks: Option<Vec<Disks>>,
    pub ioblocks: Option<Vec<IoBlock>>,
    pub memory: Option<Memory>,
    pub swap: Option<Swap>,
    pub ionets: Option<Vec<IoNet>>,
    pub created_at: chrono::NaiveDateTime,
    pub plugins: Vec<Plugin>,
}

impl Default for Data {
    fn default() -> Self {
        trace!("Init the default Data");
        let host_info = get_host_info()
            .unwrap_or_else(|err| panic!("Cannot get host_info of the host:{}", err));

        // UUID can still be empty on some Linux platform (such as WSL)
        let mut uuid =
            get_uuid().unwrap_or_else(|err| panic!("Cannot get UUID of the host:{}", err));
        // As a workaround for blank UUID, set the uuid to be the sha1 of hostname
        if uuid.is_empty() {
            uuid = sha1::Sha1::from(&host_info.hostname).digest().to_string();
        }

        Data {
            uuid,
            system: host_info.system,
            os_version: host_info.os_version,
            hostname: host_info.hostname,
            uptime: 0,
            cpu_stats: None,
            cpu_times: None,
            load_avg: None,
            disks: None,
            ioblocks: None,
            memory: None,
            swap: None,
            ionets: None,
            created_at: Utc::now().naive_local(),
            plugins: Vec::new(),
        }
    }
}

impl Data {
    /// Get each common metrics and "save" them in the Data struct
    pub fn eat_data(&mut self, load_avg: bool) {
        let eat_data_time = Utc::now().naive_local();
        trace!("eat_data: {:?}", eat_data_time);

        // Get the main host information (os, hostname, ...)
        let host_info = get_host_info()
            .unwrap_or_else(|err| panic!("Cannot get host_info of the host:{}", err));
        // Assign self value to the value from host_info
        // Convert to i64, cause as of now the server doesn't handle u64
        self.uptime = host_info.uptime as i64;
        // Get the cpustats info (interrupts, ...)
        self.cpu_stats = match get_cpustats() {
            Ok(cpustats) => Some(cpustats),
            Err(err) => {
                error!("[Eating] CpuStats fetching error: {}", err);
                None
            }
        };
        // Get the cputimes info (user, idle, ...)
        self.cpu_times = match get_cputimes() {
            Ok(cputimes) => Some(cputimes),
            Err(err) => {
                error!("[Eating] CpuTimes fetching error: {}", err);
                None
            }
        };
        // Get the avg cpu load for 1, 5, 15mins
        // Don't update everytime, depend on the config info
        self.load_avg = if load_avg {
            Some(host_info.loadavg)
        } else {
            None
        };
        // Get the disks info (mount_path, used, ...) for physical disks
        self.disks = match get_partitions_physical() {
            Ok(partitions_phy) => Some(partitions_phy),
            Err(err) => {
                error!("[Eating] Disks fetching error: {}", err);
                None
            }
        };
        // Get the iostats (read/wrtn, ...) for physical disks
        self.ioblocks = match get_physical_ioblocks() {
            Ok(ioblocks) => Some(ioblocks),
            Err(err) => {
                error!("[Eating] Ioblocks fetching error: {}", err);
                None
            }
        };
        // Get the memory info (total, free, cached, ...)
        self.memory = match get_memory() {
            Ok(memory) => Some(memory),
            Err(err) => {
                error!("[Eating] Memory fetching error: {}", err);
                None
            }
        };
        // Get the swap info (total, free, ...)
        self.swap = match get_swap() {
            Ok(swap) => Some(swap),
            Err(err) => {
                error!("[Eating] Swap fetching error: {}", err);
                None
            }
        };
        // Get the network (physical) iocounters
        self.ionets = match get_physical_ionets() {
            Ok(ionets) => Some(ionets),
            Err(err) => {
                error!("[Eating] Ionets fetching error: {}", err);
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
        self.plugins.push(plugin);
    }

    /// Clear previous Plugin struct from the plugins field of Data
    pub fn clear_plugins(&mut self) {
        self.plugins.clear();
    }
}
