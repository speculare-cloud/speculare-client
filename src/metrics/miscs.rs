#[cfg(target_os = "linux")]
use super::read_and_trim;

use crate::models;

use models::{HostInfo, LoadAvg, Memory};
use nix::sys;
use std::io::{Error, ErrorKind};

#[cfg(target_os = "macos")]
use core_foundation_sys::{
    base::CFTypeRef,
    string::{CFStringGetCString, CFStringRef},
};
#[cfg(target_os = "macos")]
use io_kit_sys::*;
#[cfg(target_os = "macos")]
use libc::c_char;
#[cfg(target_os = "macos")]
use std::ffi::{CStr, CString};

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

/// Get the machine UUID (Linux) as a String.
/// LINUX => Will read it from /etc/machine-id or /var/lib/dbus/machine-id.
#[cfg(target_os = "linux")]
pub fn get_uuid() -> Result<String, Error> {
    match read_and_trim("/etc/machine-id") {
        Ok(machine_id) => Ok(machine_id),
        Err(_) => Ok(read_and_trim("/var/lib/dbus/machine-id")?),
    }
}
#[cfg(target_os = "macos")]
pub fn get_uuid() -> Result<String, Error> {
    let serial_number;
    let buffer = match CString::new(String::with_capacity(64)) {
        Ok(val) => val,
        Err(x) => return Err(Error::new(ErrorKind::Other, x)),
    };
    #[allow(unused_assignments)]
    let mut serial: CFStringRef = std::ptr::null();
    unsafe {
        // We need to keep track of CString else it will cause a freed error
        // cf: https://github.com/rust-lang/rust/issues/56603
        let ioexpertdevice = match CString::new("IOPlatformExpertDevice") {
            Ok(val) => val,
            Err(x) => return Err(Error::new(ErrorKind::Other, x)),
        };
        let plat_exp = IOServiceGetMatchingService(0, IOServiceMatching(ioexpertdevice.as_ptr()));
        if plat_exp != 0 {
            let ioplatserialnumber = match CString::new("IOPlatformSerialNumber") {
                Ok(val) => val,
                Err(x) => return Err(Error::new(ErrorKind::Other, x)),
            };
            let serial_number_cft: CFTypeRef = IORegistryEntryCreateCFProperty(
                plat_exp,
                CFSTR(ioplatserialnumber.as_ptr()),
                std::ptr::null(),
                0,
            );
            if serial_number_cft != std::ptr::null() {
                serial = serial_number_cft as CFStringRef;
            } else {
                return Err(Error::new(ErrorKind::Other, "Cannot get serial_number_cft"));
            }
            if CFStringGetCString(serial, buffer.as_ptr() as *mut c_char, 64, 134217984) != 0 {
                serial_number = match CStr::from_ptr(buffer.as_ptr()).to_str() {
                    Ok(val) => val.to_owned(),
                    Err(x) => return Err(Error::new(ErrorKind::Other, x)),
                }
            } else {
                return Err(Error::new(
                    ErrorKind::Other,
                    "Cannot convert serial number to a String",
                ));
            }
            IOObjectRelease(plat_exp);
        } else {
            return Err(Error::new(
                ErrorKind::Other,
                "Cannot get the IOServiceGetMatchingService",
            ));
        }
    };
    Ok(serial_number)
}
