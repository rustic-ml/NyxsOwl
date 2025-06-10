//! # NyxsOwl
//!
//! A comprehensive Rust library for trading, forecasting, and financial analysis
//! Named after Nyx (Greek goddess of night, symbolizing strategic advantage) and
//! Bubo (wise owl, representing wisdom and strategy).
//!
//! ## Quick Start
//!
//! ```rust
//! use nyxs_owl::prelude::*;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let prices = vec![100.0, 102.0, 101.5, 103.0, 104.5];
//!
//! // Simple technical analysis
//! let sma_values = sma(&prices, 3)?;
//! let rsi_values = rsi(&prices, 14)?;
//!
//! println!("SMA: {:?}", sma_values);
//! # Ok(())
//! # }
//! ```

#![warn(missing_docs)]
#![warn(clippy::all)]

// Common types and utilities
pub mod common;

pub mod prelude {
    //! Common imports for typical NyxsOwl usage
    //!
    //! This module provides a convenient way to import the most commonly used
    //! types and functions with a single `use nyxs_owl::prelude::*;` statement.

    // Re-export common types and new simplified types
    pub use crate::simple_types::{NyxsOwlError, Price, Result, Signal};

    // Re-export forecasting specific items if any become necessary for prelude
    // #[cfg(feature = "forecasting")]
    // pub use crate::forecasting::some_forecast_type; // Example
}

pub mod simple_types {
    //! Shared types and error definitions for the simplified API

    /// Price type alias for clarity
    pub type Price = f64;

    /// Trading signal enumeration
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum Signal {
        /// Hold current position
        Hold = 0,
        /// Buy signal
        Buy = 1,
        /// Sell signal
        Sell = 2,
    }

    impl Signal {
        /// Convert signal to integer representation
        pub fn to_int(self) -> i32 {
            self as i32
        }
    }

    /// Position type for trading signals
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum PositionType {
        Long,
        Short,
        Hold,
    }

    /// Enhanced trading signal with additional metadata (for examples)
    #[derive(Debug, Clone, PartialEq)]
    pub struct SignalData {
        pub signal: Signal,
        pub position_type: PositionType,
        pub timestamp: Option<chrono::DateTime<chrono::Utc>>,
        pub confidence: f64,
        pub metadata: Option<std::collections::HashMap<String, f64>>,
    }

    impl SignalData {
        pub fn new(signal: Signal) -> Self {
            let position_type = match signal {
                Signal::Buy => PositionType::Long,
                Signal::Sell => PositionType::Short,
                Signal::Hold => PositionType::Hold,
            };

            Self {
                signal,
                position_type,
                timestamp: None,
                confidence: 1.0,
                metadata: None,
            }
        }

        pub fn with_confidence(mut self, confidence: f64) -> Self {
            self.confidence = confidence;
            self
        }

        pub fn with_timestamp(mut self, timestamp: chrono::DateTime<chrono::Utc>) -> Self {
            self.timestamp = Some(timestamp);
            self
        }

        pub fn with_metadata(mut self, metadata: std::collections::HashMap<String, f64>) -> Self {
            self.metadata = Some(metadata);
            self
        }
    }

    /// Main error type for NyxsOwl operations
    #[derive(Debug, thiserror::Error)]
    pub enum NyxsOwlError {
        /// Invalid parameter provided
        #[error("Invalid parameter: {0}")]
        InvalidParameter(String),

        /// Data processing error
        #[error("Data error: {0}")]
        DataError(String),

        /// Strategy execution error
        #[error("Strategy error: {0}")]
        StrategyError(String),

        /// Backtest execution error
        #[error("Backtest error: {0}")]
        BacktestError(String),

        /// Missing required data
        #[error("Missing data: {0}")]
        MissingData(String),

        /// Feature not implemented
        #[error("Feature not implemented: {0}")]
        NotImplemented(String),

        /// Validation error
        #[error("Validation error: {0}")]
        ValidationError(String),

        /// Indicator calculation error
        #[error("Indicator error: {0}")]
        IndicatorError(String),

        /// Model error
        #[error("Model error: {0}")]
        ModelError(String),
    }

    /// Result type alias for convenience
    pub type Result<T> = std::result::Result<T, NyxsOwlError>;

    // Add conversion from PolarsError
    impl From<polars::error::PolarsError> for NyxsOwlError {
        fn from(err: polars::error::PolarsError) -> Self {
            NyxsOwlError::DataError(format!("Polars error: {}", err))
        }
    }
}

// Export main modules
pub mod async_parallel;
pub mod memory_optimized; // Cache-conscious data structures and memory optimization
pub mod performance_utils; // High-performance SIMD-accelerated operations
pub mod trade_math; // Core module for technical indicator calculations // Async/parallel processing for concurrent forecasting

#[cfg(feature = "forecasting")] // Keep forecasting feature-gated if it's substantial
pub mod forecasting;

// Enable technical strategies module
pub mod technical_strategies;

// Remove old/unused module declarations and re-exports
// #[cfg(feature = "day-trading")]
// pub mod day_trade;
//
// #[cfg(feature = "minute-trading")]
// pub mod minute_trade;
//
// #[cfg(feature = "backtesting")]
// pub mod strategy_lib;
//
// pub mod performance_utils;
//
// pub mod advanced_optimizations;

// Remove re-exports of items from deleted modules
// #[cfg(feature = "day-trading")]
// pub use day_trade::{DailyOhlcv, Signal, TradingStrategy};
//
// #[cfg(feature = "minute-trading")]
// pub use minute_trade::{MinuteOhlcv, ScalpingStrategy};

// All Owl struct, impl Owl, and Owl tests are removed from this point onwards.
// The previous edit commented them out; this edit ensures full removal.
