use crate::models;

use models::LoadAvg;
use psutil::host;

/// Return the avg cpu_freq across all core as i64
pub fn get_avg_cpufreq() -> i64 {
    match cpuid::clock_frequency() {
        Some(val) => val.into(),
        None => 0,
    }
}

/// Return LoadAvg struct containing the 1, 5 and 15 percentil cpu average load
pub fn get_avg_load() -> LoadAvg {
    let load_avg = host::loadavg().unwrap();
    LoadAvg {
        one: load_avg.one,
        five: load_avg.five,
        fifteen: load_avg.fifteen,
    }
}
