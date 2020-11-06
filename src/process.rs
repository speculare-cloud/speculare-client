use crate::gather;
use crate::models;
use crate::utils;

use gather::*;
use log::info;
use models::*;
use reqwest::blocking::Client;
use std::error::Error;
use sysinfo::{ProcessorExt, System, SystemExt};
use utils::syslog;

// TODO
// Cut this function is some small relevant function
pub fn collect_and_send(sys: &mut System, client: &Client) -> Result<(), Box<dyn Error>> {
    // Refresh data within the sys
    sys.refresh_all();

    // Construct the Data structure with all the info needed
    let data = Data {
        os: get_os_version(),
        hostname: get_hostname(),
        uptime: get_uptime(&sys),
        uuid: get_uuid(),
        // TODO - Change the way we get the cpu freq
        cpu_freq: sys.get_processors()[0].get_frequency() as i64,
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
