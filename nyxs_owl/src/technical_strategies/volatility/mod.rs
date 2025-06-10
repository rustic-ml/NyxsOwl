// nyxs_owl/src/technical_strategies/volatility/mod.rs
//! Strategies based on volatility indicators.

pub mod bollinger_bands_strategy;
pub mod atr_strategy;

// Re-export key strategy types
pub use atr_strategy::{ATRStrategy, ATRConfig};

// Re-export key strategy functions
pub use bollinger_bands_strategy::bollinger_bands_signals;
