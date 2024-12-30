pub mod config;
pub mod http;
pub mod image;
pub mod server;
pub mod utils;

// Re-export public items from modules if needed
pub use config::load_config;
pub use http::handlers::{health_check, process_image};