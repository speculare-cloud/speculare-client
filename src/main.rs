extern crate cpuid;
extern crate libc;
extern crate mac_address;
extern crate reqwest;

mod metrics;
mod models;
mod process;
mod utils;

use process::collect_and_send;
use std::{thread, time::Duration};
use utils::syslog;

/// Main which start the process and loop indefinietly.
/// No other way to stop it than killing the process.
fn main() {
    // Load the config into the env to use accross the prog
    let dotenv = dotenv::from_path("/etc/speculare.config");
    if dotenv.is_err() {
        println!("Cannot find /etc/speculare.config\nRun `speculare --init` to create it or create it manually");
        return;
    }
    // Define log as info during development time
    std::env::set_var(
        "RUST_LOG",
        std::env::var("debug_level").unwrap_or_else(|_| String::from("info")),
    );
    // Init the logger
    env_logger::init();

    // Create the client instance for each loop
    // Do not create a new one each time
    let timeout = Duration::new(15, 0);
    // Create a single client instance for the app
    let client = reqwest::blocking::ClientBuilder::new()
        .timeout(timeout)
        .connect_timeout(timeout)
        .build()
        .expect("Failed to create the blocking client");

    // Prepare to send, get the url (where to send)
    let url = std::env::var("api_url").expect("Missing api_url");

    // Start the app loop
    loop {
        match collect_and_send(&client, &url) {
            Ok(x) => x,
            Err(x) => syslog(x.to_string(), false, true),
        };
        // Sleep for the interval defined above
        // don't spam the CPU nor the server
        thread::sleep(Duration::from_secs(1));
    }
}
