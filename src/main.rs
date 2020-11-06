#[macro_use]
extern crate lazy_static;
extern crate mac_address;
extern crate reqwest;
extern crate sysinfo;
extern crate cpuid;

mod gather;
mod models;
mod process;
mod utils;

use models::*;
use std::{
    sync,
    sync::{atomic::AtomicBool, Mutex},
    thread,
    time::Duration,
};
use utils::syslog;

lazy_static! {
    static ref G_INFO: Mutex<Global> = Mutex::new(Global {
        mthread: None,
        alive: sync::Arc::new(AtomicBool::new(false)),
    });
}

/// Main which start the process and loop indefinietly
/// No other way to stop it than killing the process
fn main() {
    // Define log as info during development time
    std::env::set_var("RUST_LOG", "info");
    env_logger::init();

    // Load the config into the env to use accross the prog
    dotenv::from_path("/etc/speculare.config").unwrap_or_else(|_error| {
        syslog(
            "failed to load /etc/speculare.config".to_string(),
            true,
            true,
            false,
        );
    });

    // Define the sentry guard
    let _guard = sentry::init(std::env::var("sentry_endpoint").expect("missing sentry endpoint"));

    {
        // The mutex 'data' will be dropped
        // once outside of the scope, so no need
        // to drop it manually
        G_INFO.lock().unwrap().start(Some(1));
    }

    // TODO - Start an actix web server instead
    // The actix web server will recieve order from the
    // master server to run in burst mode for a certain time.
    // But burst mode we call it sending info more than once every 5min.
    loop {
        thread::sleep(Duration::from_millis(10000));
        /* G_INFO.lock().unwrap().burst_on() */
    }
}
