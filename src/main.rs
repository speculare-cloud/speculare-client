#[macro_use]
extern crate log;

mod harvest;
mod logger;

use config::*;
use harvest::data_harvest::Data;
use hyper::{Body, Client, Method, Request};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::{
    io::{Error, ErrorKind},
    thread,
    time::Duration,
};

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct InnerConfig {
    pub api_token: String,
    pub api_url: String,
    pub harvest_interval: u64,
    pub syncing_interval: u64,
    pub loadavg_interval: u64,
}

/// Generate the Hyper Client needed for the sync requests
fn build_client() -> Client<hyper_rustls::HttpsConnector<hyper::client::HttpConnector>> {
    // Create a Https "client" to be used in the Hyper Client
    let https_conn = hyper_rustls::HttpsConnector::with_native_roots();
    // Create a single Client instance for the app
    Client::builder().build::<_, hyper::Body>(https_conn)
}

/// Generate the Request to be sent by the Hyper Client
fn build_request(
    api_url: &str,
    token: &str,
    data_cache: &[Data],
) -> Result<hyper::Request<hyper::Body>, Error> {
    match Request::builder()
        .method(Method::POST)
        .uri(api_url)
        .header("content-type", "application/json")
        .header("SPTK", token)
        .body(Body::from(serde_json::to_string(data_cache).unwrap()))
    {
        Ok(req) => Ok(req),
        Err(err_req) => Err(Error::new(ErrorKind::Other, err_req)),
    }
}

/// Entrypoint which start the process and loop indefinietly.
///
/// No other way to stop it than killing the process (for now).
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Init the logger and set the debug level correctly
    logger::configure();

    // Get arguments
    let args: Vec<String> = std::env::args().collect();

    // Verify if we have the correct number of arguments
    if args.len() != 2 {
        println!(
            "speculare-client: too {} arguments\n❯ speculare-client \"path/to/Config.toml\"",
            if args.len() > 2 { "many" } else { "few" }
        );
        std::process::exit(1);
    }

    // Load the configuration from the file passed as param
    let mut config = Config::default();
    config.merge(File::from(Path::new(&args[1]))).unwrap();

    // Get the config structure
    let config: InnerConfig = config.try_into().unwrap();

    // Build the client instance (HTTP client)
    let client = build_client();

    // Int keeping track of the sending status
    let mut sync_track: i64 = -1;
    let mut load_track: i64 = -1;

    // Compute after how many harvest_interval the data has to be sent, and loadavg gathered
    let sync_threshold = (config.harvest_interval * config.syncing_interval) as i64;
    let loadavg_threshold = (config.harvest_interval * config.loadavg_interval) as i64;

    // Get the default Data instance
    let mut data: Data = Data::default();

    // Syncing memory cache
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
            trace!("sending POST request");
            match client.request(request.unwrap()).await {
                Ok(resp_body) => {
                    trace!("the POST request resulted in {:?}", resp_body);
                    // If no error, clear the data_cache
                    data_cache.clear();
                    trace!("data_cache has cleared");
                    // Reset the tracking counter
                    sync_track = 0;
                }
                Err(hyper_err) => {
                    error!("the POST request resulted in {:?}", hyper_err);
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
        thread::sleep(Duration::from_secs(config.harvest_interval));
    }

    Ok(())
}
