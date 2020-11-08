use psutil::host;

/// Get the os version (Mac/Linux/Windows) in a safe String.
/// Take approx 0,080ms to load the info 'os_info::get()'.
/// But it's okay since we'll only call this function once in a while.
pub fn get_os_version() -> String {
    let info = os_info::get();
    let mut os_version = info.os_type().to_string();
    os_version.push_str(&info.version().to_string());
    return os_version;
}

/// Get the machine UUID (Mac/Linux/Windows) as a String.
/// 
/// TODO
pub fn get_uuid() -> String {
    todo!()
}

/// Get the hostname (Mac/Linux/Windows) in a safe String.
pub fn get_hostname() -> String {
    host::info().hostname().to_string()
}

/// Return the uptime of the current host.
/// In seconds and as i64 due to the database not handling u64.
pub fn get_uptime() -> i64 {
    match host::uptime() {
        Ok(val) => val.as_secs() as i64,
        Err(_) => 0,
    }
}
