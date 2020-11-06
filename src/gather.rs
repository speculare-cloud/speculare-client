use crate::models;
use crate::sysinfo::DiskExt;
use crate::utils;

use models::{Disks, LoadAvg, Sensors};
use sentry::integrations::anyhow::capture_anyhow;
use std::process::Command;
use sysinfo::{ComponentExt, ProcessorExt, System, SystemExt};
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
    match hostname::get() {
        Ok(val) => val.to_string_lossy().to_string(),
        Err(x) => {
            sentry::capture_error(&x);
            syslog(x.to_string(), false, true, false);
            x.to_string()
        }
    }
}

/// Get the machine UUID (Mac/Linux/Windows) as a String
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
pub fn get_senors_data(sys: &System) -> Vec<Sensors> {
    let components = sys.get_components();
    let mut sensors: Vec<Sensors> = Vec::with_capacity(components.len());
    for component in components {
        sensors.push(Sensors {
            label: component.get_label().to_string(),
            temp: f64::from(component.get_temperature()),
        })
    }
    sensors
}

/// Retrieve the disks and return them as a Vec<Disks>
pub fn get_disks_data(sys: &System) -> Vec<Disks> {
    let disks = sys.get_disks();
    let mut vdisks: Vec<Disks> = Vec::with_capacity(disks.len());
    for disk in disks {
        vdisks.push(Disks {
            name: disk.get_name().to_str().unwrap_or("?").to_string(),
            mount_point: disk.get_mount_point().display().to_string(),
            total_space: (disk.get_total_space() / 100000) as i64,
            avail_space: (disk.get_available_space() / 100000) as i64,
        })
    }
    vdisks
}

/// Return LoadAvg struct containing the 1, 5 and 15 percentil
/// cpu average load
pub fn get_avg_load(sys: &System) -> LoadAvg {
    let load_avg = sys.get_load_average();
    LoadAvg {
        one: load_avg.one,
        five: load_avg.five,
        fifteen: load_avg.fifteen,
    }
}

/// Return the uptime of the current host
/// In seconds and as i64 due to the database not handling u64
pub fn get_uptime(sys: &System) -> i64 {
    sys.get_uptime() as i64
}

/// Return the avg cpu_freq across all core as i64
pub fn get_avg_cpufreq(sys: &System) -> i64 {
    let mut cfm: i64 = 0;
    let mut item = 1;
    for cpu in sys.get_processors().iter() {
        cfm += cpu.get_frequency() as i64;
        item += 1;
    }
    cfm / item
}
