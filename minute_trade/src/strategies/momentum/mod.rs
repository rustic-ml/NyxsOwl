//! Momentum-based intraday trading strategies
//!
//! This module contains strategies that capitalize on price movement continuation:
//!
//! - **Scalping Strategy**: Ultra-short term trades capturing small price movements
//! - **Momentum Breakout Strategy**: Trades breakouts with volume confirmation

mod momentum_breakout_strategy;
mod scalping_strategy;

pub use momentum_breakout_strategy::MomentumBreakoutStrategy;
pub use scalping_strategy::ScalpingStrategy;
