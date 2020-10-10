use serde::Serialize;

#[derive(Serialize)]
pub struct Sensors {
    pub label: String,
    pub temp: f32,
}

#[derive(Serialize)]
pub struct Data {
    pub os: String,
    pub hostname: String,
    pub uptime: u64,
    pub uuid: String,
    pub cpu_freq: u64,
    pub sensors: Vec<Sensors>,
    pub user: String,
    pub mac_address: String,
}
