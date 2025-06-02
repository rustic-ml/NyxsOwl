//! # NyxsOwl
//!
//! A comprehensive Rust library for trading, forecasting, and financial analysis.
//!
//! This library provides:
//! - Day trading strategies and analysis
//! - Minute-level trading strategies
//! - Financial forecasting with OxiDiviner integration
//! - Mathematical utilities for trading
//! - Strategy libraries built on technical indicators
//!
//! ## Example
//!
//! ```
//! use nyxs_owl::Owl;
//!
//! let my_owl = Owl::new("Hedwig");
//! assert_eq!(my_owl.name(), "Hedwig");
//! ```
//!
//! ## Modules
//!
//! - [`day_trade`] - Day trading strategies using daily OHLCV data
//! - [`minute_trade`] - Minute-level intraday trading strategies
//! - [`forecast_trade`] - Financial forecasting and prediction models
//! - [`trade_math`] - Mathematical calculations for trading indicators
//! - [`strategy_lib`] - Trading strategies built on technical indicators
//! - [`performance_utils`] - Common performance utilities
//! - [`advanced_optimizations`] - Advanced optimizations for trading strategies
//! - [`common`] - Common utilities and shared functionality

// Common utilities and shared functionality
pub mod common;

// Core trading functionality
#[cfg(feature = "trading-math")]
pub mod trade_math;

// Feature-gated modules
#[cfg(feature = "day-trading")]
pub mod day_trade;

#[cfg(feature = "minute-trading")]
pub mod minute_trade;

#[cfg(feature = "forecasting")]
pub mod forecast_trade;

#[cfg(feature = "strategies")]
pub mod strategy_lib;

#[cfg(any(feature = "day-trading", feature = "minute-trading"))]
pub mod performance_utils;

pub mod advanced_optimizations;

// Re-export commonly used types for convenience
pub use trade_math::{MathError, Result as MathResult};

#[cfg(feature = "day-trading")]
pub use day_trade::{DailyOhlcv, Signal, TradingStrategy};

#[cfg(feature = "minute-trading")]
pub use minute_trade::{MinuteOhlcv, ScalpingStrategy};

#[cfg(feature = "forecasting")]
pub use forecast_trade::{ForecastModel, ForecastResult, TimeSeriesData};

/// An owl from the NyxsOwl project.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Owl {
    name: String,
    wisdom_level: u32,
}

impl Owl {
    /// Creates a new owl with the given name and a default wisdom level of 10.
    ///
    /// # Examples
    ///
    /// ```
    /// use nyxs_owl::Owl;
    ///
    /// let owl = Owl::new("Hedwig");
    /// assert_eq!(owl.name(), "Hedwig");
    /// assert_eq!(owl.wisdom_level(), 10);
    /// ```
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            wisdom_level: 10,
        }
    }

    /// Creates a new owl with a specific name and wisdom level.
    ///
    /// # Examples
    ///
    /// ```
    /// use nyxs_owl::Owl;
    ///
    /// let owl = Owl::with_wisdom("Archimedes", 100);
    /// assert_eq!(owl.name(), "Archimedes");
    /// assert_eq!(owl.wisdom_level(), 100);
    /// ```
    pub fn with_wisdom(name: &str, wisdom_level: u32) -> Self {
        Self {
            name: name.to_string(),
            wisdom_level,
        }
    }

    /// Returns the name of the owl.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the wisdom level of the owl.
    pub fn wisdom_level(&self) -> u32 {
        self.wisdom_level
    }

    /// Increases the owl's wisdom by the given amount.
    pub fn gain_wisdom(&mut self, amount: u32) {
        self.wisdom_level += amount;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_owl() {
        let owl = Owl::new("Hedwig");
        assert_eq!(owl.name(), "Hedwig");
        assert_eq!(owl.wisdom_level(), 10);
    }

    #[test]
    fn test_with_wisdom() {
        let owl = Owl::with_wisdom("Archimedes", 100);
        assert_eq!(owl.name(), "Archimedes");
        assert_eq!(owl.wisdom_level(), 100);
    }

    #[test]
    fn test_gain_wisdom() {
        let mut owl = Owl::new("Hedwig");
        owl.gain_wisdom(5);
        assert_eq!(owl.wisdom_level(), 15);
    }
}
