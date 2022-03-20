use crate::harvest::data_harvest::Data;

use hyper::{Body, Client, Method, Request};
use std::io::{Error, ErrorKind};

/// Generate the Hyper Client needed for the sync requests
pub fn build_client() -> Client<hyper_rustls::HttpsConnector<hyper::client::HttpConnector>> {
    // Create a Https "client" to be used in the Hyper Client
    let https_conn = hyper_rustls::HttpsConnectorBuilder::new()
        .with_native_roots()
        .https_or_http()
        .enable_http2()
        .build();
    // Create a single Client instance for the app
    Client::builder().build::<_, hyper::Body>(https_conn)
}

/// Generate the Request to be sent by the Hyper Client
pub fn build_request(
    api_url: &str,
    token: &str,
    uuid: &str,
    data_cache: &[Data],
) -> Result<hyper::Request<hyper::Body>, Error> {
    match Request::builder()
        .method(Method::POST)
        .uri(api_url)
        .header("content-type", "application/json")
        .header("SPTK", token)
        .header("SP-UUID", uuid)
        .body(Body::from(simd_json::to_string(data_cache).unwrap()))
    {
        Ok(req) => Ok(req),
        Err(err_req) => Err(Error::new(ErrorKind::Other, err_req)),
    }
}
