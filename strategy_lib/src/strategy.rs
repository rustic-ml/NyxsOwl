//! # Trading Strategies
//! 
//! This module provides various trading strategies based on technical indicators.

use polars::prelude::*;
use serde::{Serialize, Deserialize};
use thiserror::Error;

pub mod trend_following;
pub mod mean_reversion;
pub mod momentum;
pub mod volatility;
pub mod volume;
pub mod multi_indicator;

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
}

/// Strategy signals representing trade actions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Signal {
    Buy,
    Sell,
    Hold,
}

/// Configuration for strategy parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyConfig {
    pub parameters: Series,
}

/// Trait defining the interface for all trading strategies
pub trait Strategy {
    /// Initialize the strategy with given configuration
    fn new(config: StrategyConfig) -> Self where Self: Sized;
    
    /// Generate trading signals based on the provided data
    fn generate_signals(&self, data: &DataFrame) -> Result<Series, StrategyError>;
    
    /// Name of the strategy
    fn name(&self) -> &str;
    
    /// Description of the strategy
    fn description(&self) -> &str;
    
    /// Required data columns for this strategy
    fn required_columns(&self) -> Vec<&str>;
} 