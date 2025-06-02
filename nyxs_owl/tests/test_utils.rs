//! Test utilities for NyxsOwl testing

use chrono::{DateTime, Duration, Utc};
use rand::prelude::*;

/// Test data generator for creating test datasets
pub struct TestDataGenerator;

impl TestDataGenerator {
    /// Generate stock price data with random walk
    pub fn generate_stock_prices(
        count: usize,
        initial_price: f64,
        volatility: f64,
    ) -> Vec<(DateTime<Utc>, f64)> {
        let mut rng = thread_rng();
        let mut prices = Vec::with_capacity(count);
        let mut current_price = initial_price;
        let start_date = Utc::now() - Duration::days(count as i64);

        for i in 0..count {
            let date = start_date + Duration::days(i as i64);
            let change = rng.gen_range(-volatility..volatility);
            current_price *= 1.0 + change;
            prices.push((date, current_price));
        }

        prices
    }
}

/// Assertion utilities for testing
pub mod assertions {
    /// Assert that a forecast is valid (non-empty, finite values)
    pub fn assert_forecast_valid(forecast: &[f64], expected_length: usize) {
        assert_eq!(
            forecast.len(),
            expected_length,
            "Forecast length doesn't match expected"
        );
        assert!(
            forecast.iter().all(|&x| x.is_finite()),
            "Forecast contains non-finite values"
        );
        assert!(!forecast.is_empty(), "Forecast should not be empty");
    }
}
