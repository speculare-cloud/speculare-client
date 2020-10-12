use crate::models;
use crate::utils;

use models::Sensors;
use std::process::Command;
use sys_info::hostname;
use sysinfo::ComponentExt;
use sysinfo::{System, SystemExt};
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
    return match mac_address {
        Ok(val) => String::from_utf8_lossy(&val.stdout).to_string(),
        Err(x) => {
            syslog(x.to_string(), false);
            x.to_string()
        }
    };
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
    return match mac_address {
        Ok(val) => String::from_utf8_lossy(&val.stdout).to_string(),
        Err(x) => {
            syslog(x.to_string(), false, true);
            x.to_string()
        }
    };
}

/* Get the user currently logged, if more than 1 user, return the last one */
pub fn get_logged_user() -> String {
    let logged_users = Command::new("bash")
        .arg("-c")
        .arg("users | awk -F' ' '{print $NF}' | tr -d '\n'")
        .output();
    return match logged_users {
        Ok(val) => String::from_utf8_lossy(&val.stdout).to_string(),
        Err(x) => {
            syslog(x.to_string(), false, true);
            x.to_string()
        }
    };
}

/* Get the os version (Mac/Linux/Windows) in a safe String */
pub fn get_os_version() -> String {
    let os_release = os_version::detect();
    return match os_release {
        Ok(val) => val.to_string(),
        Err(x) => {
            syslog(x.to_string(), false, true);
            x.to_string()
        }
    };
}

/* Get the hostname (Mac/Linux/Windows) in a safe String */
pub fn get_hostname() -> String {
    return match hostname() {
        Ok(val) => val.to_string(),
        Err(x) => {
            syslog(x.to_string(), false, true);
            x.to_string()
        }
    };
}

/* Get the uuid of the host (Mac/Linux/Windows) in a safe String */
pub fn get_uuid() -> String {
    return match machine_uid::get() {
        Ok(val) => val.to_string(),
        Err(x) => {
            syslog(x.to_string(), false, true);
            x.to_string()
        }
    };
}

/* Retrieve sensors data in form of vector */
pub fn get_senors_data(sys: &System) -> Vec<Sensors> {
    let components = sys.get_components();
    let mut sensors: Vec<Sensors> = Vec::with_capacity(components.len());
    for component in components {
        sensors.push(Sensors {
            label: component.get_label().to_string(),
            temp: component.get_temperature(),
        })
    }
    return sensors;
}
