use crate::gather;
use crate::models;
use crate::utils;

use gather::*;
use log::info;
use models::*;
use reqwest::blocking::Client;
use std::error::Error;
use sysinfo::{RefreshKind, System, SystemExt};
use utils::syslog;

/// Collect all the metrics and send them to the server instance
pub fn collect_and_send(sys: &mut System, client: &Client) -> Result<(), Box<dyn Error>> {
    // Refresh data within the sys
    //sys.refresh_all();
    sys.refresh_specifics(
        RefreshKind::new()
            .with_disks_list()
            .with_processes()
            .with_components_list()
            .with_cpu(),
    );

    let mcpuf: i64 = match cpuid::clock_frequency() {
        Some(val) => val.into(),
        None => 0,
    };

    // Construct the Data structure with all the info needed
    let data = Data {
        os: get_os_version(),
        hostname: get_hostname(),
        uptime: get_uptime(&sys),
        uuid: get_uuid(),
        cpu_freq: mcpuf,
        load_avg: get_avg_load(&sys),
        user: get_logged_user(),
        sensors: get_senors_data(&sys),
        disks: get_disks_data(&sys),
        mac_address: get_mac_address(),
    };

    // Prepare to send
    // Get the url (where to send)
    let mut url: String = String::new();
    match std::env::var("api_url") {
        Ok(val) => url.push_str(&val),
        Err(x) => {
            syslog(x.to_string(), true, true, false);
        }
    };
    // Get the token (Authorization)
    let mut token: String = String::new();
    match std::env::var("api_token") {
        Ok(val) => token.push_str(&val),
        Err(x) => {
            syslog(x.to_string(), true, true, false);
        }
    };

    // Send the request
    let res = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", token))
        .json(&data)
        .send();

    // Analyze the output and log it + send to sentry in case of error
    match res {
        Ok(res) => info!("return status : {}", res.status()),
        Err(x) => {
            sentry::capture_error(&x);
            syslog(format!("calling error : {}", x), false, true, false);
        }
    }
    Ok(())
}
