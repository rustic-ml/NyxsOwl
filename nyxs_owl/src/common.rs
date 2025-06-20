//! Common types, traits, and utilities used throughout NyxsOwl
//!
//! This module provides the foundational types and error handling
//! that are shared across all components of the trading library.

use chrono::{DateTime, Utc};
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

    /// Configuration error
    ConfigurationError(String),

    /// Data validation error
    DataValidationError(String),

    /// Indicator calculation error
    IndicatorError(String),

    /// Signal generation error
    SignalError(String),

    /// State management error
    StateError(String),

    /// Strategy initialization error
    InitializationError(String),

    /// Strategy execution error
    ExecutionError(String),

    /// Insufficient data error
    InsufficientData { required: usize, available: usize },

    /// Serialization error
    SerializationError(String),

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
            NyxsOwlError::ConfigurationError(msg) => write!(f, "Configuration Error: {}", msg),
            NyxsOwlError::DataValidationError(msg) => write!(f, "Data Validation Error: {}", msg),
            NyxsOwlError::IndicatorError(msg) => write!(f, "Indicator Error: {}", msg),
            NyxsOwlError::SignalError(msg) => write!(f, "Signal Error: {}", msg),
            NyxsOwlError::StateError(msg) => write!(f, "State Error: {}", msg),
            NyxsOwlError::InitializationError(msg) => write!(f, "Initialization Error: {}", msg),
            NyxsOwlError::ExecutionError(msg) => write!(f, "Execution Error: {}", msg),
            NyxsOwlError::InsufficientData {
                required,
                available,
            } => write!(
                f,
                "Insufficient Data: required {}, available {}",
                required, available
            ),
            NyxsOwlError::SerializationError(msg) => write!(f, "Serialization Error: {}", msg),
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

impl From<serde_json::Error> for NyxsOwlError {
    fn from(error: serde_json::Error) -> Self {
        NyxsOwlError::SerializationError(error.to_string())
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

/// NyxsOwl signal types for advanced strategies
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NyxsOwlSignal {
    /// Signal timestamp
    pub timestamp: DateTime<Utc>,
    /// Signal type
    pub signal_type: NyxsOwlSignalType,
    /// Signal strength (0.0 to 1.0)
    pub strength: f64,
    /// Confidence level (0.0 to 1.0)
    pub confidence: f64,
    /// Target price (if applicable)
    pub target_price: Option<f64>,
    /// Stop loss price (if applicable)
    pub stop_loss: Option<f64>,
    /// Take profit price (if applicable)
    pub take_profit: Option<f64>,
    /// Signal metadata
    pub metadata: HashMap<String, serde_json::Value>,
    /// Strategy that generated the signal
    pub strategy_name: String,
    /// Signal expiration time
    pub expires_at: Option<DateTime<Utc>>,
}

/// NyxsOwl signal types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum NyxsOwlSignalType {
    /// Buy signal
    Buy,
    /// Sell signal
    Sell,
    /// Hold signal
    Hold,
    /// Close position signal
    Close,
    /// Reduce position signal
    Reduce(f64),
    /// Increase position signal
    Increase(f64),
}

/// Strategy category enumeration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NyxsOwlStrategyCategory {
    /// Trend following strategies
    TrendFollowing,
    /// Mean reversion strategies
    MeanReversion,
    /// Momentum-based strategies
    Momentum,
    /// Technical indicator strategies
    TechnicalIndicators,
    /// Forecasting model strategies
    ForecastingModels,
    /// Statistical arbitrage strategies
    StatisticalArbitrage,
    /// Machine learning strategies
    MachineLearning,
    /// Ensemble strategies
    Ensemble,
    /// Custom user-defined strategies
    Custom(String),
}

/// Strategy complexity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StrategyComplexity {
    /// Simple strategies with basic logic
    Simple,
    /// Moderate complexity with multiple conditions
    Moderate,
    /// Complex strategies with advanced logic
    Complex,
    /// Advanced strategies with ML/AI components
    Advanced,
}

/// Market condition types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MarketCondition {
    /// Trending markets
    Trending,
    /// Sideways/ranging markets
    Sideways,
    /// High volatility markets
    Volatile,
    /// Low volatility markets
    LowVolatility,
    /// Bull markets
    BullMarket,
    /// Bear markets
    BearMarket,
    /// All market conditions
    AllConditions,
}

/// Expected performance characteristics
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExpectedPerformance {
    /// Typical win rate (0.0 to 1.0)
    pub typical_win_rate: f64,
    /// Typical Sharpe ratio
    pub typical_sharpe_ratio: f64,
    /// Expected maximum drawdown (0.0 to 1.0)
    pub max_drawdown_expectation: f64,
    /// Expected volatility range (min, max)
    pub volatility_range: (f64, f64),
    /// Expected annual return range (min, max)
    pub annual_return_range: (f64, f64),
}

/// Computational cost estimation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComputationalCost {
    /// Low cost (< 1ms per data point)
    Low,
    /// Medium cost (1-10ms per data point)
    Medium,
    /// High cost (10-100ms per data point)
    High,
    /// Very high cost (> 100ms per data point)
    VeryHigh,
}

/// Strategy parameter definition
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StrategyParameter {
    /// Parameter name
    pub name: String,
    /// Parameter description
    pub description: String,
    /// Parameter type
    pub param_type: ParameterType,
    /// Default value
    pub default_value: ParameterValue,
    /// Valid range (for numeric parameters)
    pub range: Option<(f64, f64)>,
    /// Valid options (for enum parameters)
    pub options: Option<Vec<String>>,
    /// Whether parameter is required
    pub required: bool,
}

/// Parameter type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ParameterType {
    /// Integer parameter
    Integer,
    /// Float parameter
    Float,
    /// String parameter
    String,
    /// Boolean parameter
    Boolean,
    /// Enum parameter with predefined options
    Enum,
}

/// Parameter value enumeration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ParameterValue {
    /// Integer value
    Integer(i64),
    /// Float value
    Float(f64),
    /// String value
    String(String),
    /// Boolean value
    Boolean(bool),
}

/// Risk profile definition
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RiskProfile {
    /// Risk level (1-10 scale)
    pub risk_level: u8,
    /// Maximum position size as percentage of portfolio
    pub max_position_size: f64,
    /// Maximum drawdown tolerance
    pub max_drawdown_tolerance: f64,
    /// Volatility tolerance
    pub volatility_tolerance: f64,
    /// Correlation with market
    pub market_correlation: f64,
}

/// Strategy metadata structure
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NyxsOwlStrategyMetadata {
    /// Strategy name
    pub name: String,
    /// Strategy description
    pub description: String,
    /// Strategy category
    pub category: NyxsOwlStrategyCategory,
    /// Strategy version
    pub version: String,
    /// Strategy author
    pub author: String,
    /// Strategy complexity level
    pub complexity: StrategyComplexity,
    /// Suitable timeframes
    pub timeframe_suitability: Vec<String>,
    /// Market conditions where strategy performs well
    pub market_conditions: Vec<MarketCondition>,
    /// Expected performance characteristics
    pub expected_performance: ExpectedPerformance,
    /// Computational cost estimation
    pub computational_cost: ComputationalCost,
    /// Required technical indicators
    pub required_indicators: Vec<String>,
    /// Strategy parameters
    pub parameters: Vec<StrategyParameter>,
    /// Risk characteristics
    pub risk_profile: RiskProfile,
}

/// Position sizing methods
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PositionSizingMethod {
    /// Fixed dollar amount
    FixedAmount(f64),
    /// Fixed percentage of portfolio
    FixedPercentage(f64),
    /// Kelly criterion
    Kelly,
    /// Risk parity
    RiskParity,
    /// Volatility targeting
    VolatilityTargeting(f64),
}

/// Risk management parameters
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RiskParameters {
    /// Maximum position size
    pub max_position_size: f64,
    /// Stop loss percentage
    pub stop_loss: Option<f64>,
    /// Take profit percentage
    pub take_profit: Option<f64>,
    /// Maximum drawdown before stopping
    pub max_drawdown: f64,
    /// Position sizing method
    pub position_sizing: PositionSizingMethod,
}

/// Rebalancing frequency options
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RebalancingFrequency {
    /// Never rebalance
    Never,
    /// Daily rebalancing
    Daily,
    /// Weekly rebalancing
    Weekly,
    /// Monthly rebalancing
    Monthly,
    /// Custom frequency in days
    Custom(u32),
}

/// Execution parameters
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExecutionParameters {
    /// Minimum signal strength to act on
    pub min_signal_strength: f64,
    /// Signal confirmation requirements
    pub confirmation_required: bool,
    /// Maximum holding period
    pub max_holding_period: Option<chrono::Duration>,
    /// Rebalancing frequency
    pub rebalancing_frequency: RebalancingFrequency,
}

/// Data frequency options
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DataFrequency {
    /// Tick data
    Tick,
    /// 1-minute bars
    Minute1,
    /// 5-minute bars
    Minute5,
    /// 15-minute bars
    Minute15,
    /// 1-hour bars
    Hour1,
    /// Daily bars
    Daily,
    /// Weekly bars
    Weekly,
    /// Monthly bars
    Monthly,
}

/// Data requirements specification
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DataRequirements {
    /// Minimum data points required
    pub min_data_points: usize,
    /// Required data frequency
    pub required_frequency: DataFrequency,
    /// Required indicators
    pub required_indicators: Vec<String>,
    /// Lookback period in days
    pub lookback_period: u32,
}

/// NyxsOwl strategy parameters
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NyxsOwlParameters {
    /// Strategy-specific parameters
    pub parameters: HashMap<String, ParameterValue>,
    /// Risk management parameters
    pub risk_params: RiskParameters,
    /// Execution parameters
    pub execution_params: ExecutionParameters,
}

/// Strategy state management
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NyxsOwlStrategyState {
    /// Whether strategy is initialized
    pub initialized: bool,
    /// Current state data
    pub state_data: HashMap<String, serde_json::Value>,
    /// Last update timestamp
    pub last_update: DateTime<Utc>,
    /// Number of signals generated
    pub signals_generated: u64,
    /// Strategy performance since initialization
    pub performance: NyxsOwlPerformanceMetrics,
}

/// NyxsOwl performance metrics
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NyxsOwlPerformanceMetrics {
    /// Total return
    pub total_return: f64,
    /// Annualized return
    pub annualized_return: f64,
    /// Sharpe ratio
    pub sharpe_ratio: f64,
    /// Sortino ratio
    pub sortino_ratio: f64,
    /// Maximum drawdown
    pub max_drawdown: f64,
    /// Win rate
    pub win_rate: f64,
    /// Profit factor
    pub profit_factor: f64,
    /// Total trades
    pub total_trades: u64,
    /// Average trade return
    pub avg_trade_return: f64,
    /// Volatility
    pub volatility: f64,
    /// Calmar ratio
    pub calmar_ratio: f64,
    /// Information ratio
    pub information_ratio: f64,
}

/// NyxsOwl configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NyxsOwlConfig {
    /// Enabled strategies
    pub enabled_strategies: Vec<String>,
    /// Strategy parameters
    pub strategy_parameters: HashMap<String, NyxsOwlParameters>,
    /// Global risk settings
    pub global_risk_settings: RiskParameters,
    /// Execution settings
    pub execution_settings: ExecutionParameters,
    /// Data requirements
    pub data_requirements: DataRequirements,
}

/// OHLCV (Open, High, Low, Close, Volume) candlestick data
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OHLCV {
    /// Timestamp of the candlestick
    pub timestamp: DateTime<Utc>,
    /// Opening price
    pub open: f64,
    /// Highest price during the period
    pub high: f64,
    /// Lowest price during the period
    pub low: f64,
    /// Closing price
    pub close: f64,
    /// Volume traded during the period
    pub volume: f64,
}

impl OHLCV {
    /// Create a new OHLCV candlestick
    pub fn new(
        timestamp: DateTime<Utc>,
        open: f64,
        high: f64,
        low: f64,
        close: f64,
        volume: f64,
    ) -> Self {
        Self {
            timestamp,
            open,
            high,
            low,
            close,
            volume,
        }
    }

    /// Get the typical price (HLC/3)
    pub fn typical_price(&self) -> f64 {
        (self.high + self.low + self.close) / 3.0
    }
}

/// Core NyxsOwl strategy trait
pub trait NyxsOwlStrategy: Send + Sync {
    /// Get the strategy name
    fn name(&self) -> &str;

    /// Get the strategy version
    fn version(&self) -> &str;

    /// Get the strategy category
    fn category(&self) -> NyxsOwlStrategyCategory;

    /// Get strategy metadata
    fn metadata(&self) -> NyxsOwlStrategyMetadata;

    /// Initialize the strategy with parameters
    fn initialize(&mut self, parameters: &NyxsOwlParameters) -> Result<(), NyxsOwlError>;

    /// Process market data and generate signals
    fn process(&mut self, data: &[OHLCV]) -> Result<Vec<NyxsOwlSignal>, NyxsOwlError>;

    /// Update strategy state with new market data
    fn update(&mut self, data: &OHLCV) -> Result<Option<NyxsOwlSignal>, NyxsOwlError>;

    /// Get current strategy state
    fn state(&self) -> NyxsOwlStrategyState;

    /// Reset strategy to initial state
    fn reset(&mut self) -> Result<(), NyxsOwlError>;

    /// Validate strategy configuration
    fn validate_config(&self, config: &NyxsOwlConfig) -> Result<(), NyxsOwlError>;

    /// Get required indicators for this strategy
    fn required_indicators(&self) -> Vec<String>;

    /// Get strategy performance metrics
    fn performance_metrics(&self) -> NyxsOwlPerformanceMetrics;

    /// Clone the strategy (for parallel execution)
    fn clone_strategy(&self) -> Box<dyn NyxsOwlStrategy>;
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
            crate::simple_types::NyxsOwlError::ValidationError(format!(
                "Parameter '{}' not found or not an integer",
                key
            ))
        })
    }

    /// Get a float parameter (compatible with forecasting interface)
    pub fn get_float_compat(&self, key: &str) -> Result<f64, crate::simple_types::NyxsOwlError> {
        self.get_float(key).ok_or_else(|| {
            crate::simple_types::NyxsOwlError::ValidationError(format!(
                "Parameter '{}' not found or not a float",
                key
            ))
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

impl Default for NyxsOwlPerformanceMetrics {
    fn default() -> Self {
        Self {
            total_return: 0.0,
            annualized_return: 0.0,
            sharpe_ratio: 0.0,
            sortino_ratio: 0.0,
            max_drawdown: 0.0,
            win_rate: 0.0,
            profit_factor: 0.0,
            total_trades: 0,
            avg_trade_return: 0.0,
            volatility: 0.0,
            calmar_ratio: 0.0,
            information_ratio: 0.0,
        }
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

        let float_val = ConfigValue::from(std::f64::consts::PI);
        assert_eq!(float_val.as_float(), Some(std::f64::consts::PI));

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
