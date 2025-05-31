//! # Trend Following Strategies
//!
//! This module provides trend following trading strategies based on technical indicators.

pub mod moving_average_crossover;

// Re-export for convenient access
pub use moving_average_crossover::MovingAverageCrossover;
