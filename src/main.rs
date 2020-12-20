#[macro_use]
extern crate text_io;
#[macro_use]
extern crate log;

mod clap;
mod harvest;
mod logger;
mod options;
mod setup_config;

use harvest::data_harvest::Data;
use hyper::{Body, Client, Method, Request};
use hyper_tls::HttpsConnector;
use options::config::{self, Config};
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
        setup_config::config_mode();
        return;
    }

    // Get the config structure
    let config: Config = config::get_config(&args);

    // Create a single Client instance for the app
    let mut https_conn = HttpsConnector::new();
    https_conn.https_only(true);
    
    let client = Client::builder().http2_only(true).build::<_, hyper::Body>(https_conn);

    // Get the default Data instance
    let mut data: Data = Data::default();
    // Start the app loop (collect metrics and send them)
    loop {
        // Refresh / Populate the Data structure
        data.eat_data();

        // For development purpose
        dbg!(&data);

        // Testing
        let request = Request::builder()
            .method(Method::POST)
            .uri(&config.api_url)
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_string(&data).unwrap()))
            .unwrap();
        
        let res = client.request(request).await;
        // Debug
        dbg!(res);
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
