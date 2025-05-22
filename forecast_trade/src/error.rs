//! Error types for the forecast_trade crate

use polars::prelude::PolarsError;
use std::fmt;
use std::io;
use std::num::ParseIntError;
use std::str::ParseBoolError;
use thiserror::Error;

/// Custom error type for forecasting operations
#[derive(Error, Debug)]
pub enum ForecastError {
    /// Data loading or parsing errors
    #[error("Data error: {0}")]
    DataError(String),

    /// Model fitting errors
    #[error("Model fitting error: {0}")]
    ModelFitError(String),

    /// Forecasting errors
    #[error("Forecasting error: {0}")]
    ForecastingError(String),

    /// Input validation errors
    #[error("Validation error: {0}")]
    ValidationError(String),

    /// Parameter validation errors
    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),

    /// I/O errors
    #[error("I/O error: {0}")]
    IoError(#[from] io::Error),

    /// CSV parsing errors
    #[error("CSV error: {0}")]
    CsvError(#[from] csv::Error),

    /// Polars errors
    #[error("Polars error: {0}")]
    PolarsError(String),

    /// OxiDiviner core errors
    #[error("OxiDiviner error: {0}")]
    OxiDivinerError(String),

    /// Parse error
    #[error("Parse error: {0}")]
    ParseError(String),

    /// Other error
    #[error("Error: {0}")]
    Other(String),
}

/// Result type for forecasting operations
pub type Result<T> = std::result::Result<T, ForecastError>;

/// Convert String errors to ForecastError
impl From<String> for ForecastError {
    fn from(err: String) -> Self {
        ForecastError::Other(err)
    }
}

impl From<ParseIntError> for ForecastError {
    fn from(err: ParseIntError) -> Self {
        ForecastError::ParseError(err.to_string())
    }
}

impl From<ParseBoolError> for ForecastError {
    fn from(err: ParseBoolError) -> Self {
        ForecastError::ParseError(err.to_string())
    }
}

impl From<PolarsError> for ForecastError {
    fn from(err: PolarsError) -> Self {
        ForecastError::PolarsError(err.to_string())
    }
}

impl From<&str> for ForecastError {
    fn from(err: &str) -> Self {
        ForecastError::Other(err.to_string())
    }
}
