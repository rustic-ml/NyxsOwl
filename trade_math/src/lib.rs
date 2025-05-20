//! # Trade Math
//!
//! Mathematical calculations for trading indicators and strategies.
//! This crate provides implementations of common technical indicators and
//! trading calculations.

use thiserror::Error;

// Indicator modules
pub mod forecasting;
pub mod moving_averages;
pub mod oscillators;
pub mod volatility;
pub mod volume;

/// Errors that can occur in trading-related calculations
#[derive(Error, Debug)]
pub enum MathError {
    #[error("Insufficient data for calculation: {0}")]
    InsufficientData(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Calculation error: {0}")]
    CalculationError(String),
}

/// Result type for trading math operations
pub type Result<T> = std::result::Result<T, MathError>;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
