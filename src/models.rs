use serde::Serialize;

#[derive(Serialize)]
pub struct Sensors {
    pub label: String,
    pub temp: f64,
}

#[derive(Serialize)]
pub struct Disks {
    pub name: String,
    pub mount_point: String,
    pub total_space: i64,
    pub avail_space: i64,
}

#[derive(Serialize)]
pub struct LoadAvg {
    pub one: f64,
    pub five: f64,
    pub fifteen: f64,
}

#[derive(Serialize)]
pub struct StaticData {
    pub uuid: String,
    pub os: String,
    pub hostname: String,
    pub mac_address: String,
}

#[derive(Serialize)]
pub struct DynData {
    pub uuid: String,
    pub uptime: i64,
    pub cpu_freq: i64,
    pub load_avg: LoadAvg,
    pub sensors: Vec<Sensors>,
    pub disks: Vec<Disks>,
}
