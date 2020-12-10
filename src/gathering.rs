use crate::models::Data;

use sys_metrics::{cpu::*, disks::*, host::*};

/// Collect the metrics
///
/// Assume this function don't return a Result<> because if this
/// function fail, the whole app won't run. So it's better to panic here.
pub fn gathering() -> Data {
    let host_info = match get_host_info() {
        Ok(val) => val,
        Err(x) => panic!("Cannot get the host_info: {}", x),
    };

    Data {
        uuid: match get_uuid() {
            Ok(val) => val,
            Err(x) => panic!("Cannot get the UUID of the machine: {}", x),
        },
        os: host_info.os_version,
        hostname: host_info.hostname,
        uptime: host_info.uptime as i64,
        cpu_freq: get_cpufreq().unwrap_or(-1.0) as i64,
        load_avg: host_info.loadavg,
        disks: match get_partitions_physical() {
            Ok(val) => Some(val),
            Err(_) => None,
        },
        iostats: match get_iostats_physical() {
            Ok(val) => Some(val),
            Err(_) => None,
        },
        memory: host_info.memory,
        users: match get_users() {
            Ok(val) => Some(val),
            Err(_) => None,
        },
    }
}
