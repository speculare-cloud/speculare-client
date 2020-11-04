use crate::gather;
use crate::models;
use crate::utils;

use gather::*;
use models::*;
use utils::syslog;

use log::info;
use std::{error::Error, time::Duration};
use sysinfo::{ProcessorExt, System, SystemExt};

// TODO
// Cut this function is some small relevant function
// Comment it
pub fn collect_and_send() -> Result<(), Box<dyn Error>> {
    let sys = System::new_all();

    let data = Data {
        os: get_os_version(),
        hostname: get_hostname(),
        uptime: sys.get_uptime() as i64,
        uuid: get_uuid(),
        cpu_freq: sys.get_processors()[0].get_frequency() as i64,
        load_avg: get_avg_load(&sys),
        user: get_logged_user(),
        sensors: get_senors_data(&sys),
        disks: get_disks_data(&sys),
        mac_address: get_mac_address(),
    };

    let mut url: String = String::new();
    match std::env::var("api_url") {
        Ok(val) => url.push_str(&val),
        Err(x) => {
            syslog(x.to_string(), true, true, false);
        }
    };

    let mut token: String = String::new();
    match std::env::var("api_token") {
        Ok(val) => token.push_str(&val),
        Err(x) => {
            syslog(x.to_string(), true, true, false);
        }
    };

    let timeout = Duration::new(15, 0);
    let client = reqwest::blocking::ClientBuilder::new()
        .timeout(timeout)
        .connect_timeout(timeout)
        .build()?;

    let res = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", token))
        .json(&data)
        .send();

    match res {
        Ok(res) => info!("return status : {}", res.status()),
        Err(x) => {
            sentry::capture_error(&x);
            syslog(format!("calling error : {}", x), false, true, false);
        }
    }
    Ok(())
}
