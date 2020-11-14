use crate::models::{Config, DynData};

use metrics_rs::{cpu::*, disks::*, memory::*, miscs::*, sensors::*, users::*};
use reqwest::blocking::Client;
use std::error::Error;

/// Collect all the metrics and send them to the server instance.
pub fn collect_and_send(client: &Client, config: &Config) -> Result<(), Box<dyn Error>> {
    // Construct the Data structure with all the info needed
    let dyndata = DynData {
        uuid: get_uuid().expect("Cannot retrieve UUID"),
        uptime: get_uptime(),
        cpu_freq: get_avg_cpufreq(),
        load_avg: match get_avg_load() {
            Ok(val) => Some(val),
            Err(_) => None,
        },
        sensors: todo!(),
        disks: get_disks_data(),
        iostats: match get_iostats() {
            Ok(val) => Some(val),
            Err(_) => None,
        },
        memory: get_memory(),
        users: get_users(),
    };

    dbg!(dyndata);

    // // Send the request
    // let res = client.post(url).json(&dyndata).send();

    // // Analyze the output and log it + send to sentry in case of error
    // // TODO - Implement a status_code from the server to tell the client
    // // "Hey, I'm missing some of your infos, can you send me those ?"
    // match res {
    //     Ok(res) => info!("return status : {}", res.status()),
    //     Err(x) => {
    //         syslog(format!("calling error : {}", x), false, true);
    //     }
    // }
    Ok(())
}
