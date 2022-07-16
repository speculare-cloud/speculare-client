#[macro_use]
extern crate log;

use crate::client::SpClient;
use crate::utils::config::Config;

use clap::Parser;
use clap_verbosity_flag::InfoLevel;
use once_cell::sync::Lazy;
use std::{ffi::OsStr, path::Path};

mod client;
mod harvest;
mod utils;

#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Args {
    #[clap(short = 'c', long = "config")]
    config_path: Option<String>,

    #[clap(flatten)]
    verbose: clap_verbosity_flag::Verbosity<InfoLevel>,
}

static CONFIG: Lazy<Config> = Lazy::new(|| match Config::new() {
    Ok(config) => config,
    Err(err) => {
        error!("Cannot build the Config: {}", err);
        std::process::exit(1);
    }
});

static API_URL: Lazy<String> = Lazy::new(|| {
    info!(
        "API_URL: {}",
        CONFIG.api_url.clone() + "?uuid=" + &CONFIG.uuid
    );
    CONFIG.api_url.clone() + "?uuid=" + &CONFIG.uuid
});

#[cfg(feature = "auth")]
static SSO_URL: Lazy<String> = Lazy::new(|| {
    info!(
        "SSO_URL: {}",
        CONFIG.sso_url.clone() + "?uuid=" + &CONFIG.uuid
    );
    CONFIG.sso_url.clone() + "?uuid=" + &CONFIG.uuid
});

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
async fn main() -> std::io::Result<()> {
    let args = Args::parse();

    // Define log level
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var(
            "RUST_LOG",
            format!(
                "{}={level}",
                &prog().map_or_else(|| "speculare_client".to_owned(), |f| f.replace('-', "_")),
                level = args.verbose.log_level_filter()
            ),
        )
    }

    // Init logger/tracing
    tracing_subscriber::fmt::init();

    SpClient::default().serve().await
}
