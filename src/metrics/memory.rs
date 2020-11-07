use crate::models;

use models::Memory;
use psutil::memory;

/// Retrieve the Memory from Virt and Swap and return them as a Memory
pub fn get_memory() -> Memory {
    let (total_virt, avail_virt) = match memory::virtual_memory() {
        Ok(val) => (val.total() as i64, val.available() as i64),
        Err(_) => (0, 0),
    };
    let (total_swap, avail_swap) = match memory::swap_memory() {
        Ok(val) => (val.total() as i64, val.free() as i64),
        Err(_) => (0, 0),
    };

    Memory {
        total_virt,
        total_swap,
        avail_virt,
        avail_swap,
    }
}
