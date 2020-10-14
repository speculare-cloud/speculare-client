#[macro_use]
extern crate lazy_static;

extern crate reqwest;
extern crate sys_info;
extern crate sysinfo;

mod gather;
mod models;
mod process;
mod utils;

use gather::get_disks_data;
use models::*;
use sysinfo::SystemExt;
use utils::syslog;

use std::{
    sync,
    sync::{atomic::AtomicBool, Mutex},
    thread,
    time::Duration,
};

lazy_static! {
    static ref G_INFO: Mutex<Global> = Mutex::new(Global {
        mthread: None,
        alive: sync::Arc::new(AtomicBool::new(false)),
        skip: sync::Arc::new(AtomicBool::new(false))
    });
}

fn main() {
    /* Define log as info during development time */
    std::env::set_var("RUST_LOG", "info");

    /* Load the config into the env to use accross the prog */
    dotenv::from_path("/etc/speculare.config").unwrap_or_else(|_error| {
        syslog(
            "failed to load /etc/speculare.config".to_string(),
            true,
            true,
        );
    });

    env_logger::init();

    let system = sysinfo::System::new_all();
    get_disks_data(&system);

    return;

    {
        /*
         *  The mutex 'data' will be dropped
         *  once outside of the scope, so no need
         *  to drop it manually
         */
        G_INFO.lock().unwrap().start(None);
    }

    /*
     *  TODO - Start an actix web server instead
     *  The actix web server will recieve order from the
     *  master server to run in burst mode for a certain time.
     *  But burst mode we call it sending info more than once every 5min.
     */
    loop {
        thread::sleep(Duration::from_millis(10000));
        /* G_INFO.lock().unwrap().burst_on() */
    }
}
