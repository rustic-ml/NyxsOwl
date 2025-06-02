//! # Forecast Trade
//!
//! A Rust library for financial time series forecasting and trading strategy development.
//!
//! ## Features
//!
//! - Time series data handling (OHLCV data)
//! - Advanced forecasting models from OxiDiviner:
//!   - **Exponential Smoothing** (Simple, Holt Linear, Holt-Winters)
//!   - **ETS** (Error, Trend, Seasonal) state space models
//!   - **ARIMA/SARIMA** (AutoRegressive Integrated Moving Average with seasonal support)
//!   - **AR** (AutoRegressive) models
//!   - **GARCH** (Generalized AutoRegressive Conditional Heteroskedasticity) for volatility
//!   - **Moving Average** models with adaptive window optimization
//! - Trading strategies (Mean Reversion, Trend Following, Volatility Breakout, ARIMA, Volatility)
//! - Strategy backtesting with performance metrics
//! - Support for both daily and minute-level data
//! - Advanced model comparison and automatic selection capabilities
//!
//! ## Advanced OxiDiviner Integration
//!
//! This library now provides **full integration** with OxiDiviner v0.4.3, including all advanced variants:
//!
//! ### Core Models
//! - **Simple ES**: Basic exponential smoothing for stable series
//! - **Holt Linear**: Linear trend modeling
//! - **Holt-Winters**: Seasonal pattern forecasting
//! - **ETS**: Error-Trend-Seasonal state space models with automatic component selection
//! - **ARIMA**: Full ARIMA(p,d,q) support with seasonal extensions
//! - **AR**: AutoRegressive models for temporal dependencies
//! - **GARCH**: Volatility modeling and risk forecasting
//!
//! ### Easy API
//! ```rust
//! use forecast_trade::models::oxidiviner::easy;
//!
//! // Automatic model selection with performance comparison
//! let (forecast, best_model) = easy::auto_forecast(dates, values, 5)?;
//!
//! // Advanced model comparison with metrics
//! let comparison = easy::model_comparison(dates, values, 5, 0.8)?;
//!
//! // Specific advanced models
//! let ets_forecast = easy::ets_forecast(dates, values, 5, Some(12))?;
//! let ar_forecast = easy::ar_forecast(dates, values, 5, Some(3))?;
//! let garch_forecast = easy::garch_forecast(dates, values, 5, Some(1), Some(1))?;
//! ```
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
//! - **Direct Integration**: Work with both `crate::day_trade::DailyOhlcv` and `crate::minute_trade::MinuteOhlcv` types
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

pub mod backtest;
pub mod data;
pub mod error;
pub mod models;
pub mod strategies;

// Re-export common types
pub use crate::forecast_trade::data::{DataLoader, TimeSeriesData};
pub use crate::forecast_trade::error::ForecastError;

// Re-export from models
pub use models::oxidiviner::{
    ARAdapter,
    ArimaAdapter,
    // Advanced adapters for backward compatibility
    ETSAdapter,
    ExponentialSmoothingAdapter,
    GarchAdapter,
    MovingAverageAdapter,
    OxiDivinerAdapter,
};
pub use models::{ErrorMetrics, ForecastModel, ForecastResult};

// Re-export from strategies
pub use strategies::{
    arima_strategy::{create_sarima_strategy, ArimaStrategy},
    volatility_strategy::{create_garch_strategy, VolatilityStrategy},
    BacktestResult, ForecastStrategy, TimeGranularity, TradingSignal,
};

// Re-export common utilities
pub use backtest::run_backtest;

// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const NAME: &str = env!("CARGO_PKG_NAME");
