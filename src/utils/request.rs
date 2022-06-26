#[cfg(feature = "auth")]
use crate::SSO_URL;
use crate::{harvest::data_harvest::Data, API_URL};

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
    token: &str,
    data_cache: &[Data],
) -> Result<hyper::Request<hyper::Body>, Error> {
    match Request::builder()
        .method(Method::POST)
        .uri(API_URL.clone())
        .header("content-type", "application/json")
        .header("SPTK", token)
        .body(Body::from(simd_json::to_string(data_cache).unwrap()))
    {
        Ok(req) => Ok(req),
        Err(err_req) => Err(Error::new(ErrorKind::Other, err_req)),
    }
}

#[cfg(feature = "auth")]
pub fn build_update(token: &str) -> Result<hyper::Request<hyper::Body>, Error> {
    match Request::builder()
        .method(Method::PATCH)
        .uri(SSO_URL.clone())
        .header("content-type", "application/json")
        .header("SPTK", token)
        .body(Body::default())
    {
        Ok(req) => Ok(req),
        Err(err_req) => Err(Error::new(ErrorKind::Other, err_req)),
    }
}
