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
}

impl Default for Data {
    fn default() -> Self {
        trace!("Init the default Data");
        let host_info = match get_host_info() {
            Ok(info) => info,
            Err(e) => {
                error!("cannot get the host_info(): {}", e);
                std::process::exit(1);
            }
        };

        // UUID can still be empty on some Linux platform (such as WSL)
        // as per https://man7.org/linux/man-pages/man5/machine-id.5.html
        // the machine-id should never be shared on the network or other.
        let uuid = match get_uuid() {
            Ok(uuid) => sha1_smol::Sha1::from(&uuid).hexdigest(),
            Err(_) => sha1_smol::Sha1::from(&host_info.hostname).hexdigest(),
        };

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
        }
    }
}

impl Data {
    /// Get each common metrics and "save" them in the Data struct
    pub fn eat_data(&mut self, load_avg: bool) {
        let eat_data_time = Utc::now().naive_local();
        trace!("eat_data: {:?}", eat_data_time);

        // Get the main host information (os, hostname, ...)
        let host_info = match get_host_info() {
            Ok(info) => info,
            Err(e) => {
                error!("cannot get the host_info(): {}", e);
                // TODO - Do we want to make this fatal?
                return;
            }
        };
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
}
