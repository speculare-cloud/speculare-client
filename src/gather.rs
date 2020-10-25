use crate::models;
use crate::sysinfo::DiskExt;
use crate::utils;

use models::{Disks, LoadAvg, Sensors};
use sentry::integrations::anyhow::capture_anyhow;
use std::process::Command;
use sys_info::hostname;
use sysinfo::{ComponentExt, System, SystemExt};
use utils::syslog;

/*
 *  MAC - linux specific get_mac_address
 *  using default interface in a safe string
 */
#[cfg(target_os = "linux")]
pub fn get_mac_address() -> String {
    let interface = Command::new("bash")
        .arg("-c")
        .arg("route | grep '^default' | grep -o '[^ ]*$' | sed '$!d' | tr -d '\n'")
        .output()
        .expect("failed to retrieve default interface");
    let mac_address = Command::new("bash")
        .arg("-c")
        .arg(format!(
            "ifconfig {} | grep 'ether ' | awk {} | tr -d '\n'",
            String::from_utf8_lossy(&interface.stdout),
            "'{print $2}'"
        ))
        .output();
    match mac_address {
        Ok(val) => String::from_utf8_lossy(&val.stdout).to_string(),
        Err(x) => {
            sentry::capture_error(&x);
            syslog(x.to_string(), false, true, false);
            x.to_string()
        }
    }
}

/*
 *  MAC - macos specific get_mac_address
 *  using default interface in a safe string
 */
#[cfg(target_os = "macos")]
pub fn get_mac_address() -> String {
    let interface = Command::new("bash")
        .arg("-c")
        .arg("route -n get default | grep 'interface:' | grep -o '[^ ]*$' | sed '$!d' | tr -d '\n'")
        .output()
        .expect("failed to retrieve default interface");
    let mac_address = Command::new("bash")
        .arg("-c")
        .arg(format!(
            "ifconfig {} | grep 'ether ' | awk {} | tr -d '\n'",
            String::from_utf8_lossy(&interface.stdout),
            "'{print $2}'"
        ))
        .output();
    match mac_address {
        Ok(val) => String::from_utf8_lossy(&val.stdout).to_string(),
        Err(x) => {
            sentry::capture_error(&x);
            syslog(x.to_string(), false, true, false);
            x.to_string()
        }
    }
}

/* Get the user currently logged, if more than 1 user, return the last one */
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

/* Get the os version (Mac/Linux/Windows) in a safe String */
pub fn get_os_version() -> String {
    let os_release = os_version::detect();
    match os_release {
        Ok(val) => val.to_string(),
        Err(x) => {
            capture_anyhow(&x);
            syslog(x.to_string(), false, true, false);
            x.to_string()
        }
    }
}

/* Get the hostname (Mac/Linux/Windows) in a safe String */
pub fn get_hostname() -> String {
    match hostname() {
        Ok(val) => val,
        Err(x) => {
            sentry::capture_error(&x);
            syslog(x.to_string(), false, true, false);
            x.to_string()
        }
    }
}

/* Get the uuid of the host (Mac/Linux/Windows) in a safe String */
pub fn get_uuid() -> String {
    match machine_uid::get() {
        Ok(val) => val,
        Err(x) => {
            syslog(x.to_string(), false, true, true);
            x.to_string()
        }
    }
}

/* Retrieve sensors data in form of vector */
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

/* Retrieve disks data in form of vector */
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

/* Return the avg load of the system in 1,5,15min */
pub fn get_avg_load(sys: &System) -> LoadAvg {
    let load_avg = sys.get_load_average();
    LoadAvg {
        one: load_avg.one,
        five: load_avg.five,
        fifteen: load_avg.fifteen,
    }
}
