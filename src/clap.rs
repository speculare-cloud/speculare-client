use clap::{App, Arg, ArgMatches};

/// Init the clap menu/args handling and return the ArgMatches instance.
pub fn init_clap() -> ArgMatches {
    App::new("Speculare-client")
        .version("0.1.0")
        .author("Martin A. <ma@rtin.fyi>")
        .about("Collect metrics and send them to the server")
        .arg(
            Arg::new("config")
                .short('c')
                .long("config")
                .about("Enter the interactive config mode")
                .takes_value(false),
        )
        .arg(
            Arg::new("path")
                .short('p')
                .long("path")
                .about("Path to the config file")
                .takes_value(true),
        )
        .get_matches()
}
