#[cfg(target_os = "linux")]
use super::read_and_trim;

use crate::models;

#[cfg(target_os = "macos")]
use core_foundation_sys::{
    base::CFTypeRef,
    string::{CFStringGetCString, CFStringRef},
};
#[cfg(target_os = "macos")]
use crypto::digest::Digest;
#[cfg(target_os = "macos")]
use io_kit_sys::*;
#[cfg(target_os = "macos")]
use libc::c_char;
#[cfg(target_family = "unix")]
use libc::{c_double, getloadavg};
#[cfg(target_os = "macos")]
use libc::{c_void, sysctl, timeval};
use models::{HostInfo, LoadAvg, Memory};
use nix::sys;
#[cfg(target_os = "macos")]
use std::ffi::{CStr, CString};
use std::io::{Error, ErrorKind};
#[cfg(target_os = "macos")]
use std::time::Duration;

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

#[cfg(target_os = "macos")]
pub fn get_uptime() -> Result<Duration, Error> {
    let mut data: timeval = unsafe { std::mem::zeroed() };
    let mib = [1, 21];
    let ret = unsafe {
        sysctl(
            &mib[0] as *const _ as *mut _,
            mib.len() as u32,
            &mut data as *mut _ as *mut c_void,
            &mut std::mem::size_of::<timeval>(),
            std::ptr::null_mut(),
            0,
        )
    };
    if ret < 0 {
        Err(Error::new(ErrorKind::Other, "Invalid return for sysctl"))
    } else {
        Ok(Duration::from_secs(data.tv_sec as u64))
    }
}

#[cfg(target_family = "unix")]
pub fn get_loadavg() -> Result<LoadAvg, Error> {
    let mut data: [c_double; 3] = [0.0, 0.0, 0.0];
    if unsafe { getloadavg(data.as_mut_ptr(), 3) } == -1 {
        return Err(Error::new(
            ErrorKind::Other,
            "Invalid return for getloadavg",
        ));
    }
    Ok(LoadAvg {
        one: data[0],
        five: data[1],
        fifteen: data[2],
    })
}

/// Get both hostname and os_version from the same single uname instance.
#[cfg(target_os = "linux")]
pub fn get_host_info() -> Result<HostInfo, Error> {
    let x = sys::utsname::uname();
    let y = match sys::sysinfo::sysinfo() {
        Ok(val) => val,
        Err(x) => return Err(Error::new(ErrorKind::Other, x)),
    };
    let uptime = y.uptime().as_secs() as i64;
    let loadavg_raw = y.load_average();
    let loadavg = LoadAvg {
        one: loadavg_raw.0,
        five: loadavg_raw.1,
        fifteen: loadavg_raw.2,
    };
    let memory = Memory {
        total_virt: y.ram_total() as i64,
        total_swap: y.swap_total() as i64,
        avail_virt: y.ram_unused() as i64,
        avail_swap: y.swap_free() as i64,
    };
    Ok(HostInfo {
        loadavg,
        memory,
        uptime,
        os_version: x.sysname().to_owned() + "/" + x.release(),
        hostname: x.nodename().to_owned(),
    })
}

/// Get both hostname and os_version from the same single uname instance.
#[cfg(target_os = "macos")]
pub fn get_host_info() -> Result<HostInfo, Error> {
    let x = sys::utsname::uname();
    let uptime = get_uptime().unwrap().as_secs() as i64;
    let loadavg = get_loadavg().unwrap();
    // TODO
    let memory = Memory {
        total_virt: 0,
        total_swap: 0,
        avail_virt: 0,
        avail_swap: 0,
    };
    Ok(HostInfo {
        loadavg,
        memory,
        uptime,
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
/// Get the machine Serial Number (macOS) as a String.
/// macOS => Will get it from some black magic extern C function.
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
            if !serial_number_cft.is_null() {
                serial = serial_number_cft as CFStringRef;
            } else {
                return Err(Error::new(ErrorKind::Other, "Cannot get serial_number_cft"));
            }
            if CFStringGetCString(serial, buffer.as_ptr() as *mut c_char, 64, 134217984) != 0 {
                let mut hasher = crypto::sha3::Sha3::sha3_256();
                serial_number = match CStr::from_ptr(buffer.as_ptr()).to_str() {
                    Ok(val) => {
                        hasher.input_str(val);
                        hasher.result_str()
                    }
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
