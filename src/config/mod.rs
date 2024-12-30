use serde::Deserialize;
use std::path::PathBuf;
use anyhow::Result;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub security: SecurityConfig,
    pub storage: StorageConfig,
}

#[derive(Debug, Deserialize)]
pub struct ServerConfig {
    pub port: u16,
    pub host: String,
    pub read_timeout: u64,
    pub write_timeout: u64,
    pub concurrency: usize,
}

#[derive(Debug, Deserialize)]
pub struct SecurityConfig {
    pub key: Option<String>,
    pub salt: Option<String>,
    pub allowed_origins: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct StorageConfig {
    pub temp_dir: PathBuf,
    pub max_cache_size: usize,
}

pub fn load_config() -> Result<Config> {
    let builder = config::Config::builder()
        .add_source(config::File::with_name("config/default"))
        .add_source(config::Environment::with_prefix("IMAGINARY"));

    Ok(builder.build()?.try_deserialize()?)
}
