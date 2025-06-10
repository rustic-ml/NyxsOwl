// nyxs_owl/src/technical_strategies/volatility/mod.rs
//! Strategies based on volatility indicators.

pub mod atr_strategy;
pub mod bollinger_bands_strategy;

// Re-export key strategy types
pub use atr_strategy::{ATRConfig, ATRStrategy};

// Re-export key strategy functions
pub use bollinger_bands_strategy::bollinger_bands_signals;
