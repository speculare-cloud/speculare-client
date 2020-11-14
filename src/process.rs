use crate::models::{Config, Data};

use metrics_rs::{cpu::*, disks::*, miscs::*, sensors::*, users::*};
use reqwest::blocking::Client;
use std::io::{Error, ErrorKind};

/// Collect all the metrics and send them to the server instance.
pub fn collect_and_send(client: &Client, config: &Config) -> Result<(), Error> {
    // Construct the Data structure with all the info needed
    let host_info = match get_host_info() {
        Ok(val) => val,
        Err(x) => return Err(Error::new(ErrorKind::Other, x)),
    };

    let dyndata = Data {
        uuid: get_uuid().expect("Cannot retrieve UUID"),
        os: host_info.os_version,
        hostname: host_info.hostname,
        uptime: host_info.uptime,
        cpu_freq: get_avg_cpufreq(),
        load_avg: host_info.loadavg,
        sensors: get_sensors_data(),
        disks: get_disks_data(),
        iostats: match get_iostats() {
            Ok(val) => Some(val),
            Err(_) => None,
        },
        memory: host_info.memory,
        users: get_users(),
    };

    dbg!(dyndata);

    // // Send the request
    // let res = client.post(url).json(&dyndata).send();

    // // Detect error for the post request and log potential info
    // match res {
    //     Ok(res) => info!("return status : {}", res.status()),
    //     Err(x) => {
    //         syslog(format!("calling error : {}", x), false, true);
    //     }
    // }
    Ok(())
}
