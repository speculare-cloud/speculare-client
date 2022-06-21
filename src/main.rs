#[macro_use]
extern crate log;

#[cfg(feature = "auth")]
use crate::request::build_update;
use crate::request::{build_client, build_request};
use crate::utils::config::Config;

use clap::Parser;
use clap_verbosity_flag::InfoLevel;
use harvest::data_harvest::Data;
use hyper::StatusCode;
use std::{ffi::OsStr, path::Path, thread, time::Duration};

mod harvest;
mod request;
mod utils;

#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Args {
    #[clap(short = 'c', long = "config")]
    config_path: Option<String>,

    #[clap(flatten)]
    verbose: clap_verbosity_flag::Verbosity<InfoLevel>,
}

lazy_static::lazy_static! {
    static ref CONFIG: Config = match Config::new() {
        Ok(config) => config,
        Err(err) => {
            error!("Cannot build the Config: {}", err);
            std::process::exit(1);
        }
    };

    static ref API_URL: String = {
        info!("API_URL: {}", CONFIG.api_url.clone() + "?uuid=" + &CONFIG.uuid);
        CONFIG.api_url.clone() + "?uuid=" + &CONFIG.uuid
    };
}

#[cfg(feature = "auth")]
lazy_static::lazy_static! {
    static ref SSO_URL: String = {
        info!("SSO_URL: {}", CONFIG.sso_url.clone() + "?uuid=" + &CONFIG.uuid);
        CONFIG.sso_url.clone() + "?uuid=" + &CONFIG.uuid
    };
}

fn prog() -> Option<String> {
    std::env::args()
        .next()
        .as_ref()
        .map(Path::new)
        .and_then(Path::file_name)
        .and_then(OsStr::to_str)
        .map(String::from)
}

/// Entrypoint which start the process and loop indefinitely.
///
/// No other way to stop it than killing the process (for now).
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // Init logger
    env_logger::Builder::new()
        .filter_module(
            &prog().map_or_else(|| "speculare_client".to_owned(), |f| f.replace('-', "_")),
            args.verbose.log_level_filter(),
        )
        .init();

    let config = match Config::new() {
        Ok(config) => config,
        Err(err) => {
            error!("Cannot build the Config: {}", err);
            std::process::exit(1);
        }
    };

    // Build the client instance (HTTP.S client)
    let client = build_client();

    // Int keeping track of the sending status & load (if we should gather loadavg)
    let mut sync_track: i64 = -1;

    // Compute after how many harvest_interval the data has to be sent, and loadavg gathered
    let sync_threshold = (config.harvest_interval * config.syncing_interval) as i64;
    let loadavg_threshold = (config.harvest_interval * config.loadavg_interval) as i64;

    // Get the default Data instance
    let mut data: Data = Data::default();

    // Syncing memory cache (min 16 items)
    let cache_size = std::cmp::max(sync_threshold, config.cache_size);
    let mut data_cache: Vec<Data> = Vec::with_capacity(cache_size as usize);
    info!("data_cache with size = {} spaces", cache_size);

    // Start the app loop (collect metrics and send them)
    loop {
        // Increment track of our syncing status
        sync_track += 1;

        // Refresh / Populate the Data structure
        data.eat_data(sync_track % loadavg_threshold == 0);

        // Saving data in a temp var/space if we don't sync it right away
        data_cache.push(data.clone());
        trace!(
            "data_cache pushed, current occupation: {} / {}",
            data_cache.len(),
            data_cache.capacity()
        );

        // Checking if we should sync
        if sync_track % sync_threshold == 0 {
            // Building the request to be sent to the server
            let request = match build_request(&config.api_token, &data_cache) {
                Ok(req) => req,
                Err(err) => {
                    error!("build_request: error: {}", err);
                    std::process::exit(1);
                }
            };

            let mut should_drain = false;
            // Actually send the request to the server
            match client.request(request).await {
                Ok(resp_body) => {
                    trace!("request: response: {:?}", resp_body);
                    if resp_body.status() == StatusCode::OK {
                        data_cache.clear();
                        trace!("data_cache has been cleared");
                    }

                    #[cfg(feature = "auth")]
                    if resp_body.status() == StatusCode::PRECONDITION_FAILED {
                        warn!("The host_uuid is not defined for this key, updating...");
                        // Post the PATCH update and if no error, continue
                        let update = match build_update(&config.api_token) {
                            Ok(req) => req,
                            Err(err) => {
                                error!("build_update: error: {}", err);
                                std::process::exit(1);
                            }
                        };
                        // Do the call to update
                        match client.request(update).await {
                            Ok(_) => {}
                            Err(err) => error!("request: error: cannot update host_uuid: {}", err),
                        }

                        // Retry to send the request, but we need to rebuild it first...
                        // We don't care if this fail, this is simply a means to push asap
                        // the failed data.
                        if let Ok(request) = build_request(&config.api_token, &data_cache) {
                            match client.request(request).await {
                                Ok(resp_body) => {
                                    if resp_body.status() == StatusCode::OK {
                                        data_cache.clear();
                                        trace!("data_cache has been cleared");
                                    }
                                }
                                Err(_) => {
                                    should_drain = true;
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    error!("request: error: {}", e);
                    should_drain = true;
                }
            }

            // We reach here in case of error in the client.request above
            // If data_cache contains too many items due to previous error
            if should_drain && data_cache.len() as i64 >= cache_size * 2 {
                // drain the first (older) items to avoid taking too much memory
                let to_drain = cache_size / 2;
                data_cache.drain(0..to_drain as usize);
                warn!("draining [0..{}] items of the data_cache", to_drain)
            }
        }

        // Wait config.harvest_interval before running again
        // For syncing interval must be greater or equals to the harvest_interval
        // so just base this sleep on the harvest_interval value.
        //
        // Doing so doesn't guarantee that we'll gather values every config.harvest_interval
        // due to the time we take to gather data and send it over the network.
        // Gathering and sending is not async so it's more like (time_to_gather_&_send + config.harvest_interval).
        thread::sleep(Duration::from_secs(config.harvest_interval));
    }
}
