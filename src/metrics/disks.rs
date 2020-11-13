use crate::models;

use models::{Disks, IoStats};
use psutil::disk;
use std::{
    fs::File,
    io::{prelude::*, BufReader, Error},
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
    let mut buffer = String::with_capacity(64);
    let mut viostats: Vec<IoStats> = Vec::new();

    while reader.read_line(&mut buffer).unwrap_or(0) > 0 {
        let fields = buffer.split_whitespace().collect::<Vec<&str>>();
        if fields.len() < 14 {
            continue;
        }
        viostats.push(IoStats {
            device_name: fields[2].to_owned(),
            sectors_read: fields[5].as_ptr() as i64,
            sectors_wrtn: fields[9].as_ptr() as i64,
        });
        buffer.clear();
    }

    Ok(viostats)
}
