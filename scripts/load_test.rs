//! Load testing script for imaginary-rs
//! 
//! This script tests the application under various concurrent load scenarios
//! to establish performance characteristics and identify bottlenecks.
//!
//! Usage: cargo run --bin load_test
//!
//! Requirements:
//! 1. The imaginary-rs service should be running on http://localhost:8080
//! 2. A test image should be available at ./test_assets/test_image.jpg

use std::sync::Arc;
use std::time::{Duration, Instant};
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::time::sleep;
use reqwest::{Client, multipart};
use serde_json::json;
use std::path::Path;

#[derive(Clone)]
struct LoadTestScenario {
    name: String,
    concurrent_users: u32,
    requests_per_user: u32,
    delay_between_requests: Duration,
}

#[derive(Clone)]
struct TestMetrics {
    total_requests: Arc<AtomicU64>,
    successful_requests: Arc<AtomicU64>,
    failed_requests: Arc<AtomicU64>,
    total_response_time: Arc<AtomicU64>,
    min_response_time: Arc<AtomicU64>,
    max_response_time: Arc<AtomicU64>,
}

impl TestMetrics {
    fn new() -> Self {
        Self {
            total_requests: Arc::new(AtomicU64::new(0)),
            successful_requests: Arc::new(AtomicU64::new(0)),
            failed_requests: Arc::new(AtomicU64::new(0)),
            total_response_time: Arc::new(AtomicU64::new(0)),
            min_response_time: Arc::new(AtomicU64::new(u64::MAX)),
            max_response_time: Arc::new(AtomicU64::new(0)),
        }
    }

    fn record_request(&self, response_time_ms: u64, success: bool) {
        self.total_requests.fetch_add(1, Ordering::Relaxed);
        
        if success {
            self.successful_requests.fetch_add(1, Ordering::Relaxed);
        } else {
            self.failed_requests.fetch_add(1, Ordering::Relaxed);
        }

        self.total_response_time.fetch_add(response_time_ms, Ordering::Relaxed);
        
        // Update min response time
        loop {
            let current_min = self.min_response_time.load(Ordering::Relaxed);
            if response_time_ms >= current_min {
                break;
            }
            if self.min_response_time.compare_exchange_weak(
                current_min,
                response_time_ms,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ).is_ok() {
                break;
            }
        }

        // Update max response time
        loop {
            let current_max = self.max_response_time.load(Ordering::Relaxed);
            if response_time_ms <= current_max {
                break;
            }
            if self.max_response_time.compare_exchange_weak(
                current_max,
                response_time_ms,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ).is_ok() {
                break;
            }
        }
    }

    fn get_stats(&self) -> (u64, u64, u64, f64, u64, u64) {
        let total = self.total_requests.load(Ordering::Relaxed);
        let successful = self.successful_requests.load(Ordering::Relaxed);
        let failed = self.failed_requests.load(Ordering::Relaxed);
        let total_time = self.total_response_time.load(Ordering::Relaxed);
        let min_time = self.min_response_time.load(Ordering::Relaxed);
        let max_time = self.max_response_time.load(Ordering::Relaxed);
        
        let avg_time = if total > 0 {
            total_time as f64 / total as f64
        } else {
            0.0
        };

        (total, successful, failed, avg_time, min_time, max_time)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Starting Load Testing for imaginary-rs");
    println!("==========================================");

    // Check if test image exists
    let test_image_path = "./test_assets/test_image.jpg";
    if !Path::new(test_image_path).exists() {
        println!("❌ Error: Test image not found at {}", test_image_path);
        println!("Please create a test image or run: mkdir -p test_assets && curl -o test_assets/test_image.jpg https://httpbin.org/image/jpeg");
        return Ok(());
    }

    // Test configuration
    let base_url = "http://localhost:8080";
    
    // Load test scenarios
    let scenarios = vec![
        LoadTestScenario {
            name: "Light Load".to_string(),
            concurrent_users: 10,
            requests_per_user: 5,
            delay_between_requests: Duration::from_millis(100),
        },
        LoadTestScenario {
            name: "Medium Load".to_string(),
            concurrent_users: 50,
            requests_per_user: 10,
            delay_between_requests: Duration::from_millis(50),
        },
        // Updated scenarios matching PRD targets
        LoadTestScenario {
            name: "Heavy Load".to_string(),
            concurrent_users: 100,
            requests_per_user: 20,
            delay_between_requests: Duration::from_millis(20),
        },
        LoadTestScenario {
            name: "Extreme Load".to_string(),
            concurrent_users: 500,
            requests_per_user: 25,
            delay_between_requests: Duration::from_millis(15),
        },
        LoadTestScenario {
            name: "Stress Test".to_string(),
            concurrent_users: 1000,
            requests_per_user: 30,
            delay_between_requests: Duration::from_millis(10),
        },
    ];

    // Test pipeline operations
    let test_operations = vec![
        // Basic resize operation
        vec![json!({
            "operation": "resize",
            "params": {
                "width": 400,
                "height": 300,
                "maintain_aspect_ratio": true
            },
            "ignore_failure": false
        })],
        // Crop and rotate
        vec![
            json!({
                "operation": "crop",
                "params": {
                    "x": 0,
                    "y": 0,
                    "width": 200,
                    "height": 200
                },
                "ignore_failure": false
            }),
            json!({
                "operation": "rotate",
                "params": {
                    "angle": 90.0
                },
                "ignore_failure": false
            })
        ],
        // Complex pipeline
        vec![
            json!({
                "operation": "resize",
                "params": {
                    "width": 800,
                    "height": 600,
                    "maintain_aspect_ratio": true
                },
                "ignore_failure": false
            }),
            json!({
                "operation": "grayscale",
                "params": {},
                "ignore_failure": false
            }),
            json!({
                "operation": "blur",
                "params": {
                    "sigma": 2.0
                },
                "ignore_failure": false
            })
        ],
    ];

    // Run load tests
    for scenario in scenarios {
        println!("\n📊 Running scenario: {}", scenario.name);
        println!("   Users: {}, Requests per user: {}", 
                 scenario.concurrent_users, scenario.requests_per_user);
        
        let metrics = run_load_test_scenario(
            &scenario,
            base_url,
            test_image_path,
            &test_operations,
        ).await?;
        
        print_test_results(&scenario, &metrics);
    }

    println!("\n✅ Load testing completed!");
    Ok(())
}

async fn run_load_test_scenario(
    scenario: &LoadTestScenario,
    base_url: &str,
    test_image_path: &str,
    test_operations: &[Vec<serde_json::Value>],
) -> Result<TestMetrics, Box<dyn std::error::Error>> {
    let metrics = TestMetrics::new();
    let client = Arc::new(Client::new());
    
    let start_time = Instant::now();
    
    // Spawn concurrent users
    let mut handles = Vec::new();
    
    for user_id in 0..scenario.concurrent_users {
        let scenario_clone = scenario.clone();
        let metrics_clone = metrics.clone();
        let client_clone = client.clone();
        let base_url = base_url.to_string();
        let test_image_path = test_image_path.to_string();
        let test_operations = test_operations.to_vec();
        
        let handle = tokio::spawn(async move {
            simulate_user(
                user_id,
                &scenario_clone,
                &metrics_clone,
                &client_clone,
                &base_url,
                &test_image_path,
                &test_operations,
            ).await
        });
        
        handles.push(handle);
    }
    
    // Wait for all users to complete
    for handle in handles {
        if let Err(e) = handle.await {
            eprintln!("User simulation error: {}", e);
        }
    }
    
    let total_duration = start_time.elapsed();
    println!("   Total test duration: {:.2}s", total_duration.as_secs_f64());
    
    Ok(metrics)
}

async fn simulate_user(
    user_id: u32,
    scenario: &LoadTestScenario,
    metrics: &TestMetrics,
    client: &Client,
    base_url: &str,
    test_image_path: &str,
    test_operations: &[Vec<serde_json::Value>],
) {
    for request_id in 0..scenario.requests_per_user {
        // Select random operation set
        let operations = &test_operations[request_id as usize % test_operations.len()];
        
        let start_time = Instant::now();
        let success = match make_pipeline_request(
            client,
            base_url,
            test_image_path,
            operations,
        ).await {
            Ok(_) => true,
            Err(e) => {
                eprintln!("Request failed for user {}, request {}: {}", 
                         user_id, request_id, e);
                false
            }
        };
        
        let response_time = start_time.elapsed().as_millis() as u64;
        metrics.record_request(response_time, success);
        
        // Add delay between requests
        if request_id < scenario.requests_per_user - 1 {
            sleep(scenario.delay_between_requests).await;
        }
    }
}

async fn make_pipeline_request(
    client: &Client,
    base_url: &str,
    test_image_path: &str,
    operations: &[serde_json::Value],
) -> Result<(), Box<dyn std::error::Error>> {
    // Read test image
    let image_data = tokio::fs::read(test_image_path).await?;
    
    // Create multipart form
    let form = multipart::Form::new()
        .part("image", multipart::Part::bytes(image_data)
            .file_name("test_image.jpg")
            .mime_str("image/jpeg")?)
        .text("operations", serde_json::to_string(operations)?);
    
    // Make request
    let response = client
        .post(&format!("{}/pipeline", base_url))
        .multipart(form)
        .send()
        .await?;
    
    if !response.status().is_success() {
        return Err(format!("HTTP error: {}", response.status()).into());
    }
    
    // Consume response body to complete the request
    let _body = response.bytes().await?;
    
    Ok(())
}

fn print_test_results(scenario: &LoadTestScenario, metrics: &TestMetrics) {
    let (total, successful, failed, avg_time, min_time, max_time) = metrics.get_stats();
    
    let success_rate = if total > 0 {
        (successful as f64 / total as f64) * 100.0
    } else {
        0.0
    };
    
    let total_expected = scenario.concurrent_users as u64 * scenario.requests_per_user as u64;
    let throughput = successful as f64 / (total_expected as f64 / scenario.concurrent_users as f64);
    
    println!("   Results:");
    println!("     Total Requests: {}/{}", total, total_expected);
    println!("     Successful: {} ({:.1}%)", successful, success_rate);
    println!("     Failed: {}", failed);
    println!("     Response Times:");
    println!("       Average: {:.1}ms", avg_time);
    println!("       Min: {}ms", if min_time == u64::MAX { 0 } else { min_time });
    println!("       Max: {}ms", max_time);
    println!("     Throughput: {:.1} req/s", throughput);
}