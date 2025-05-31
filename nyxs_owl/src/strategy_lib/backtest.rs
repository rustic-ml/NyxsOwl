//! # Strategy Backtesting
//!
//! This module provides functionality for backtesting trading strategies.

use crate::strategy_lib::strategy::{Strategy, StrategyError};
use polars::prelude::*;

/// Configuration for backtest parameters
#[derive(Debug, Clone)]
pub struct BacktestConfig {
    /// Initial capital for the backtest
    pub initial_capital: f64,
    /// Commission per trade (fraction of trade value)
    pub commission: f64,
    /// Slippage per trade (fraction of price)
    pub slippage: f64,
    /// Position size as a fraction of capital
    pub position_size: f64,
}

impl Default for BacktestConfig {
    fn default() -> Self {
        Self {
            initial_capital: 10000.0,
            commission: 0.001,  // 0.1%
            slippage: 0.0005,   // 0.05%
            position_size: 0.1, // 10% of capital
        }
    }
}

/// Results from running a backtest
#[derive(Debug)]
pub struct BacktestResults {
    /// Final equity curve
    pub equity_curve: Series,
    /// Trade history
    pub trades: DataFrame,
    /// Performance metrics
    pub metrics: BacktestMetrics,
}

/// Performance metrics for a backtest
#[derive(Debug)]
pub struct BacktestMetrics {
    /// Total return percentage
    pub total_return: f64,
    /// Annualized return percentage
    pub annualized_return: f64,
    /// Maximum drawdown percentage
    pub max_drawdown: f64,
    /// Sharpe ratio
    pub sharpe_ratio: f64,
    /// Win rate (percentage of winning trades)
    pub win_rate: f64,
    /// Profit factor (gross profit / gross loss)
    pub profit_factor: f64,
}

/// Run a backtest for a given strategy on historical data
pub fn run_backtest<S: Strategy>(
    strategy: &S,
    data: &DataFrame,
    config: BacktestConfig,
) -> Result<BacktestResults, StrategyError> {
    // Verify the data contains required columns
    for &col in strategy.required_columns().iter() {
        // Try to get the column - if it fails, the column doesn't exist
        if data.column(col).is_err() {
            return Err(StrategyError::MissingData(format!(
                "Required column '{}' not found in data",
                col
            )));
        }
    }

    // Generate signals using the strategy
    // We need to generate signals even if we're not using them yet,
    // to make sure the strategy can process the data successfully
    let _signals = strategy.generate_signals(data)?;

    // This is a simplified implementation - a full backtest engine would include:
    // - Position sizing
    // - Order execution with slippage
    // - Portfolio management
    // - Detailed trade tracking
    // - Performance metric calculation

    // For now, we'll just return a placeholder
    let metrics = BacktestMetrics {
        total_return: 0.0,
        annualized_return: 0.0,
        max_drawdown: 0.0,
        sharpe_ratio: 0.0,
        win_rate: 0.0,
        profit_factor: 0.0,
    };

    let equity_curve = Series::new("equity".into(), vec![config.initial_capital]);

    // Create a trade DataFrame
    let trade_type = Series::new("type".into(), vec!["placeholder"]);
    let trade_price = Series::new("price".into(), vec![0.0]);
    let trades = DataFrame::new(vec![trade_type.into(), trade_price.into()]).unwrap();

    Ok(BacktestResults {
        equity_curve,
        trades,
        metrics,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::strategy_lib::strategy::{Signal, StrategyConfig};

    /// A simple mock strategy for testing
    struct MockStrategy {
        name: String,
        description: String,
        required_cols: Vec<String>,
        signals: Series,
    }

    impl Strategy for MockStrategy {
        fn new(_config: StrategyConfig) -> Self {
            Self {
                name: "Mock Strategy".to_string(),
                description: "A mock strategy for testing".to_string(),
                required_cols: vec!["close".to_string()],
                signals: Series::new(
                    "signal".into(),
                    vec![Signal::Hold as i32, Signal::Buy as i32, Signal::Sell as i32],
                ),
            }
        }

        fn generate_signals(&self, _data: &DataFrame) -> Result<Series, StrategyError> {
            Ok(self.signals.clone())
        }

        fn name(&self) -> &str {
            &self.name
        }

        fn description(&self) -> &str {
            &self.description
        }

        fn required_columns(&self) -> Vec<&str> {
            self.required_cols.iter().map(|s| s.as_str()).collect()
        }
    }

    /// Create a test DataFrame
    fn create_test_data() -> DataFrame {
        let close = Series::new("close".into(), &[100.0, 101.0, 99.0]);
        DataFrame::new(vec![close.into()]).unwrap()
    }

    #[test]
    fn test_backtest_config_default() {
        let config = BacktestConfig::default();

        assert_eq!(config.initial_capital, 10000.0);
        assert_eq!(config.commission, 0.001);
        assert_eq!(config.slippage, 0.0005);
        assert_eq!(config.position_size, 0.1);
    }

    #[test]
    fn test_run_backtest_with_mock_strategy() {
        let data = create_test_data();
        let mock_strategy = MockStrategy {
            name: "Mock Strategy".to_string(),
            description: "A mock strategy for testing".to_string(),
            required_cols: vec!["close".to_string()],
            signals: Series::new(
                "signal".into(),
                vec![Signal::Hold as i32, Signal::Buy as i32, Signal::Sell as i32],
            ),
        };

        let config = BacktestConfig::default();
        let results = run_backtest(&mock_strategy, &data, config).unwrap();

        // Check that we got back a valid results object
        assert_eq!(results.equity_curve.len(), 1);
        assert_eq!(results.trades.height(), 1);

        // Metrics should be the placeholder values
        assert_eq!(results.metrics.total_return, 0.0);
        assert_eq!(results.metrics.annualized_return, 0.0);
        assert_eq!(results.metrics.max_drawdown, 0.0);
        assert_eq!(results.metrics.sharpe_ratio, 0.0);
        assert_eq!(results.metrics.win_rate, 0.0);
        assert_eq!(results.metrics.profit_factor, 0.0);
    }

    #[test]
    fn test_run_backtest_missing_column() {
        // Create data without the required column
        let wrong_col = Series::new("wrong_col".into(), &[1.0, 2.0, 3.0]);
        let data = DataFrame::new(vec![wrong_col.into()]).unwrap();

        let mock_strategy = MockStrategy {
            name: "Mock Strategy".to_string(),
            description: "A mock strategy for testing".to_string(),
            required_cols: vec!["close".to_string()],
            signals: Series::new(
                "signal".into(),
                vec![Signal::Hold as i32, Signal::Buy as i32, Signal::Sell as i32],
            ),
        };

        let config = BacktestConfig::default();
        let result = run_backtest(&mock_strategy, &data, config);

        // Should error because the required column is missing
        assert!(result.is_err());
        match result.unwrap_err() {
            StrategyError::MissingData(msg) => {
                assert!(msg.contains("close"));
            }
            _ => panic!("Expected MissingData error"),
        }
    }
}
