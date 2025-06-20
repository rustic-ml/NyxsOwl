//! NyxsOwl: A comprehensive financial analysis library for Rust
//!
//! NyxsOwl provides production-ready quantitative finance tools including:
//! - Technical indicators (40+ indicators with SIMD optimization)
//! - Advanced forecasting strategies (7 models with adaptive features)
//! - High-performance backtesting framework
//! - Real-time market data processing
//! - Institutional-grade risk management
//!
//! ## Quick Start
//!
//! ```rust
//! use nyxs_owl::prelude::*;
//! use nyxs_owl::common::time_series::sma;
//! use nyxs_owl::trade_math::momentum::calculate_rsi;
//! use polars::prelude::*;
//!
//! # fn main() -> Result<()> {
//! let prices = vec![100.0, 102.0, 101.5, 103.0, 104.5];
//!
//! // Simple technical analysis
//! let sma_values = sma(&prices, 3);
//! let price_series = Series::new("price".into(), &prices);
//! let rsi_series = calculate_rsi(&price_series, 3).unwrap();
//!
//! println!("SMA: {:?}", sma_values);
//! println!("RSI length: {}", rsi_series.len());
//! # Ok(())
//! # }
//! ```

#![warn(missing_docs)]
#![warn(clippy::all)]

// Common types and utilities
pub mod common;
pub mod simple_types;

pub mod prelude {
    //! Common imports for typical NyxsOwl usage
    //!
    //! This module provides a convenient way to import the most commonly used
    //! types and functions with a single `use nyxs_owl::prelude::*;` statement.

    // Re-export common types and new simplified types
    pub use crate::simple_types::{NyxsOwlError, Price, Result, Signal, SignalData, PositionType};

    // Re-export enhanced Ichimoku functionality
    pub use crate::technical_strategies::trend::{
        enhanced_ichimoku_signals, EnhancedIchimokuConfig, EnhancedIchimokuSignal, MarketRegime,
    };

    // Re-export enhanced RSI functionality
    pub use crate::technical_strategies::momentum::{
        enhanced_rsi_signals, enhanced_rsi_signals_with_config, EnhancedRsiConfig, EnhancedRsiStrategy,
    };

    // Re-export technical strategy framework
    pub use crate::technical_strategies::{
        Strategy, StrategyConfig, TechnicalSignal, TechnicalStrategy,
    };

    // Re-export forecasting specific items if any become necessary for prelude
    #[cfg(feature = "forecasting")]
    pub use crate::forecasting::{
        ConfigValue as ForecastConfigValue,
        Strategy as ForecastStrategy,
        StrategyConfig as ForecastStrategyConfig,
    };
}

// Re-export the comprehensive types from simple_types module
pub use simple_types::*;

// Export main modules
/// Async and parallel processing capabilities for concurrent forecasting
#[cfg(feature = "async-support")]
pub mod async_parallel;
/// Memory optimization utilities and cache-conscious data structures
pub mod memory_optimized; // Cache-conscious data structures and memory optimization
/// High-performance SIMD-accelerated operations and utilities
pub mod performance_utils; // High-performance SIMD-accelerated operations
/// Core module for technical indicator calculations and trading math
pub mod trade_math; // Core module for technical indicator calculations // Async/parallel processing for concurrent forecasting

#[cfg(feature = "forecasting")] // Keep forecasting feature-gated if it's substantial
/// Advanced forecasting models and strategies
pub mod forecasting;

// Enable technical strategies module
/// Technical analysis strategies and implementations
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
