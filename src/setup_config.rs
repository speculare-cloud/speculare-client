use crate::models::Config;

use std::fs::{set_permissions, write, Permissions};
use std::io::{stdout, Write};
use std::os::unix::fs::PermissionsExt;

pub fn config_mode() {
    // Get the api_url
    print!("What is your api_token ?\n > ");
    stdout().flush().unwrap();
    let api_token: String = read!("{}\n");

    // Get the api_url
    print!("What is your api_url ?\n > ");
    stdout().flush().unwrap();
    let api_url: String = read!("{}\n");

    // Determine the path where the config should be saved
    let mut path: String = match dirs::home_dir() {
        Some(val) => val.to_string_lossy().into_owned(),
        None => String::from("/"),
    };
    // Asking the user if we should change the default
    print!("Where should we save the config ? [{}]\n > ", path);
    stdout().flush().unwrap();
    let ask_path: String = read!("{}\n");
    // If the return is not "", set path to the value
    if ask_path != "" {
        path = ask_path;
    }

    // Create the config object
    let config = Config { api_token, api_url };
    // Construct our entire path
    let path = format!("{}/speculare.config", path);
    // Write the config the our file
    let res = write(&path, serde_json::to_string(&config).unwrap());
    if res.is_err() {
        error!("Can't write file to {}\nError: {:?}", &path, res.err());
        return;
    }
    println!("\nThe config has been written at {}/speculare.config", path);

    // Change permission over the file, only the current user can read/modify it
    match set_permissions(&path, Permissions::from_mode(0o600)) {
        Ok(_) => println!("Successfully changing permission of the config file to be accessible only by the current user."),
        Err(x) => println!("Failed to change the permission of the config file (attempted 600) due to {}", x)
    };
}
