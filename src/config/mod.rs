use serde::Deserialize;
use anyhow::{Result, Context};
use clap::ArgMatches;
use crate::server::ServerConfig;
use crate::security::SecurityConfig;
use crate::storage::StorageConfig;
use std::fs;
use std::path::Path;
use toml::Value;
use crate::http::errors::AppError;
pub mod cli;

#[derive(Debug, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub server: ServerConfig,
    #[serde(default)]
    pub security: SecurityConfig,
    #[serde(default)]
    pub storage: StorageConfig,
    #[serde(default = "default_data")]
    pub data: Vec<u8>,
}

fn default_data() -> Vec<u8> {
    b"example data".to_vec()
}

pub fn load_config(matches: &ArgMatches) -> Result<Config, AppError> {
    let config_path = matches.get_one::<String>("config").map(|s| s.as_str()).unwrap_or("config/default.toml");
    let config_path = Path::new(config_path);

    if !config_path.exists() {
        create_default_config(config_path)?;
    }

    let config_content = fs::read_to_string(config_path).map_err(|_| AppError::FileSystemError("Failed to read config file".to_string()))?;
    let mut config: Value = toml::from_str(&config_content).map_err(|_| AppError::FileSystemError("Failed to parse config file".to_string()))?;

    override_with_cli_args(&mut config, matches);

    let config: Config = config.try_into().map_err(|_| AppError::FileSystemError("Failed to deserialize config".to_string()))?;
    Ok(config)
}

fn create_default_config(config_path: &Path) -> Result<(), AppError> {
    let default_config = r#"
[server]
port = 8080
host = "127.0.0.1"
read_timeout = 30
write_timeout = 30
concurrency = 4
max_body_size = 10485760

[security]
key = ""
salt = ""
allowed_origins = ["*"]

[storage]
temp_dir = "temp"
max_cache_size = 1073741824

[data]
value = "example data"
"#;

    fs::create_dir_all(config_path.parent().unwrap()).map_err(|_| AppError::FileSystemError("Failed to create config directory".to_string()))?;
    fs::write(config_path, default_config).map_err(|_| AppError::FileSystemError("Failed to write default config file".to_string()))?;
    Ok(())
}

fn override_with_cli_args(config: &mut Value, matches: &ArgMatches) {
    if let Some(port) = matches.get_one::<String>("port") {
        config["server"]["port"] = Value::Integer(port.parse::<i64>().unwrap());
    }
    if let Some(host) = matches.get_one::<String>("host") {
        config["server"]["host"] = Value::String(host.clone());
    }
    if let Some(read_timeout) = matches.get_one::<String>("read-timeout") {
        config["server"]["read_timeout"] = Value::Integer(read_timeout.parse::<i64>().unwrap());
    }
    if let Some(write_timeout) = matches.get_one::<String>("write-timeout") {
        config["server"]["write_timeout"] = Value::Integer(write_timeout.parse::<i64>().unwrap());
    }
    if let Some(concurrency) = matches.get_one::<String>("concurrency") {
        config["server"]["concurrency"] = Value::Integer(concurrency.parse::<i64>().unwrap());
    }
    if let Some(max_body_size) = matches.get_one::<String>("max-body-size") {
        config["server"]["max_body_size"] = Value::Integer(max_body_size.parse::<i64>().unwrap());
    }
    if let Some(key) = matches.get_one::<String>("key") {
        config["security"]["key"] = Value::String(key.clone());
    }
    if let Some(salt) = matches.get_one::<String>("salt") {
        config["security"]["salt"] = Value::String(salt.clone());
    }
    if let Some(allowed_origins) = matches.get_one::<String>("allowed-origins") {
        config["security"]["allowed_origins"] = Value::Array(
            allowed_origins.split(',').map(|s| Value::String(s.to_string())).collect()
        );
    }
    if let Some(temp_dir) = matches.get_one::<String>("temp-dir") {
        config["storage"]["temp_dir"] = Value::String(temp_dir.clone());
    }
    if let Some(max_cache_size) = matches.get_one::<String>("max-cache-size") {
        config["storage"]["max_cache_size"] = Value::Integer(max_cache_size.parse::<i64>().unwrap());
    }
}
