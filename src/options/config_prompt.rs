use crate::Config;

use std::fs::{create_dir_all, set_permissions, write, Permissions};
use std::io::{stdout, Write};
use std::os::unix::fs::PermissionsExt;
use termion::{color, style};

macro_rules! cwrite {
    ($($arg:expr),*) => {
        $(print!("{}", $arg);)*
        stdout().flush().unwrap();
    };
}

pub fn get_config_prompt() {
    // Get the api_url
    cwrite!(format!(
        "What is your api_token ?\n > {}",
        color::Fg(color::Blue)
    ));
    let api_token: String = read!("{}\n");

    // Get the api_url
    cwrite!(format!(
        "{}What is your api_url ? (https://[DOMAIN]:[PORT]/api/guard/hosts)\n > {}",
        color::Fg(color::Reset),
        color::Fg(color::Blue)
    ));
    let api_url: String = read!("{}\n");

    // Get the harvest_interval
    let mut harvest_interval: u64 = 1;
    cwrite!(format!(
        "{}How often do you want to harvest data from the host? (secs) [default: {}]\n > {}",
        color::Fg(color::Reset),
        harvest_interval,
        color::Fg(color::Blue)
    ));
    let ask_harvest_interval: String = read!("{}\n");
    if !ask_harvest_interval.is_empty() {
        harvest_interval = ask_harvest_interval.parse::<u64>().unwrap_or(1);
    }

    // Get the syncing_interval
    let mut syncing_interval: u64 = 1;
    cwrite!(format!(
        "{}How often do you want to send data to the server? (secs) [default: {}]\n{}{}{}\n > {}",
        color::Fg(color::Reset),
        syncing_interval,
        style::Italic,
        "Note: this must be a multiple of the harvest_interval.",
        style::Reset,
        color::Fg(color::Blue)
    ));
    let ask_syncing_interval: String = read!("{}\n");
    if !ask_syncing_interval.is_empty() {
        syncing_interval = ask_syncing_interval.parse::<u64>().unwrap_or(1);
    }

    // Asking the user if we should change the configs path
    let mut conf_path = "/usr/share/speculare/configs";
    cwrite!(format!(
        "{}Where to save the config ? [default: {}]\n > {}",
        color::Fg(color::Reset),
        conf_path,
        color::Fg(color::Blue)
    ));

    let ask_path: String = read!("{}\n");
    // If the ask_path is not empty, set it as our path
    if !ask_path.is_empty() {
        conf_path = &ask_path;
    }

    // Asking the user if we should change the plugin path
    let mut plug_path = "/usr/share/speculare/plugins";
    cwrite!(format!(
        "{}Where to look for plugins ? [default: {}]\n > {}",
        color::Fg(color::Reset),
        plug_path,
        color::Fg(color::Blue)
    ));

    let ask_path: String = read!("{}\n");
    // If the ask_path is not empty, set it as our path
    if !ask_path.is_empty() {
        plug_path = &ask_path;
    }
    // Reset color of the terminal
    cwrite!(format!("{}{}", style::Reset, color::Fg(color::Reset)));
    // Create the config object
    let config = Config {
        api_token,
        api_url,
        harvest_interval,
        syncing_interval,
        plugins_path: plug_path.to_owned(),
    };
    // Create the configs folder
    match create_dir_all(conf_path) {
        Ok(_) => {}
        Err(x) => {
            println!("Cannot create folders `{}` due to {}", conf_path, x);
            return;
        }
    };
    // Create the plugins folder
    match create_dir_all(plug_path) {
        Ok(_) => {}
        Err(x) => {
            println!("Cannot create folders `{}` due to {}", plug_path, x);
            return;
        }
    };
    // Construct our entire path
    let path = format!("{}/speculare.config", conf_path);
    // Write the config the our file
    let res = write(&path, serde_json::to_string_pretty(&config).unwrap());
    if res.is_err() {
        error!("Can't write file to {}\nError: {:?}", &path, res.err());
        return;
    }
    println!("\nThe configuration has been saved at {}", path);

    // Change permission over the file, only the current user can read/modify it
    match set_permissions(&path, Permissions::from_mode(0o600)) {
        Ok(_) => println!("Successfully changed the permissions of the config file (chmod 600)."),
        Err(x) => println!(
            "Failed to change the permission of the config file (attempted 600) due to {}",
            x
        ),
    };
}
