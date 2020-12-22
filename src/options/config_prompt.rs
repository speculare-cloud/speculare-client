use crate::Config;

use std::fs::{create_dir_all, set_permissions, write, Permissions};
use std::io::{stdout, Write};
use std::os::unix::fs::PermissionsExt;

pub fn get_config_prompt() {
    // Get the api_url
    print!("What is your api_token ?\n > ");
    stdout().flush().unwrap();
    let api_token: String = read!("{}\n");

    // Get the api_url
    print!("What is your api_url ?\n > ");
    stdout().flush().unwrap();
    let api_url: String = read!("{}\n");

    // Get the harvest_interval
    let mut harvest_interval: u64 = 1;
    print!(
        "At which interval do you want to harvest metrics from the host (secs) ? [default: {}]\n > ",
        harvest_interval
    );
    stdout().flush().unwrap();
    let ask_harvest_interval: String = read!("{}\n");
    if !ask_harvest_interval.is_empty() {
        harvest_interval = ask_harvest_interval.parse::<u64>().unwrap_or(1);
    }

    // Get the syncing_interval
    let mut syncing_interval: u64 = 1;
    print!(
        "At which interval do you want to send the data to the server (secs) ? [default: {}]\nNote: this must be a multiple of the harvest_interval.\n > ",
        syncing_interval
    );
    stdout().flush().unwrap();
    let ask_syncing_interval: String = read!("{}\n");
    if !ask_syncing_interval.is_empty() {
        syncing_interval = ask_syncing_interval.parse::<u64>().unwrap_or(1);
    }

    // Asking the user if we should change the default path
    let mut path = "/etc/speculare";
    print!("Where should we save the config ? [{}]\n > ", path);
    stdout().flush().unwrap();
    let ask_path: String = read!("{}\n");
    // If the ask_path is not empty, set it as our path
    if !ask_path.is_empty() {
        path = &ask_path;
    }

    // Create the config object
    let config = Config {
        api_token,
        api_url,
        harvest_interval,
        syncing_interval,
    };
    // Create the folders (for the path)
    match create_dir_all(path) {
        Ok(_) => {}
        Err(x) => {
            println!("Cannot create folders `{}` due to {}", path, x);
            return;
        }
    };
    // Construct our entire path
    let path = format!("{}/speculare.config", path);
    // Write the config the our file
    let res = write(&path, serde_json::to_string(&config).unwrap());
    if res.is_err() {
        error!("Can't write file to {}\nError: {:?}", &path, res.err());
        return;
    }
    println!("\nThe config has been written at {}", path);

    // Change permission over the file, only the current user can read/modify it
    match set_permissions(&path, Permissions::from_mode(0o600)) {
        Ok(_) => println!("Successfully changing permission of the config file (600)."),
        Err(x) => println!(
            "Failed to change the permission of the config file (attempted 600) due to {}",
            x
        ),
    };
}
