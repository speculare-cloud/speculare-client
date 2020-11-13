use nix::sys;
use psutil::host;
use std::io::{Error, ErrorKind};

/// Get the os version (Mac/Linux/Windows) in a safe String.
/// Take approx 0,080ms to load the info 'os_info::get()'.
/// But it's okay since we'll only call this function once in a while.
pub fn get_os_version() -> String {
    let x = sys::utsname::uname();
    x.sysname().to_owned() + " " + x.release()
}

/// Get the machine UUID (Mac/Linux/Windows) as a String.
/// This one is slow as the underlying lib invoke a shell command
/// if the host os is Mac. Else it will read it from /etc/machine-id or /var/lib/dbus/machine-id.
pub fn get_uuid() -> Result<String, Error> {
    match host::get_machine_id() {
        Ok(val) => Ok(val),
        Err(x) => Err(Error::new(ErrorKind::Other, x)),
    }
}

/// Get the hostname (Mac/Linux/Windows) in a safe String.
pub fn get_hostname() -> String {
    host::info().hostname().to_owned()
}

/// Return the uptime of the current host.
/// In seconds and as i64 due to the database not handling u64.
pub fn get_uptime() -> i64 {
    match host::uptime() {
        Ok(val) => val.as_secs() as i64,
        Err(_) => 0,
    }
}
