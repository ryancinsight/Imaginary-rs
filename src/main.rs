//! Main entry point for Imaginary-rs.
//!
//! ## HTTP/1.1 and HTTP/2 Support
//! - `--http-version <http1|http2>`: Select HTTP version (default: http1)
//! - `--tls-mode <self-signed|signed>`: TLS mode (default: self-signed)
//! - `--cert-path <PATH>`: Path to TLS certificate (default: cert.pem)
//! - `--key-path <PATH>`: Path to TLS private key (default: key.pem)
//!
//! By default, runs HTTP/1.1 on port 8080. In HTTP/2 mode, serves HTTPS on 3000 and redirects HTTP/1.1 on 8080.
//!
//! Documentation is updated with every major change, following [best practices](https://www.linkedin.com/advice/0/what-best-practices-keeping-your-software-documentation-28sje).
use crate::config::cli;
use std::sync::Arc;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
mod config;
mod http;
mod image;
mod security;
mod server;
mod storage;
mod utils;
use crate::http::errors::AppError;
use crate::http::info::AppInfo;
use crate::security::{ApiKey, ApiSalt};
use axum_server::tls_rustls::RustlsConfig;
use std::net::SocketAddr;
use tokio::sync::Semaphore;

use axum_server::Server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Parse command line arguments
    let matches = cli::build_cli().get_matches();

    // Handle health check command
    if matches.get_flag("health-check") {
        return perform_health_check(&matches).await;
    }

    // Initialize health metrics
    crate::http::handlers::health_handler::init_health_metrics();

    // Load configuration
    let config = config::load_config(&matches)?;

    // Generate a new API key if not already set
    //let mut security_config = SecurityConfig::default();
    //if config.security.key.is_none() || config.security.key.as_ref().unwrap().is_empty() {
    //    let api_key = security_config.generate_api_key();
    //    config.security.set_api_key(api_key);
    //}

    let config = Arc::new(config);

    // Use the security configuration
    let allow_insecure =
        std::env::var("IMAGINARY_ALLOW_INSECURE").unwrap_or_else(|_| "1".to_string()) == "1";
    if let Err(e) = config.security.validate_secure() {
        if allow_insecure {
            eprintln!("\n*** WARNING: Running in INSECURE mode! ***\n{}\nThis is NOT safe for production.\nSet IMAGINARY_ALLOW_INSECURE=0 to require secure config.\n", e);
        } else {
            return Err(AppError::InternalServerError(format!(
                "Security configuration is not secure: {}. Refusing to start.",
                e
            ))
            .into());
        }
    } else {
        info!("Security configuration validated: secure defaults enforced.");
    }

    // Example usage of allowed_origins and methods
    let origin = format!("http://{}", config.server.host);
    if !config.security.is_origin_allowed(&origin) {
        return Err(AppError::Unauthorized(format!("Origin {} is not allowed.", origin)).into());
    }

    let data = &config.data;
    let signature = config.security.generate_signature(data).map_err(|e| {
        AppError::InternalServerError(format!("Failed to generate signature: {}", e))
    })?;
    info!("{}", AppInfo::GeneratedSignature(signature.clone()));

    let is_valid = config
        .security
        .validate_signature(data, &signature)
        .map_err(|e| {
            AppError::InternalServerError(format!("Failed to validate signature: {}", e))
        })?;
    info!("{}", AppInfo::ValidatedSignature(is_valid));

    // Print expected and received API keys
    if let Some(expected_api_key) = config.security.key() {
        info!("{}", AppInfo::ExpectedApiKey(expected_api_key.to_string()));
    }

    // If binding to localhost/127.0.0.1 and no key/salt is set, auto-generate them for local use only
    let is_localhost = config.server.host == "127.0.0.1" || config.server.host == "localhost";
    if is_localhost {
        let mut insecure = false;
        if config.security.key().is_none() {
            let key_string = security::generate_local_machine_secret();
            let api_key = ApiKey::from(key_string.clone());
            Arc::get_mut(&mut Arc::clone(&config))
                .unwrap()
                .security
                .set_key(api_key);
            eprintln!("\n*** WARNING: Auto-generated local security key for localhost. Not safe for remote use!\nKey: {}\n", key_string);
            insecure = true;
        }
        if config.security.salt().is_none() {
            let salt_string = security::generate_local_machine_secret();
            let api_salt = ApiSalt::from(salt_string.clone());
            Arc::get_mut(&mut Arc::clone(&config))
                .unwrap()
                .security
                .set_salt(api_salt);
            eprintln!("\n*** WARNING: Auto-generated local security salt for localhost. Not safe for remote use!\nSalt: {}\n", salt_string);
            insecure = true;
        }
        if insecure {
            eprintln!("\n*** WARNING: Local system-specific key/salt in use. Do not use this configuration for remote or production deployments!\n");
        }
    }

    let concurrency: u32 = *matches.get_one::<u32>("concurrency").unwrap_or(&0);
    let _semaphore = if concurrency > 0 {
        Some(Arc::new(Semaphore::new(concurrency as usize)))
    } else {
        None
    };

    // Pass the semaphore to the server
    // server::run_server(config, semaphore).await?;

    let http_version = matches
        .get_one::<String>("http-version")
        .map(|s| s.as_str())
        .unwrap_or("http1");
    let tls_mode = matches
        .get_one::<String>("tls-mode")
        .map(|s| s.as_str())
        .unwrap_or("self-signed");
    let cert_path = matches
        .get_one::<String>("cert-path")
        .map(|s| s.as_str())
        .unwrap_or("cert.pem");
    let key_path = matches
        .get_one::<String>("key-path")
        .map(|s| s.as_str())
        .unwrap_or("key.pem");

    let cert_exists = std::path::Path::new(cert_path).exists();
    let key_exists = std::path::Path::new(key_path).exists();

    if http_version == "http2" {
        // TLS cert logic
        if tls_mode == "signed" {
            if !cert_exists || !key_exists {
                eprintln!("TLS mode is 'signed' but certificate or key not found at specified paths.\nCert: {}\nKey: {}", cert_path, key_path);
                std::process::exit(1);
            }
        } else if !cert_exists || !key_exists {
            // Generate self-signed cert
            let subj = "/CN=localhost";
            let output = std::process::Command::new("openssl")
                .args([
                    "req", "-x509", "-newkey", "rsa:4096", "-keyout", key_path, "-out", cert_path,
                    "-days", "365", "-nodes", "-subj", subj,
                ])
                .output()
                .expect("Failed to run openssl to generate self-signed certificate");
            if !output.status.success() {
                eprintln!(
                    "Failed to generate self-signed certificate:\n{}",
                    String::from_utf8_lossy(&output.stderr)
                );
                std::process::exit(1);
            }
            println!(
                "Generated self-signed certificate at {} and {}",
                cert_path, key_path
            );
        }
        // Start HTTPS/2 on 3000
        let addr_https = SocketAddr::from(([0, 0, 0, 0], 3000));
        let app = server::create_router(config.clone());
        let config_tls = RustlsConfig::from_pem_file(cert_path, key_path)
            .await
            .unwrap();
        println!("listening on https://{} (HTTP/2 enabled)", addr_https);
        let https_handle = tokio::spawn(async move {
            axum_server::bind_rustls(addr_https, config_tls)
                .serve(app.into_make_service())
                .await
                .unwrap();
        });
        // Start HTTP/1.1 redirect on 8080
        let addr_http = SocketAddr::from(([0, 0, 0, 0], 8080));
        let redirect_router = axum::Router::new().fallback(axum::routing::any(
            move |req: axum::http::Request<axum::body::Body>| async move {
                let host = req
                    .headers()
                    .get("host")
                    .and_then(|h| h.to_str().ok())
                    .unwrap_or("localhost");
                let uri = req
                    .uri()
                    .path_and_query()
                    .map(|pq| pq.as_str())
                    .unwrap_or("/");
                let redirect_url = format!("https://{}:3000{}", host, uri);
                axum::response::Redirect::permanent(&redirect_url)
            },
        ));
        println!(
            "listening on http://{} (redirects to https://host:3000)",
            addr_http
        );
        let http_handle = tokio::spawn(async move {
            Server::bind(addr_http)
                .serve(redirect_router.into_make_service())
                .await
                .unwrap();
        });
        https_handle.await?;
        http_handle.await?;
    } else {
        // HTTP/1.1 only on 8080
        let addr_http = SocketAddr::from(([0, 0, 0, 0], 8080));
        let app = server::create_router(config.clone());
        println!("listening on http://{} (HTTP/1.1)", addr_http);
        Server::bind(addr_http)
            .serve(app.into_make_service())
            .await
            .unwrap();
    }
    Ok(())
}

/// Perform a health check by making an HTTP request to the health endpoint
async fn perform_health_check(
    matches: &clap::ArgMatches,
) -> Result<(), Box<dyn std::error::Error>> {
    use reqwest;

    // Get host and port from CLI arguments or defaults
    let host = matches
        .get_one::<String>("host")
        .map(|s| s.as_str())
        .unwrap_or("127.0.0.1");
    let port = matches
        .get_one::<String>("port")
        .map(|s| s.as_str())
        .unwrap_or("8080");

    let client = reqwest::Client::new();
    let url = format!("http://{}:{}/health", host, port);

    match client
        .get(&url)
        .timeout(std::time::Duration::from_secs(5))
        .send()
        .await
    {
        Ok(response) => {
            if response.status().is_success() {
                println!("Health check: OK");
                std::process::exit(0);
            } else {
                eprintln!("Health check failed: HTTP {}", response.status());
                std::process::exit(1);
            }
        }
        Err(e) => {
            eprintln!("Health check failed: {}", e);
            std::process::exit(1);
        }
    }
}
