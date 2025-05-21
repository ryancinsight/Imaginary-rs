use std::sync::Arc;
use crate::config::cli;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
mod storage;
mod security;   
mod config;
mod server;
mod http;
mod image;
mod utils;
use crate::http::errors::AppError;
use crate::http::info::AppInfo;
use crate::security::{ApiKey, ApiSalt};

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
    let allow_insecure = std::env::var("IMAGINARY_ALLOW_INSECURE").unwrap_or_else(|_| "1".to_string()) == "1";
    if let Err(e) = config.security.validate_secure() {
        if allow_insecure {
            eprintln!("\n*** WARNING: Running in INSECURE mode! ***\n{}\nThis is NOT safe for production.\nSet IMAGINARY_ALLOW_INSECURE=0 to require secure config.\n", e);
        } else {
            return Err(AppError::InternalServerError(format!("Security configuration is not secure: {}. Refusing to start.", e)).into());
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
    let signature = config.security.generate_signature(data)
        .map_err(|e| AppError::InternalServerError(format!("Failed to generate signature: {}", e)))?;
    info!("{}", AppInfo::GeneratedSignature(signature.clone()));

    let is_valid = config.security.validate_signature(data, &signature)
        .map_err(|e| AppError::InternalServerError(format!("Failed to validate signature: {}", e)))?;
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
            Arc::get_mut(&mut Arc::clone(&config)).unwrap().security.set_key(api_key);
            eprintln!("\n*** WARNING: Auto-generated local security key for localhost. Not safe for remote use!\nKey: {}\n", key_string);
            insecure = true;
        }
        if config.security.salt().is_none() {
            let salt_string = security::generate_local_machine_secret();
            let api_salt = ApiSalt::from(salt_string.clone());
            Arc::get_mut(&mut Arc::clone(&config)).unwrap().security.set_salt(api_salt);
            eprintln!("\n*** WARNING: Auto-generated local security salt for localhost. Not safe for remote use!\nSalt: {}\n", salt_string);
            insecure = true;
        }
        if insecure {
            eprintln!("\n*** WARNING: Local system-specific key/salt in use. Do not use this configuration for remote or production deployments!\n");
        }
    }

    // Start server (now passing Arc<Config> directly)
    server::run_server(config).await?;

    Ok(())
}
