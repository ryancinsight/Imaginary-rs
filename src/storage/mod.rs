use anyhow::Result;
use cached::proc_macro::cached;
use rkyv::{Archive, Deserialize as RkyvDeserialize, Serialize as RkyvSerialize};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use tokio::fs as tokio_fs;
use tokio::io::AsyncReadExt;
use tracing::info;

#[derive(Debug, Default, Clone, Deserialize, Archive, RkyvDeserialize, RkyvSerialize)]
#[archive(check_bytes)]
#[archive_attr(derive(Debug))]
pub struct StorageConfig {
    #[serde(default = "default_temp_dir")]
    pub temp_dir: String,
    #[serde(default = "default_max_cache_size")]
    #[allow(dead_code)]
    pub max_cache_size: usize,
    #[serde(default = "default_cache_cleanup_interval")]
    pub cache_cleanup_interval: u64, // seconds
    #[serde(default = "default_cache_max_age")]
    pub cache_max_age: u64, // seconds
}

#[allow(dead_code)] // For future cache management features
pub fn ensure_temp_dir(path: &PathBuf) -> Result<()> {
    fs::create_dir_all(path)?;
    Ok(())
}

#[allow(dead_code)] // For future cache management features
pub fn cleanup_temp_files(path: &PathBuf) -> Result<()> {
    if path.exists() {
        fs::remove_dir_all(path)?;
        fs::create_dir_all(path)?;
    }
    Ok(())
}

#[allow(dead_code)] // For future cache management features
pub fn init_storage_dirs(temp_dir: &Path) -> Result<()> {
    std::fs::create_dir_all(temp_dir)?;
    Ok(())
}

pub fn calculate_hash_sync(
    image_path: &Path,
    operation: &str,
    params: &str,
) -> Result<String> {
    let mut hasher = Sha256::new();
    let mut file = fs::File::open(image_path)?;
    let mut buffer = [0; 8192]; // 8KB buffer
    loop {
        let count = file.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }
    hasher.update(operation.as_bytes());
    hasher.update(params.as_bytes());
    Ok(format!("{:x}", hasher.finalize()))
}

// Cache for storing operation hashes (avoids re-reading file to compute hash)
#[cached(
    size = 100,
    key = "String",
    convert = r#"{ format!("{}:{}:{}:{}:{}", _path_str, _mtime, _size, _operation, _params) }"#
)]
pub fn get_hash_cached(
    _path_str: String,
    _mtime: u64,
    _size: u64,
    _operation: &str,
    _params: &str
) -> Option<String> {
    // We recreate path from string to open it.
    let path = PathBuf::from(&_path_str);
    calculate_hash_sync(&path, _operation, _params).ok()
}

pub fn get_cached_result(image_path: PathBuf, operation: &str, params: &str) -> Option<PathBuf> {
    // Get metadata to ensure cache validity
    let metadata = fs::metadata(&image_path).ok()?;
    // If we can't get mtime (e.g. some filesystems), we default to 0? Or just fail?
    // Using 0 might risk stale cache if mtime is broken. But usually it works.
    let mtime = metadata.modified()
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let size = metadata.len();

    let path_str = image_path.to_string_lossy().to_string();

    let hash = get_hash_cached(path_str, mtime, size, operation, params)?;

    let temp_dir = default_temp_dir_path();
    let cache_path = temp_dir.join(format!("{}.img", hash));

    if cache_path.exists() {
        Some(cache_path)
    } else {
        None
    }
}

// Cache for storing file metadata hashes
#[cached(
    size = 100,
    key = "String",
    convert = r#"{ format!("{}:{}:{}", _filename, _content_length, _content_type) }"#
)]
pub fn get_metadata_hash(
    _filename: String,
    _content_length: usize,
    _content_type: String,
) -> Option<String> {
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
pub async fn generate_operation_hash(
    image_path: &Path,
    operation: &str,
    params: &str,
) -> Result<String> {
    let mut hasher = Sha256::new();

    // Hash the image content
    let mut file = tokio_fs::File::open(image_path).await?;
    let mut buffer = [0; 8192];
    loop {
        let count = file.read(&mut buffer).await?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }

    // Hash the operation and parameters
    hasher.update(operation.as_bytes());
    hasher.update(params.as_bytes());

    Ok(format!("{:x}", hasher.finalize()))
}

pub async fn cache_result(image_path: &Path, operation: &str, params: &str, result_path: &Path) {
    if let Ok(hash) = generate_operation_hash(image_path, operation, params).await {
        let temp_dir = default_temp_dir_path();
        if !temp_dir.exists() {
            let _ = fs::create_dir_all(&temp_dir);
        }
        let cache_path = temp_dir.join(format!("{}.img", hash));

        // Copy result to cache
        // We use tokio::fs::copy for async context
        match tokio_fs::copy(result_path, &cache_path).await {
            Ok(_) => {
                info!("Cached result for operation: {} at {:?}", hash, cache_path);
            }
            Err(e) => {
                tracing::error!("Failed to cache result: {}", e);
            }
        }
    }
}

pub fn get_result(image_path: &Path, operation: &str, params: &str) -> Option<PathBuf> {
    get_cached_result(image_path.to_path_buf(), operation, params)
}

pub fn calculate_hash_from_memory(
    image_data: &[u8],
    operation: &str,
    params: &str,
) -> String {
    let mut hasher = Sha256::new();
    hasher.update(image_data);
    hasher.update(operation.as_bytes());
    hasher.update(params.as_bytes());
    format!("{:x}", hasher.finalize())
}

pub async fn get_cached_path(hash: &str) -> Option<PathBuf> {
    let temp_dir = default_temp_dir_path();
    let cache_path = temp_dir.join(format!("{}.img", hash));

    if tokio_fs::metadata(&cache_path).await.is_ok() {
        Some(cache_path)
    } else {
        None
    }
}

pub async fn save_buffer_to_cache(hash: &str, data: &[u8]) -> Result<()> {
    let temp_dir = default_temp_dir_path();
    if tokio_fs::metadata(&temp_dir).await.is_err() {
        tokio_fs::create_dir_all(&temp_dir).await?;
    }

    let cache_filename = format!("{}.img", hash);
    let cache_path = temp_dir.join(&cache_filename);
    let temp_path = temp_dir.join(format!("{}.tmp", cache_filename));

    // Write to temp file first
    tokio_fs::write(&temp_path, data).await?;

    // Atomically rename
    tokio_fs::rename(&temp_path, &cache_path).await?;
    Ok(())
}

// Cleanup old cache entries
#[allow(dead_code)] // For future cache management features
pub fn cleanup_old_cache(temp_dir: &PathBuf, max_age: std::time::Duration) -> Result<()> {
    let now = std::time::SystemTime::now();

    for entry in fs::read_dir(temp_dir)?.flatten() {
        if let Ok(metadata) = entry.metadata() {
            if let Ok(created) = metadata.created() {
                if now.duration_since(created).unwrap_or_default() > max_age {
                    let _ = fs::remove_file(entry.path());
                }
            }
        }
    }

    Ok(())
}

fn default_temp_dir() -> String {
    "temp".to_string()
}

pub(crate) fn default_temp_dir_path() -> PathBuf {
    PathBuf::from("temp")
}

fn default_max_cache_size() -> usize {
    1024 * 1024 * 1024 // 1GB
}

fn default_cache_cleanup_interval() -> u64 {
    3600 // 1 hour
}

fn default_cache_max_age() -> u64 {
    86400 // 24 hours
}

pub fn start_cache_cleanup_task(config: StorageConfig) {
    tokio::spawn(async move {
        let interval_duration = std::time::Duration::from_secs(config.cache_cleanup_interval);
        let max_age_duration = std::time::Duration::from_secs(config.cache_max_age);
        let temp_dir = PathBuf::from(config.temp_dir);

        // skip first tick
        let mut interval = tokio::time::interval(interval_duration);
        interval.tick().await;

        loop {
            interval.tick().await;
            info!("Starting cache cleanup...");
            let temp_dir_clone = temp_dir.clone();
            let result = tokio::task::spawn_blocking(move || {
                cleanup_old_cache(&temp_dir_clone, max_age_duration)
            })
            .await;

            match result {
                Ok(Ok(_)) => info!("Cache cleanup completed successfully."),
                Ok(Err(e)) => tracing::error!("Cache cleanup failed: {}", e),
                Err(e) => tracing::error!("Cache cleanup task panicked: {}", e),
            }
        }
    });
}
