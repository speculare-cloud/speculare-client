use crate::metrics;
use crate::models;
use crate::utils;

use log::info;
use metrics::{cpu::*, disks::*, miscs::*, sensors::*};
use models::DynData;
use reqwest::blocking::Client;
use std::error::Error;
use utils::syslog;

/// Collect all the metrics and send them to the server instance
pub fn collect_and_send(client: &Client, url: &String) -> Result<(), Box<dyn Error>> {
    // Construct the Data structure with all the info needed
    let dyndata = DynData {
        uuid: get_uuid(),
        uptime: get_uptime(),
        cpu_freq: get_avg_cpufreq(),
        load_avg: get_avg_load(),
        sensors: get_sensors_data(),
        disks: get_disks_data(),
    };

    // Send the request
    let res = client.post(url).json(&dyndata).send();

    // Analyze the output and log it + send to sentry in case of error
    // TODO - Implement a status_code from the server to tell the client
    // "Hey, I'm missing some of your infos, can you send me those ?"
    match res {
        Ok(res) => info!("return status : {}", res.status()),
        Err(x) => {
            sentry::capture_error(&x);
            syslog(format!("calling error : {}", x), false, true, false);
        }
    }
    Ok(())
}
