#[macro_use]
extern crate text_io;
#[macro_use]
extern crate log;

mod clap;
mod harvest;
mod logger;
mod options;

use harvest::data_harvest::Data;
use hyper::{Body, Client, Method, Request};
use hyper_tls::HttpsConnector;
use options::{
    config::{self},
    config_prompt,
    plugins_init::{self},
    Config, PluginsMap,
};
use std::{
    io::{Error, ErrorKind},
    thread,
    time::Duration,
};

/// Generate the Hyper Client needed for the sync requests
fn build_client() -> Client<hyper_tls::HttpsConnector<hyper::client::HttpConnector>> {
    // Create a Https "client" to be used in the Hyper Client
    let mut https_conn = HttpsConnector::new();
    https_conn.https_only(true);
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

    // Construct the --help menu and parse args more efficiently
    let args = clap::init_clap();

    // Detect if the user asked for config mode
    if args.is_present("config") {
        config_prompt::get_config_prompt();
        return Ok(());
    }

    // Get the config structure
    let config: Config = config::get_config(&args);

    // Build the client instance (*HTTP client)
    let client = build_client();

    // Int keeping track of the sending status
    let mut sync_track = 0;
    let mut load_track = 0;
    // Compute after how many harvest_interval the data has to be sent, and loadavg gathered
    let sync_threshold = config.harvest_interval * config.syncing_interval;
    let loadavg_threshold = config.harvest_interval * config.loadavg_interval;

    // Get the default Data instance
    let mut data: Data = Data::default();

    // Syncing memory cache
    let mut data_cache: Vec<Data> = Vec::with_capacity(sync_threshold as usize);
    info!("data_cache with size = {} spaces", sync_threshold);

    // Load Plugins (if any)
    let mut plugins = std::mem::MaybeUninit::<PluginsMap>::uninit();
    let mut has_plugins: bool = false;
    match plugins_init::get_plugins(&config) {
        Ok(plug_map) => {
            has_plugins = true;
            info!("plugins successfully loaded");
            // Use of unsafe is safe in this case as :
            //  - we're not reading from the as_mut_ptr
            //  - write is the first and only one we do to plugins,
            //    so no fear to loose any previous value without dropping it.
            unsafe { plugins.as_mut_ptr().write(plug_map) };
        }
        Err(plug_err) => {
            // This initialization as zeroed is used to prevent SEGFault
            // when reaching end of the program, cause Rust try to drop the value
            // as we assume plugins as init later.
            plugins = unsafe { std::mem::zeroed() };
            warn!("plugins_init throw: {}", plug_err);
        }
    };
    // //!\\ WARN UNSAFETY //!\\
    // I assume it's safe to assume_init because I know what I'm doing with it.
    // It MUST always be used behind a check if has_plugins is true or not.
    // Used without this protection can lead to undefined behavior and potential overflow/...
    let plugins = unsafe { plugins.assume_init() };

    // Start the app loop (collect metrics and send them)
    loop {
        // Increment track of our syncing status
        sync_track += 1;
        load_track += 1;
        // Refresh / Populate the Data structure
        data.eat_data(sync_track % loadavg_threshold == 0);
        // Reset loadavg tracker
        if load_track % loadavg_threshold == 0 {
            load_track = 0;
        }
        // Gather data from plugins
        // Only if has_plugins
        if has_plugins {
            data.eat_plugins(&plugins);
        }
        // Saving data in a temp var/space if we don't sync it right away
        data_cache.push(data.clone());
        trace!("data_cache filled");
        // Clear the plugin Vec only if has_plugins
        if has_plugins {
            data.clear_plugins();
            trace!("plugins data cleared");
        }
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
                    if data_cache.len() as u64 >= sync_threshold * 10 {
                        // drain the first (older) items to avoid taking too much memory
                        data_cache.drain(0..(sync_threshold * 2) as usize);
                        warn!("draining 0..{} items of the data_cache", sync_threshold * 2)
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
