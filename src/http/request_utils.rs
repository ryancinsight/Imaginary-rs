use axum::body::Bytes;
use crate::config::Config;
use crate::http::errors::AppError;
use once_cell::sync::Lazy;
use std::net::{IpAddr, Ipv4Addr};
use url::Url;

pub const MAX_IMAGE_SIZE: usize = 10 * 1024 * 1024; // 10 MB

// Reusable HTTP client for performance
pub static HTTP_CLIENT: Lazy<reqwest::Client> = Lazy::new(|| {
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .user_agent("imaginary-rs/0.1.0")
        .build()
        .expect("Failed to create HTTP client")
});

/// Checks if an IP address is safe for external requests (not private/internal)
pub fn is_safe_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(ipv4) => {
            // Use De Morgan's law to simplify boolean expression
            !(ipv4.is_private()
                || ipv4.is_loopback()
                || ipv4.is_link_local()
                || ipv4.is_broadcast()
                || ipv4.is_multicast()
                || (ipv4.octets()[0] == 100 && (64..128).contains(&ipv4.octets()[1]))
                || ipv4 == Ipv4Addr::new(169, 254, 169, 254)
                || (ipv4.octets()[0] == 192 && ipv4.octets()[1] == 0 && ipv4.octets()[2] == 2)
                || (ipv4.octets()[0] == 198 && ipv4.octets()[1] == 51 && ipv4.octets()[2] == 100)
                || (ipv4.octets()[0] == 203 && ipv4.octets()[1] == 0 && ipv4.octets()[2] == 113)
                || (ipv4.octets()[0] == 192 && ipv4.octets()[1] == 88 && ipv4.octets()[2] == 99))
        }
        IpAddr::V6(ipv6) => {
            // Use De Morgan's law and simplified expressions
            !(ipv6.is_loopback()
                || ipv6.is_multicast()
                || ipv6.segments()[0] & 0xffc0 == 0xfe80
                || ipv6.segments()[0] & 0xfe00 == 0xfc00
                || (ipv6.segments()[0] == 0x2001 && ipv6.segments()[1] == 0x0db8))
        }
    }
}

pub async fn fetch_image_from_url(url_str: &str, config: &Config) -> Result<Bytes, AppError> {
    // Parse and validate URL
    let url =
        Url::parse(url_str).map_err(|e| AppError::BadRequest(format!("Invalid URL: {}", e)))?;

    // Validate URL scheme
    match url.scheme() {
        "http" | "https" => {}
        _ => {
            return Err(AppError::BadRequest(
                "Only HTTP and HTTPS URLs are supported".to_string(),
            ))
        }
    }

    // Validate hostname exists
    let hostname = url
        .host_str()
        .ok_or_else(|| AppError::BadRequest("URL must contain a valid hostname".to_string()))?;

    // Resolve hostname to IP addresses
    let addrs = tokio::net::lookup_host((
        hostname,
        url.port()
            .unwrap_or(if url.scheme() == "https" { 443 } else { 80 }),
    ))
    .await
    .map_err(|e| {
        AppError::BadRequest(format!("Failed to resolve hostname '{}': {}", hostname, e))
    })?;

    // Check if any resolved IP is safe
    let safe_ips: Vec<_> = addrs
        .filter_map(|addr| {
            let ip = addr.ip();
            if is_safe_ip(ip) {
                Some(ip)
            } else {
                None
            }
        })
        .collect();

    if safe_ips.is_empty() {
        return Err(AppError::BadRequest(format!(
            "URL '{}' resolves to private/internal IP addresses and is not allowed for security reasons",
            hostname
        )));
    }

    let max_size = config.server.max_body_size.min(MAX_IMAGE_SIZE);
    fetch_bytes_with_limit(url_str, max_size).await
}

pub(crate) async fn fetch_bytes_with_limit(
    url_str: &str,
    max_size: usize,
) -> Result<Bytes, AppError> {
    // Make the HTTP request using the reusable client
    let mut response = HTTP_CLIENT
        .get(url_str)
        .send()
        .await
        .map_err(|e| AppError::BadRequest(format!("Failed to fetch image from URL: {}", e)))?;

    if !response.status().is_success() {
        return Err(AppError::BadRequest(format!(
            "HTTP error when fetching image: {}",
            response.status()
        )));
    }

    // Check content length
    let content_length = response.content_length().unwrap_or(0);
    if content_length > max_size as u64 {
        return Err(AppError::PayloadTooLarge(format!(
            "Image size {} exceeds limit of {} bytes",
            content_length, max_size
        )));
    }

    // Read response body with size limit
    let mut bytes = Vec::with_capacity(content_length as usize);
    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|e| AppError::BadRequest(format!("Failed to read image data: {}", e)))?
    {
        if bytes.len() + chunk.len() > max_size {
            return Err(AppError::PayloadTooLarge(format!(
                "Image size exceeds limit of {} bytes",
                max_size
            )));
        }
        bytes.extend_from_slice(&chunk);
    }

    Ok(Bytes::from(bytes))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_safe_ip_private_ranges() {
        use std::net::{IpAddr, Ipv4Addr};

        // Private IPv4 ranges should be rejected
        assert!(!is_safe_ip(IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1))));
        assert!(!is_safe_ip(IpAddr::V4(Ipv4Addr::new(172, 16, 0, 1))));
        assert!(!is_safe_ip(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1))));

        // Loopback should be rejected
        assert!(!is_safe_ip(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1))));

        // Link-local should be rejected
        assert!(!is_safe_ip(IpAddr::V4(Ipv4Addr::new(169, 254, 1, 1))));

        // Cloud metadata service should be rejected
        assert!(!is_safe_ip(IpAddr::V4(Ipv4Addr::new(169, 254, 169, 254))));

        // Carrier-grade NAT should be rejected
        assert!(!is_safe_ip(IpAddr::V4(Ipv4Addr::new(100, 64, 0, 1))));

        // Test networks should be rejected
        assert!(!is_safe_ip(IpAddr::V4(Ipv4Addr::new(192, 0, 2, 1))));
        assert!(!is_safe_ip(IpAddr::V4(Ipv4Addr::new(198, 51, 100, 1))));
        assert!(!is_safe_ip(IpAddr::V4(Ipv4Addr::new(203, 0, 113, 1))));
    }

    #[test]
    fn test_is_safe_ip_public_ranges() {
        use std::net::{IpAddr, Ipv4Addr};

        // Public IPv4 addresses should be allowed
        assert!(is_safe_ip(IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8)))); // Google DNS
        assert!(is_safe_ip(IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1)))); // Cloudflare DNS
        assert!(is_safe_ip(IpAddr::V4(Ipv4Addr::new(208, 67, 222, 222)))); // OpenDNS
    }

    #[test]
    fn test_is_safe_ip_ipv6() {
        use std::net::{IpAddr, Ipv6Addr};

        // IPv6 loopback should be rejected
        assert!(!is_safe_ip(IpAddr::V6(Ipv6Addr::new(
            0, 0, 0, 0, 0, 0, 0, 1
        ))));

        // IPv6 link-local should be rejected
        assert!(!is_safe_ip(IpAddr::V6(Ipv6Addr::new(
            0xfe80, 0, 0, 0, 0, 0, 0, 1
        ))));

        // IPv6 unique local should be rejected
        assert!(!is_safe_ip(IpAddr::V6(Ipv6Addr::new(
            0xfc00, 0, 0, 0, 0, 0, 0, 1
        ))));

        // IPv6 documentation should be rejected
        assert!(!is_safe_ip(IpAddr::V6(Ipv6Addr::new(
            0x2001, 0x0db8, 0, 0, 0, 0, 0, 1
        ))));

        // Public IPv6 should be allowed (Google DNS)
        assert!(is_safe_ip(IpAddr::V6(Ipv6Addr::new(
            0x2001, 0x4860, 0x4860, 0, 0, 0, 0, 0x8888
        ))));
    }

    #[tokio::test]
    async fn test_unbounded_read() {
        use axum::{routing::get, Router};
        use axum::body::Body;
        use futures_util::StreamExt;
        use std::sync::atomic::{AtomicUsize, Ordering};
        use std::sync::Arc;
        use tokio::net::TcpListener;
        use tokio::task;

        // A counter to track bytes sent by the server body stream
        let sent_bytes = Arc::new(AtomicUsize::new(0));
        let sent_bytes_clone = sent_bytes.clone();

        // Create a router that serves a large body (100MB)
        let app = Router::new().route("/large", get(move || {
            let sent_bytes = sent_bytes_clone.clone();
            async move {
                // We want to stream data so we can detect early termination
                // Create a stream of 100MB in 1KB chunks
                let total_size = 100 * 1024 * 1024;
                let chunk_size = 1024;
                let chunks = total_size / chunk_size;

                let stream = futures_util::stream::iter(0..chunks).map(move |_| {
                    sent_bytes.fetch_add(chunk_size, Ordering::SeqCst);
                    Ok::<_, std::io::Error>(vec![0u8; chunk_size])
                });

                Body::from_stream(stream)
            }
        }));

        // Start server
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        task::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });

        // Limit set to 5MB
        let max_size = 5 * 1024 * 1024;
        let url = format!("http://{}/large", addr);

        // Call the function
        let result = fetch_bytes_with_limit(&url, max_size).await;

        let bytes_sent = sent_bytes.load(Ordering::SeqCst);

        match result {
            Ok(_) => panic!("Should have failed with PayloadTooLarge"),
            Err(AppError::PayloadTooLarge(_)) => {
                // expected
            }
            Err(e) => panic!("Wrong error type: {}", e),
        }

        // Verify we didn't download everything
        // 100MB is the total size. We expect to stop much earlier.
        // Due to buffering, we might have sent more than the limit (5MB), but should be far less than 100MB.
        assert!(
            bytes_sent < 50 * 1024 * 1024,
            "Server sent too much data: {} bytes. Limit was 5MB.",
            bytes_sent
        );
    }
}
