//! Pattern recognition intraday trading strategies
//!
//! This module contains strategies that identify and trade chart patterns.

mod chart_pattern_strategy;
mod support_resistance_strategy;

// Re-export strategies
pub use self::chart_pattern_strategy::ChartPatternStrategy;
pub use self::support_resistance_strategy::SupportResistanceStrategy;
