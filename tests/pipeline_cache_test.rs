use axum::http::StatusCode;
use imaginary::server::create_router;
use imaginary::config::Config;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::test]
async fn test_pipeline_caching() {
    // Setup config manually
    let mut config = Config::default();
    config.server.max_body_size = 10 * 1024 * 1024; // 10MB
    let config = Arc::new(config);

    // Clean temp dir
    let temp_dir = std::path::Path::new("temp");
    if temp_dir.exists() {
        let _ = std::fs::remove_dir_all(temp_dir);
    }

    // Start server
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let router = create_router(config);

    tokio::spawn(async move {
        axum::serve(listener, router).await.unwrap();
    });

    // Create a simple test image (1x1 pixel)
    let image_bytes = vec![
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A,  // PNG Header
        0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52,  // IHDR
        0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01,  // 1x1
        0x08, 0x06, 0x00, 0x00, 0x00, 0x1F, 0x15, 0xC4,  // 8-bit RGBA
        0x89, 0x00, 0x00, 0x00, 0x0A, 0x49, 0x44, 0x41,  // IDAT
        0x54, 0x78, 0x9C, 0x63, 0x00, 0x01, 0x00, 0x00,  // data
        0x05, 0x00, 0x01, 0x0D, 0x0A, 0x2D, 0xB4, 0x00,  // ...
        0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4E, 0x44,  // IEND
        0xAE, 0x42, 0x60, 0x82,
    ];

    let operations = r#"[{"operation": "resize", "params": {"width": 1, "height": 1}}]"#;

    let client = reqwest::Client::new();
    let url = format!("http://{}/pipeline", addr);

    // 1. First Request (Cache Miss)
    let form = reqwest::multipart::Form::new()
        .part("image", reqwest::multipart::Part::bytes(image_bytes.clone()).file_name("test.png").mime_str("image/png").unwrap())
        .text("operations", operations);

    let response = client.post(&url)
        .multipart(form)
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let cache_header = response.headers().get("X-Cache");
    assert!(cache_header.is_some(), "X-Cache header missing");
    assert_eq!(cache_header.unwrap(), "MISS");

    // Wait for async cache write
    sleep(Duration::from_millis(500)).await;

    // Verify file exists in temp
    let entries = std::fs::read_dir("temp").unwrap();
    let count = entries.count();
    assert!(count > 0, "Cache file should exist in temp dir");

    // 2. Second Request (Cache Hit)
    let form = reqwest::multipart::Form::new()
        .part("image", reqwest::multipart::Part::bytes(image_bytes.clone()).file_name("test.png").mime_str("image/png").unwrap())
        .text("operations", operations);

    let start = std::time::Instant::now();
    let response = client.post(&url)
        .multipart(form)
        .send()
        .await
        .unwrap();
    let duration = start.elapsed();

    assert_eq!(response.status(), StatusCode::OK);

    let cache_header = response.headers().get("X-Cache");
    assert!(cache_header.is_some(), "X-Cache header missing on second request");
    assert_eq!(cache_header.unwrap(), "HIT");

    println!("Second request took: {:?}", duration);
}
