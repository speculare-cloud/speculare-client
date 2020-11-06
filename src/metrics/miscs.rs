use crate::utils;

use psutil::host;
use sentry::integrations::anyhow::capture_anyhow;
use utils::syslog;

/// Get the os version (Mac/Linux/Windows) in a safe String
/// Capture the error and send it to sentry + print it
/// TODO - Should change the return value in case of an error
pub fn get_os_version() -> String {
    match os_version::detect() {
        Ok(val) => val.to_string(),
        Err(x) => {
            capture_anyhow(&x);
            syslog(x.to_string(), false, true, false);
            x.to_string()
        }
    }
}

/// Get the machine UUID (Mac/Linux/Windows) as a String
/// TODO - Should change the return value in case of an error
pub fn get_uuid() -> String {
    match machine_uid::get() {
        Ok(val) => val,
        Err(x) => {
            syslog(x.to_string(), false, true, true);
            x.to_string()
        }
    }
}

/// Get the hostname (Mac/Linux/Windows) in a safe String
/// Capture the error and send it to sentry + print it
/// TODO - Should change the return value in case of an error
pub fn get_hostname() -> String {
    host::info().hostname().to_string()
}

/// Return the uptime of the current host
/// In seconds and as i64 due to the database not handling u64
/// TODO - It's unsecure to cast to i64 but faster :(
pub fn get_uptime() -> i64 {
    host::uptime().unwrap().as_secs() as i64
}
