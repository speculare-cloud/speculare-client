use log::{info, warn};

// Will only send the syslog on the iMac
// -> Send message to syslog.s19.be
// -> warn or info the message in the log of the program
// -> panic if needed
pub fn syslog(message: String, fail: bool, warn: bool, sentry_send: bool) {
    if warn {
        warn!("{}", message);
    } else {
        info!("{}", message);
    }
    if fail {
        panic!(message);
    } else if sentry_send {
        let sentry_level = if warn {
            sentry::Level::Warning
        } else {
            sentry::Level::Info
        };
        sentry::capture_message(&message, sentry_level);
    }
}
