use axum::http::StatusCode;
use imaginary::config::Config;
use imaginary::server::create_router;
use std::sync::Arc;

#[tokio::test]
async fn test_post_cache_control_headers() {
    // Setup config manually
    let mut config = Config::default();
    config.server.max_body_size = 10 * 1024 * 1024; // 10MB
    let config = Arc::new(config);

    // Start server
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let router = create_router(config);

    tokio::spawn(async move {
        axum::serve(listener, router).await.unwrap();
    });

    // Create a simple test image (1x1 pixel)
    let image_bytes = vec![
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, // PNG Header
        0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52, // IHDR
        0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, // 1x1
        0x08, 0x06, 0x00, 0x00, 0x00, 0x1F, 0x15, 0xC4, // 8-bit RGBA
        0x89, 0x00, 0x00, 0x00, 0x0A, 0x49, 0x44, 0x41, // IDAT
        0x54, 0x78, 0x9C, 0x63, 0x00, 0x01, 0x00, 0x00, // data
        0x05, 0x00, 0x01, 0x0D, 0x0A, 0x2D, 0xB4, 0x00, // ...
        0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4E, 0x44, // IEND
        0xAE, 0x42, 0x60, 0x82,
    ];

    let form = reqwest::multipart::Form::new()
        .part(
            "image",
            reqwest::multipart::Part::bytes(image_bytes)
                .file_name("test.png")
                .mime_str("image/png")
                .unwrap(),
        )
        .text(
            "operations",
            r#"[{"operation": "resize", "params": {"width": 1, "height": 1}}]"#,
        );

    let client = reqwest::Client::new();
    let response = client
        .post(format!("http://{}/pipeline", addr))
        .multipart(form)
        .send()
        .await
        .unwrap();

    let status = response.status();
    if status != StatusCode::OK {
        let text = response.text().await.unwrap();
        panic!("Request failed with status: {} - {}", status, text);
    }

    // Check headers for POST
    let cache_control = response.headers().get("Cache-Control");
    assert!(
        cache_control.is_some(),
        "Cache-Control header missing on POST response"
    );
    assert_eq!(
        cache_control.unwrap(),
        "no-store",
        "Unexpected Cache-Control header for POST"
    );
}

#[tokio::test]
async fn test_get_cache_control_headers() {
    // Setup config manually
    let mut config = Config::default();
    config.server.max_body_size = 10 * 1024 * 1024; // 10MB
    let config = Arc::new(config);

    // Start server
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let router = create_router(config);

    tokio::spawn(async move {
        axum::serve(listener, router).await.unwrap();
    });

    // Use a public URL that is likely to work
    let image_url = "https://www.rust-lang.org/logos/rust-logo-64x64.png";
    let operations = r#"[{"operation": "resize", "params": {"width": 10, "height": 10}}]"#;

    let client = reqwest::Client::new();
    let response = client
        .get(format!("http://{}/pipeline", addr))
        .query(&[("url", image_url), ("operations", operations)])
        .send()
        .await
        .unwrap();

    let status = response.status();
    if status != StatusCode::OK {
        let text = response.text().await.unwrap();
        println!("Request failed with status: {} - {}", status, text);
        if text.contains("resolves to private") || text.contains("Failed to fetch") {
            println!("Skipping GET test due to network restrictions or resolution issues.");
            return;
        }
        panic!("Request failed");
    }

    // Check headers for GET
    let cache_control = response.headers().get("Cache-Control");
    assert!(
        cache_control.is_some(),
        "Cache-Control header missing on GET response"
    );
    assert_eq!(
        cache_control.unwrap(),
        "public, max-age=31536000, immutable",
        "Unexpected Cache-Control header for GET"
    );
}

#[tokio::test]
async fn test_head_cache_control_headers() {
    // Setup config manually
    let mut config = Config::default();
    config.server.max_body_size = 10 * 1024 * 1024; // 10MB
    let config = Arc::new(config);

    // Start server
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let router = create_router(config);

    tokio::spawn(async move {
        axum::serve(listener, router).await.unwrap();
    });

    // Use a public URL that is likely to work
    let image_url = "https://www.rust-lang.org/logos/rust-logo-64x64.png";
    let operations = r#"[{"operation": "resize", "params": {"width": 10, "height": 10}}]"#;

    let client = reqwest::Client::new();
    let response = client
        .head(format!("http://{}/pipeline", addr))
        .query(&[("url", image_url), ("operations", operations)])
        .send()
        .await
        .unwrap();

    let status = response.status();
    // HEAD request might return the same status as GET if implemented correctly, but usually axum handles HEAD automatically for GET routes.
    // However, our handler specifically checks method.
    // Wait, does Axum route HEAD to the GET handler?
    // In `src/server/mod.rs`: `.route("/pipeline", post(process_pipeline).get(process_pipeline))`
    // It does NOT explicitly route HEAD.
    // Axum by default handles HEAD requests for GET routes by calling the GET handler and stripping the body?
    // If so, `method` will be `GET`? Or `HEAD`?
    // "By default, axum will create a HEAD handler for each GET handler that simply calls the GET handler and strips the body."
    // But does it pass `Method::HEAD` to the handler?
    // Usually yes.

    if status != StatusCode::OK {
        // Since it's HEAD, we might not get a body to print error text easily if we just read it.
        // But headers should be there.
        println!("Request failed with status: {}", status);
        // If 405 Method Not Allowed, then HEAD is not routed.
        if status == StatusCode::METHOD_NOT_ALLOWED {
            panic!("HEAD method not allowed, router configuration might be missing HEAD routing or Axum default behavior issue.");
        }

        // We can't read body from HEAD response usually.
        panic!("Request failed");
    }

    // Check headers for HEAD
    let cache_control = response.headers().get("Cache-Control");
    assert!(
        cache_control.is_some(),
        "Cache-Control header missing on HEAD response"
    );
    assert_eq!(
        cache_control.unwrap(),
        "public, max-age=31536000, immutable",
        "Unexpected Cache-Control header for HEAD"
    );
}
