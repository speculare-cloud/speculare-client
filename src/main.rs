extern crate sys_info;
extern crate sysinfo;

use std::process::Command;

use sys_info::hostname;

use sysinfo::ComponentExt;
use sysinfo::{ProcessorExt, System, SystemExt};

/* Get uptime of the computer in a special format */
fn get_uptime(sys: &System) -> String {
    let mut uptime = sys.get_uptime();
    let days = uptime / 86400;
    uptime -= days * 86400;
    let hours = uptime / 3600;
    uptime -= hours * 3600;
    let minutes = uptime / 60;
    return format!("{} days {} hours {} minutes", days, hours, minutes);
}

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

fn main() {
    let sys = System::new_all();

    let os_release = os_version::detect().unwrap();
    println!("os: {}", os_release.to_string());

    let hostname = hostname().unwrap();
    println!("hostname: {}", hostname);
    
    let uptime = get_uptime(&sys);
    println!("{}", uptime);
    
    let uuid = machine_uid::get().unwrap();
    println!("uuid {}", uuid);
    
    println!(
        "Memory: {}/{} MB",
        sys.get_used_memory() / 1000,
        sys.get_total_memory() / 1000
    );

    println!(
        "CPU : {} MHz, {} %",
        sys.get_processors()[0].get_frequency(),
        sys.get_global_processor_info().get_cpu_usage()
    );

    for component in sys.get_components() {
        println!(
            "{}: {}°C",
            component.get_label(),
            component.get_temperature()
        );
    }
    let logged_users = get_logged_user();
    println!("Users: {}", logged_users);
    
    let mac_address = get_mac_address("48:".to_string());
    println!("MAC: {}", mac_address);
}
