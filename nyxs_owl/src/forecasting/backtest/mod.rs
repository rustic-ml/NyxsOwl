//! Forecasting strategy backtesting module
//! 
//! This module provides backtesting capabilities specifically for forecasting-based strategies.

pub mod forecast_backtest;

// Re-export main backtesting functionality
pub use forecast_backtest::*; 