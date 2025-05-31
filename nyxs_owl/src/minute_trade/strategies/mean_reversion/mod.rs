//! Mean reversion intraday trading strategies
//!
//! This module contains strategies that trade on the assumption that prices
//! revert to the mean over time.

mod mean_reversion_oscillator_strategy;
mod statistical_arbitrage_strategy;

// Re-export strategies
pub use self::mean_reversion_oscillator_strategy::MeanReversionOscillatorStrategy;
pub use self::statistical_arbitrage_strategy::StatisticalArbitrageStrategy;
