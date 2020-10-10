extern crate sys_info;
extern crate sysinfo;

mod models;

use crate::models::*;

use std::process::Command;
use sys_info::hostname;
use std::error::Error;
use sysinfo::ComponentExt;
use sysinfo::{ProcessorExt, System, SystemExt};

/* Retrieve the MAC address using the prefix as target [78: for 19's iMac] */
fn get_mac_address(prefix: String) -> String {
    let mac_address = Command::new("bash")
        .arg("-c")
        .arg(format!(
            "ifconfig | grep {} | awk {} | tr -d '\n'",
            prefix, "'{print $2}'"
        ))
        .output();
    return match mac_address {
        Ok(val) => String::from_utf8_lossy(&val.stdout).to_string(),
        Err(x) => x.to_string(),
    };
}

/* Get the user currently logged, if more than 1 user, return the last one */
fn get_logged_user() -> String {
    let logged_users = Command::new("bash")
        .arg("-c")
        .arg("users | awk -F' ' '{print $NF}' | tr -d '\n'")
        .output();
    return match logged_users {
        Ok(val) => String::from_utf8_lossy(&val.stdout).to_string(),
        Err(x) => x.to_string(),
    };
}

/* Get the os version (Mac/Linux/Windows) in a safe String */
fn get_os_version() -> String {
    let os_release = os_version::detect();
    return match os_release {
        Ok(val) => val.to_string(),
        Err(x) => x.to_string()
    }
}

/* Get the hostname (Mac/Linux/Windows) in a safe String */
fn get_hostname() -> String {
    return match hostname() {
        Ok(val) => val.to_string(),
        Err(x) => x.to_string()
    }
}

/* Get the uuid of the host (Mac/Linux/Windows) in a safe String */
fn get_uuid() -> String {
    return match machine_uid::get() {
        Ok(val) => val.to_string(),
        Err(x) => x.to_string()
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let sys = System::new_all();
    let components = sys.get_components();
    let mut sensors: Vec<Sensors> = Vec::with_capacity(components.len());
    for component in components {
        sensors.push(Sensors {
            label: component.get_label().to_string(),
            temp: component.get_temperature(),
        })
    }

    // TODO - Replace the 48: in a env file
    let data = Data {
        os: get_os_version(),
        hostname: get_hostname(),
        uptime: sys.get_uptime(),
        uuid: get_uuid(),
        cpu_freq: sys.get_processors()[0].get_frequency(),
        user: get_logged_user(),
        sensors: sensors,
        mac_address: get_mac_address("48:".to_string()),
    };

    let client = reqwest::Client::new();
    // TODO - Change the URL in a env file
    let res = client
        .post("https://enmy1ryyhupko.x.pipedream.net")
        .json(&data)
        .send()
        .await?;

    // TODO - Save log to the syslog server
    println!("Return code [{}]", res.status());
    Ok(())
}
