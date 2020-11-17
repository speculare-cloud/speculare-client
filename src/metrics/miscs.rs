use super::read_and_trim;

use crate::models;

use models::{HostInfo, LoadAvg, Memory};
use nix::sys;
use std::io::{Error, ErrorKind};

use io_kit_sys::*;
use core_foundation_sys::base::CFTypeRef;
use core_foundation_sys::string::CFStringRef;
use io_kit_sys::types::io_registry_entry_t;
use core_foundation_sys::base::CFAllocatorRef;
use io_kit_sys::types::IOOptionBits;
use std::ffi::CString;
use libc::c_char;

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
    let mut serial: CFTypeRef = std::ptr::null();
    unsafe {
        // That's a u32
        let IOPlatformExpertDevice = std::ffi::CString::new("IOPlatformExpertDevice").unwrap().as_ptr() as *const c_char;
        let platformExpert = IOServiceGetMatchingService(0, IOServiceMatching(IOPlatformExpertDevice));
        if platformExpert != 0 {
            let kIOPlatformSerialNumberKey = std::ffi::CString::new("IOPlatformSerialNumber").unwrap().as_ptr() as *const c_char;
            let serialNumberAsCFString: CFTypeRef = IORegistryEntryCreateCFProperty(platformExpert, CFSTR(kIOPlatformSerialNumberKey), std::ptr::null(), 0);
            if serialNumberAsCFString != std::ptr::null() {
                serial = serialNumberAsCFString as CFTypeRef;
            }
            IOObjectRelease(platformExpert);
        }
    };
    dbg!(serial);
    return Ok(unsafe {std::ffi::CStr::from_ptr(serial as *const _)}.to_string_lossy().to_string());
}