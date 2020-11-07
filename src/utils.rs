use log::{info, warn};

// Will only send the syslog on the iMac
// -> Send message to syslog.s19.be
// -> warn or info the message in the log of the program
// -> panic if needed
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
