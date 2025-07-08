// nyxs_owl/src/technical_strategies/oscillators/mod.rs
//! Strategies based on oscillator indicators.

pub mod rsi_strategy;
pub mod williams_r_strategy;

// Re-export key strategy types
/// Williams %R configuration and strategy
pub use williams_r_strategy::{WilliamsRConfig, WilliamsRStrategy};

/// RSI strategy functions
pub use rsi_strategy::{rsi_signals, rsi_bullish_failure_swing, rsi_bearish_failure_swing, rsi_combined_signals};
