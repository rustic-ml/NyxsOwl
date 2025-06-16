use polars::prelude::*;
use thiserror::Error;

/// Custom error type for NyxsOwl operations
#[derive(Error, Debug)]
pub enum NyxsOwlError {
    #[error("Data processing error: {0}")]
    DataError(String),

    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),

    #[error("Missing data: {0}")]
    MissingData(String),

    #[error("Strategy error: {0}")]
    StrategyError(String),

    #[error("Not implemented: {0}")]
    NotImplemented(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Polars error: {0}")]
    PolarsError(#[from] PolarsError),
}

/// Result type alias for NyxsOwl operations
pub type Result<T> = std::result::Result<T, NyxsOwlError>;

/// Price type alias for clarity
pub type Price = f64;

/// Trading signal enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Signal {
    /// Hold current position
    Hold = 0,
    /// Buy signal
    Buy = 1,
    /// Sell signal
    Sell = 2,
} 