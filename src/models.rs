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
use utils::syslog;

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

pub struct Global {
    pub mthread: Option<thread::JoinHandle<()>>,
    pub alive: sync::Arc<AtomicBool>,
    pub skip: sync::Arc<AtomicBool>,
}

impl Global {
    pub fn start(&mut self, interval: Option<u64>) {
        self.alive.store(true, Ordering::SeqCst);
        let alive = self.alive.clone();
        let skip = self.skip.clone();

        let interval = if interval.is_some() {
            interval.unwrap()
        } else {
            300 // 300 default is 5 mins
        };

        self.mthread = Some(thread::spawn(move || {
            while alive.load(Ordering::SeqCst) {
                if !skip.load(Ordering::SeqCst) {
                    match collect_and_send() {
                        Ok(x) => x,
                        Err(x) => syslog(x.to_string(), false, true),
                    };
                }
                thread::sleep(Duration::from_secs(interval));
            }
        }));
    }

    pub fn burst_on(&mut self) {
        self.skip.store(true, Ordering::SeqCst);
    }

    pub fn burst_off(&mut self) {
        self.skip.store(false, Ordering::SeqCst);
    }

    pub fn stop(&mut self) {
        self.alive.store(false, Ordering::SeqCst);
        self.mthread
            .take()
            .expect("Called stop on non-running thread")
            .join()
            .expect("Could not join spawned thread");
    }
}
