use serde::Deserialize;
use anyhow::Result;
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

impl Default for Config {
    fn default() -> Self {
        Self {
            server: ServerConfig::default(),
            security: SecurityConfig::default(),
            storage: StorageConfig::default(),
            data: default_data(),
        }
    }
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

    override_with_cli_args(&mut config, matches)
        .map_err(|e| AppError::BadRequest(format!("Configuration error: {}", e)))?;

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

fn override_with_cli_args(config: &mut Value, matches: &ArgMatches) -> Result<(), String> {
    if let Some(port) = matches.get_one::<String>("port") {
        let port_val = port.parse::<i64>()
            .map_err(|_| format!("Invalid port value: {}", port))?;
        if port_val < 1 || port_val > 65535 {
            return Err(format!("Port must be between 1 and 65535, got: {}", port_val));
        }
        config["server"]["port"] = Value::Integer(port_val);
    }
    if let Some(host) = matches.get_one::<String>("host") {
        config["server"]["host"] = Value::String(host.clone());
    }
    if let Some(read_timeout) = matches.get_one::<String>("read-timeout") {
        let timeout_val = read_timeout.parse::<i64>()
            .map_err(|_| format!("Invalid read timeout value: {}", read_timeout))?;
        if timeout_val < 1 {
            return Err(format!("Read timeout must be positive, got: {}", timeout_val));
        }
        config["server"]["read_timeout"] = Value::Integer(timeout_val);
    }
    if let Some(write_timeout) = matches.get_one::<String>("write-timeout") {
        let timeout_val = write_timeout.parse::<i64>()
            .map_err(|_| format!("Invalid write timeout value: {}", write_timeout))?;
        if timeout_val < 1 {
            return Err(format!("Write timeout must be positive, got: {}", timeout_val));
        }
        config["server"]["write_timeout"] = Value::Integer(timeout_val);
    }
    if let Some(concurrency) = matches.get_one::<u32>("concurrency") {
        config["server"]["concurrency"] = Value::Integer(*concurrency as i64);
    }
    if let Some(max_body_size) = matches.get_one::<String>("max-body-size") {
        let size_val = max_body_size.parse::<i64>()
            .map_err(|_| format!("Invalid max body size value: {}", max_body_size))?;
        if size_val < 1024 {
            return Err(format!("Max body size must be at least 1024 bytes, got: {}", size_val));
        }
        config["server"]["max_body_size"] = Value::Integer(size_val);
    }
    if let Some(key) = matches.get_one::<String>("key") {
        if key.len() < 32 {
            return Err("Security key must be at least 32 characters long".to_string());
        }
        config["security"]["key"] = Value::String(key.clone());
    }
    if let Some(salt) = matches.get_one::<String>("salt") {
        if salt.len() < 32 {
            return Err("Security salt must be at least 32 characters long".to_string());
        }
        config["security"]["salt"] = Value::String(salt.clone());
    }
    if let Some(allowed_origins) = matches.get_one::<String>("allowed-origins") {
        config["security"]["allowed_origins"] = Value::Array(
            allowed_origins.split(',').map(|s| Value::String(s.trim().to_string())).collect()
        );
    }
    if let Some(temp_dir) = matches.get_one::<String>("temp-dir") {
        config["storage"]["temp_dir"] = Value::String(temp_dir.clone());
    }
    if let Some(max_cache_size) = matches.get_one::<String>("max-cache-size") {
        let cache_size_val = max_cache_size.parse::<i64>()
            .map_err(|_| format!("Invalid max cache size value: {}", max_cache_size))?;
        if cache_size_val < 1024 * 1024 {
            return Err(format!("Max cache size must be at least 1MB, got: {}", cache_size_val));
        }
        config["storage"]["max_cache_size"] = Value::Integer(cache_size_val);
    }
    Ok(())
}
