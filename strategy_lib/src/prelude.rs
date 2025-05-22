//! # Strategy Library Prelude
//!
//! This module re-exports the most commonly used types and functions from the strategy library.
//! Importing this module with `use strategy_lib::prelude::*;` is the recommended way to 
//! get started with the library.

// Core strategy types
pub use crate::strategy::{Strategy, StrategyConfig, Signal, StrategyError};

// Backtesting
pub use crate::backtest::{run_backtest, BacktestConfig, BacktestResults, BacktestMetrics};

// Specific strategies
pub use crate::strategy::trend_following::MovingAverageCrossover;

// Utility functions
pub use crate::utils::{
    raw_values_to_signals,
    calculate_crossovers,
    calculate_pct_change,
}; 