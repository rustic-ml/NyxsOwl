//! # Day Trade
//!
//! `day_trade` is a Rust library for implementing day trading strategies.
//! It works with both daily and minute OHLCV (Open, High, Low, Close, Volume) data.
//!
//! ## Strategy Categories
//!
//! Strategies are organized into three main categories:
//!
//! - **Buy-focused strategies**: Identify opportunities to enter long positions (trend following, momentum)
//! - **Sell-focused strategies**: Identify opportunities to exit positions or enter short positions (reversal, overbought detection)
//! - **Hold-focused strategies**: Work well in range-bound markets or take a balanced approach (market-neutral)
//!
//! ## Usage Example
//!
//! ```no_run
//! use day_trade::{MeanReversionStrategy, TradingStrategy};
//! use day_trade::utils::generate_test_data;
//!
//! // Create test data and strategy
//! let data = generate_test_data(100, 100.0, 0.05);
//! let strategy = MeanReversionStrategy::default();
//!
//! // Generate trading signals
//! let signals = strategy.generate_signals(&data).unwrap();
//!
//! // Calculate performance
//! let performance = strategy.calculate_performance(&data, &signals).unwrap();
//! println!("Strategy performance: {}%", performance);
//! ```

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use std::hash::Hash;
use thiserror::Error;

// Strategy modules
mod strategies;
// Mock indicators that interface with rustalib and oxidiviner
pub mod mock_indicators;
// Utility functions
pub mod utils;

pub use strategies::{
    AdaptiveMovingAverageStrategy, BollingerBandsStrategy, BreakoutStrategy, CompositeStrategy,
    DualTimeframeStrategy, ForecastingStrategy, GridTradingStrategy, MACrossover, MacdStrategy,
    MeanReversionStrategy, RsiStrategy, VolumeBasedStrategy, VwapStrategy,
};

/// Errors that can occur in day trading operations
#[derive(Error, Debug)]
pub enum TradeError {
    #[error("Invalid data: {0}")]
    InvalidData(String),

    #[error("Insufficient data for strategy: {0}")]
    InsufficientData(String),

    #[error("Strategy calculation error: {0}")]
    CalculationError(String),
}

/// Represents OHLCV (Open, High, Low, Close, Volume) data for a specific time period
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OhlcvData {
    /// Open price
    pub open: f64,
    /// High price
    pub high: f64,
    /// Low price
    pub low: f64,
    /// Close price
    pub close: f64,
    /// Volume
    pub volume: u64,
}

/// Daily OHLCV data with a date
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyOhlcv {
    /// Date of the data point
    pub date: NaiveDate,
    /// OHLCV data
    pub data: OhlcvData,
}

/// Minute-level OHLCV data with timestamp
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinuteOhlcv {
    /// Timestamp of the data point
    pub timestamp: DateTime<Utc>,
    /// OHLCV data
    pub data: OhlcvData,
}

/// Trading signal type representing buy/sell decisions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Signal {
    /// Buy signal - enter a long position
    Buy,
    /// Sell signal - exit a position or enter a short position
    Sell,
    /// Hold signal - maintain current position
    Hold,
}

/// Trait defining a trading strategy
pub trait TradingStrategy {
    /// Analyze data and generate trading signals
    fn generate_signals(&self, data: &[DailyOhlcv]) -> Result<Vec<Signal>, TradeError>;

    /// Calculate performance metrics for the strategy
    fn calculate_performance(
        &self,
        data: &[DailyOhlcv],
        signals: &[Signal],
    ) -> Result<f64, TradeError>;
}

/// Trait defining an intraday trading strategy using minute data
pub trait IntradayTradingStrategy {
    /// Analyze minute data and generate trading signals
    fn generate_signals(&self, data: &[MinuteOhlcv]) -> Result<Vec<Signal>, TradeError>;

    /// Calculate performance metrics for the intraday strategy
    fn calculate_performance(
        &self,
        data: &[MinuteOhlcv],
        signals: &[Signal],
    ) -> Result<f64, TradeError>;
}

/// Trait defining a realtime trading strategy that generates signals based on streaming data
pub trait RealtimeTradingStrategy {
    /// Update the strategy with new OHLCV data
    fn update(
        &mut self,
        timestamp: DateTime<Utc>,
        open: f64,
        high: f64,
        low: f64,
        close: f64,
        volume: f64,
    ) -> Result<(), String>;

    /// Generate a trading signal
    /// Returns an i8 value representing the strength and direction of the signal:
    /// - Positive values (1, 2) represent buy signals of increasing strength
    /// - Zero represents a hold/neutral signal
    /// - Negative values (-1, -2) represent sell signals of increasing strength
    fn generate_signal(&self) -> Result<i8, String>;

    /// Get the name of the strategy
    fn name(&self) -> &str;

    /// Reset the strategy's internal state
    fn reset(&mut self);
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    fn create_test_data() -> Vec<DailyOhlcv> {
        vec![
            DailyOhlcv {
                date: NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(),
                data: OhlcvData {
                    open: 100.0,
                    high: 105.0,
                    low: 99.0,
                    close: 102.0,
                    volume: 1000,
                },
            },
            DailyOhlcv {
                date: NaiveDate::from_ymd_opt(2023, 1, 2).unwrap(),
                data: OhlcvData {
                    open: 102.0,
                    high: 106.0,
                    low: 101.0,
                    close: 105.0,
                    volume: 1200,
                },
            },
            // Add more data points as needed
        ]
    }

    #[test]
    fn test_ohlcv_data_creation() {
        let data = create_test_data();
        assert_eq!(data.len(), 2);
        assert_eq!(data[0].data.close, 102.0);
    }

    #[test]
    fn test_generate_test_data() {
        let data = utils::generate_test_data(50, 100.0, 0.05);
        assert_eq!(data.len(), 50);
        assert!(data[0].data.open == 100.0);

        // Check that dates are sequential
        for i in 1..data.len() {
            assert!(data[i].date > data[i - 1].date);
        }
    }
}
