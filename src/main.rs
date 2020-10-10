extern crate sys_info;
extern crate sysinfo;

use std::process::Command;

use sys_info::hostname;

use sysinfo::{ProcessorExt, System, SystemExt};
use sysinfo::ComponentExt;

fn get_uptime(sys: &System) -> String {
    let mut uptime = sys.get_uptime();
    let days = uptime / 86400;
    uptime -= days * 86400;
    let hours = uptime / 3600;
    uptime -= hours * 3600;
    let minutes = uptime / 60;
    return format!("{} days {} hours {} minutes", days, hours, minutes);
}

fn main() {
    let sys = System::new_all();

    let hostname = hostname().unwrap();
    let os_release = os_version::detect().unwrap();

    let uuid = machine_uid::get().unwrap();

    println!("os: {}", os_release.to_string());
    println!("hostname: {}", hostname);
    println!("{}", get_uptime(&sys));
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
        println!("{}: {}°C", component.get_label(), component.get_temperature());
    }
    let logged_users = Command::new("bash")
        .arg("-c")
        .arg("users | awk -F' ' '{print $NF}' | tr -d '\n'")
        .output()
        .expect("failed to execute process");
    println!("Users: {}", String::from_utf8_lossy(&logged_users.stdout));

    // Get the mac address - only valid for the iMac at 19 as they start by 78:
    let mac_address = Command::new("bash")
        .arg("-c")
        .arg("ifconfig | grep 78: | awk -F' ' '{print $NF}' | tr -d '\n'")
        .output()
        .expect("failed to execute process");
    println!("MAC: {}", String::from_utf8_lossy(&mac_address.stdout));
}
