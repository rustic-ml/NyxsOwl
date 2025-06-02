//! Error types for the forecast_trade crate

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Main error type for forecast trading operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ForecastTradeError {
    /// Data processing errors
    DataError(String),
    /// Model training errors  
    ModelError(String),
    /// Forecasting errors
    ForecastError(String),
    /// Strategy errors
    StrategyError(String),
    /// IO errors
    IoError(String),
    /// Configuration errors
    ConfigError(String),
    /// Validation errors
    ValidationError(String),
}

/// Custom error types for the forecast_trade crate
#[derive(Debug, Error)]
pub enum ForecastError {
    /// Error related to data validation or processing
    #[error("Data error: {0}")]
    DataError(String),

    /// Error related to forecasting operations
    #[error("Forecasting error: {0}")]
    ForecastingError(String),

    /// Error related to parameter validation
    #[error("Validation error: {0}")]
    ValidationError(String),

    /// Error from mathematical operations
    #[error("Math error: {0}")]
    MathError(String),

    /// Error from invalid parameters
    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),

    /// Error from IO operations
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// Error from Polars operations
    #[error("Polars error: {0}")]
    PolarsError(String),

    /// Model-related error
    #[error("Model error: {0}")]
    ModelError(String),
}

/// Result type with our custom error
pub type Result<T> = std::result::Result<T, ForecastError>;

impl From<polars::prelude::PolarsError> for ForecastError {
    fn from(err: polars::prelude::PolarsError) -> Self {
        ForecastError::PolarsError(err.to_string())
    }
}

impl From<String> for ForecastError {
    fn from(error: String) -> Self {
        ForecastError::ModelError(error)
    }
}

impl From<&str> for ForecastError {
    fn from(error: &str) -> Self {
        ForecastError::ModelError(error.to_string())
    }
}

impl From<Box<dyn std::error::Error + Send + Sync>> for ForecastError {
    fn from(error: Box<dyn std::error::Error + Send + Sync>) -> Self {
        ForecastError::ModelError(error.to_string())
    }
}
