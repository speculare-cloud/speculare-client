use crate::utils;

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
