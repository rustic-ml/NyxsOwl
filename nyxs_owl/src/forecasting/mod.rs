// nyxs_owl/src/forecasting/mod.rs
//! Forecasting module: Defines common traits, types, and errors for forecasting strategies.
//! It also declares available forecasting model submodules like ARIMA.

use polars::prelude::{DataFrame, Series};
use std::collections::HashMap;
// use thiserror::Error; // NyxsOwlError is already derived with thiserror
use log::{error, warn};

// Import common types from crate root
use crate::simple_types::{NyxsOwlError, Result as NyxsOwlResult}; // Assuming Result is NyxsOwlError based

// Declare submodules within the forecasting module
pub mod arima;

/// Trading strategy integration with forecasting
pub mod forecast_trade;

/// Backtest module for strategy evaluation
pub mod backtest;

/// Strategy implementations for different forecasting approaches
pub mod strategies;

/// Utility functions for forecasting operations
pub mod utils;

// TODO: Add other forecasting strategy sub-modules here as they are developed.
// e.g., pub mod exponential_smoothing;
// e.g., pub mod prophet_model;

// Re-exports for easier access
pub use crate::forecasting::strategies::*;

/// Configuration value types for flexible parameter handling
#[derive(Debug, Clone, PartialEq)]
pub enum ConfigValue {
    /// Integer configuration value
    Int(i64),
    /// Floating-point configuration value
    Float(f64),
    /// String configuration value
    String(String),
    /// Boolean configuration value
    Bool(bool),
}

// From implementations for ConfigValue for easier parameter setting
impl From<i32> for ConfigValue {
    fn from(i: i32) -> Self {
        ConfigValue::Int(i as i64)
    }
}
impl From<i64> for ConfigValue {
    fn from(i: i64) -> Self {
        ConfigValue::Int(i)
    }
}
impl From<f64> for ConfigValue {
    fn from(f: f64) -> Self {
        ConfigValue::Float(f)
    }
}
impl From<String> for ConfigValue {
    fn from(s: String) -> Self {
        ConfigValue::String(s)
    }
}
impl From<&str> for ConfigValue {
    fn from(s: &str) -> Self {
        ConfigValue::String(s.to_string())
    }
}
impl From<bool> for ConfigValue {
    fn from(b: bool) -> Self {
        ConfigValue::Bool(b)
    }
}

/// Holds configuration parameters for a trading strategy.
///
/// Parameters are stored in a `HashMap` and can be of various types
/// (`Int`, `Float`, `String`, `Bool`) via the `ConfigValue` enum.
#[derive(Debug, Clone, Default)]
pub struct StrategyConfig {
    parameters: HashMap<String, ConfigValue>,
}

impl StrategyConfig {
    /// Creates a new, empty `StrategyConfig`.
    pub fn new() -> Self {
        StrategyConfig {
            parameters: HashMap::new(),
        }
    }

    /// Adds or updates a parameter in the configuration.
    ///
    /// # Examples
    /// ```
    /// # use nyxs_owl::forecasting::{StrategyConfig, ConfigValue}; // Assuming this path
    /// let config = StrategyConfig::new()
    ///     .with_parameter("lookback_period", 30)
    ///     .with_parameter("risk_percentage", 0.01)
    ///     .with_parameter("asset_name", "BTC/USD");
    /// ```
    pub fn with_parameter<K: Into<String>, V: Into<ConfigValue>>(
        mut self,
        key: K,
        value: V,
    ) -> Self {
        self.parameters.insert(key.into(), value.into());
        self
    }

    /// Retrieves an integer parameter.
    pub fn get_int(&self, key: &str) -> NyxsOwlResult<i64> {
        match self.parameters.get(key) {
            Some(ConfigValue::Int(val)) => Ok(*val),
            Some(other_type) => Err(NyxsOwlError::StrategyError(format!(
                "Parameter '{}' is not an integer. Found: {:?}",
                key, other_type
            ))),
            None => Err(NyxsOwlError::StrategyError(format!(
                "Required integer parameter '{}' not found.",
                key
            ))),
        }
    }

    /// Retrieves a float parameter. Allows integers to be cast to floats.
    pub fn get_float(&self, key: &str) -> NyxsOwlResult<f64> {
        match self.parameters.get(key) {
            Some(ConfigValue::Float(val)) => Ok(*val),
            Some(ConfigValue::Int(val)) => Ok(*val as f64), // Allow int to be coerced to float
            Some(other_type) => Err(NyxsOwlError::StrategyError(format!(
                "Parameter '{}' is not a float. Found: {:?}",
                key, other_type
            ))),
            None => Err(NyxsOwlError::StrategyError(format!(
                "Required float parameter '{}' not found.",
                key
            ))),
        }
    }

    /// Retrieves a string parameter.
    pub fn get_string(&self, key: &str) -> NyxsOwlResult<&str> {
        match self.parameters.get(key) {
            Some(ConfigValue::String(val)) => Ok(val.as_str()),
            Some(other_type) => Err(NyxsOwlError::StrategyError(format!(
                "Parameter '{}' is not a string. Found: {:?}",
                key, other_type
            ))),
            None => Err(NyxsOwlError::StrategyError(format!(
                "Required string parameter '{}' not found.",
                key
            ))),
        }
    }

    /// Retrieves a boolean parameter.
    #[allow(dead_code)] // Keep for completeness
    pub fn get_bool(&self, key: &str) -> NyxsOwlResult<bool> {
        match self.parameters.get(key) {
            Some(ConfigValue::Bool(val)) => Ok(*val),
            Some(other_type) => Err(NyxsOwlError::StrategyError(format!(
                "Parameter '{}' is not a boolean. Found: {:?}",
                key, other_type
            ))),
            None => Err(NyxsOwlError::StrategyError(format!(
                "Required boolean parameter '{}' not found.",
                key
            ))),
        }
    }

    /// Validates that all specified keys are present in the configuration.
    ///
    /// Returns `Ok(())` if all keys are present, otherwise `Err` with a message
    /// listing the missing keys.
    pub fn validate(&self, required_keys: &[&str]) -> Result<(), String> {
        // This Result is std::result::Result
        let missing_keys: Vec<String> = required_keys
            .iter()
            .filter(|&&key| !self.parameters.contains_key(key))
            .map(|&key| key.to_string())
            .collect();

        if missing_keys.is_empty() {
            Ok(())
        } else {
            Err(format!(
                "Missing required config keys: {}",
                missing_keys.join(", ")
            ))
        }
    }
}

/// Trait defining the interface for trading strategies.
///
/// All trading strategies must implement this trait to be used within the
/// NyxsOwl framework. The trait provides methods for configuration, signal
/// generation, and data validation.
pub trait Strategy: Send + Sync {
    /// Creates a new instance of the strategy with the given configuration.
    ///
    /// This typically involves parsing and validating parameters from `config`.
    fn new(config: StrategyConfig) -> Self
    where
        Self: Sized;

    /// Generates trading signals based on the provided market data.
    ///
    /// # Arguments
    /// * `data`: A Polars `DataFrame` containing historical market data.
    ///           The specific columns required depend on the strategy.
    ///
    /// # Returns
    /// A `Result` containing a Polars `Series` of signals (integers representing
    /// Buy, Sell, or Hold), or a `NyxsOwlError` if signal generation fails.
    fn generate_signals(&self, data: &DataFrame) -> NyxsOwlResult<Series>;

    /// Returns the name of the strategy.
    fn name(&self) -> &str;

    /// Provides a brief description of the strategy.
    fn description(&self) -> &str;

    /// Lists the names of data columns required by the strategy (e.g., "close", "volume").
    fn required_columns(&self) -> Vec<&str>;

    /// Returns a reference to the strategy's configuration.
    fn config(&self) -> &StrategyConfig;

    /// Specifies the minimum number of data points (rows) required by the strategy
    /// to produce meaningful signals. This is used in `validate_data`.
    fn min_data_points(&self) -> usize;

    /// Validates the input data against the strategy's requirements.
    ///
    /// The default implementation checks for:
    /// 1. Presence of all columns specified by `required_columns()`.
    /// 2. Sufficient number of data rows as per `min_data_points()`.
    ///
    /// Strategies can override this for more specific validation logic.
    fn validate_data(&self, data: &DataFrame) -> NyxsOwlResult<()> {
        // Check for required columns
        for col_name in self.required_columns() {
            if data.column(col_name).is_err() {
                let err_msg = format!("Column '{}' not found", col_name);
                error!("Strategy Validation Error (MissingData): {}", err_msg);
                return Err(NyxsOwlError::MissingData(err_msg));
            }
        }
        // Check for minimum data points
        if data.height() < self.min_data_points() {
            let err_msg = format!(
                "Insufficient data: {} requires at least {} data points, but got {}",
                self.name(),
                self.min_data_points(),
                data.height()
            );
            warn!(
                "Strategy Validation Warning (Insufficient Data): {}",
                err_msg
            );
            return Err(NyxsOwlError::StrategyError(err_msg));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use polars::prelude::*;

    #[test]
    fn strategy_config_creation_and_getters() -> NyxsOwlResult<()> {
        let config = StrategyConfig::new()
            .with_parameter("lookback_period", 30)
            .with_parameter("risk_percentage", 0.01)
            .with_parameter("asset_name", "BTC/USD")
            .with_parameter("enable_feature", true);

        assert_eq!(config.get_int("lookback_period")?, 30);
        assert_eq!(config.get_float("risk_percentage")?, 0.01);
        assert_eq!(config.get_string("asset_name")?, "BTC/USD");
        assert!(config.get_bool("enable_feature")?);

        // Test int to float coercion
        assert_eq!(config.get_float("lookback_period")?, 30.0);

        Ok(())
    }

    #[test]
    fn strategy_config_validation_logic() {
        let config = StrategyConfig::new()
            .with_parameter("param1", 10)
            .with_parameter("param2", "value");

        assert!(config.validate(&["param1", "param2"]).is_ok());
        assert!(config.validate(&["param1", "param2", "param3"]).is_err());
    }

    // Mock strategy for testing the trait
    struct MockTestStrategy {
        config: StrategyConfig,
        min_points: usize,
        req_cols: Vec<&'static str>,
    }

    impl Strategy for MockTestStrategy {
        fn new(config: StrategyConfig) -> Self {
            MockTestStrategy {
                config,
                min_points: 10,
                req_cols: vec!["close", "volume"],
            }
        }

        fn generate_signals(&self, _data: &DataFrame) -> NyxsOwlResult<Series> {
            Ok(Series::new("signals".into(), vec![0i32; 10]))
        }
        fn name(&self) -> &str {
            "MockStrategy"
        }
        fn description(&self) -> &str {
            "A mock strategy for testing"
        }
        fn required_columns(&self) -> Vec<&str> {
            self.req_cols.clone()
        }
        fn config(&self) -> &StrategyConfig {
            &self.config
        }
        fn min_data_points(&self) -> usize {
            self.min_points
        }
    }

    #[test]
    fn default_validate_data_success() -> NyxsOwlResult<()> {
        let config = StrategyConfig::new();
        let strategy = MockTestStrategy::new(config);

        let df = df! {
            "close" => [100.0, 101.0, 102.0, 103.0, 104.0, 105.0, 106.0, 107.0, 108.0, 109.0, 110.0],
            "volume" => [1000, 1100, 1200, 1300, 1400, 1500, 1600, 1700, 1800, 1900, 2000]
        }?;

        strategy.validate_data(&df)?;
        Ok(())
    }

    #[test]
    fn default_validate_data_missing_column() {
        let config = StrategyConfig::new();
        let strategy = MockTestStrategy::new(config);

        let df = df! {
            "close" => [100.0, 101.0, 102.0, 103.0, 104.0, 105.0, 106.0, 107.0, 108.0, 109.0, 110.0]
            // Missing "volume" column
        }
        .unwrap();

        let result = strategy.validate_data(&df);
        assert!(result.is_err());
        if let Err(NyxsOwlError::MissingData(msg)) = result {
            assert!(msg.contains("volume"));
        } else {
            panic!("Expected MissingData error");
        }
    }

    #[test]
    fn default_validate_data_insufficient_rows() {
        let config = StrategyConfig::new();
        let strategy = MockTestStrategy::new(config);

        let df = df! {
            "close" => [100.0, 101.0, 102.0], // Only 3 rows, but strategy requires 10
            "volume" => [1000, 1100, 1200]
        }
        .unwrap();

        let result = strategy.validate_data(&df);
        assert!(result.is_err());
        if let Err(NyxsOwlError::StrategyError(msg)) = result {
            assert!(msg.contains("Insufficient data"));
        } else {
            panic!("Expected StrategyError");
        }
    }
}
