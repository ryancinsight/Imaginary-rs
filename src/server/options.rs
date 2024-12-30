use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ServerOptions {
    pub port: u16,
    pub address: String,
    pub max_concurrent_requests: usize,
    pub request_timeout_seconds: u64,
}