use log::{info, warn};
use std::process::Command;

/* Will only send the syslog on the iMac
 * -> Send message to syslog.s19.be
 * -> warn or info the message in the log of the program
 * -> panic if needed
 */
pub fn syslog(message: String, fail: bool, warn: bool) {
    Command::new("bash").arg("-c").arg(format!(
        "/bin/syslog.py {}",
        format!("[SPECULARE] - {}", message)
    ));
    if warn {
        warn!("{}", message);
    } else {
        info!("{}", message);
    }
    if fail {
        panic!(message);
    }
}
