#[macro_use]
extern crate text_io;
#[macro_use]
extern crate log;

mod gathering;
mod models;
mod setup_config;

use clap::{App, Arg, ArgMatches};
use gathering::gathering;
use models::Config;
use std::fs::File;
use std::io::BufReader;
use std::{thread, time::Duration};

/// Init the logger (env_logger) and define the debug level
/// based on debug or release build.
fn init_logger() {
    // Define log as info for debug and error for prod
    let dbg_level = if cfg!(debug_assertions) {
        "info"
    } else {
        "error"
    };
    std::env::set_var("RUST_LOG", dbg_level);
    // Init the logger
    env_logger::init();
}

/// Init the clap menu/args handling and return the ArgMatches instance.
fn init_matches() -> ArgMatches {
    App::new("Speculare-client")
        .version("0.1.0")
        .author("Martin A. <ma@rtin.fyi>")
        .about("Collect metrics and send them to the server")
        .arg(
            Arg::new("config")
                .short('c')
                .long("config")
                .about("Enter the interactive config mode")
                .takes_value(false),
        )
        .arg(
            Arg::new("path")
                .short('p')
                .long("path")
                .about("Path to the config file")
                .takes_value(true),
        )
        .get_matches()
}

/// Get the correct path for the config, open it and read it to the Config struct
/// which is then returned.
fn get_config(matches: &ArgMatches) -> Config {
    // Determine the path of the config
    let config_path = if matches.is_present("path") {
        matches.value_of("path").unwrap()
    } else {
        "/etc/speculare/speculare.config"
    };
    // Open the config_file as File
    let config_file = match File::open(&config_path) {
        Ok(val) => val,
        Err(x) => {
            panic!("Can't open {}\nError: {}", &config_path, x);
        }
    };
    // Create a reader from the config_file
    let config_reader = BufReader::new(&config_file);
    // Convert the reader into Config struct
    match serde_json::from_reader(config_reader) {
        Ok(val) => val,
        Err(x) => {
            panic!("Can't convert {}\nError: {}", &config_path, x);
        }
    }
}

/// Main which start the process and loop indefinietly.
///
/// No other way to stop it than killing the process (for now).
#[cfg(any(target_os = "linux", target_os = "macos"))]
fn main() {
    // Init the logger and set the debug level correctly
    init_logger();
    // Construct the --help menu and handle args more efficiently
    let matches = init_matches();
    // Detect if the user asked for config mode
    if matches.is_present("config") {
        setup_config::config_mode();
        return;
    }
    // Get the config structure
    let config: Config = get_config(&matches);
    // Max 15s of timeout per requests
    let timeout = Duration::new(15, 0);
    // Create a single Client instance for the app
    // blocking (for now) cause as of now, this app try to minimize CPU impact
    let client = reqwest::blocking::ClientBuilder::new()
        .timeout(timeout)
        .connect_timeout(timeout)
        .build()
        .expect("Failed to create the blocking client");

    // Start the app loop (collect metrics and send them)
    loop {
        // Collect all (needed) metrics
        let data = gathering();
        // For development purpose
        dbg!(data);
        // Send the metrics to the server as JSON
        // match client.post(&config.api_url).json(&data).send() {
        //     Ok(_val) => info!("Data send correctly"),
        //     Err(x) => warn!("Error while sending the request: {}", x),
        // };
        // Sleep for the interval defined above
        // don't spam the CPU nor the server
        thread::sleep(Duration::from_secs(1));
    }
}
