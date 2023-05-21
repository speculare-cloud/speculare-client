use crate::utils::request::build_client;
#[cfg(feature = "auth")]
use crate::utils::request::build_update;
use crate::CONFIG;
use crate::{harvest::Data, utils::request::build_request};

use async_recursion::async_recursion;
use hyper::{client::HttpConnector, Client};
use hyper::{Body, Response, StatusCode};
use hyper_rustls::HttpsConnector;
use std::{thread, time::Duration};
use tokio::time::timeout;

pub struct SpClient {
    client: Client<HttpsConnector<HttpConnector>>,

    pub cache_size: i64,
    pub data: Data,
    pub data_cache: Vec<Data>,
    pub sync_threshold: i64,
    pub loadavg_threshold: i64,
    pub sync_track: i64,
}

impl Default for SpClient {
    fn default() -> Self {
        let sync_threshold = (CONFIG.harvest_interval * CONFIG.syncing_interval) as i64;
        let loadavg_threshold = (CONFIG.harvest_interval * CONFIG.loadavg_interval) as i64;
        let cache_size = std::cmp::max(sync_threshold, CONFIG.cache_size);

        Self {
            client: build_client(),
            cache_size,
            data: Data::default(),
            data_cache: Vec::with_capacity(cache_size as usize),
            sync_threshold,
            loadavg_threshold,
            sync_track: -1,
        }
    }
}

impl SpClient {
    fn harvest_data(&mut self) {
        // Refresh / Populate the Data structure
        self.data
            .eat_data(self.sync_track % self.loadavg_threshold == 0);

        // Saving data in a temp var/space if we don't sync it right away
        self.data_cache.push(self.data.clone());
        trace!(
            "data_cache pushed, current occupation: {} / {}",
            self.data_cache.len(),
            self.data_cache.capacity()
        );
    }

    fn enforce_cache_limit(&mut self) {
        // We reach here in case of error in the client.request above
        // If data_cache contains too many items due to previous error
        if self.data_cache.len() as i64 >= self.cache_size * 2 {
            // drain the first (older) items to avoid taking too much memory
            let to_drain = self.cache_size / 2;
            self.data_cache.drain(0..to_drain as usize);
            warn!("draining [0..{}] items of the data_cache", to_drain)
        }
    }

    fn prepare_request(&self) -> hyper::Request<hyper::Body> {
        // Building the request to be sent to the server
        match build_request(&CONFIG.api_token, &self.data_cache) {
            Ok(req) => req,
            Err(err) => {
                error!("build_request: error: {}", err);
                std::process::exit(1);
            }
        }
    }

    #[cfg(feature = "auth")]
    fn prepare_update(&self) -> hyper::Request<hyper::Body> {
        match build_update(&CONFIG.api_token) {
            Ok(req) => req,
            Err(err) => {
                error!("build_update: error: {}", err);
                std::process::exit(1);
            }
        }
    }

    async fn handle_response(&mut self, resp: Response<Body>) {
        trace!("request: response: {}", resp.status());
        if resp.status() == StatusCode::OK {
            self.data_cache.clear();
            #[cfg(feature = "auth")]
            return;
        } else {
            trace!("request: full response: {:?}", resp);
        }

        #[cfg(feature = "auth")]
        if resp.status() == StatusCode::PRECONDITION_FAILED {
            warn!("The host_uuid is not defined for this key, updating...");
            let update = self.prepare_update();

            if let Err(err) = self.client.request(update).await {
                error!("request: error: cannot update host_uuid: {}", err);
            }

            // Republish the data now that the host is registered
            // on the auth server (thanks to prepare_update)
            self.publish_data().await;
        }
    }

    #[async_recursion]
    async fn publish_data(&mut self) {
        let request = self.prepare_request();

        let future = self.client.request(request);
        match timeout(Duration::from_secs(5), future).await {
            Ok(v) => {
                match v {
                    Ok(r) => self.handle_response(r).await,
                    Err(err) => error!("request: error: {}", err),
                };
            }
            Err(_) => error!("request: error: timed out"),
        };
    }

    pub async fn serve(&mut self) -> std::io::Result<()> {
        let config_interval: Duration = Duration::from_secs(CONFIG.harvest_interval);

        loop {
            let start_overall = std::time::Instant::now();

            self.sync_track += 1;

            self.harvest_data();

            if self.sync_track % self.sync_threshold == 0 {
                self.publish_data().await;
            }

            self.enforce_cache_limit();

            let duration_overall = start_overall.elapsed();
            // Wait config.harvest_interval before running again
            // For syncing interval must be greater or equals to the harvest_interval
            // so just base this sleep on the harvest_interval value.
            //
            // Doing so doesn't guarantee that we'll gather values every config.harvest_interval
            // due to the time we take to gather data and send it over the network.
            // Gathering and sending is not async so it's more like (time_to_gather_&_send + config.harvest_interval).
            if duration_overall < config_interval {
                trace!(
                    "Sleeping: {:?} because execution took: {:?} vs {:?}",
                    config_interval - duration_overall,
                    duration_overall,
                    config_interval
                );
                thread::sleep(config_interval - duration_overall);
            } else {
                warn!(
                    "Skipping sleep as execution took longer than config_interval ({:?})",
                    duration_overall
                );
            }
        }
    }
}
