#[macro_use]
extern crate lazy_static;

extern crate reqwest;
extern crate sys_info;
extern crate sysinfo;

mod models;

use crate::models::*;

use std::{
    error::Error,
    process::Command,
    sync,
    sync::{
        atomic::{AtomicBool, Ordering},
        Mutex,
    },
    thread,
    time::Duration,
};
use sys_info::hostname;
use sysinfo::ComponentExt;
use sysinfo::{ProcessorExt, System, SystemExt};

struct Global {
    mthread: Option<thread::JoinHandle<()>>,
    alive: sync::Arc<AtomicBool>,
    skip: sync::Arc<AtomicBool>
}

lazy_static! {
    static ref G_INFO: Mutex<Global> = Mutex::new(Global {
        mthread: None,
        alive: sync::Arc::new(AtomicBool::new(false)),
        skip: sync::Arc::new(AtomicBool::new(false))
    });
}

impl Global {
    fn start(&mut self, interval: Option<u64>) {
        self.alive.store(true, Ordering::SeqCst);
        let alive = self.alive.clone();
        let skip = self.skip.clone();

        let interval = if interval.is_some() {
            interval.unwrap()
        } else {
            300 // 300 default is 5 mins
        };

        self.mthread = Some(thread::spawn(move || {
            while alive.load(Ordering::SeqCst) {
                if !skip.load(Ordering::SeqCst) {
                    match collect_and_send() {
                        Ok(x) => x,
                        Err(x) => syslog(x.to_string(), false),
                    };
                }
                thread::sleep(Duration::from_secs(interval));
            }
        }));
    }

    fn burst_on(&mut self) {
        self.skip.store(true, Ordering::SeqCst);
    }

    fn burst_off(&mut self) {
        self.skip.store(false, Ordering::SeqCst);
    }

    fn stop(&mut self) {
        self.alive.store(false, Ordering::SeqCst);
        self.mthread
            .take()
            .expect("Called stop on non-running thread")
            .join()
            .expect("Could not join spawned thread");
    }
}

/* Will only succed on iMac - send message and panic if needed */
fn syslog(message: String, fail: bool) {
    Command::new("bash").arg("-c").arg(format!(
        "/bin/syslog.py {}",
        format!("[SPECULARE] - {}", message)
    ));
    if fail {
        panic!(message);
    }
}

/*
 *  MAC - linux specific get_mac_address
 *  using default interface in a safe string
 */
#[cfg(target_os = "linux")]
fn get_mac_address() -> String {
    let interface = Command::new("bash")
        .arg("-c")
        .arg("route | grep '^default' | grep -o '[^ ]*$' | sed '$!d' | tr -d '\n'")
        .output()
        .expect("failed to retrieve default interface");
    let mac_address = Command::new("bash")
        .arg("-c")
        .arg(format!(
            "ifconfig {} | grep 'ether ' | awk {} | tr -d '\n'",
            String::from_utf8_lossy(&interface.stdout),
            "'{print $2}'"
        ))
        .output();
    return match mac_address {
        Ok(val) => String::from_utf8_lossy(&val.stdout).to_string(),
        Err(x) => {
            syslog(x.to_string(), false);
            x.to_string()
        }
    };
}

/*
 *  MAC - macos specific get_mac_address
 *  using default interface in a safe string
 */
#[cfg(target_os = "macos")]
fn get_mac_address() -> String {
    let interface = Command::new("bash")
        .arg("-c")
        .arg("route -n get default | grep 'interface:' | grep -o '[^ ]*$' | sed '$!d' | tr -d '\n'")
        .output()
        .expect("failed to retrieve default interface");
    let mac_address = Command::new("bash")
        .arg("-c")
        .arg(format!(
            "ifconfig {} | grep 'ether ' | awk {} | tr -d '\n'",
            String::from_utf8_lossy(&interface.stdout),
            "'{print $2}'"
        ))
        .output();
    return match mac_address {
        Ok(val) => String::from_utf8_lossy(&val.stdout).to_string(),
        Err(x) => {
            syslog(x.to_string(), false);
            x.to_string()
        }
    };
}

/* Get the user currently logged, if more than 1 user, return the last one */
fn get_logged_user() -> String {
    let logged_users = Command::new("bash")
        .arg("-c")
        .arg("users | awk -F' ' '{print $NF}' | tr -d '\n'")
        .output();
    return match logged_users {
        Ok(val) => String::from_utf8_lossy(&val.stdout).to_string(),
        Err(x) => {
            syslog(x.to_string(), false);
            x.to_string()
        }
    };
}

/* Get the os version (Mac/Linux/Windows) in a safe String */
fn get_os_version() -> String {
    let os_release = os_version::detect();
    return match os_release {
        Ok(val) => val.to_string(),
        Err(x) => {
            syslog(x.to_string(), false);
            x.to_string()
        }
    };
}

/* Get the hostname (Mac/Linux/Windows) in a safe String */
fn get_hostname() -> String {
    return match hostname() {
        Ok(val) => val.to_string(),
        Err(x) => {
            syslog(x.to_string(), false);
            x.to_string()
        }
    };
}

/* Get the uuid of the host (Mac/Linux/Windows) in a safe String */
fn get_uuid() -> String {
    return match machine_uid::get() {
        Ok(val) => val.to_string(),
        Err(x) => {
            syslog(x.to_string(), false);
            x.to_string()
        }
    };
}

fn collect_and_send() -> Result<(), Box<dyn Error>> {
    syslog("collecting info...".to_string(), false);
    let sys = System::new_all();

    let components = sys.get_components();
    let mut sensors: Vec<Sensors> = Vec::with_capacity(components.len());
    for component in components {
        sensors.push(Sensors {
            label: component.get_label().to_string(),
            temp: component.get_temperature(),
        })
    }

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
    syslog("got all the data needed...".to_string(), false);

    let mut url: String = String::new();
    match std::env::var("api_url") {
        Ok(val) => url.push_str(&val),
        Err(x) => {
            syslog(x.to_string(), true);
        }
    };

    let mut token: String = String::new();
    match std::env::var("api_token") {
        Ok(val) => token.push_str(&val),
        Err(x) => {
            syslog(x.to_string(), true);
        }
    };

    // Still having issue if the server is not reachable, this will be blocking indefinitely
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(3))
        .build()?;

    let res = client.post(&url)
        .header("Authorization", format!("Bearer {}", token))
        .json(&data)
        .send()?;

    println!("Return code [{}]", res.status());
    syslog(format!("return code [{}]", res.status()), false);
    Ok(())
}

fn main() {
    dotenv::from_path("/etc/speculare.config").unwrap_or_else(|_error| {
        syslog("failed to load /etc/speculare.config".to_string(), true);
    });

    {
        /*
         *  The mutex 'data' will be dropped
         *  once outside of the scope, so no need
         *  to drop it manually
         */
        G_INFO.lock().unwrap().start(None);
    }

    /*
     *  Start an actix web server instead
     *  The actix web server will recieve order from the
     *  master server to run in burst mode for a certain time.
     *  But burst mode we call it sending info more than once every 5min.
     */
    loop {
        thread::sleep(Duration::from_millis(10000));
        // G_INFO.lock().unwrap().burst_on()
    }
}
