use serde::Deserialize;
use std::path::PathBuf;
use anyhow::{Result, Context};
use clap::ArgMatches;
use crate::server::ServerConfig;
use crate::security::SecurityConfig;
use crate::storage::StorageConfig;  // Updated import path
pub mod cli;

#[derive(Debug, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub server: ServerConfig,
    #[serde(default)]
    pub security: SecurityConfig,
    #[serde(default)]
    pub storage: StorageConfig,
}

// StorageConfig moved to storage/options.rs

pub fn load_config(matches: &ArgMatches) -> Result<Config> {
    let config_file = matches.get_one::<String>("config")
        .map(String::as_str)
        .unwrap_or("config/default");

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
