use axum::Router;
use std::net::SocketAddr;
use tracing_subscriber;
use tokio::net::TcpListener;
use crate::config::cli::build_cli;
use crate::server::routes::create_router;

mod config;
mod http;
mod image;
mod server;
mod utils;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing for logging
    tracing_subscriber::fmt::init();

    // Parse command line arguments
    let matches = build_cli().get_matches();

    // Load configuration
    let config = config::load_config(&matches).expect("Failed to load configuration");

    // Create the router with routes and middleware
    let app = create_router();

    // Define the address to bind the server
    let addr = SocketAddr::from(([0, 0, 0, 0], config.server.port));

    tracing::info!("Server running on {}", addr);

    // Create a TcpListener and serve the application
    let listener = TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
