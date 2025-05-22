//! # Forecast Trade
//!
//! A Rust library for financial time series forecasting and trading strategy development.
//!
//! ## Features
//!
//! - Time series data handling (OHLCV data)
//! - Forecasting models (Exponential Smoothing, Moving Average, ARIMA)
//! - Trading strategies (Mean Reversion, Trend Following, Volatility Breakout)
//! - Strategy backtesting with performance metrics
//! - Support for both daily and minute-level data
//!
//! ## Time Granularity Support
//!
//! This library supports both daily and minute-level data through the `TimeGranularity` enum:
//!
//! ```rust
//! pub enum TimeGranularity {
//!     Daily,
//!     Minute,
//! }
//! ```
//!
//! Trading strategies automatically adjust parameters based on the time granularity:
//!
//! - **Parameter Scaling**: Window sizes, momentum thresholds, and other parameters
//! - **Transaction Costs**: Different commission and slippage models based on granularity
//! - **Direct Integration**: Work with both `day_trade::DailyOhlcv` and `minute_trade::MinuteOhlcv` types
//!
//! ## Quick Start
//!
//! ```rust
//! use forecast_trade::data::DataLoader;
//! use forecast_trade::models::exponential_smoothing::ExponentialSmoothing;
//! use forecast_trade::strategies::mean_reversion::MeanReversionStrategy;
//! use forecast_trade::strategies::{ForecastStrategy, TimeGranularity};
//!
//! // Load data
//! let data = DataLoader::from_csv("data.csv")?;
//!
//! // Create a forecasting model
//! let model = ExponentialSmoothing::new(0.7)?;
//!
//! // Create a trading strategy for daily data
//! let daily_strategy = MeanReversionStrategy::new_with_granularity(
//!     model.clone(),
//!     2.0, // Threshold
//!     TimeGranularity::Daily
//! )?;
//!
//! // Create a trading strategy for minute data
//! let minute_strategy = MeanReversionStrategy::new_with_granularity(
//!     model.clone(),
//!     1.5, // Lower threshold for minute data
//!     TimeGranularity::Minute
//! )?;
//!
//! // Generate trading signals
//! let signals = daily_strategy.generate_signals(&data)?;
//!
//! // Run backtest
//! let results = daily_strategy.backtest(&data, 10000.0)?;
//! ```

pub mod data;
pub mod error;
pub mod models;
pub mod strategies;

// Re-export commonly used types
pub use crate::data::{DataLoader, TimeSeriesData};
pub use crate::error::ForecastError;
pub use crate::models::{ForecastModel, ForecastResult};
pub use crate::strategies::ForecastStrategy;

// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const NAME: &str = env!("CARGO_PKG_NAME");
