#[macro_use]
extern crate text_io;
#[macro_use]
extern crate log;

mod config_mode;
mod models;
mod process;

use models::Config;
use process::collect_and_send;
use std::fs::File;
use std::io::BufReader;
use std::{thread, time::Duration};

/// Main which start the process and loop indefinietly.
/// No other way to stop it than killing the process.
#[cfg(not(windows))]
fn main() {
    // Define log as info for debug and error for prod
    let dbg_level = if cfg!(debug_assertions) {
        "info"
    } else {
        "error"
    };
    std::env::set_var("RUST_LOG", dbg_level);
    // Init the logger
    env_logger::init();

    // Detect if we should run in init config mode
    let args: Vec<String> = std::env::args().collect();
    if args.len() == 2 && args[1] == "--config" {
        config_mode::entry_point();
        return;
    } else if args.len() == 2 {
        error!(
            "Wrong number of paramters or wrong parameters.\nUse with --config or no paramters."
        );
        return;
    }

    // Load the config into the env to use accross the prog
    let home: String = match dirs::home_dir() {
        Some(val) => val.to_string_lossy().into_owned(),
        None => String::from("/"),
    };
    // Open the config_file as File
    let config_path = format!("{}/speculare.config", home);
    let config_file = match File::open(&config_path) {
        Ok(val) => val,
        Err(x) => {
            error!("Can't open {}\nError: {}", &config_path, x);
            return;
        }
    };
    // Create a reader from the config_file
    let config_reader = BufReader::new(&config_file);

    // Convert the reader into Config struct
    let config: Config = match serde_json::from_reader(config_reader) {
        Ok(val) => val,
        Err(x) => {
            error!("Can't convert {}\nError: {}", &config_path, x);
            return;
        }
    };

    // Create the client instance for each loop
    // Do not create a new one each time
    let timeout = Duration::new(15, 0);
    // Create a single client instance for the app
    let client = reqwest::blocking::ClientBuilder::new()
        .timeout(timeout)
        .connect_timeout(timeout)
        .build()
        .expect("Failed to create the blocking client");

    // Start the app loop
    loop {
        match collect_and_send(&client, &config) {
            Ok(x) => x,
            Err(x) => warn!("Error when collect_and_send: {}", x),
        };
        // Sleep for the interval defined above
        // don't spam the CPU nor the server
        thread::sleep(Duration::from_secs(1));
    }
}
