use std::sync::Arc;
use crate::config::{Config, cli};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod config;
mod server;
mod http;
mod image;
mod utils;

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
    let config = Arc::new(config);

    // Start server (now passing Arc<Config> directly)
    server::run_server(config).await?;

    Ok(())
}
