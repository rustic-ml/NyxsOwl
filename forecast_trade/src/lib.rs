//! # Forecast Trade
//!
//! A Rust library for financial time series forecasting and trading strategy development.
//!
//! ## Features
//!
//! - Time series data handling (OHLCV data)
//! - Forecasting models from OxiDiviner (Exponential Smoothing, Moving Average, ARIMA, GARCH)
//! - Trading strategies (Mean Reversion, Trend Following, Volatility Breakout, ARIMA, Volatility)
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
//! use forecast_trade::models::oxidiviner::ExponentialSmoothingAdapter;
//! use forecast_trade::strategies::arima_strategy::ArimaStrategy;
//! use forecast_trade::strategies::{ForecastStrategy, TimeGranularity};
//!
//! // Load data
//! let data = DataLoader::from_csv("data.csv")?;
//!
//! // Create a forecasting model
//! let model = ExponentialSmoothingAdapter::new(0.7)?;
//!
//! // Create a trading strategy for daily data
//! let daily_strategy = ArimaStrategy::new_with_granularity(
//!     model.clone(),
//!     2.0, // Threshold
//!     TimeGranularity::Daily
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
pub mod backtest;

// Re-export common types
pub use crate::data::{DataLoader, TimeSeriesData};
pub use crate::error::ForecastError;

// Re-export from models
pub use models::ForecastModel;
pub use models::ForecastResult;
pub use models::oxidiviner::{
    ExponentialSmoothingAdapter, MovingAverageAdapter, 
    ArimaAdapter, GarchAdapter
};

// Re-export from strategies
pub use strategies::{
    BacktestResult, ForecastStrategy, TimeGranularity, TradingSignal,
    arima_strategy::{ArimaStrategy, create_sarima_strategy}, 
    volatility_strategy::{VolatilityStrategy, create_garch_strategy},
};

// Re-export common utilities
pub use backtest::run_backtest;

// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const NAME: &str = env!("CARGO_PKG_NAME");
