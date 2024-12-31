use std::sync::Arc;
use crate::config::{Config, cli};
use tracing::{info, error};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use axum::response::IntoResponse;
mod storage;
mod security;   
mod config;
mod server;
mod http;
mod image;
mod utils;
use crate::security::SecurityConfig;
use crate::http::errors::AppError;
use crate::http::info::AppInfo;

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
    let mut config = config::load_config(&matches)?;

    // Generate a new API key if not already set
    //let mut security_config = SecurityConfig::default();
    //if config.security.key.is_none() || config.security.key.as_ref().unwrap().is_empty() {
    //    let api_key = security_config.generate_api_key();
    //    config.security.set_api_key(api_key);
    //}

    let config = Arc::new(config);

    // Use the security configuration
    if config.security.is_secure() {
        info!("{}", AppInfo::SecurityConfigSecure);
    } else {
        return Err(AppError::InternalServerError("Security configuration is not secure.".to_string()).into());
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
    if let Some(expected_api_key) = &config.security.key {
        info!("{}", AppInfo::ExpectedApiKey(expected_api_key.clone()));
    }

    // Start server (now passing Arc<Config> directly)
    server::run_server(config).await?;

    Ok(())
}
