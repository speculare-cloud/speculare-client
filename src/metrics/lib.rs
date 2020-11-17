pub mod cpu;
pub mod disks;
pub mod miscs;
pub mod models;
pub mod network;
pub mod sensors;
pub mod users;

use std::{fs, io::Error};

/// Read from path to content, trim it and return the String
pub fn read_and_trim(path: &str) -> Result<String, Error> {
    let content = fs::read_to_string(path)?;
    Ok(content.trim().to_owned())
}
