use clap::ArgMatches;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::BufReader;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub api_token: String,
    pub api_url: String,
}

/// Get the correct path for the config, open it and read it to the Config struct
/// which is then returned.
pub fn get_config(args: &ArgMatches) -> Config {
    // Determine the path of the config
    let config_path = if args.is_present("path") {
        args.value_of("path").unwrap()
    } else {
        "/etc/speculare/speculare.config"
    };

    // Open the config_file as File
    let config_file = match File::open(&config_path) {
        Ok(val) => val,
        Err(x) => {
            panic!("Can't open {}\nError: {}", &config_path, x);
        }
    };

    // Create a reader from the config_file
    let config_reader = BufReader::new(&config_file);

    // Convert the reader into Config struct
    match serde_json::from_reader(config_reader) {
        Ok(val) => val,
        Err(x) => {
            panic!("Can't convert {}\nError: {}", &config_path, x);
        }
    }
}
