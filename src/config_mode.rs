use crate::models;

use models::Config;
use std::io::{self, Write};

pub fn entry_point() {
    print!("\nWhat is your api_token ?\n > ");
    io::stdout().flush().unwrap();
    let api_token: String = read!("{}\n");
    print!("What is your api_url ?\n > ");
    io::stdout().flush().unwrap();
    let api_url: String = read!("{}\n");

    let home: String = match dirs::home_dir() {
        Some(val) => val.to_string_lossy().into_owned(),
        None => String::from("/"),
    };

    let config = Config { api_token, api_url };

    let path = format!("{}/speculare.config", home);
    let res = std::fs::write(&path, serde_json::to_string(&config).unwrap());
    if res.is_err() {
        error!("Can't write file to {}\nError: {:?}", &path, res.err());
        return;
    }
    println!("\nThe config has been written at {}/speculare.config", home);
}
