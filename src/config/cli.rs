/// CLI argument definitions for Imaginary-rs.
///
/// ## HTTP/1.1 and HTTP/2 Support
/// - `--http-version <http1|http2>`: Select HTTP version (default: http1)
/// - `--tls-mode <self-signed|signed>`: TLS mode (default: self-signed)
/// - `--cert-path <PATH>`: Path to TLS certificate (default: cert.pem)
/// - `--key-path <PATH>`: Path to TLS private key (default: key.pem)
///
/// Documentation is updated with every major change, following [best practices](https://www.linkedin.com/advice/0/what-best-practices-keeping-your-software-documentation-28sje).
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
                .default_value("127.0.0.1"),
        )
        .arg(
            Arg::new("read-timeout")
                .long("read-timeout")
                .value_name("SECONDS")
                .help("Sets the server read timeout in seconds")
                .default_value("30"),
        )
        .arg(
            Arg::new("write-timeout")
                .long("write-timeout")
                .value_name("SECONDS")
                .help("Sets the server write timeout in seconds")
                .default_value("30"),
        )
        .arg(
            Arg::new("concurrency")
                .long("concurrency")
                .value_name("N")
                .help("Maximum number of concurrent HTTP requests to process (0 = unlimited, default: 0)")
                .value_parser(clap::value_parser!(u32))
                .default_value("0"),
        )
        .arg(
            Arg::new("max-body-size")
                .long("max-body-size")
                .value_name("BYTES")
                .help("Sets the maximum request body size in bytes")
                .default_value("10485760"),
        )
        .arg(
            Arg::new("key")
                .long("key")
                .value_name("KEY")
                .help("Sets the security key")
                .default_value(""),
        )
        .arg(
            Arg::new("salt")
                .long("salt")
                .value_name("SALT")
                .help("Sets the security salt")
                .default_value(""),
        )
        .arg(
            Arg::new("allowed-origins")
                .long("allowed-origins")
                .value_name("ORIGINS")
                .help("Sets the allowed CORS origins (comma-separated)")
                .default_value("*"),
        )
        .arg(
            Arg::new("temp-dir")
                .long("temp-dir")
                .value_name("DIR")
                .help("Sets the temporary directory path")
                .default_value("temp"),
        )
        .arg(
            Arg::new("max-cache-size")
                .long("max-cache-size")
                .value_name("BYTES")
                .help("Sets the maximum cache size in bytes")
                .default_value("1073741824"),
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
        .arg(
            Arg::new("http-version")
                .long("http-version")
                .value_name("VERSION")
                .help("HTTP version to use: http1 or http2")
                .default_value("http1"),
        )
        .arg(
            Arg::new("tls-mode")
                .long("tls-mode")
                .value_name("MODE")
                .help("TLS mode: self-signed or signed")
                .default_value("self-signed"),
        )
        .arg(
            Arg::new("cert-path")
                .long("cert-path")
                .value_name("PATH")
                .help("Path to TLS certificate file")
                .default_value("cert.pem"),
        )
        .arg(
            Arg::new("key-path")
                .long("key-path")
                .value_name("PATH")
                .help("Path to TLS private key file")
                .default_value("key.pem"),
        )
        .arg(
            Arg::new("health-check")
                .long("health-check")
                .help("Perform health check and exit")
                .action(clap::ArgAction::SetTrue),
        )
}
