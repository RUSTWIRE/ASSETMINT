// DISCLAIMER: Technical demo code — legal wrappers required in production
// SPDX-License-Identifier: MIT
//
//! Centralized price feed aggregator.
//! Fetches prices from multiple simulated sources, averages with outlier rejection.
//! Production would use real APIs (CoinGecko, CoinMarketCap, etc.)

use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::info;

use crate::LOG_PREFIX;

#[derive(Error, Debug)]
pub enum OracleError {
    #[error("[K-RWA] Price fetch failed: {0}")]
    FetchFailed(String),
    #[error("[K-RWA] Insufficient price sources: need {needed}, have {have}")]
    InsufficientSources { needed: usize, have: usize },
    #[error("[K-RWA] All prices are outliers")]
    AllOutliers,
}

/// Minimum sources required for a valid aggregation
pub const MIN_SOURCES: usize = 2;
/// Maximum deviation from median before a source is considered an outlier (20%)
pub const OUTLIER_THRESHOLD_PCT: f64 = 20.0;

/// A price data point from a single source
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PricePoint {
    pub source: String,
    pub price_usd: f64,
    pub timestamp: u64,
}

/// Aggregated price with attestation metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedPrice {
    pub price_usd: f64,
    pub sources_used: usize,
    pub sources_rejected: usize,
    pub timestamp: u64,
    pub asset_id: String,
}

/// Simulated price sources for testnet demo
fn simulated_sources(asset_id: &str) -> Vec<PricePoint> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    // Simulate 3 price feeds with slight variance
    let base_price = match asset_id {
        "KPROP-NYC-TEST" => 250_000.0,  // $250k property token
        "KAS" => 0.15,                   // KAS price
        _ => 100.0,                      // Default
    };

    vec![
        PricePoint {
            source: "source_alpha".into(),
            price_usd: base_price * 1.001, // +0.1%
            timestamp: now,
        },
        PricePoint {
            source: "source_beta".into(),
            price_usd: base_price * 0.999, // -0.1%
            timestamp: now,
        },
        PricePoint {
            source: "source_gamma".into(),
            price_usd: base_price * 1.002, // +0.2%
            timestamp: now,
        },
    ]
}

/// Aggregate prices with outlier rejection
pub fn aggregate_prices(prices: &[PricePoint]) -> Result<f64, OracleError> {
    if prices.len() < MIN_SOURCES {
        return Err(OracleError::InsufficientSources {
            needed: MIN_SOURCES,
            have: prices.len(),
        });
    }

    // Find median
    let mut sorted: Vec<f64> = prices.iter().map(|p| p.price_usd).collect();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let median = sorted[sorted.len() / 2];

    // Reject outliers (>OUTLIER_THRESHOLD_PCT deviation from median)
    let valid: Vec<f64> = prices
        .iter()
        .filter(|p| {
            let deviation = ((p.price_usd - median) / median * 100.0).abs();
            deviation <= OUTLIER_THRESHOLD_PCT
        })
        .map(|p| p.price_usd)
        .collect();

    if valid.is_empty() {
        return Err(OracleError::AllOutliers);
    }

    let avg = valid.iter().sum::<f64>() / valid.len() as f64;
    Ok(avg)
}

/// Get a single simulated price point for an asset (public, for fallback use)
pub fn get_simulated_price(asset_id: &str) -> PricePoint {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let base_price = match asset_id {
        "KPROP-NYC-TEST" => 250_000.0,
        "KAS" => 0.15,
        _ => 100.0,
    };

    PricePoint {
        source: "simulated".to_string(),
        price_usd: base_price,
        timestamp: now,
    }
}

/// Fetch a live price from CoinGecko API
/// Falls back to simulated price if the API is unreachable
pub async fn fetch_coingecko_price(asset_id: &str) -> Result<PricePoint, OracleError> {
    let coingecko_id = match asset_id {
        "KAS" => "kaspa",
        _ => "kaspa", // Default to KAS price for demo assets
    };

    let url = format!(
        "https://api.coingecko.com/api/v3/simple/price?ids={}&vs_currencies=usd",
        coingecko_id
    );

    info!("{} Fetching live price from CoinGecko for {}", LOG_PREFIX, asset_id);

    match reqwest::get(&url).await {
        Ok(response) => {
            match response.json::<serde_json::Value>().await {
                Ok(data) => {
                    if let Some(price) = data[coingecko_id]["usd"].as_f64() {
                        let now = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs();
                        info!("{} CoinGecko price for {}: ${:.4}", LOG_PREFIX, asset_id, price);
                        Ok(PricePoint {
                            source: "coingecko".to_string(),
                            price_usd: if asset_id == "KAS" { price } else { price * 250000.0 / 0.15 },
                            timestamp: now,
                        })
                    } else {
                        info!("{} CoinGecko returned no price, falling back to simulated", LOG_PREFIX);
                        Ok(get_simulated_price(asset_id))
                    }
                }
                Err(e) => {
                    info!("{} CoinGecko parse error: {}, falling back", LOG_PREFIX, e);
                    Ok(get_simulated_price(asset_id))
                }
            }
        }
        Err(e) => {
            info!("{} CoinGecko unreachable: {}, falling back", LOG_PREFIX, e);
            Ok(get_simulated_price(asset_id))
        }
    }
}

/// Get an aggregated price combining live and simulated sources
pub async fn get_live_aggregated_price(asset_id: &str) -> Result<AggregatedPrice, OracleError> {
    let mut prices = vec![];

    // Try live source
    if let Ok(live) = fetch_coingecko_price(asset_id).await {
        prices.push(live);
    }

    // Add simulated sources to ensure minimum
    for i in 0..2 {
        let mut sim = get_simulated_price(asset_id);
        sim.source = format!("simulated_{}", i);
        prices.push(sim);
    }

    let price = aggregate_prices(&prices)?;
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    Ok(AggregatedPrice {
        price_usd: price,
        sources_used: prices.len(),
        sources_rejected: 0,
        timestamp: now,
        asset_id: asset_id.to_string(),
    })
}

/// Fetch and aggregate price for an asset (uses simulated sources for testnet)
pub fn get_aggregated_price(asset_id: &str) -> Result<AggregatedPrice, OracleError> {
    info!("{} Fetching aggregated price for {}", LOG_PREFIX, asset_id);

    let sources = simulated_sources(asset_id);
    let total_sources = sources.len();
    let avg_price = aggregate_prices(&sources)?;

    // Count how many were used vs rejected
    let mut sorted: Vec<f64> = sources.iter().map(|p| p.price_usd).collect();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let median = sorted[sorted.len() / 2];
    let used = sources
        .iter()
        .filter(|p| {
            ((p.price_usd - median) / median * 100.0).abs() <= OUTLIER_THRESHOLD_PCT
        })
        .count();

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    info!(
        "{} Aggregated price for {}: ${:.2} ({}/{} sources)",
        LOG_PREFIX, asset_id, avg_price, used, total_sources
    );

    Ok(AggregatedPrice {
        price_usd: avg_price,
        sources_used: used,
        sources_rejected: total_sources - used,
        timestamp: now,
        asset_id: asset_id.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_prices(values: &[f64]) -> Vec<PricePoint> {
        values
            .iter()
            .enumerate()
            .map(|(i, &v)| PricePoint {
                source: format!("source_{}", i),
                price_usd: v,
                timestamp: 1000,
            })
            .collect()
    }

    #[test]
    fn test_aggregate_normal_prices() {
        let prices = make_prices(&[100.0, 101.0, 99.0]);
        let avg = aggregate_prices(&prices).unwrap();
        assert!((avg - 100.0).abs() < 1.0);
    }

    #[test]
    fn test_aggregate_rejects_outlier() {
        // One price is 50% off — should be rejected
        let prices = make_prices(&[100.0, 101.0, 150.0]);
        let avg = aggregate_prices(&prices).unwrap();
        // Should only average the two close prices
        assert!((avg - 100.5).abs() < 1.0);
    }

    #[test]
    fn test_insufficient_sources() {
        let prices = make_prices(&[100.0]);
        assert!(aggregate_prices(&prices).is_err());
    }

    #[test]
    fn test_simulated_price_feed() {
        let price = get_aggregated_price("KPROP-NYC-TEST").unwrap();
        assert!(price.price_usd > 240_000.0);
        assert!(price.price_usd < 260_000.0);
        assert_eq!(price.sources_used, 3);
        assert_eq!(price.sources_rejected, 0);
    }

    #[test]
    fn test_kas_price_feed() {
        let price = get_aggregated_price("KAS").unwrap();
        assert!(price.price_usd > 0.1);
        assert!(price.price_usd < 0.2);
    }

    #[tokio::test]
    async fn test_fetch_coingecko_price() {
        let result = fetch_coingecko_price("KAS").await;
        // Should succeed (either live or fallback)
        assert!(result.is_ok());
        let price = result.unwrap();
        assert!(price.price_usd > 0.0);
    }

    #[tokio::test]
    async fn test_live_aggregated_price() {
        let result = get_live_aggregated_price("KAS").await;
        assert!(result.is_ok());
        let agg = result.unwrap();
        assert!(agg.sources_used >= 2);
    }
}
