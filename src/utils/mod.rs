use sys_metrics::host::{get_host_info, get_uuid};

pub mod config;

pub fn cget_uuid() -> String {
    // UUID can still be empty on some Linux platform (such as WSL)
    // as per https://man7.org/linux/man-pages/man5/machine-id.5.html
    // the machine-id should never be shared on the network or other.
    match get_uuid() {
        Ok(uuid) => sha1_smol::Sha1::from(&uuid).hexdigest(),
        Err(_) => {
            let host_info = match get_host_info() {
                Ok(info) => info,
                Err(e) => {
                    error!("cannot get the host_info(): {}", e);
                    std::process::exit(1);
                }
            };
            sha1_smol::Sha1::from(&host_info.hostname).hexdigest()
        }
    }
}
