use tracing_subscriber;
use tracing::Level;

pub fn init_logger() {
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();
}