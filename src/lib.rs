mod config;
mod http;
mod image;
mod server;
mod storage;  // Ensure this is public
mod security; // Ensure this is public

// Re-export public items from modules if needed
pub use config::load_config;
pub use http::handlers::{health_check, process_image};
