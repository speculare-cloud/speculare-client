/// Init the logger (env_logger) and define the debug level
/// based on debug or release build.
pub fn configure() {
    // Define log as info for debug and error for prod
    let dbg_level = if cfg!(debug_assertions) {
        "info"
    } else {
        "error"
    };
    std::env::set_var("RUST_LOG", dbg_level);
    // Init the logger
    env_logger::init();
}
