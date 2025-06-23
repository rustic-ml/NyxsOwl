//! Volatility indicators and calculations module
//!
//! This module provides implementations of various volatility-based technical indicators
//! used in financial analysis and trading strategies.

/// Average True Range (ATR) indicator implementation
pub mod atr;

/// Bollinger Bands indicator implementation
pub mod bollinger_bands;

/// Chandelier Exit indicator implementation
pub mod chandelier_exit;

/// SuperTrend indicator implementation
pub mod supertrend;

pub use atr::calculate_atr;
pub use bollinger_bands::calculate_bollinger_bands;
pub use chandelier_exit::calculate_chandelier_exit;
pub use supertrend::calculate_supertrend;

// ... existing code ... 