#[macro_use]
extern crate text_io;
#[macro_use]
extern crate log;
extern crate libloading as lib;
extern crate num_integer;

mod clap;
mod harvest;
mod logger;
mod options;

use harvest::data_harvest::{Data, Plugin};
use hyper::{Body, Client, Method, Request};
use hyper_tls::HttpsConnector;
use num_integer::Integer;
use options::{
    config::{self},
    config_prompt, Config,
};
use std::collections::HashMap;
use std::{io::Error, thread, time::Duration};

fn build_client() -> Client<hyper_tls::HttpsConnector<hyper::client::HttpConnector>> {
    // Create a Https "client" to be used in the Hyper Client
    let mut https_conn = HttpsConnector::new();
    https_conn.https_only(true);
    // Create a single Client instance for the app
    Client::builder().build::<_, hyper::Body>(https_conn)
}

#[derive(Debug)]
struct PluginInfo {
    pub lib: lib::Library,
    pub func: fn() -> Result<String, Error>,
}

fn get_plugins() -> Option<HashMap<String, PluginInfo>> {
    let mut plugins: HashMap<String, PluginInfo> = HashMap::new();
    let paths = match std::fs::read_dir("./plugins_compiled") {
        Ok(paths_res) => {
            trace!("successfully read the plugins folder");
            paths_res
        }
        Err(err_path) => {
            error!("cannot read dir for plugins: {}", err_path);
            return None;
        }
    };
    for path in paths {
        let path = path.unwrap();
        debug!("check if {:?} is a plugin", path.path());
        let lib = match lib::Library::new(path.path()) {
            Ok(library) => {
                trace!(
                    "the plugin ({:?}) has been loaded correctly",
                    path.file_name()
                );
                library
            }
            Err(err_lib) => {
                error!("the plugin failed to load: {:?}", err_lib);
                continue;
            }
        };
        let func: fn() -> Result<String, Error> = *(unsafe { lib.get(b"entrypoint") }.unwrap());
        plugins.insert(
            path.file_name().into_string().unwrap(),
            PluginInfo { lib, func },
        );
    }
    if !plugins.is_empty() {
        Some(plugins)
    } else {
        None
    }
}

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

    // Build the client instance (*HTTP client)
    let client = build_client();

    // Int keeping track of the sending status// Compute the lcm of harvest_interval and syncing_interval to know when we should sync the data
    // Compute the lcm of harvest_interval and syncing_interval to know when we should sync the data
    let (mut sync_track, sync_threshold) =
        (0, config.harvest_interval.lcm(&config.syncing_interval));

    // Get the default Data instance
    let mut data: Data = Data::default();

    // Syncing memory cache
    let mut data_cache: Vec<Data> = Vec::with_capacity(sync_threshold as usize);
    info!("data_cache with size = {} spaces", sync_threshold);

    // Testing the dynamic lib loading
    // Hardcoded for now, will load dynamically in function of what is present in a folder
    // Need a lot of work to get a great plugin system
    // TODO:
    //  - Find how to get a fixed return type
    //  - Find how to send the info so that the server can understand it correctly
    //  - Add a helper function in the plugin so that the main client can know more info about it (?)
    let raw_plugins = get_plugins();
    let has_plugins = raw_plugins.is_some();
    info!("has plugin: {}", has_plugins);
    let mut plugins = std::mem::MaybeUninit::<HashMap<String, PluginInfo>>::uninit();
    if has_plugins {
        unsafe { plugins.as_mut_ptr().write(raw_plugins.unwrap()) };
    }
    let plugins = unsafe { plugins.assume_init() };

    // Start the app loop (collect metrics and send them)
    loop {
        // Increment track of our syncing status
        sync_track += 1;
        // Refresh / Populate the Data structure
        data.eat_data();
        // Gather data from plugins
        // Only if has_plugins
        if has_plugins {
            // Iterate over each items in the HashMap to gather metrics from all plugins
            for (key, val) in &plugins {
                // Execute the entrypoint and get the return of it
                let res = match (val.func)() {
                    Ok(res_func) => {
                        info!("PLUGIN {} returned: {:?}", key, res_func);
                        res_func
                    }
                    Err(err) => {
                        error!("PLUGIN {} failed with: {}", key, err);
                        continue;
                    }
                };
                // Add the plugin data to the Data struct
                data.add_plugin(Plugin {
                    key: key.to_owned(),
                    val: res,
                });
            }
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
            // TODO - Get rid of these unsafe unwrap
            let request = Request::builder()
                .method(Method::POST)
                .uri(&config.api_url)
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&data_cache).unwrap()))
                .unwrap();
            // Execute the request
            trace!("sending POST request");
            match client.request(request).await {
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
}
