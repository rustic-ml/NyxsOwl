//! # Trading Strategies
//!
//! This module provides various trading strategies based on technical indicators.

use polars::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

pub mod mean_reversion;
pub mod momentum;
pub mod multi_indicator;
pub mod trend_following;
pub mod volatility;
pub mod volume;

/// Errors that can occur during strategy execution
#[derive(Error, Debug)]
pub enum StrategyError {
    #[error("Missing required data: {0}")]
    MissingData(String),

    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),

    #[error("Polars error: {0}")]
    PolarsError(#[from] PolarsError),

    #[error("Indicator error: {0}")]
    IndicatorError(String),

    #[error("Configuration error: {0}")]
    ConfigurationError(String),

    #[error("Validation error: {0}")]
    ValidationError(String),
}

/// Strategy signals representing trade actions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Signal {
    Buy,
    Sell,
    Hold,
}

impl Signal {
    /// Convert signal to integer representation
    pub fn to_int(&self) -> i32 {
        match self {
            Signal::Hold => 0,
            Signal::Buy => 1,
            Signal::Sell => 2,
        }
    }

    /// Create signal from integer representation
    pub fn from_int(value: i32) -> Self {
        match value {
            1 => Signal::Buy,
            2 => Signal::Sell,
            _ => Signal::Hold,
        }
    }
}

/// Enhanced configuration for strategy parameters with type safety
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StrategyConfig {
    /// Strategy-specific parameters
    pub parameters: HashMap<String, ParameterValue>,
    /// Risk management settings
    pub risk_settings: Option<RiskSettings>,
    /// Performance optimization settings
    pub optimization: Option<OptimizationSettings>,
}

impl StrategyConfig {
    /// Create a new configuration with parameters
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a parameter to the configuration
    pub fn with_parameter<T: Into<ParameterValue>>(mut self, key: &str, value: T) -> Self {
        self.parameters.insert(key.to_string(), value.into());
        self
    }

    /// Get a parameter value
    pub fn get_parameter(&self, key: &str) -> Option<&ParameterValue> {
        self.parameters.get(key)
    }

    /// Get an integer parameter
    pub fn get_int(&self, key: &str) -> Result<i64, StrategyError> {
        match self.get_parameter(key) {
            Some(ParameterValue::Integer(val)) => Ok(*val),
            Some(_) => Err(StrategyError::InvalidParameter(format!(
                "Parameter '{}' is not an integer",
                key
            ))),
            None => Err(StrategyError::MissingData(format!(
                "Parameter '{}' not found",
                key
            ))),
        }
    }

    /// Get a float parameter
    pub fn get_float(&self, key: &str) -> Result<f64, StrategyError> {
        match self.get_parameter(key) {
            Some(ParameterValue::Float(val)) => Ok(*val),
            Some(ParameterValue::Integer(val)) => Ok(*val as f64),
            Some(_) => Err(StrategyError::InvalidParameter(format!(
                "Parameter '{}' is not a number",
                key
            ))),
            None => Err(StrategyError::MissingData(format!(
                "Parameter '{}' not found",
                key
            ))),
        }
    }

    /// Get a string parameter
    pub fn get_string(&self, key: &str) -> Result<&str, StrategyError> {
        match self.get_parameter(key) {
            Some(ParameterValue::String(val)) => Ok(val),
            Some(_) => Err(StrategyError::InvalidParameter(format!(
                "Parameter '{}' is not a string",
                key
            ))),
            None => Err(StrategyError::MissingData(format!(
                "Parameter '{}' not found",
                key
            ))),
        }
    }

    /// Get a boolean parameter
    pub fn get_bool(&self, key: &str) -> Result<bool, StrategyError> {
        match self.get_parameter(key) {
            Some(ParameterValue::Boolean(val)) => Ok(*val),
            Some(_) => Err(StrategyError::InvalidParameter(format!(
                "Parameter '{}' is not a boolean",
                key
            ))),
            None => Err(StrategyError::MissingData(format!(
                "Parameter '{}' not found",
                key
            ))),
        }
    }

    /// Validate the configuration against required parameters
    pub fn validate(&self, required_params: &[&str]) -> Result<(), StrategyError> {
        for &param in required_params {
            if !self.parameters.contains_key(param) {
                return Err(StrategyError::ValidationError(format!(
                    "Required parameter '{}' is missing",
                    param
                )));
            }
        }
        Ok(())
    }
}

/// Parameter value types for type-safe configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParameterValue {
    Integer(i64),
    Float(f64),
    String(String),
    Boolean(bool),
    Array(Vec<ParameterValue>),
}

impl From<i64> for ParameterValue {
    fn from(value: i64) -> Self {
        ParameterValue::Integer(value)
    }
}

impl From<f64> for ParameterValue {
    fn from(value: f64) -> Self {
        ParameterValue::Float(value)
    }
}

impl From<String> for ParameterValue {
    fn from(value: String) -> Self {
        ParameterValue::String(value)
    }
}

impl From<&str> for ParameterValue {
    fn from(value: &str) -> Self {
        ParameterValue::String(value.to_string())
    }
}

impl From<bool> for ParameterValue {
    fn from(value: bool) -> Self {
        ParameterValue::Boolean(value)
    }
}

impl From<usize> for ParameterValue {
    fn from(value: usize) -> Self {
        ParameterValue::Integer(value as i64)
    }
}

/// Risk management settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskSettings {
    /// Maximum position size as a fraction of capital
    pub max_position_size: f64,
    /// Stop loss percentage
    pub stop_loss: Option<f64>,
    /// Take profit percentage
    pub take_profit: Option<f64>,
    /// Maximum number of concurrent positions
    pub max_positions: Option<usize>,
}

impl Default for RiskSettings {
    fn default() -> Self {
        Self {
            max_position_size: 0.1,  // 10% of capital
            stop_loss: Some(0.05),   // 5% stop loss
            take_profit: Some(0.15), // 15% take profit
            max_positions: Some(1),  // Single position
        }
    }
}

/// Performance optimization settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationSettings {
    /// Use parallel processing when available
    pub use_parallel: bool,
    /// Cache intermediate calculations
    pub use_cache: bool,
    /// Batch size for processing
    pub batch_size: Option<usize>,
}

impl Default for OptimizationSettings {
    fn default() -> Self {
        Self {
            use_parallel: true,
            use_cache: true,
            batch_size: Some(1000),
        }
    }
}

/// Enhanced trait defining the interface for all trading strategies
pub trait Strategy: Send + Sync {
    /// Initialize the strategy with given configuration
    fn new(config: StrategyConfig) -> Self
    where
        Self: Sized;

    /// Generate trading signals based on the provided data
    fn generate_signals(&self, data: &DataFrame) -> Result<Series, StrategyError>;

    /// Name of the strategy
    fn name(&self) -> &str;

    /// Description of the strategy
    fn description(&self) -> &str;

    /// Required data columns for this strategy
    fn required_columns(&self) -> Vec<&str>;

    /// Get the strategy configuration
    fn config(&self) -> &StrategyConfig;

    /// Validate input data before processing
    fn validate_data(&self, data: &DataFrame) -> Result<(), StrategyError> {
        // Check if all required columns are present
        for &col in self.required_columns().iter() {
            if data.column(col).is_err() {
                return Err(StrategyError::MissingData(format!(
                    "Required column '{}' not found",
                    col
                )));
            }
        }

        // Check if data has sufficient rows
        if data.height() < self.min_data_points() {
            return Err(StrategyError::ValidationError(format!(
                "Insufficient data: need at least {} rows, got {}",
                self.min_data_points(),
                data.height()
            )));
        }

        Ok(())
    }

    /// Minimum number of data points required for the strategy
    fn min_data_points(&self) -> usize {
        50 // Default minimum
    }

    /// Whether the strategy supports real-time processing
    fn supports_realtime(&self) -> bool {
        true
    }

    /// Get strategy metadata
    fn metadata(&self) -> StrategyMetadata {
        StrategyMetadata {
            name: self.name().to_string(),
            description: self.description().to_string(),
            required_columns: self
                .required_columns()
                .iter()
                .map(|s| s.to_string())
                .collect(),
            min_data_points: self.min_data_points(),
            supports_realtime: self.supports_realtime(),
            version: "1.0.0".to_string(),
        }
    }
}

/// Metadata about a strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyMetadata {
    pub name: String,
    pub description: String,
    pub required_columns: Vec<String>,
    pub min_data_points: usize,
    pub supports_realtime: bool,
    pub version: String,
}

/// Utility functions for strategy development
pub mod utils {
    use super::*;

    /// Convert signals series to integer series for easier processing
    pub fn signals_to_int_series(signals: &[Signal]) -> Series {
        let int_signals: Vec<i32> = signals.iter().map(|s| s.to_int()).collect();
        Series::new("signals".into(), int_signals)
    }

    /// Convert integer series back to signals
    pub fn int_series_to_signals(series: &Series) -> Result<Vec<Signal>, StrategyError> {
        let int_values = series.i32().map_err(|_| {
            StrategyError::InvalidParameter("Cannot convert series to integers".to_string())
        })?;

        Ok(int_values
            .into_iter()
            .map(|opt_val| Signal::from_int(opt_val.unwrap_or(0)))
            .collect())
    }

    /// Create a default configuration for common strategy parameters
    pub fn create_ma_config(fast_period: usize, slow_period: usize) -> StrategyConfig {
        StrategyConfig::new()
            .with_parameter("fast_period", fast_period)
            .with_parameter("slow_period", slow_period)
    }

    /// Create a default configuration for RSI strategy
    pub fn create_rsi_config(period: usize, oversold: f64, overbought: f64) -> StrategyConfig {
        StrategyConfig::new()
            .with_parameter("period", period)
            .with_parameter("oversold_threshold", oversold)
            .with_parameter("overbought_threshold", overbought)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signal_conversion() {
        assert_eq!(Signal::Buy.to_int(), 1);
        assert_eq!(Signal::Sell.to_int(), 2);
        assert_eq!(Signal::Hold.to_int(), 0);

        assert_eq!(Signal::from_int(1), Signal::Buy);
        assert_eq!(Signal::from_int(2), Signal::Sell);
        assert_eq!(Signal::from_int(0), Signal::Hold);
        assert_eq!(Signal::from_int(99), Signal::Hold); // Invalid values default to Hold
    }

    #[test]
    fn test_strategy_config() {
        let config = StrategyConfig::new()
            .with_parameter("period", 20_i64)
            .with_parameter("threshold", 0.5)
            .with_parameter("enabled", true)
            .with_parameter("name", "test_strategy");

        assert_eq!(config.get_int("period").unwrap(), 20);
        assert_eq!(config.get_float("threshold").unwrap(), 0.5);
        assert_eq!(config.get_bool("enabled").unwrap(), true);
        assert_eq!(config.get_string("name").unwrap(), "test_strategy");
    }

    #[test]
    fn test_config_validation() {
        let config = StrategyConfig::new().with_parameter("period", 20_i64);

        // Should pass validation for existing parameter
        assert!(config.validate(&["period"]).is_ok());

        // Should fail validation for missing parameter
        assert!(config.validate(&["period", "missing_param"]).is_err());
    }
}
