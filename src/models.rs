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
pub struct Data {
    pub os: String,
    pub hostname: String,
    pub uptime: i64,
    pub uuid: String,
    pub cpu_freq: i64,
    pub load_avg: LoadAvg,
    pub sensors: Vec<Sensors>,
    pub disks: Vec<Disks>,
    pub user: String,
    pub mac_address: String,
}
