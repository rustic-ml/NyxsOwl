//! Hold-focused and market-neutral trading strategies
//!
//! These strategies focus on range-bound markets, neutral conditions, or balanced approaches

mod bollinger_bands;
mod dual_timeframe;
mod forecasting_strategy;
mod grid_trading_strategy;
mod multi_indicator_strategy;
mod vwap;

pub use bollinger_bands::BollingerBandsStrategy;
pub use dual_timeframe::DualTimeframeStrategy;
pub use forecasting_strategy::ForecastingStrategy;
pub use grid_trading_strategy::GridTradingStrategy;
pub use multi_indicator_strategy::CompositeStrategy;
pub use vwap::VwapStrategy;
