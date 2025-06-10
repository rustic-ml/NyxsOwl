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
pub mod forecast_trade;

// Forecasting strategies submodules
pub mod backtest;
pub mod strategies;
pub mod utils;

// TODO: Add other forecasting strategy sub-modules here as they are developed.
// e.g., pub mod exponential_smoothing;
// e.g., pub mod prophet_model;

/// Represents a configuration value for a strategy parameter.
#[derive(Debug, Clone)]
pub enum ConfigValue {
    Int(i64),
    Float(f64),
    String(String),
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

/// Defines the common interface for all trading strategies.
///
/// Strategies must be `Send + Sync` to be usable across threads, for example,
/// in concurrent backtesting or live trading scenarios.
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
    use crate::simple_types::Signal as CrateSignal; // Use aliased import for clarity in tests

    // Tests for local Signal and StrategyError are removed as the types themselves are removed.
    // Tests for StrategyConfig might need adjustment if they relied on the local StrategyError.
    // Test for Signal to/from int conversion is removed. If this logic is needed,
    // it should be part of CrateSignal or handled by consumers.

    #[test]
    fn strategy_config_creation_and_getters() -> NyxsOwlResult<()> {
        let config = StrategyConfig::new()
            .with_parameter("period", 20)
            .with_parameter("factor", 2.5)
            .with_parameter("name", "Test Strategy".to_string())
            .with_parameter("enabled", true);

        assert_eq!(config.get_int("period")?, 20);
        assert_eq!(config.get_float("factor")?, 2.5);
        // Allow int to be retrieved as float
        assert_eq!(config.get_float("period")?, 20.0);
        assert_eq!(config.get_string("name")?, "Test Strategy");
        assert_eq!(config.get_bool("enabled")?, true);

        assert!(config.get_int("factor").is_err()); // Wrong type
        assert!(config.get_string("period").is_err()); // Wrong type
        assert!(config.get_bool("name").is_err()); // Wrong type
        assert!(config.get_float("enabled").is_err()); // Wrong type

        assert!(config.get_int("non_existent").is_err());
        Ok(())
    }

    #[test]
    fn strategy_config_validation_logic() {
        let config = StrategyConfig::new()
            .with_parameter("p1", 10)
            .with_parameter("p2", "value");

        assert_eq!(config.validate(&["p1", "p2"]), Ok(()));
        assert!(config.validate(&["p1", "p3"]).is_err());
        assert_eq!(
            config.validate(&["p1", "p3", "p4"]),
            Err("Missing required config keys: p3, p4".to_string())
        );
    }

    // Mock strategy for testing default validate_data
    struct MockTestStrategy {
        config: StrategyConfig,
        min_points: usize,
        req_cols: Vec<&'static str>,
    }

    impl Strategy for MockTestStrategy {
        fn new(config: StrategyConfig) -> Self {
            MockTestStrategy {
                min_points: config.get_int("min_points").unwrap_or(10) as usize,
                req_cols: vec!["close", "volume"], // Example
                config,
            }
        }
        fn generate_signals(&self, _data: &DataFrame) -> NyxsOwlResult<Series> {
            // Return a series of CrateSignal::Hold (as i32) for simplicity
            Ok(Series::from_iter(vec![CrateSignal::Hold as i32; 5]).with_name("signals".into()))
        }
        fn name(&self) -> &str {
            "MockTestStrategy"
        }
        fn description(&self) -> &str {
            "A mock strategy for testing."
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
        // Using default validate_data
    }

    #[test]
    fn default_validate_data_success() -> NyxsOwlResult<()> {
        let config = StrategyConfig::new().with_parameter("min_points", 5);
        let strategy = MockTestStrategy::new(config);
        let df = polars::prelude::df! {
            "close" => &[1.0, 2.0, 3.0, 4.0, 5.0],
            "volume" => &[100.0, 110.0, 120.0, 130.0, 140.0]
        }?;
        strategy.validate_data(&df)
    }

    #[test]
    fn default_validate_data_missing_column() {
        let config = StrategyConfig::new().with_parameter("min_points", 5);
        let strategy = MockTestStrategy::new(config);
        let df = polars::prelude::df! {
            "close" => &[1.0, 2.0, 3.0, 4.0, 5.0]
            // "volume" column is missing
        }
        .unwrap();

        match strategy.validate_data(&df) {
            Err(NyxsOwlError::MissingData(msg)) => {
                assert!(msg.contains("Column 'volume' not found"));
            }
            _ => panic!("Expected MissingData error"),
        }
    }

    #[test]
    fn default_validate_data_insufficient_rows() {
        let config = StrategyConfig::new().with_parameter("min_points", 10); // Requires 10 points
        let strategy = MockTestStrategy::new(config);
        let df = polars::prelude::df! {
            "close" => &[1.0, 2.0, 3.0],
            "volume" => &[100.0, 110.0, 120.0]
        }
        .unwrap();

        match strategy.validate_data(&df) {
            Err(NyxsOwlError::StrategyError(msg)) => {
                // Changed from ValidationError
                assert!(msg.contains("requires at least 10 data points, but got 3"));
            }
            _ => panic!("Expected StrategyError for insufficient rows"),
        }
    }
}
