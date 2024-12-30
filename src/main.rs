use axum::{
    routing::{get, post},
    Router,
    serve,
};
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber;
use tokio::net::TcpListener;

mod config;
mod http;
mod image;
mod server;
mod utils;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing for logging
    tracing_subscriber::fmt::init();

    // Load configuration
    let config = config::load_config().expect("Failed to load configuration");

    // Create the router with routes and middleware
    let app = Router::new()
        .route("/", get(http::handlers::health_check))
        .route("/process", post(http::handlers::process_image))
        .layer(CorsLayer::new().allow_origin(Any));

    // Define the address to bind the server
    let addr = SocketAddr::from(([0, 0, 0, 0], config.server.port));

    tracing::info!("Server running on {}", addr);

    // Create a TcpListener and serve the application
    let listener = TcpListener::bind(addr).await?;
    serve(listener, app).await?;

    Ok(())
}
