use std::path::{Path, PathBuf};
use serde::Deserialize;
use anyhow::Result;
use std::fs;

#[derive(Debug, Deserialize, Default)]
pub struct StorageConfig {
    #[serde(default = "default_temp_dir")]
    pub temp_dir: PathBuf,
    #[serde(default = "default_max_cache_size")]
    pub max_cache_size: usize,
}

fn default_temp_dir() -> PathBuf {
    PathBuf::from("temp")
}

fn default_max_cache_size() -> usize {
    1024 * 1024 * 1024 // 1GB
}

// Add storage utility functions
pub fn ensure_temp_dir(path: &PathBuf) -> Result<()> {
    fs::create_dir_all(path)?;
    Ok(())
}

pub fn cleanup_temp_files(path: &PathBuf) -> Result<()> {
    if path.exists() {
        fs::remove_dir_all(path)?;
        fs::create_dir_all(path)?;
    }
    Ok(())
}

pub fn init_storage_dirs(temp_dir: &Path) -> Result<()> {
    std::fs::create_dir_all(temp_dir)?;
    Ok(())
}
