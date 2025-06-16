//! Common types, traits, and utilities used throughout NyxsOwl
//!
//! This module provides the foundational types and error handling
//! that are shared across all components of the trading library.

use polars::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// Main result type for NyxsOwl operations
pub type NyxsOwlResult<T> = Result<T, NyxsOwlError>;

/// Comprehensive error types for NyxsOwl
#[derive(Debug, Clone, PartialEq)]
pub enum NyxsOwlError {
    /// Data-related errors (missing columns, invalid format, etc.)
    DataError(String),

    /// Strategy configuration or parameter errors
    StrategyError(String),

    /// Validation errors (invalid parameters, etc.)
    ValidationError(String),

    /// Invalid parameter error
    InvalidParameter(String),

    /// Missing required data
    MissingData(String),

    /// Calculation or mathematical errors
    CalculationError(String),

    /// Optimization-related errors
    OptimizationError(String),

    /// I/O errors (file operations, network, etc.)
    IoError(String),

    /// Polars DataFrame errors
    PolarsError(String),

    /// Generic error for unspecified issues
    Other(String),
}

impl fmt::Display for NyxsOwlError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NyxsOwlError::DataError(msg) => write!(f, "Data Error: {}", msg),
            NyxsOwlError::StrategyError(msg) => write!(f, "Strategy Error: {}", msg),
            NyxsOwlError::ValidationError(msg) => write!(f, "Validation Error: {}", msg),
            NyxsOwlError::InvalidParameter(msg) => write!(f, "Invalid Parameter: {}", msg),
            NyxsOwlError::MissingData(msg) => write!(f, "Missing Data: {}", msg),
            NyxsOwlError::CalculationError(msg) => write!(f, "Calculation Error: {}", msg),
            NyxsOwlError::OptimizationError(msg) => write!(f, "Optimization Error: {}", msg),
            NyxsOwlError::IoError(msg) => write!(f, "I/O Error: {}", msg),
            NyxsOwlError::PolarsError(msg) => write!(f, "Polars Error: {}", msg),
            NyxsOwlError::Other(msg) => write!(f, "Error: {}", msg),
        }
    }
}

impl std::error::Error for NyxsOwlError {}

impl From<PolarsError> for NyxsOwlError {
    fn from(error: PolarsError) -> Self {
        NyxsOwlError::PolarsError(error.to_string())
    }
}

impl From<std::io::Error> for NyxsOwlError {
    fn from(error: std::io::Error) -> Self {
        NyxsOwlError::IoError(error.to_string())
    }
}

/// Trading signal types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Signal {
    /// Buy signal
    Buy,

    /// Sell signal
    Sell,

    /// Hold (no action) signal
    Hold,
}

impl Signal {
    /// Convert signal to integer representation
    pub fn to_int(&self) -> i32 {
        match self {
            Signal::Buy => 1,
            Signal::Sell => -1,
            Signal::Hold => 0,
        }
    }

    /// Create signal from integer representation
    pub fn from_int(value: i32) -> Self {
        match value {
            1 => Signal::Buy,
            -1 => Signal::Sell,
            _ => Signal::Hold,
        }
    }
}

impl fmt::Display for Signal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Signal::Buy => write!(f, "BUY"),
            Signal::Sell => write!(f, "SELL"),
            Signal::Hold => write!(f, "HOLD"),
        }
    }
}

/// Enhanced technical signal with metadata
#[derive(Debug, Clone, PartialEq)]
pub struct TechnicalSignal {
    /// Base trading signal
    pub signal: Signal,

    /// Signal strength (0.0 to 1.0)
    pub strength: f64,

    /// Signal confidence (0.0 to 1.0)
    pub confidence: f64,

    /// Additional metadata
    pub metadata: HashMap<String, f64>,

    /// Timestamp (optional)
    pub timestamp: Option<i64>,
}

impl TechnicalSignal {
    /// Create a new technical signal
    pub fn new(signal: Signal) -> Self {
        Self {
            signal,
            strength: 1.0,
            confidence: 1.0,
            metadata: HashMap::new(),
            timestamp: None,
        }
    }

    /// Set signal strength
    pub fn with_strength(mut self, strength: f64) -> Self {
        self.strength = strength.max(0.0).min(1.0);
        self
    }

    /// Set signal confidence
    pub fn with_confidence(mut self, confidence: f64) -> Self {
        self.confidence = confidence.max(0.0).min(1.0);
        self
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: &str, value: f64) -> Self {
        self.metadata.insert(key.to_string(), value);
        self
    }

    /// Set timestamp
    pub fn with_timestamp(mut self, timestamp: i64) -> Self {
        self.timestamp = Some(timestamp);
        self
    }
}

impl Default for TechnicalSignal {
    fn default() -> Self {
        Self::new(Signal::Hold)
    }
}

/// Performance metrics for strategy evaluation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// Total return as percentage
    pub total_return: f64,

    /// Sharpe ratio (risk-adjusted return)
    pub sharpe_ratio: f64,

    /// Maximum drawdown as percentage
    pub max_drawdown: f64,

    /// Win rate (percentage of winning trades)
    pub win_rate: f64,

    /// Total number of trades
    pub total_trades: i32,

    /// Average return per trade
    pub avg_trade_return: f64,

    /// Return volatility (standard deviation)
    pub volatility: f64,
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self {
            total_return: 0.0,
            sharpe_ratio: 0.0,
            max_drawdown: 0.0,
            win_rate: 0.0,
            total_trades: 0,
            avg_trade_return: 0.0,
            volatility: 0.0,
        }
    }
}

/// Configuration value types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ConfigValue {
    /// Integer value
    Int(i64),

    /// Floating point value
    Float(f64),

    /// String value
    String(String),

    /// Boolean value
    Bool(bool),
}

impl ConfigValue {
    /// Extract integer value
    pub fn as_int(&self) -> Option<i64> {
        match self {
            ConfigValue::Int(v) => Some(*v),
            ConfigValue::Float(v) => Some(*v as i64),
            _ => None,
        }
    }

    /// Extract float value
    pub fn as_float(&self) -> Option<f64> {
        match self {
            ConfigValue::Float(v) => Some(*v),
            ConfigValue::Int(v) => Some(*v as f64),
            _ => None,
        }
    }

    /// Extract string value
    pub fn as_string(&self) -> Option<&str> {
        match self {
            ConfigValue::String(v) => Some(v),
            _ => None,
        }
    }

    /// Extract boolean value
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            ConfigValue::Bool(v) => Some(*v),
            _ => None,
        }
    }
}

/// Strategy configuration management
#[derive(Debug, Clone, Default)]
pub struct StrategyConfig {
    /// Configuration parameters
    pub parameters: HashMap<String, ConfigValue>,
}

impl StrategyConfig {
    /// Create a new empty configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a parameter to the configuration
    pub fn with_parameter<T: Into<ConfigValue>>(mut self, key: &str, value: T) -> Self {
        self.parameters.insert(key.to_string(), value.into());
        self
    }

    /// Get an integer parameter
    pub fn get_int(&self, key: &str) -> Option<i64> {
        self.parameters.get(key)?.as_int()
    }

    /// Get a float parameter
    pub fn get_float(&self, key: &str) -> Option<f64> {
        self.parameters.get(key)?.as_float()
    }
    
    /// Get an integer parameter (compatible with forecasting interface)
    pub fn get_int_compat(&self, key: &str) -> Result<i64, crate::simple_types::NyxsOwlError> {
        self.get_int(key).ok_or_else(|| {
            crate::simple_types::NyxsOwlError::ValidationError(format!("Parameter '{}' not found or not an integer", key))
        })
    }
    
    /// Get a float parameter (compatible with forecasting interface)
    pub fn get_float_compat(&self, key: &str) -> Result<f64, crate::simple_types::NyxsOwlError> {
        self.get_float(key).ok_or_else(|| {
            crate::simple_types::NyxsOwlError::ValidationError(format!("Parameter '{}' not found or not a float", key))
        })
    }

    /// Get a string parameter
    pub fn get_string(&self, key: &str) -> Option<&str> {
        self.parameters.get(key)?.as_string()
    }

    /// Get a boolean parameter
    pub fn get_bool(&self, key: &str) -> Option<bool> {
        self.parameters.get(key)?.as_bool()
    }

    /// Set a parameter
    pub fn set_parameter<T: Into<ConfigValue>>(&mut self, key: &str, value: T) {
        self.parameters.insert(key.to_string(), value.into());
    }
}

// Implement From traits for ConfigValue
impl From<i32> for ConfigValue {
    fn from(value: i32) -> Self {
        ConfigValue::Int(value as i64)
    }
}

impl From<i64> for ConfigValue {
    fn from(value: i64) -> Self {
        ConfigValue::Int(value)
    }
}

impl From<f32> for ConfigValue {
    fn from(value: f32) -> Self {
        ConfigValue::Float(value as f64)
    }
}

impl From<f64> for ConfigValue {
    fn from(value: f64) -> Self {
        ConfigValue::Float(value)
    }
}

impl From<String> for ConfigValue {
    fn from(value: String) -> Self {
        ConfigValue::String(value)
    }
}

impl From<&str> for ConfigValue {
    fn from(value: &str) -> Self {
        ConfigValue::String(value.to_string())
    }
}

impl From<bool> for ConfigValue {
    fn from(value: bool) -> Self {
        ConfigValue::Bool(value)
    }
}

/// Utility functions for data validation
pub mod validation {
    use super::*;

    /// Validate that a DataFrame has required columns
    pub fn validate_required_columns(data: &DataFrame, required: &[&str]) -> NyxsOwlResult<()> {
        let columns = data.get_column_names();

        for &col in required {
            if !columns.iter().any(|c| c.as_str() == col) {
                return Err(NyxsOwlError::MissingData(format!(
                    "Required column '{}' not found in data",
                    col
                )));
            }
        }

        Ok(())
    }

    /// Validate that data has minimum required rows
    pub fn validate_min_data_points(data: &DataFrame, min_points: usize) -> NyxsOwlResult<()> {
        if data.height() < min_points {
            return Err(NyxsOwlError::DataError(format!(
                "Insufficient data: {} rows required, {} provided",
                min_points,
                data.height()
            )));
        }

        Ok(())
    }

    /// Validate that numeric parameters are within acceptable ranges
    pub fn validate_parameter_range(
        value: f64,
        min: f64,
        max: f64,
        param_name: &str,
    ) -> NyxsOwlResult<()> {
        if value < min || value > max {
            return Err(NyxsOwlError::InvalidParameter(format!(
                "Parameter '{}' value {} is outside valid range [{}, {}]",
                param_name, value, min, max
            )));
        }

        Ok(())
    }
}

/// Utility functions for time series operations
pub mod time_series {
    use super::*;

    /// Calculate simple moving average
    pub fn sma(values: &[f64], period: usize) -> Vec<f64> {
        if period == 0 || values.len() < period {
            return vec![f64::NAN; values.len()];
        }

        let mut result = vec![f64::NAN; period - 1];

        for i in (period - 1)..values.len() {
            let sum: f64 = values[(i + 1 - period)..=i].iter().sum();
            result.push(sum / period as f64);
        }

        result
    }

    /// Calculate exponential moving average
    pub fn ema(values: &[f64], period: usize) -> Vec<f64> {
        if period == 0 || values.is_empty() {
            return vec![f64::NAN; values.len()];
        }

        let alpha = 2.0 / (period + 1) as f64;
        let mut result = Vec::with_capacity(values.len());

        // First value is just the first price
        result.push(values[0]);

        for i in 1..values.len() {
            let ema_val = alpha * values[i] + (1.0 - alpha) * result[i - 1];
            result.push(ema_val);
        }

        result
    }

    /// Calculate standard deviation over a rolling window
    pub fn rolling_std(values: &[f64], period: usize) -> Vec<f64> {
        if period == 0 || values.len() < period {
            return vec![f64::NAN; values.len()];
        }

        let mut result = vec![f64::NAN; period - 1];

        for i in (period - 1)..values.len() {
            let window = &values[(i + 1 - period)..=i];
            let mean: f64 = window.iter().sum::<f64>() / period as f64;

            let variance: f64 =
                window.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / period as f64;

            result.push(variance.sqrt());
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signal_conversion() {
        assert_eq!(Signal::Buy.to_int(), 1);
        assert_eq!(Signal::Sell.to_int(), -1);
        assert_eq!(Signal::Hold.to_int(), 0);

        assert_eq!(Signal::from_int(1), Signal::Buy);
        assert_eq!(Signal::from_int(-1), Signal::Sell);
        assert_eq!(Signal::from_int(0), Signal::Hold);
    }

    #[test]
    fn test_technical_signal() {
        let signal = TechnicalSignal::new(Signal::Buy)
            .with_strength(0.8)
            .with_confidence(0.9)
            .with_metadata("rsi", 75.0);

        assert_eq!(signal.signal, Signal::Buy);
        assert_eq!(signal.strength, 0.8);
        assert_eq!(signal.confidence, 0.9);
        assert_eq!(signal.metadata.get("rsi"), Some(&75.0));
    }

    #[test]
    fn test_config_value() {
        let int_val = ConfigValue::from(42);
        assert_eq!(int_val.as_int(), Some(42));

        let float_val = ConfigValue::from(3.14);
        assert_eq!(float_val.as_float(), Some(3.14));

        let string_val = ConfigValue::from("test");
        assert_eq!(string_val.as_string(), Some("test"));

        let bool_val = ConfigValue::from(true);
        assert_eq!(bool_val.as_bool(), Some(true));
    }

    #[test]
    fn test_strategy_config() {
        let config = StrategyConfig::new()
            .with_parameter("period", 14)
            .with_parameter("threshold", 0.02)
            .with_parameter("enabled", true);

        assert_eq!(config.get_int("period"), Some(14));
        assert_eq!(config.get_float("threshold"), Some(0.02));
        assert_eq!(config.get_bool("enabled"), Some(true));
    }

    #[test]
    fn test_time_series_sma() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let sma = time_series::sma(&values, 3);

        assert!(sma[0].is_nan());
        assert!(sma[1].is_nan());
        assert_eq!(sma[2], 2.0); // (1+2+3)/3
        assert_eq!(sma[3], 3.0); // (2+3+4)/3
        assert_eq!(sma[4], 4.0); // (3+4+5)/3
    }
}
