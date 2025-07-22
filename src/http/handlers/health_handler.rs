use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde_json::json;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};
use sysinfo::{Disks, System};
use tracing::info;

const VERSION: &str = env!("CARGO_PKG_VERSION");

// Global counters for metrics (in production, use proper metrics library like prometheus)
static REQUEST_COUNT: AtomicU64 = AtomicU64::new(0);
static ERROR_COUNT: AtomicU64 = AtomicU64::new(0);
static START_TIME: std::sync::OnceLock<SystemTime> = std::sync::OnceLock::new();

/// Initialize the start time for uptime calculation
pub fn init_health_metrics() {
    START_TIME.set(SystemTime::now()).unwrap_or(());
}

/// Increment request counter
pub fn increment_request_count() {
    REQUEST_COUNT.fetch_add(1, Ordering::Relaxed);
}

/// Increment error counter
pub fn increment_error_count() {
    ERROR_COUNT.fetch_add(1, Ordering::Relaxed);
}

/// Basic health check endpoint
pub async fn health_check() -> impl IntoResponse {
    info!("Health check endpoint called");
    Json(json!({
        "status": "healthy",
        "message": "Service is running",
        "version": VERSION,
        "timestamp": SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }))
}

/// Detailed readiness check endpoint
pub async fn readiness_check() -> impl IntoResponse {
    info!("Readiness check endpoint called");

    // Perform basic system checks
    let memory_check = check_memory_usage();
    let disk_check = check_disk_space();

    let is_ready = memory_check && disk_check;
    let status_code = if is_ready {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };

    (
        status_code,
        Json(json!({
            "status": if is_ready { "ready" } else { "not_ready" },
            "version": VERSION,
            "checks": {
                "memory": memory_check,
                "disk": disk_check
            },
            "timestamp": SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
        })),
    )
}

/// Metrics endpoint for monitoring
pub async fn metrics() -> impl IntoResponse {
    info!("Metrics endpoint called");

    let uptime_seconds = START_TIME
        .get()
        .and_then(|start| SystemTime::now().duration_since(*start).ok())
        .map(|duration| duration.as_secs())
        .unwrap_or(0);

    Json(json!({
        "version": VERSION,
        "uptime_seconds": uptime_seconds,
        "requests_total": REQUEST_COUNT.load(Ordering::Relaxed),
        "errors_total": ERROR_COUNT.load(Ordering::Relaxed),
        "memory_usage_bytes": get_memory_usage(),
        "timestamp": SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }))
}

/// Check memory usage - returns true if usage is reasonable (less than 90%)
fn check_memory_usage() -> bool {
    let mut system = System::new();
    system.refresh_memory();

    let total_memory = system.total_memory();
    let used_memory = system.used_memory();

    if total_memory == 0 {
        return true; // Can't determine, assume healthy
    }

    let usage_percentage = (used_memory as f64 / total_memory as f64) * 100.0;
    usage_percentage < 90.0
}

/// Check disk space - returns true if available space is more than 10%
fn check_disk_space() -> bool {
    let disks = Disks::new_with_refreshed_list();

    // Check the root filesystem
    for disk in &disks {
        if disk.mount_point().to_str().unwrap_or("") == "/" {
            let total_space = disk.total_space();
            let available_space = disk.available_space();

            if total_space == 0 {
                return true; // Can't determine, assume healthy
            }

            let usage_percentage =
                ((total_space - available_space) as f64 / total_space as f64) * 100.0;
            return usage_percentage < 90.0;
        }
    }

    // If we can't find root filesystem, assume healthy
    true
}

/// Get current memory usage in bytes
fn get_memory_usage() -> u64 {
    let mut system = System::new();
    system.refresh_memory();

    // Return used memory in bytes
    system.used_memory()
}
