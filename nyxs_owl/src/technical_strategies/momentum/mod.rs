// nyxs_owl/src/technical_strategies/momentum/mod.rs
//! Strategies based on momentum indicators.

pub mod enhanced_rsi_strategy;
pub mod macd_strategy;
pub mod stochastic_strategy;

// Re-export key strategy functions and types
/// Enhanced RSI signals and configuration
pub use enhanced_rsi_strategy::{
    enhanced_rsi_signals, enhanced_rsi_signals_with_config, EnhancedRsiConfig, EnhancedRsiStrategy,
};
/// MACD signals
pub use macd_strategy::macd_signals;
/// Stochastic signals
pub use stochastic_strategy::stochastic_signals;
