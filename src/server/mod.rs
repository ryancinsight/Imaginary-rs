pub mod middleware;
pub mod options;
pub mod routes;

pub use routes::create_router;

use std::net::SocketAddr;
use crate::config::Config;
use std::sync::Arc;
use tracing::{info, error};
use axum::serve;
use tokio::net::TcpListener;

pub async fn run_server(config: Arc<Config>) -> Result<(), Box<dyn std::error::Error>> {
    let addr: SocketAddr = format!("{}:{}", config.server.host, config.server.port)
        .parse()
        .expect("Failed to parse address");

    let app = routes::create_router()
        .with_state(config);

    info!("Starting server on {}", addr);
    
    let listener = TcpListener::bind(addr).await?;
    serve(listener, app).await?;

    Ok(())
}