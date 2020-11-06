use crate::gather;
use crate::models;
use crate::utils;

use gather::*;
use log::info;
use models::*;
use reqwest::blocking::Client;
use std::error::Error;
use utils::syslog;

/// Collect all the metrics and send them to the server instance
pub fn collect_and_send(client: &Client) -> Result<(), Box<dyn Error>> {
    // Construct the Data structure with all the info needed
    let data = Data {
        os: get_os_version(),
        hostname: get_hostname(),
        uptime: get_uptime(),
        uuid: get_uuid(),
        cpu_freq: get_avg_cpufreq(),
        load_avg: get_avg_load(),
        user: get_logged_user(),
        sensors: get_senors_data(),
        disks: get_disks_data(),
        mac_address: get_mac_address(),
    };

    // Prepare to send
    // Get the url (where to send)
    // TODO - Do it only one time
    let mut url: String = String::new();
    match std::env::var("api_url") {
        Ok(val) => url.push_str(&val),
        Err(x) => {
            syslog(x.to_string(), true, true, false);
        }
    };
    // Get the token (Authorization)
    // TODO - Do it only one time
    // let mut token: String = String::new();
    // match std::env::var("api_token") {
    //     Ok(val) => token.push_str(&val),
    //     Err(x) => {
    //         syslog(x.to_string(), true, true, false);
    //     }
    // };

    // Send the request
    let res = client
        .post(&url)
        //.header("Authorization", format!("Bearer {}", token))
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
