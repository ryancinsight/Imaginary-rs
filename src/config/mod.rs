use serde::Deserialize;
use std::path::PathBuf;
use anyhow::{Result, Context};
use clap::ArgMatches;

pub mod cli;

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
    #[serde(default = "default_max_body_size")]
    pub max_body_size: usize,
}

fn default_max_body_size() -> usize {
    10 * 1024 * 1024  // 10MB default
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

pub fn load_config(matches: &ArgMatches) -> Result<Config> {
    let config_file = matches.get_one::<String>("config").map(String::as_str).unwrap_or("config/default");
    let mut builder = config::Config::builder()
        .add_source(config::File::with_name(config_file).required(false))
        .add_source(config::Environment::with_prefix("IMAGINARY"));

    if let Some(port) = matches.get_one::<String>("port") {
        builder = builder.set_override("server.port", port.as_str())?;
    }

    if let Some(host) = matches.get_one::<String>("host") {
        builder = builder.set_override("server.host", host.as_str())?;
    }

    if matches.contains_id("cors") {
        builder = builder.set_override("security.cors", "true")?;
    }

    builder.build()
        .context("Failed to build configuration")?
        .try_deserialize()
        .context("Failed to deserialize configuration")
}
