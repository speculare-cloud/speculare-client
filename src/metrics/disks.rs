use crate::models;

#[cfg(target_os = "macos")]
use futures::executor;
#[cfg(target_os = "macos")]
use futures_util::stream::StreamExt;
use models::{Disks, IoStats};
use psutil::disk;
#[cfg(target_family = "unix")]
use std::io::Error;
#[cfg(target_os = "linux")]
use std::{
    fs::File,
    io::{prelude::*, BufReader},
};

/// Retrieve the disks and return them as a Vec<Disks>.
pub fn get_disks_data() -> Vec<Disks> {
    let partitions = disk::partitions_physical().unwrap();
    let mut vdisks: Vec<Disks> = Vec::with_capacity(partitions.len());

    for disk in partitions {
        let mount = disk.mountpoint();
        let disk_usage = disk::disk_usage(mount).unwrap();
        vdisks.push(Disks {
            name: disk.device().to_string(),
            mount_point: mount.display().to_string(),
            total_space: (disk_usage.total() / 100000) as i64,
            avail_space: (disk_usage.free() / 100000) as i64,
        })
    }

    vdisks
}

#[cfg(target_os = "linux")]
pub fn get_iostats() -> Result<Vec<IoStats>, Error> {
    let file = match File::open("/proc/diskstats") {
        Ok(val) => val,
        Err(x) => return Err(x),
    };
    let mut reader = BufReader::new(file);
    let mut buffer = String::with_capacity(128);
    let mut viostats: Vec<IoStats> = Vec::new();

    while reader.read_line(&mut buffer).unwrap_or(0) > 0 {
        let fields = buffer.split_whitespace().collect::<Vec<&str>>();
        if fields.len() < 14 {
            buffer.clear();
            continue;
        }
        viostats.push(IoStats {
            device_name: fields[2].to_owned(),
            sectors_read: fields[5].parse::<i64>().unwrap(),
            sectors_wrtn: fields[9].parse::<i64>().unwrap(),
        });
        buffer.clear();
    }

    Ok(viostats)
}

#[cfg(target_os = "macos")]
pub fn get_iostats() -> Result<Vec<IoStats>, Error> {
    let mut viostats: Vec<IoStats> = Vec::new();

    let mut counters = heim_disk::io_counters();
    while let Some(count) = executor::block_on(counters.next()) {
        let count = count.unwrap();
        viostats.push(IoStats {
            device_name: count.device_name().to_string_lossy().to_string(),
            sectors_read: count.read_bytes().value as i64,
            sectors_wrtn: count.write_bytes().value as i64,
        });
    }

    Ok(viostats)
}
