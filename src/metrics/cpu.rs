use std::{
    fs::File,
    io::{prelude::*, BufReader, Error},
};

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
    todo!()
}
