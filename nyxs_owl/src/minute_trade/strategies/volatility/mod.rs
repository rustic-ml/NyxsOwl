//! Volatility-based intraday trading strategies
//!
//! This module contains strategies that trade based on market volatility patterns.

mod bollinger_band_contraction_strategy;
mod volatility_breakout_strategy;

// Re-export strategies
pub use self::bollinger_band_contraction_strategy::BollingerBandContractionStrategy;
pub use self::volatility_breakout_strategy::VolatilityBreakoutStrategy;
