// nyxs_owl/src/technical_strategies/volatility/mod.rs
//! Strategies based on volatility indicators.

pub mod atr_strategy;
pub mod bollinger_bands_strategy;

// Re-export key strategy types
/// ATR configuration and strategy
pub use atr_strategy::{ATRConfig, ATRStrategy};

// Re-export key strategy functions
/// Bollinger Bands signals
pub use bollinger_bands_strategy::bollinger_bands_signals;
