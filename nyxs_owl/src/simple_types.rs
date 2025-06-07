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