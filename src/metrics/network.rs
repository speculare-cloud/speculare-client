use std::io::{Error, ErrorKind};
use std::process::Command;

/// Return the default interface on Linux.
///
/// SLOW.
#[cfg(target_os = "linux")]
fn get_default_interface() -> String {
    let interface = Command::new("bash")
        .arg("-c")
        .arg("route | grep '^default' | grep -o '[^ ]*$' | sed '$!d' | tr -d '\n'")
        .output()
        .expect("failed to retrieve default interface");
    String::from_utf8_lossy(&interface.stdout).to_string()
}

/// Return the default interface on MacOS.
///
/// SLOW.
#[cfg(target_os = "macos")]
fn get_default_interface() -> String {
    let interface = Command::new("bash")
        .arg("-c")
        .arg("route -n get default | grep 'interface:' | grep -o '[^ ]*$' | sed '$!d' | tr -d '\n'")
        .output()
        .expect("failed to retrieve default interface");
    String::from_utf8_lossy(&interface.stdout).to_string()
}

/// Get the MAC Address (MacOS/Linux) in a safe String.
///
/// WARNING - This function is slow due to the call with Command from get_default_interface.
#[cfg(target_family = "unix")]
pub fn get_mac_address() -> Result<String, Error> {
    match mac_address::mac_address_by_name(&get_default_interface()) {
        Ok(Some(val)) => Ok(val.to_string()),
        Ok(None) => Ok(String::from("unknown")),
        Err(x) => Err(Error::new(ErrorKind::Other, x)),
    }
}

/// Get the MAC Address (Windows) in a safe String.
///
/// WARNING - This function is slow due to the call with Command from get_default_interface.
#[cfg(target_family = "windows")]
pub fn get_mac_address() -> Result<String, Error> {
    match mac_address::get_mac_address() {
        Ok(Some(val)) => Ok(val.to_string()),
        Ok(None) => Ok(String::from("unknown")),
        Err(x) => Err(Error::new(ErrorKind::Other, x)),
    }
}
