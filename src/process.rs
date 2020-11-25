use crate::models::{Config, Data};

use sys_metrics::{cpu::*, disks::*, miscs::*, users::*};
use reqwest::blocking::Client;
use std::io::{Error, ErrorKind};

/// Collect all the metrics and send them to the server instance.
pub fn collect_and_send(_client: &Client, _config: &Config) -> Result<(), Error> {
    // Construct the Data structure with all the info needed
    let host_info = match get_host_info() {
        Ok(val) => val,
        Err(x) => return Err(Error::new(ErrorKind::Other, x)),
    };

    let dyndata = Data {
        uuid: get_uuid().expect("Cannot retrieve UUID"),
        os: host_info.os_version,
        hostname: host_info.hostname,
        uptime: host_info.uptime as i64,
        cpu_freq: match get_cpufreq() {
            Ok(val) => val as i64,
            Err(_) => -1,
        },
        load_avg: host_info.loadavg,
        disks: match get_partitions_physical() {
            Ok(val) => Some(val),
            Err(_) => None,
        },
        iostats: match get_iostats() {
            Ok(val) => Some(val),
            Err(_) => None,
        },
        memory: host_info.memory,
        users: match get_users() {
            Ok(val) => Some(val),
            Err(_) => None,
        },
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
