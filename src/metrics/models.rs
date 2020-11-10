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
