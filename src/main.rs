#[macro_use]
extern crate text_io;
#[macro_use]
extern crate log;
extern crate num_integer;

mod clap;
mod harvest;
mod logger;
mod options;

use harvest::data_harvest::Data;
use hyper::{Body, Client, Method, Request};
use hyper_tls::HttpsConnector;
use num_integer::Integer;
use options::{
    config::{self},
    config_prompt, Config,
};
use std::{thread, time::Duration};

/// Entrypoint which start the process and loop indefinietly.
///
/// No other way to stop it than killing the process (for now).
#[tokio::main]
async fn main() {
    // Init the logger and set the debug level correctly
    logger::configure();

    // Construct the --help menu and parse args more efficiently
    let args = clap::init_clap();

    // Detect if the user asked for config mode
    if args.is_present("config") {
        config_prompt::get_config_prompt();
        return;
    }

    // Get the config structure
    let config: Config = config::get_config(&args);

    // Create a Https "client" to be used in the Hyper Client
    let mut https_conn = HttpsConnector::new();
    https_conn.https_only(true);
    // Create a single Client instance for the app
    let client = Client::builder().build::<_, hyper::Body>(https_conn);

    // Int keeping track of the sending status
    let mut syncing_track: u64 = 0;
    let syncing_threshold = config.harvest_interval.lcm(&config.syncing_interval);
    // Debug printing
    dbg!(syncing_threshold);

    // Get the default Data instance
    let mut data: Data = Data::default();

    // Start the app loop (collect metrics and send them)
    loop {
        // Increment track of our syncing status
        syncing_track += 1;
        // Refresh / Populate the Data structure
        data.eat_data();
        // Saving data in a temp var/space if we don't sync it right away
        // TODO
        // Checking if we should sync
        if syncing_track == syncing_threshold {
            syncing_track = 0;
            // Sending request to the server
            // TODO - Get rid of these unsafe unwrap
            let request = Request::builder()
                .method(Method::POST)
                .uri(&config.api_url)
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&data).unwrap()))
                .unwrap();
            // Execute the request
            let res = client.request(request).await;
            // Debug printing
            dbg!(res);
        }
        // Wait config.harvest_interval before running again
        // For syncing interval must be greater or equals to the harvest_interval
        // so just base this sleep on the harvest_interval value.
        thread::sleep(Duration::from_secs(config.harvest_interval));
    }
}
