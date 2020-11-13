use crate::models;

use models::LoadAvg;
use psutil::host;
use std::io::{Error, ErrorKind};

/// Return the avg cpu_freq across all core as i64.
pub fn get_avg_cpufreq() -> i64 {
    match cpuid::clock_frequency() {
        Some(val) => val as i64,
        None => 0,
    }
}

/// Return LoadAvg struct containing the 1, 5 and 15 percentil cpu average load.
#[cfg(target_family = "unix")]
pub fn get_avg_load() -> Result<LoadAvg, Error> {
    match host::loadavg() {
        Ok(val) => Ok(LoadAvg {
            one: val.one,
            five: val.five,
            fifteen: val.fifteen,
        }),
        Err(x) => Err(Error::new(ErrorKind::Other, x)),
    }
}
