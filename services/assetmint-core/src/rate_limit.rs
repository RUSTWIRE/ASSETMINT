// DISCLAIMER: Technical demo code — legal wrappers required in production
// SPDX-License-Identifier: MIT
//
// Simple in-memory rate limiter for the AssetMint compliance API.
// Uses a sliding window counter per IP address.

use axum::{
    extract::ConnectInfo,
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Rate limiter state: tracks request counts per IP in a sliding window.
#[derive(Clone)]
pub struct RateLimiter {
    /// Map from IP string to (window_start, request_count)
    state: Arc<Mutex<HashMap<String, (Instant, u32)>>>,
    /// Maximum requests per window
    max_requests: u32,
    /// Window duration
    window: Duration,
}

impl RateLimiter {
    /// Create a new rate limiter.
    /// - `max_requests`: maximum requests allowed per window
    /// - `window_secs`: window duration in seconds
    pub fn new(max_requests: u32, window_secs: u64) -> Self {
        Self {
            state: Arc::new(Mutex::new(HashMap::new())),
            max_requests,
            window: Duration::from_secs(window_secs),
        }
    }

    /// Check if a request from this IP should be allowed.
    /// Returns true if allowed, false if rate limited.
    pub fn check(&self, ip: &str) -> bool {
        let mut state = self.state.lock().unwrap();
        let now = Instant::now();

        let entry = state.entry(ip.to_string()).or_insert((now, 0));

        // If window has expired, reset
        if now.duration_since(entry.0) > self.window {
            *entry = (now, 1);
            return true;
        }

        // Increment and check
        entry.1 += 1;
        entry.1 <= self.max_requests
    }
}

/// Axum middleware function for rate limiting.
pub async fn rate_limit_middleware(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    request: Request<axum::body::Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    // Extract rate limiter from request extensions
    let limiter = request
        .extensions()
        .get::<RateLimiter>()
        .cloned();

    if let Some(limiter) = limiter {
        let ip = addr.ip().to_string();
        if !limiter.check(&ip) {
            return Err(StatusCode::TOO_MANY_REQUESTS);
        }
    }

    Ok(next.run(request).await)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limiter_allows_under_limit() {
        let limiter = RateLimiter::new(5, 60);
        for _ in 0..5 {
            assert!(limiter.check("127.0.0.1"));
        }
    }

    #[test]
    fn test_rate_limiter_blocks_over_limit() {
        let limiter = RateLimiter::new(3, 60);
        assert!(limiter.check("127.0.0.1"));
        assert!(limiter.check("127.0.0.1"));
        assert!(limiter.check("127.0.0.1"));
        assert!(!limiter.check("127.0.0.1")); // 4th request blocked
    }

    #[test]
    fn test_rate_limiter_separate_ips() {
        let limiter = RateLimiter::new(2, 60);
        assert!(limiter.check("10.0.0.1"));
        assert!(limiter.check("10.0.0.1"));
        assert!(!limiter.check("10.0.0.1")); // blocked
        assert!(limiter.check("10.0.0.2")); // different IP, allowed
        assert!(limiter.check("10.0.0.2"));
        assert!(!limiter.check("10.0.0.2")); // blocked
    }

    #[test]
    fn test_rate_limiter_window_reset() {
        let limiter = RateLimiter::new(2, 0); // 0-second window = instant reset
        assert!(limiter.check("127.0.0.1"));
        assert!(limiter.check("127.0.0.1"));
        // Window expires immediately, so next check resets
        std::thread::sleep(std::time::Duration::from_millis(10));
        assert!(limiter.check("127.0.0.1")); // allowed after window reset
    }
}
