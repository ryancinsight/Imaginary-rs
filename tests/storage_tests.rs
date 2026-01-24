use imaginary::storage::{get_cached_result, cache_result};
use std::path::PathBuf;
use std::time::{Instant, SystemTime, UNIX_EPOCH};
use tokio::fs;
use tokio::io::AsyncWriteExt;

#[tokio::test]
async fn test_cache_performance() {
    // Setup
    let test_dir = PathBuf::from("test_assets_perf");

    // Clean up previous runs
    if test_dir.exists() {
        fs::remove_dir_all(&test_dir).await.unwrap();
    }

    // Create unique content to ensure unique hash
    let unique_id = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    let image_content = format!("fake image content {}", unique_id);

    if !test_dir.exists() {
        fs::create_dir_all(&test_dir).await.unwrap();
    }

    // Create dummy image file
    let image_path = test_dir.join("input.jpg");
    let mut file = fs::File::create(&image_path).await.unwrap();
    file.write_all(image_content.as_bytes()).await.unwrap();

    // Create dummy result file
    let result_path = test_dir.join("output.jpg");
    let mut file = fs::File::create(&result_path).await.unwrap();
    file.write_all(b"fake processed content").await.unwrap();

    let operation = "resize";
    let params = r#"{"width": 100}"#;

    // 1. Measure Cache Miss (Baseline)
    let start = Instant::now();
    let result = get_cached_result(image_path.clone(), operation, params);
    let duration_miss = start.elapsed();
    println!("Cache Miss Duration: {:?}", duration_miss);
    assert!(result.is_none(), "Expected cache miss initially");

    // 2. Populate Cache
    cache_result(&image_path, operation, params, &result_path).await;

    // 3. Measure Cache Hit
    let start = Instant::now();
    let result = get_cached_result(image_path.clone(), operation, params);
    let duration_hit = start.elapsed();
    println!("Cache Hit Duration: {:?}", duration_hit);

    assert!(result.is_some(), "Expected cache hit after population");

    if let Some(path) = result {
        println!("Cache Hit: Found {:?}", path);
        // Verify path exists
        assert!(path.exists());
        // Clean up the cache file
        let _ = fs::remove_file(path).await;
    }

    // Cleanup
    let _ = fs::remove_dir_all(test_dir).await;
}
