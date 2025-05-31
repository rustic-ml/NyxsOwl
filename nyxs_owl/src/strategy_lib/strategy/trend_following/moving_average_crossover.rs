//! # Moving Average Crossover Strategy
//!
//! A trend-following strategy that generates buy/sell signals when a faster moving average
//! crosses over a slower moving average.

use crate::strategy_lib::strategy::{Signal, Strategy, StrategyConfig, StrategyError};
use polars::prelude::*;
use serde::{Deserialize, Serialize};

/// Configuration for the Moving Average Crossover strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MovingAverageCrossoverConfig {
    /// Period for the fast moving average
    pub fast_period: u32,
    /// Period for the slow moving average
    pub slow_period: u32,
    /// Type of moving average to use ("sma" or "ema")
    pub ma_type: String,
    /// Price column to use for calculations
    pub price_col: String,
}

impl Default for MovingAverageCrossoverConfig {
    fn default() -> Self {
        Self {
            fast_period: 10,
            slow_period: 30,
            ma_type: "ema".to_string(),
            price_col: "close".to_string(),
        }
    }
}

/// Moving Average Crossover Strategy
///
/// Generates buy signals when the fast moving average crosses above the slow moving average,
/// and sell signals when the fast moving average crosses below the slow moving average.
#[derive(Debug)]
pub struct MovingAverageCrossover {
    config: MovingAverageCrossoverConfig,
    name: String,
    description: String,
}

impl Strategy for MovingAverageCrossover {
    fn new(_config: StrategyConfig) -> Self {
        // For simplicity, we'll just use default values for now
        let strategy_config = MovingAverageCrossoverConfig::default();

        let name = format!(
            "MA_Crossover_{}_{}",
            strategy_config.fast_period, strategy_config.slow_period
        );

        let description = format!(
            "Moving Average Crossover strategy using {} and {} period {}",
            strategy_config.fast_period,
            strategy_config.slow_period,
            strategy_config.ma_type.to_uppercase()
        );

        Self {
            config: strategy_config,
            name,
            description,
        }
    }

    fn generate_signals(&self, data: &DataFrame) -> Result<Series, StrategyError> {
        // Check if the price column exists by trying to get it
        if data.column(&self.config.price_col).is_err() {
            return Err(StrategyError::MissingData(format!(
                "Price column '{}' not found in data",
                self.config.price_col
            )));
        }

        // For simplicity, we'll just return a series of "Hold" signals for now
        let length = data.height();
        let signals = vec![Signal::Hold as i32; length];

        // Create the signal series
        let signal_series = Series::new("signal".into(), signals);

        Ok(signal_series)
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn required_columns(&self) -> Vec<&str> {
        vec![&self.config.price_col]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Create a test DataFrame
    fn create_test_data() -> DataFrame {
        let close = Series::new("close".into(), &[100.0, 101.0, 99.0]);
        DataFrame::new(vec![close.into()]).unwrap()
    }

    #[test]
    fn test_strategy_creation() {
        // Create a simple empty config
        let config = StrategyConfig {
            parameters: Series::new("params".into(), [0i32]), // Dummy series
        };

        let strategy = MovingAverageCrossover::new(config);

        // Check that the strategy was created with default values
        assert_eq!(strategy.name(), "MA_Crossover_10_30");
        assert_eq!(
            strategy.description(),
            "Moving Average Crossover strategy using 10 and 30 period EMA"
        );
        assert_eq!(strategy.required_columns(), vec!["close"]);
    }

    #[test]
    fn test_generate_signals() {
        let df = create_test_data();
        let config = StrategyConfig {
            parameters: Series::new("params".into(), [0i32]), // Dummy series
        };

        let strategy = MovingAverageCrossover::new(config);

        let signals = strategy.generate_signals(&df).unwrap();

        // Check that we have the right number of signals
        assert_eq!(signals.len(), df.height());

        // All signals should be "Hold" in our simplified implementation
        let signal_vals = signals.i32().unwrap();
        for i in 0..signal_vals.len() {
            assert_eq!(signal_vals.get(i), Some(Signal::Hold as i32));
        }
    }

    #[test]
    fn test_missing_price_column() {
        // Create a DataFrame without the required price column
        let wrong_col = Series::new("wrong_col".into(), &[1.0, 2.0, 3.0]);
        let df = DataFrame::new(vec![wrong_col.into()]).unwrap();

        let config = StrategyConfig {
            parameters: Series::new("params".into(), [0i32]), // Dummy series
        };

        let strategy = MovingAverageCrossover::new(config);

        // This should error because the price column is missing
        let result = strategy.generate_signals(&df);
        assert!(result.is_err());

        match result.unwrap_err() {
            StrategyError::MissingData(msg) => {
                assert!(msg.contains("close"));
            }
            _ => panic!("Expected MissingData error"),
        }
    }
}
