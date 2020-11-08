use log::{info, warn};

/// Syslog is the function to call anytime you want to log something
/// or you want to crash, depend on what kind of error you're facing.
pub fn syslog(message: String, fail: bool, warn: bool) {
    if !fail {
        if warn {
            warn!("{}", message);
        } else {
            info!("{}", message);
        }
    }
    panic!(message);
}
