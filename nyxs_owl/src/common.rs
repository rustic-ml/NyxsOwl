//! Common utilities and types used across the trading system
//!
//! This module provides shared functionality to reduce code duplication

pub mod strategy_params {
    //! Common parameter validation and configuration structures
    
    use crate::minute_trade::utils::{validate_period, validate_positive};
    
    /// Common strategy configuration with lookback period and thresholds
    #[derive(Debug, Clone)]
    pub struct LookbackConfig {
        pub lookback_period: usize,
        pub threshold: f64,
    }

    impl LookbackConfig {
        pub fn new(lookback_period: usize, threshold: f64, min_period: usize) -> Result<Self, String> {
            validate_period(lookback_period, min_period)?;
            validate_positive(threshold, "Threshold")?;
            Ok(Self { lookback_period, threshold })
        }
    }

    /// Common volume-based strategy configuration
    #[derive(Debug, Clone)]
    pub struct VolumeConfig {
        pub lookback_period: usize,
        pub volume_threshold: f64,
        pub price_change_threshold: f64,
    }

    impl VolumeConfig {
        pub fn new(lookback_period: usize, volume_threshold: f64, price_change_threshold: f64) -> Result<Self, String> {
            validate_period(lookback_period, 10)?;
            validate_positive(volume_threshold, "Volume threshold")?;
            validate_positive(price_change_threshold, "Price change threshold")?;
            
            if volume_threshold < 1.2 {
                return Err("Volume threshold should be at least 1.2 to detect meaningful anomalies.".to_string());
            }
            
            if price_change_threshold > 5.0 {
                return Err("Price change threshold seems too high (>5%). Consider using a lower value.".to_string());
            }
            
            Ok(Self { lookback_period, volume_threshold, price_change_threshold })
        }
    }

    /// Common momentum/breakout strategy configuration
    #[derive(Debug, Clone)]
    pub struct BreakoutConfig {
        pub period: usize,
        pub volume_threshold: f64,
    }

    impl BreakoutConfig {
        pub fn new(period: usize, volume_threshold: f64) -> Result<Self, String> {
            validate_period(period, 5)?;
            validate_positive(volume_threshold, "Volume threshold")?;
            Ok(Self { period, volume_threshold })
        }
    }
}

pub mod test_utils {
    //! Common test data generation and utilities
    
    use crate::minute_trade::{MinuteOhlcv, create_test_data};
    use chrono::NaiveDate;
    
    /// Generate test data with realistic volume variations
    pub fn create_realistic_test_data(count: usize) -> Vec<MinuteOhlcv> {
        create_test_data(count)
    }
    
    /// Create a test date for use in test functions
    pub fn test_date(year: i32, month: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(year, month, day).unwrap_or_default()
    }
}

pub mod imports {
    //! Commonly used imports grouped for convenience
    
    // Core trading types
    pub use crate::minute_trade::{IntradayStrategy, MinuteOhlcv, Signal, TradeError};
    pub use crate::day_trade::{DailyOhlcv, TradingStrategy};
    
    // Utility functions
    pub use crate::minute_trade::utils::{validate_period, validate_positive, calculate_basic_performance};
    
    // Time-related imports
    pub use chrono::{DateTime, NaiveDate, Timelike, Utc};
} 