use std::path::{Path, PathBuf};
use serde::Deserialize;
use anyhow::Result;
use cached::proc_macro::cached;
use cached::SizedCache;
use sha2::{Sha256, Digest};
use std::fs;
use std::io::Read;
use tracing::info;

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

// Cache for storing operation results
#[cached(
    type = "SizedCache<String, Option<PathBuf>>",
    create = "{ SizedCache::with_size(100) }",
    convert = r#"{ format!("{}:{}:{}", _image_path.to_string_lossy(), _operation, _params) }"#
)]
pub fn get_cached_result(_image_path: PathBuf, _operation: &str, _params: &str) -> Option<PathBuf> {
    None // Initial cache miss
}

// Cache for storing file metadata hashes
#[cached(
    type = "SizedCache<String, Option<String>>",
    create = "{ SizedCache::with_size(100) }",
    convert = r#"{ format!("{}:{}:{}", _filename, _content_length, _content_type) }"#
)]
pub fn get_metadata_hash(_filename: String, _content_length: usize, _content_type: String) -> Option<String> {
    None
}

pub fn check_cached_metadata(
    filename: &str,
    content_length: usize,
    content_type: &str,
    _operation: &str,
    _params: &str,
) -> Option<PathBuf> {
    let _metadata_hash = get_metadata_hash(
        filename.to_string(),
        content_length,
        content_type.to_string(),
    )?;

    // Construct the expected output path
    let output_path = PathBuf::from("temp").join(format!("processed_{}", filename));
    if output_path.exists() {
        Some(output_path)
    } else {
        None
    }
}

// Generate operation hash
pub fn generate_operation_hash(image_path: &PathBuf, operation: &str, params: &str) -> Result<String> {
    let mut hasher = Sha256::new();
    
    // Hash the image content
    let mut file = fs::File::open(image_path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    hasher.update(&buffer);
    
    // Hash the operation and parameters
    hasher.update(operation.as_bytes());
    hasher.update(params.as_bytes());
    
    Ok(format!("{:x}", hasher.finalize()))
}

pub fn cache_result(image_path: &PathBuf, operation: &str, params: &str, _result_path: &PathBuf) {
    if let Ok(hash) = generate_operation_hash(image_path, operation, params) {
        let cached = get_cached_result(image_path.clone(), operation, params);
        if cached.is_none() {
            info!("Cached result for operation: {}", hash);
        }
    }
}

pub fn get_result(image_path: &PathBuf, operation: &str, params: &str) -> Option<PathBuf> {
    get_cached_result(image_path.clone(), operation, params)
}

// Cleanup old cache entries
pub fn cleanup_old_cache(temp_dir: &PathBuf, max_age: std::time::Duration) -> Result<()> {
    let now = std::time::SystemTime::now();
    
    for entry in fs::read_dir(temp_dir)? {
        if let Ok(entry) = entry {
            if let Ok(metadata) = entry.metadata() {
                if let Ok(created) = metadata.created() {
                    if now.duration_since(created).unwrap_or_default() > max_age {
                        let _ = fs::remove_file(entry.path());
                    }
                }
            }
        }
    }
    
    Ok(())
}
