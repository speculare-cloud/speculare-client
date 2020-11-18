#[cfg(target_os = "macos")]
use libc::{c_uint, c_void, sysctl};
#[cfg(target_family = "unix")]
use std::io::Error;
#[cfg(target_os = "macos")]
use std::io::ErrorKind;
#[cfg(target_os = "linux")]
use std::{
    fs::File,
    io::{prelude::*, BufReader},
};

/// Return the avg (not yet a true avg) cpu_freq as f64.
#[cfg(target_os = "linux")]
pub fn get_avg_cpufreq() -> Result<f64, Error> {
    let file = File::open("/proc/cpuinfo")?;
    let file = BufReader::with_capacity(1024, file);

    for line in file.lines() {
        let line = line.unwrap();
        let lenght = line.len();
        if lenght > 7 && lenght < 48 && &line[..7] == "cpu MHz" {
            match line[11..lenght - 1].parse::<f64>() {
                Ok(val) => return Ok(val),
                Err(_) => continue,
            };
        }
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
