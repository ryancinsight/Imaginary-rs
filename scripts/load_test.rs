#!/usr/bin/env cargo +nightly -Zscript

//! Load testing script for imaginary-rs
//! 
//! This script tests the application under various concurrent load scenarios
//! to establish performance characteristics and identify bottlenecks.

use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::time::sleep;
use reqwest::Client;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Starting Load Testing for imaginary-rs");
    println!("==========================================");

    // Test configuration
    let base_url = "http://localhost:8080";
    let test_image_url = "https://httpbin.org/image/jpeg"; // Small test image
    
    // Load test scenarios
    let scenarios = vec![
        LoadTestScenario {
            name: "Light Load".to_string(),
            concurrent_users: 10,
            requests_per_user: 5,
            delay_between_requests: Duration::from_millis(100),
        },
        LoadTestScenario {
            name: "Moderate Load".to_string(),
            concurrent_users: 50,
            requests_per_user: 10,
            delay_between_requests: Duration::from_millis(50),
        },
        LoadTestScenario {
            name: "Heavy Load".to_string(),
            concurrent_users: 100,
            requests_per_user: 5,
            delay_between_requests: Duration::from_millis(25),
        },
        LoadTestScenario {
            name: "Stress Test".to_string(),
            concurrent_users: 200,
            requests_per_user: 3,
            delay_between_requests: Duration::from_millis(10),
        },
    ];

    // Check if server is running
    let client = Client::new();
    match client.get(&format!("{}/health", base_url)).send().await {
        Ok(response) if response.status().is_success() => {
            println!("✅ Server is running and healthy");
        }
        _ => {
            println!("❌ Server is not running. Please start the server first:");
            println!("   cargo run --release");
            return Ok(());
        }
    }

    // Run load test scenarios
    for scenario in scenarios {
        println!("\n📊 Running scenario: {}", scenario.name);
        println!("   Concurrent users: {}", scenario.concurrent_users);
        println!("   Requests per user: {}", scenario.requests_per_user);
        
        let results = run_load_test_scenario(&client, base_url, &scenario).await?;
        print_results(&scenario, &results);
        
        // Cool down between scenarios
        println!("   Cooling down for 2 seconds...");
        sleep(Duration::from_secs(2)).await;
    }

    println!("\n🎉 Load testing completed!");
    Ok(())
}

#[derive(Clone)]
struct LoadTestScenario {
    name: String,
    concurrent_users: u32,
    requests_per_user: u32,
    delay_between_requests: Duration,
}

struct LoadTestResults {
    total_requests: u32,
    successful_requests: u32,
    failed_requests: u32,
    total_duration: Duration,
    average_response_time: Duration,
    min_response_time: Duration,
    max_response_time: Duration,
    requests_per_second: f64,
}

async fn run_load_test_scenario(
    client: &Client,
    base_url: &str,
    scenario: &LoadTestScenario,
) -> Result<LoadTestResults, Box<dyn std::error::Error>> {
    let start_time = Instant::now();
    let client = Arc::new(client.clone());
    
    // Create tasks for concurrent users
    let mut tasks = Vec::new();
    
    for user_id in 0..scenario.concurrent_users {
        let client = Arc::clone(&client);
        let base_url = base_url.to_string();
        let scenario = scenario.clone();
        
        let task = tokio::spawn(async move {
            simulate_user(client, &base_url, user_id, &scenario).await
        });
        
        tasks.push(task);
    }
    
    // Wait for all tasks to complete
    let mut all_results = Vec::new();
    for task in tasks {
        match task.await {
            Ok(user_results) => all_results.extend(user_results),
            Err(e) => eprintln!("Task failed: {}", e),
        }
    }
    
    let total_duration = start_time.elapsed();
    
    // Calculate statistics
    let total_requests = all_results.len() as u32;
    let successful_requests = all_results.iter().filter(|r| r.success).count() as u32;
    let failed_requests = total_requests - successful_requests;
    
    let response_times: Vec<Duration> = all_results.iter()
        .filter(|r| r.success)
        .map(|r| r.response_time)
        .collect();
    
    let average_response_time = if response_times.is_empty() {
        Duration::from_millis(0)
    } else {
        let total_ms: u64 = response_times.iter().map(|d| d.as_millis() as u64).sum();
        Duration::from_millis(total_ms / response_times.len() as u64)
    };
    
    let min_response_time = response_times.iter().min().copied().unwrap_or_default();
    let max_response_time = response_times.iter().max().copied().unwrap_or_default();
    
    let requests_per_second = if total_duration.as_secs_f64() > 0.0 {
        successful_requests as f64 / total_duration.as_secs_f64()
    } else {
        0.0
    };
    
    Ok(LoadTestResults {
        total_requests,
        successful_requests,
        failed_requests,
        total_duration,
        average_response_time,
        min_response_time,
        max_response_time,
        requests_per_second,
    })
}

#[derive(Debug)]
struct RequestResult {
    success: bool,
    response_time: Duration,
    status_code: Option<u16>,
}

async fn simulate_user(
    client: Arc<Client>,
    base_url: &str,
    user_id: u32,
    scenario: &LoadTestScenario,
) -> Vec<RequestResult> {
    let mut results = Vec::new();
    
    // Test operations to perform
    let operations = vec![
        json!([
            {"operation": "resize", "params": {"width": 200, "height": 200}},
            {"operation": "grayscale", "params": {}}
        ]),
        json!([
            {"operation": "resize", "params": {"width": 400, "height": 300}},
            {"operation": "blur", "params": {"sigma": 1.0}}
        ]),
        json!([
            {"operation": "crop", "params": {"x": 10, "y": 10, "width": 100, "height": 100}},
            {"operation": "rotate", "params": {"degrees": 90.0}}
        ]),
    ];
    
    for request_num in 0..scenario.requests_per_user {
        let start_time = Instant::now();
        
        // Select operation based on request number
        let operation = &operations[request_num as usize % operations.len()];
        
        // Make request to pipeline endpoint
        let result = match client
            .get(&format!("{}/pipeline", base_url))
            .query(&[
                ("url", "https://httpbin.org/image/jpeg"),
                ("operations", &operation.to_string()),
            ])
            .send()
            .await
        {
            Ok(response) => {
                let status_code = response.status().as_u16();
                let success = response.status().is_success();
                
                // Consume response body to complete the request
                let _ = response.bytes().await;
                
                RequestResult {
                    success,
                    response_time: start_time.elapsed(),
                    status_code: Some(status_code),
                }
            }
            Err(_) => RequestResult {
                success: false,
                response_time: start_time.elapsed(),
                status_code: None,
            },
        };
        
        results.push(result);
        
        // Delay between requests
        if request_num < scenario.requests_per_user - 1 {
            sleep(scenario.delay_between_requests).await;
        }
    }
    
    results
}

fn print_results(scenario: &LoadTestScenario, results: &LoadTestResults) {
    println!("   📈 Results:");
    println!("      Total requests: {}", results.total_requests);
    println!("      Successful: {} ({:.1}%)", 
             results.successful_requests, 
             (results.successful_requests as f64 / results.total_requests as f64) * 100.0);
    println!("      Failed: {} ({:.1}%)", 
             results.failed_requests,
             (results.failed_requests as f64 / results.total_requests as f64) * 100.0);
    println!("      Test duration: {:.2}s", results.total_duration.as_secs_f64());
    println!("      Requests/sec: {:.2}", results.requests_per_second);
    println!("      Avg response time: {}ms", results.average_response_time.as_millis());
    println!("      Min response time: {}ms", results.min_response_time.as_millis());
    println!("      Max response time: {}ms", results.max_response_time.as_millis());
}