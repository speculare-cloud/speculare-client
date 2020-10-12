use crate::gather;
use crate::models;
use crate::utils;

use gather::*;
use models::*;
use utils::syslog;

use log::info;
use std::{error::Error, time::Duration};
use sysinfo::{ProcessorExt, System, SystemExt};

pub fn collect_and_send() -> Result<(), Box<dyn Error>> {
    syslog("collecting info...".to_string(), false, false);
    let sys = System::new_all();

    let sensors: Vec<Sensors> = get_senors_data(&sys);

    let data = Data {
        os: get_os_version(),
        hostname: get_hostname(),
        uptime: sys.get_uptime(),
        uuid: get_uuid(),
        cpu_freq: sys.get_processors()[0].get_frequency(),
        user: get_logged_user(),
        sensors: sensors,
        mac_address: get_mac_address(),
    };
    syslog("got all the data needed...".to_string(), false, false);

    let mut url: String = String::new();
    match std::env::var("api_url") {
        Ok(val) => url.push_str(&val),
        Err(x) => {
            syslog(x.to_string(), true, true);
        }
    };

    let mut token: String = String::new();
    match std::env::var("api_token") {
        Ok(val) => token.push_str(&val),
        Err(x) => {
            syslog(x.to_string(), true, true);
        }
    };

    let timeout = Duration::new(5, 0);
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
            syslog(format!("calling error : {}", x), false, true);
        }
    }
    Ok(())
}
