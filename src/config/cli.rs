use clap::{Arg, Command};

pub fn build_cli() -> Command {
    Command::new("imaginary-rs")
        .version(env!("CARGO_PKG_VERSION"))
        .about("Fast, simple, scalable HTTP microservice for high-level image processing")
        .arg(
            Arg::new("port")
                .short('p')
                .long("port")
                .value_name("PORT")
                .help("Sets the server port")
                .default_value("8080"),
        )
        .arg(
            Arg::new("host")
                .short('H')
                .long("host")
                .value_name("HOST")
                .help("Sets the server host")
                .num_args(1)
                .default_value("127.0.0.1"),
        )
        .arg(
            Arg::new("config")
                .short('c')
                .long("config")
                .value_name("FILE")
                .help("Sets a custom config file")
                .num_args(1)
                .default_value("config/default"),
        )
        .arg(
            Arg::new("log-level")
                .long("log-level")
                .value_name("LEVEL")
                .help("Sets the log level")
                .num_args(1)
                .default_value("info"),
        )
        .arg(
            Arg::new("cors")
                .long("cors")
                .help("Enables CORS support")
                .num_args(0),
        )
}
