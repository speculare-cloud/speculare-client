use crate::models;

use models::{HostInfo, LoadAvg, Memory};
use nix::sys;
use psutil::host;
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
pub fn get_host_info() -> Result<HostInfo, Error> {
    let x = sys::utsname::uname();
    let y = match sys::sysinfo::sysinfo() {
        Ok(val) => val,
        Err(x) => return Err(Error::new(ErrorKind::Other, x)),
    };
    let loadavg = y.load_average();
    Ok(HostInfo {
        loadavg: LoadAvg {
            one: loadavg.0,
            five: loadavg.1,
            fifteen: loadavg.2,
        },
        memory: Memory {
            total_virt: y.ram_total() as i64,
            total_swap: y.swap_total() as i64,
            avail_virt: y.ram_unused() as i64,
            avail_swap: y.swap_free() as i64,
        },
        uptime: y.uptime().as_secs() as i64,
        os_version: x.sysname().to_owned() + "/" + x.release(),
        hostname: x.nodename().to_owned(),
    })
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
