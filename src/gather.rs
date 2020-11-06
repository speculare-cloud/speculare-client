use crate::models;
use crate::utils;

use models::{Disks, LoadAvg, Sensors};
use psutil::sensors::TemperatureSensor;
use psutil::*;
use sentry::integrations::anyhow::capture_anyhow;
use std::process::Command;
use utils::syslog;

/// Return the default interface on Linux
#[cfg(target_os = "linux")]
fn get_default_interface() -> String {
    let interface = Command::new("bash")
        .arg("-c")
        .arg("route | grep '^default' | grep -o '[^ ]*$' | sed '$!d' | tr -d '\n'")
        .output()
        .expect("failed to retrieve default interface");
    String::from_utf8_lossy(&interface.stdout).to_string()
}

/// Return the default interface on MacOS
#[cfg(target_os = "macos")]
fn get_default_interface() -> String {
    let interface = Command::new("bash")
        .arg("-c")
        .arg("route -n get default | grep 'interface:' | grep -o '[^ ]*$' | sed '$!d' | tr -d '\n'")
        .output()
        .expect("failed to retrieve default interface");
    String::from_utf8_lossy(&interface.stdout).to_string()
}

/// Get the MAC Address (MacOS/Linux) in a safe String
/// Capture the error and send it to sentry + print it
/// TODO - Should change the return value in case of an error
#[cfg(target_os = "linux")]
pub fn get_mac_address() -> String {
    match mac_address::mac_address_by_name(&get_default_interface()) {
        Ok(Some(val)) => val.to_string(),
        Ok(None) => String::from("none"),
        Err(x) => {
            sentry::capture_error(&x);
            syslog(x.to_string(), false, true, false);
            x.to_string()
        }
    }
}

/// Get the MAC Address (Windows) in a safe String
/// Capture the error and send it to sentry + print it
/// TODO - Should change the return value in case of an error
#[cfg(target_os = "windows")]
pub fn get_mac_address() -> String {
    match mac_address::get_mac_address() {
        Ok(Some(val)) => val.to_string(),
        Ok(None) => String::from("none"),
        Err(x) => {
            sentry::capture_error(&x);
            syslog(x.to_string(), false, true, false);
            x.to_string()
        }
    }
}

/// Get the logged users and only keep the last/first one
/// Will be updated to return a Vec<String> instead
/// TODO - Should change the return value in case of an error
pub fn get_logged_user() -> String {
    let logged_users = Command::new("bash")
        .arg("-c")
        .arg("users | awk -F' ' '{print $NF}' | tr -d '\n'")
        .output();
    match logged_users {
        Ok(val) => String::from_utf8_lossy(&val.stdout).to_string(),
        Err(x) => {
            sentry::capture_error(&x);
            syslog(x.to_string(), false, true, false);
            x.to_string()
        }
    }
}

/// Get the os version (Mac/Linux/Windows) in a safe String
/// Capture the error and send it to sentry + print it
/// TODO - Should change the return value in case of an error
pub fn get_os_version() -> String {
    match os_version::detect() {
        Ok(val) => val.to_string(),
        Err(x) => {
            capture_anyhow(&x);
            syslog(x.to_string(), false, true, false);
            x.to_string()
        }
    }
}

/// Get the hostname (Mac/Linux/Windows) in a safe String
/// Capture the error and send it to sentry + print it
/// TODO - Should change the return value in case of an error
pub fn get_hostname() -> String {
    host::info().hostname().to_string()
}

/// Get the machine UUID (Mac/Linux/Windows) as a String
/// TODO - Should change the return value in case of an error
pub fn get_uuid() -> String {
    match machine_uid::get() {
        Ok(val) => val,
        Err(x) => {
            syslog(x.to_string(), false, true, true);
            x.to_string()
        }
    }
}

/// Retrieve the sensors and return them as a Vec<String>
pub fn get_senors_data() -> Vec<Sensors> {
    let temperatures: Vec<TemperatureSensor> = sensors::temperatures()
        .into_iter()
        .filter_map(Result::ok)
        .collect();
    let mut sensors: Vec<Sensors> = Vec::with_capacity(temperatures.len());
    for temp in temperatures {
        sensors.push(Sensors {
            label: temp.label().unwrap_or("?").to_string(),
            temp: f64::from(temp.current().celsius()),
        })
    }
    sensors
}

/// Retrieve the disks and return them as a Vec<Disks>
pub fn get_disks_data() -> Vec<Disks> {
    let partitions = disk::partitions_physical().unwrap();
    let mut vdisks: Vec<Disks> = Vec::with_capacity(partitions.len());
    for disk in partitions {
        let mount = disk.mountpoint();
        let disk_usage = disk::disk_usage(mount).unwrap();
        vdisks.push(Disks {
            name: disk.device().to_string(),
            mount_point: mount.display().to_string(),
            total_space: (disk_usage.total() / 100000) as i64,
            avail_space: (disk_usage.free() / 100000) as i64,
        })
    }
    vdisks
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

/// Return the uptime of the current host
/// In seconds and as i64 due to the database not handling u64
/// TODO - It's unsecure to cast to i64 but faster :(
pub fn get_uptime() -> i64 {
    host::uptime().unwrap().as_secs() as i64
}

/// Return the avg cpu_freq across all core as i64
pub fn get_avg_cpufreq() -> i64 {
    match cpuid::clock_frequency() {
        Some(val) => val.into(),
        None => 0,
    }
}
