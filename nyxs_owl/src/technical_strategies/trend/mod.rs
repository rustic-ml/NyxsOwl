// nyxs_owl/src/technical_strategies/trend/mod.rs
//! Strategies based on trend indicators.

pub mod adx_di_strategy;
pub mod aroon_strategy;
pub mod ichimoku_strategy;
pub mod psar_strategy;
pub mod vortex_strategy;

// Re-export key strategy functions and types
/// Enhanced Ichimoku signals and configuration
pub use ichimoku_strategy::{
    enhanced_ichimoku_signals, ichimoku_kumo_breakout_signals, EnhancedIchimokuConfig,
    EnhancedIchimokuSignal, MarketRegime,
};

// Re-export key strategy functions if desired, e.g.:
// pub use adx_di_strategy::adx_di_signals;
// pub use aroon_strategy::aroon_signals;
// pub use ichimoku_strategy::ichimoku_signals;
// pub use psar_strategy::psar_signals;
// pub use vortex_strategy::vortex_signals;
