//! Financial mathematics and technical analysis functions
//!
//! This module provides comprehensive mathematical functions for financial trading
//! including momentum indicators, moving averages, trend analysis, and volatility calculations.

/// Momentum-based technical indicators and calculations
pub mod momentum;

/// Moving average calculations and trend-following indicators
pub mod moving_averages;

/// Trend analysis functions including directional indicators
pub mod trend;

/// Volatility measurement and risk analysis functions
pub mod volatility;

/// Re-exported momentum indicator functions
pub use momentum::{calculate_macd, calculate_rsi, calculate_stochastic};
/// Re-exported moving average functions
pub use moving_averages::{calculate_ema, calculate_sma, calculate_vwap, calculate_wma};
/// Re-exported trend indicator functions
pub use trend::{
    calculate_adx_di, calculate_adxr, calculate_aroon, calculate_aroon_oscillator,
    calculate_directional_movement_components, calculate_vortex,
};
/// Re-exported volatility indicator functions
pub use volatility::{
    calculate_atr, calculate_bollinger_bands, calculate_ease_of_movement, calculate_obv,
    calculate_volume_price_trend,
};

// TODO: Add other categories of math/indicator functions as modules
// e.g.:
// pub mod volume;
// pub mod oscillators; // (if not part of momentum, e.g. RSI might go here or its own module)
