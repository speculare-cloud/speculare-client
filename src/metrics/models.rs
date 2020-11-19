use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Sensors {
    pub label: String,
    pub temp: f64,
}

#[derive(Debug, Serialize)]
pub struct Disks {
    pub name: String,
    pub mount_point: String,
    pub total_space: i64,
    pub avail_space: i64,
}

#[derive(Debug, Serialize)]
pub struct IoStats {
    pub device_name: String,
    pub sectors_read: i64,
    pub sectors_wrtn: i64,
}

#[derive(Debug, Serialize)]
pub struct LoadAvg {
    pub one: f64,
    pub five: f64,
    pub fifteen: f64,
}

#[derive(Debug, Serialize)]
pub struct Memory {
    pub total_virt: i64,
    pub avail_virt: i64,
    pub total_swap: i64,
    pub avail_swap: i64,
}

#[derive(Debug)]
pub struct HostInfo {
    pub loadavg: LoadAvg,
    pub memory: Memory,
    pub os_version: String,
    pub hostname: String,
    pub uptime: i64,
}
