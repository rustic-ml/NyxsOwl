//! Buy-focused trading strategies
//!
//! These strategies primarily focus on identifying opportunities to enter long positions

mod adaptive_moving_average_strategy;
mod breakout_strategy;
mod ma_crossover;
mod macd;

pub use adaptive_moving_average_strategy::AdaptiveMovingAverageStrategy;
pub use breakout_strategy::BreakoutStrategy;
pub use ma_crossover::MACrossover;
pub use macd::MacdStrategy;
