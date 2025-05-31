//! # Strategy Library Prelude
//!
//! This module re-exports the most commonly used types and functions from the strategy library.
//! Importing this module with `use strategy_lib::prelude::*;` is the recommended way to
//! get started with the library.

// Core strategy types
pub use crate::strategy_lib::strategy::{Signal, Strategy, StrategyConfig, StrategyError};

// Backtesting
pub use crate::strategy_lib::backtest::{
    run_backtest, BacktestConfig, BacktestMetrics, BacktestResults,
};

// Specific strategies
pub use crate::strategy_lib::strategy::trend_following::MovingAverageCrossover;

// Utility functions
pub use crate::strategy_lib::utils::{
    calculate_crossovers, calculate_pct_change, raw_values_to_signals,
};
