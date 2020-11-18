use std::{
    fs::File,
    io::{prelude::*, BufReader, Error},
};

#[cfg(target_os = "macos")]
use std::io::ErrorKind;
#[cfg(target_os = "macos")]
use libc::{c_uint, sysctl, c_void};

/// Return the avg (not yet a true avg) cpu_freq as f64.
#[cfg(target_os = "linux")]
pub fn get_avg_cpufreq() -> Result<f64, Error> {
    let file = match File::open("/proc/cpuinfo") {
        Ok(val) => val,
        Err(x) => return Err(x),
    };

    let mut reader = BufReader::with_capacity(1024, file);
    let mut buffer = String::with_capacity(1024);

    while reader.read_line(&mut buffer).unwrap_or(0) > 0 {
        let lenght = buffer.len();
        if lenght > 7 && lenght < 48 && &buffer[..7] == "cpu MHz" {
            match buffer[11..lenght - 1].parse::<f64>() {
                Ok(val) => return Ok(val),
                Err(_) => {
                    buffer.clear();
                    continue;
                }
            };
        }
        buffer.clear();
    }

    Ok(-1.0)
}

#[cfg(target_os = "macos")]
pub fn get_avg_cpufreq() -> Result<f64, Error> {
    let mut data: c_uint = 0;
    let mib = [6, 15];
    let ret = unsafe {
        sysctl(
            &mib[0] as *const _ as *mut _,
            mib.len() as u32,
            &mut data as *mut _ as *mut c_void,
            &mut std::mem::size_of::<c_uint>(),
            std::ptr::null_mut(),
            0,
        )
    };
    if ret < 0 {
        Err(Error::new(ErrorKind::Other, "Invalid return for sysctl"))
    } else {
        Ok(data as f64)
    }
}
