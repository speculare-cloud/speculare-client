use crate::process;
use crate::utils;

use process::collect_and_send;
use serde::Serialize;
use std::{
    sync,
    sync::atomic::{AtomicBool, Ordering},
    thread,
    time::Duration,
};
use sysinfo::SystemExt;
use utils::syslog;

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

pub struct Global {
    pub mthread: Option<thread::JoinHandle<()>>,
    pub alive: sync::Arc<AtomicBool>,
}

impl Global {
    pub fn start(&mut self, interval: Option<u64>) {
        // Declare that the thread has been started
        self.alive.store(true, Ordering::SeqCst);

        // Clone the boolean to use them in the thread
        let alive = self.alive.clone();

        // 300, as default is 5 mins
        let interval = if interval.is_some() {
            interval.unwrap()
        } else {
            300
        };

        // Create a single instance of System
        // And refresh it in collect_and_send
        // Instead of creating a new one each time
        let mut sys = sysinfo::System::new();
        // Create the client instance for each loop
        // Do not create a new one each time
        let timeout = Duration::new(15, 0);
        let client = match reqwest::blocking::ClientBuilder::new()
            .timeout(timeout)
            .connect_timeout(timeout)
            .build()
        {
            Ok(val) => val,
            Err(x) => panic!(x),
        };

        // Start and save the thread in a Some
        // allowing us to check wether it exists (potentially)
        self.mthread = Some(thread::spawn(move || {
            // While the atomic boolean is true
            while alive.load(Ordering::SeqCst) {
                match collect_and_send(&mut sys, &client) {
                    Ok(x) => x,
                    Err(x) => syslog(x.to_string(), false, true, true),
                };

                // Sleep for the interval defined above
                // don't spam the CPU nor the server
                thread::sleep(Duration::from_secs(interval));
            }
        }));
    }
}
