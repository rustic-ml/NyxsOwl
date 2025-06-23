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

/// Volume analysis functions
pub mod volume;

/// Pattern recognition and geometric analysis
pub mod patterns;

/// Re-exported momentum indicator functions
pub use momentum::{
    calculate_cci, calculate_macd, calculate_mfi, calculate_roc, calculate_rsi,
    calculate_stochastic,
};
/// Re-exported moving average functions
pub use moving_averages::{calculate_ema, calculate_sma, calculate_vwap, calculate_wma};
/// Re-exported pattern recognition functions
pub use patterns::{
    calculate_fibonacci_extensions, calculate_fibonacci_retracements, detect_fibonacci_retracements,
};
/// Re-exported trend indicator functions
pub use trend::{
    calculate_adx_di, calculate_adxr, calculate_aroon, calculate_aroon_oscillator,
    calculate_directional_movement_components, calculate_vortex,
};
/// Re-exported volatility indicator functions
pub use volatility::{
    atr::calculate_atr, chandelier_exit::calculate_chandelier_exit,
    supertrend::calculate_supertrend,
};
/// Re-exported volume indicator functions
pub use volume::{
    calculate_adl, calculate_cmf, calculate_obv, calculate_vroc, calculate_vwap_with_bands,
};

// TODO: Add other categories of math/indicator functions as modules
// e.g.:
// pub mod volume;
// pub mod oscillators; // (if not part of momentum, e.g. RSI might go here or its own module)
