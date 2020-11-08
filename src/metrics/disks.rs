use crate::models;

use models::Disks;
use psutil::disk;

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
