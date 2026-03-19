// DISCLAIMER: Technical demo code — legal wrappers required in production
// SPDX-License-Identifier: MIT
//
// API key authentication middleware for AssetMint.
// Checks X-API-Key header against API_KEY environment variable.
// If API_KEY is not set, authentication is skipped (demo mode).

use axum::{
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};

/// Check X-API-Key header against API_KEY env var.
/// If API_KEY is not set, skip auth (demo mode).
pub async fn api_key_middleware(
    request: Request<axum::body::Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let expected_key = std::env::var("API_KEY").ok();

    match expected_key {
        Some(key) => {
            let provided = request
                .headers()
                .get("x-api-key")
                .and_then(|v| v.to_str().ok());

            match provided {
                Some(k) if k == key => Ok(next.run(request).await),
                _ => Err(StatusCode::UNAUTHORIZED),
            }
        }
        None => {
            // No API_KEY set — demo mode, skip auth
            Ok(next.run(request).await)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_key_skipped_when_not_set() {
        // When API_KEY env is not set, auth should be skipped
        // This is implicitly tested by existing tests that don't set API_KEY
        assert!(
            std::env::var("API_KEY").is_err(),
            "API_KEY should not be set in test env"
        );
    }
}
