//! Sell-focused trading strategies
//!
//! These strategies primarily focus on identifying opportunities to exit positions or enter short positions

mod mean_reversion_strategy;
mod rsi;
mod volume_based_strategy;

pub use mean_reversion_strategy::MeanReversionStrategy;
pub use rsi::RsiStrategy;
pub use volume_based_strategy::VolumeBasedStrategy;
