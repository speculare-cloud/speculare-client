use crate::models;

use nix::sys;
use psutil::host;
use models::HostInfo;
use std::io::{Error, ErrorKind};

/// Get the os version (Mac/Linux/Windows) in a safe String.
/// Take approx 0,080ms to load the info 'os_info::get()'.
/// But it's okay since we'll only call this function once in a while.
pub fn get_os_version() -> String {
    let x = sys::utsname::uname();
    x.sysname().to_owned() + "/" + x.release()
}

/// Get the hostname (Mac/Linux/Windows) in a safe String.
pub fn get_hostname() -> String {
    let x = sys::utsname::uname();
    x.nodename().to_owned()
}

/// Get both hostname and os_version from the same single uname instance.
pub fn get_host_info() -> HostInfo {
    let x = sys::utsname::uname();
    HostInfo {
        os_version: x.sysname().to_owned() + "/" + x.release(),
        hostname: x.nodename().to_owned()
    }
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

/// Return the uptime of the current host.
/// In seconds and as i64 due to the database not handling u64.
pub fn get_uptime() -> i64 {
    match host::uptime() {
        Ok(val) => val.as_secs() as i64,
        Err(_) => 0,
    }
}
