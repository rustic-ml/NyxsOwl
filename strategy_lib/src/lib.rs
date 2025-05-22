//! # Trading Strategies Library
//! 
//! A Rust library implementing various trading strategies based on technical indicators
//! from the RusTaLib crate. This library is part of the NyxsOwl workspace.
//!
//! ## Usage Options
//!
//! There are three main ways to use this library:
//!
//! ### Option 1: Direct imports from root (most concise)
//!
//! ```rust
//! // Import specific items directly from the root
//! use strategy_lib::{Strategy, StrategyConfig, run_backtest, BacktestConfig};
//! use strategy_lib::strategy::trend_following::MovingAverageCrossover;
//! 
//! // Create and use a strategy
//! let strategy = MovingAverageCrossover::new(config);
//! let results = run_backtest(&strategy, &data, BacktestConfig::default()).unwrap();
//! ```
//!
//! ### Option 2: Using the prelude module (recommended)
//!
//! ```rust
//! // Import everything commonly needed
//! use strategy_lib::prelude::*;
//! 
//! // Create and use a strategy
//! let strategy = MovingAverageCrossover::new(config);
//! let results = run_backtest(&strategy, &data, BacktestConfig::default()).unwrap();
//! ```
//!
//! ### Option 3: Using the nested module structure
//!
//! ```rust
//! // Import from specific modules
//! use strategy_lib::strategy::{Strategy, StrategyConfig};
//! use strategy_lib::strategy::trend_following::MovingAverageCrossover;
//! use strategy_lib::backtest::{run_backtest, BacktestConfig};
//! 
//! // Create and use a strategy
//! let strategy = MovingAverageCrossover::new(config);
//! let results = run_backtest(&strategy, &data, BacktestConfig::default()).unwrap();
//! ```
//! 
//! ## Strategy Categories
//! 
//! This library organizes strategies into several categories:
//! 
//! - **Trend-Following Strategies**: Strategies that aim to capture market trends
//! - **Mean-Reversion Strategies**: Strategies that capitalize on price returning to the mean
//! - **Momentum Strategies**: Strategies that trade based on market momentum
//! - **Volatility Strategies**: Strategies that trade based on market volatility
//! - **Volume Strategies**: Strategies that use volume as a primary signal
//! - **Multi-Indicator Strategies**: Strategies that combine multiple indicators

pub mod strategy;
pub mod backtest;
pub mod utils;
pub mod prelude;

// Re-export commonly used items for convenience
pub use strategy::{Strategy, StrategyConfig, Signal, StrategyError};
pub use backtest::{run_backtest, BacktestConfig, BacktestResults, BacktestMetrics};

// Re-export specific strategies for direct access
pub use strategy::trend_following::MovingAverageCrossover;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
