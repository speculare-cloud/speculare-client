#[macro_use]
extern crate log;

mod harvest;
mod request;

use config::*;
use harvest::data_harvest::Data;
use serde::{Deserialize, Serialize};
use std::{path::Path, thread, time::Duration};

use crate::request::{build_client, build_request};

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct InnerConfig {
    pub api_token: String,
    pub api_url: String,
    pub harvest_interval: u64,
    pub syncing_interval: u64,
    pub loadavg_interval: u64,
}

/// Init the logger (env_logger) and define the debug level
/// based on debug or release build.
fn configure_logger() {
    // Check if the RUST_LOG already exist in the sys
    if std::env::var_os("RUST_LOG").is_none() {
        // if it doesn't, assign a default value to RUST_LOG
        // Define RUST_LOG as trace for debug and error for prod
        std::env::set_var(
            "RUST_LOG",
            if cfg!(debug_assertions) {
                "info,speculare_client=trace"
            } else {
                "warn"
            },
        );
    }
    // Init the logger
    env_logger::init();
}

/// Entrypoint which start the process and loop indefinietly.
///
/// No other way to stop it than killing the process (for now).
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Init the logger and set the debug level correctly
    configure_logger();

    // Get arguments
    let args: Vec<String> = std::env::args().collect();

    // Verify if we have the correct number of arguments
    if args.len() != 2 {
        println!(
            "speculare-client: too {} arguments: missing a \"path/to/Config.toml\"",
            if args.len() > 2 { "many" } else { "few" }
        );
        std::process::exit(1);
    }

    // Load the configuration from the file passed as param
    let mut config = Config::default();
    config.merge(File::from(Path::new(&args[1]))).unwrap();
    // Convert the config (.toml) to the InnerConfig struct
    let config: InnerConfig = config.try_into().unwrap();

    // Build the client instance (HTTP.S client)
    let client = build_client();

    // Int keeping track of the sending status & load (if we should gather loadavg)
    let (mut sync_track, mut load_track): (i64, i64) = (-1, -1);

    // Compute after how many harvest_interval the data has to be sent, and loadavg gathered
    let sync_threshold = (config.harvest_interval * config.syncing_interval) as i64;
    let loadavg_threshold = (config.harvest_interval * config.loadavg_interval) as i64;

    // Get the default Data instance
    let mut data: Data = Data::default();

    // Syncing memory cache (min 16 items)
    let cache_size = std::cmp::max(sync_threshold, 16);
    let mut data_cache: Vec<Data> = Vec::with_capacity(cache_size as usize);
    info!("data_cache with size = {} spaces", cache_size);

    // Start the app loop (collect metrics and send them)
    loop {
        // Increment track of our syncing status
        sync_track += 1;
        load_track += 1;
        // Define if we should gather loadavg or not
        let get_loadavg = load_track % loadavg_threshold == 0;
        // Refresh / Populate the Data structure
        data.eat_data(get_loadavg);
        // Reset loadavg tracker
        if load_track % loadavg_threshold == 0 {
            load_track = 0;
        }
        // Saving data in a temp var/space if we don't sync it right away
        data_cache.push(data.clone());
        trace!("data_cache filled");
        // Checking if we should sync
        if sync_track % sync_threshold == 0 {
            // Sending request to the server
            let request = build_request(&config.api_url, &config.api_token, &data_cache);
            // If the request couldn't be created, exit and print
            if request.is_err() {
                error!("request builder: {}", request.unwrap_err());
                break;
            }
            trace!("request is ready to be sent");

            // Execute the request
            match client.request(request.unwrap()).await {
                Ok(resp_body) => {
                    trace!("the request resulted in {:?}", resp_body);
                    // If no error, clear the data_cache
                    data_cache.clear();
                    trace!("data_cache has been cleared");
                    // Reset the tracking counter
                    sync_track = 0;
                }
                Err(hyper_err) => {
                    error!("the request resulted in {:?}", hyper_err);
                    // If data_cache contains too many items due to previous error
                    if data_cache.len() as i64 >= cache_size * 2 {
                        // drain the first (older) items to avoid taking too much memory
                        let to_drain = cache_size / 2;
                        data_cache.drain(0..to_drain as usize);
                        warn!("draining 0..{} items of the data_cache", to_drain)
                    }
                }
            }
        }
        // Wait config.harvest_interval before running again
        // For syncing interval must be greater or equals to the harvest_interval
        // so just base this sleep on the harvest_interval value.
        // => Doing so doesn't guaratee that we'll gather values every config.harvest_interval
        // due to the time we take to gather data and send it over the network.
        // Gathering and sending is not async so it's more like (time_to_gather_&_send + config.harvest_interval).
        thread::sleep(Duration::from_secs(config.harvest_interval));
    }

    Ok(())
}
