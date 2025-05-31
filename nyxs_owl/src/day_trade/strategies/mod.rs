//! Strategies module containing various trading strategy implementations
//!
//! Strategies are organized into three main categories:
//! - Buy-focused strategies (trend following, momentum)
//! - Sell-focused strategies (reversal, overbought detection)
//! - Hold-focused strategies (range-bound, market-neutral)

pub mod buy;
pub mod hold;
pub mod sell;

// Re-export all strategies for convenient access
pub use buy::{AdaptiveMovingAverageStrategy, BreakoutStrategy, MACrossover, MacdStrategy};
pub use hold::{
    BollingerBandsStrategy, CompositeStrategy, DualTimeframeStrategy, ForecastingStrategy,
    GridTradingStrategy, VwapStrategy,
};
pub use sell::{MeanReversionStrategy, RsiStrategy, VolumeBasedStrategy};
