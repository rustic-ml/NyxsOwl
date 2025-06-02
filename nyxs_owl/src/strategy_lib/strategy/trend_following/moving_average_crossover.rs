//! # Moving Average Crossover Strategy
//!
//! A trend-following strategy that generates buy/sell signals when a faster moving average
//! crosses over a slower moving average.

use crate::strategy_lib::strategy::{Signal, Strategy, StrategyConfig, StrategyError};
use crate::trade_math::moving_averages::{ExponentialMovingAverage, SimpleMovingAverage};
use polars::prelude::*;

/// Moving Average Crossover Strategy
///
/// Generates buy signals when the fast moving average crosses above the slow moving average,
/// and sell signals when the fast moving average crosses below the slow moving average.
#[derive(Debug, Clone)]
pub struct MovingAverageCrossover {
    config: StrategyConfig,
    name: String,
    description: String,
    fast_period: usize,
    slow_period: usize,
    ma_type: String,
    price_column: String,
}

impl MovingAverageCrossover {
    /// Create a new SMA crossover strategy
    pub fn sma_crossover(fast_period: usize, slow_period: usize) -> Self {
        let config = StrategyConfig::new()
            .with_parameter("fast_period", fast_period)
            .with_parameter("slow_period", slow_period)
            .with_parameter("ma_type", "sma")
            .with_parameter("price_column", "close");

        Self::new(config)
    }

    /// Create a new EMA crossover strategy
    pub fn ema_crossover(fast_period: usize, slow_period: usize) -> Self {
        let config = StrategyConfig::new()
            .with_parameter("fast_period", fast_period)
            .with_parameter("slow_period", slow_period)
            .with_parameter("ma_type", "ema")
            .with_parameter("price_column", "close");

        Self::new(config)
    }

    /// Calculate Simple Moving Average
    fn calculate_sma(
        &self,
        prices: &[f64],
        period: usize,
    ) -> Result<Vec<Option<f64>>, StrategyError> {
        let mut sma = SimpleMovingAverage::new(period)
            .map_err(|e| StrategyError::IndicatorError(e.to_string()))?;

        let mut results = Vec::new();

        for &price in prices {
            sma.update(price)
                .map_err(|e| StrategyError::IndicatorError(e.to_string()))?;

            if sma.is_ready() {
                let value = sma
                    .value()
                    .map_err(|e| StrategyError::IndicatorError(e.to_string()))?;
                results.push(Some(value));
            } else {
                results.push(None);
            }
        }

        Ok(results)
    }

    /// Calculate Exponential Moving Average
    fn calculate_ema(
        &self,
        prices: &[f64],
        period: usize,
    ) -> Result<Vec<Option<f64>>, StrategyError> {
        let mut ema = ExponentialMovingAverage::new(period)
            .map_err(|e| StrategyError::IndicatorError(e.to_string()))?;

        let mut results = Vec::new();

        for &price in prices {
            ema.update(price)
                .map_err(|e| StrategyError::IndicatorError(e.to_string()))?;

            if ema.is_ready() {
                let value = ema
                    .value()
                    .map_err(|e| StrategyError::IndicatorError(e.to_string()))?;
                results.push(Some(value));
            } else {
                results.push(None);
            }
        }

        Ok(results)
    }

    /// Detect crossover signals
    fn detect_crossovers(&self, fast_ma: &[Option<f64>], slow_ma: &[Option<f64>]) -> Vec<Signal> {
        let mut signals = Vec::new();
        let mut prev_fast: Option<f64> = None;
        let mut prev_slow: Option<f64> = None;

        for i in 0..fast_ma.len() {
            let current_fast = fast_ma[i];
            let current_slow = slow_ma[i];

            let signal = if let (Some(cf), Some(cs), Some(pf), Some(ps)) =
                (current_fast, current_slow, prev_fast, prev_slow)
            {
                // Check for crossover
                if pf <= ps && cf > cs {
                    // Fast MA crossed above slow MA - Buy signal
                    Signal::Buy
                } else if pf >= ps && cf < cs {
                    // Fast MA crossed below slow MA - Sell signal
                    Signal::Sell
                } else {
                    Signal::Hold
                }
            } else {
                Signal::Hold
            };

            signals.push(signal);

            // Update previous values
            if current_fast.is_some() {
                prev_fast = current_fast;
            }
            if current_slow.is_some() {
                prev_slow = current_slow;
            }
        }

        signals
    }
}

impl Strategy for MovingAverageCrossover {
    fn new(config: StrategyConfig) -> Self {
        // Extract parameters with defaults
        let fast_period = config.get_int("fast_period").unwrap_or(10) as usize;
        let slow_period = config.get_int("slow_period").unwrap_or(30) as usize;
        let ma_type = config.get_string("ma_type").unwrap_or("ema").to_string();
        let price_column = config
            .get_string("price_column")
            .unwrap_or("close")
            .to_string();

        let name = format!(
            "MA_Crossover_{}_{}_{}",
            fast_period,
            slow_period,
            ma_type.to_uppercase()
        );

        let description = format!(
            "Moving Average Crossover strategy using {} and {} period {}",
            fast_period,
            slow_period,
            ma_type.to_uppercase()
        );

        Self {
            config,
            name,
            description,
            fast_period,
            slow_period,
            ma_type,
            price_column,
        }
    }

    fn generate_signals(&self, data: &DataFrame) -> Result<Series, StrategyError> {
        // Validate data first
        self.validate_data(data)?;

        // Get price data
        let prices = data.column(&self.price_column)?.f64().map_err(|_| {
            StrategyError::InvalidParameter("Cannot convert price column to f64".to_string())
        })?;

        // Convert to Vec<f64> for processing
        let price_vec: Vec<f64> = prices
            .into_iter()
            .map(|opt_price| opt_price.unwrap_or(0.0))
            .collect();

        // Calculate moving averages
        let fast_ma = match self.ma_type.as_str() {
            "sma" => self.calculate_sma(&price_vec, self.fast_period)?,
            "ema" => self.calculate_ema(&price_vec, self.fast_period)?,
            _ => {
                return Err(StrategyError::InvalidParameter(format!(
                    "Unknown MA type: {}",
                    self.ma_type
                )))
            }
        };

        let slow_ma = match self.ma_type.as_str() {
            "sma" => self.calculate_sma(&price_vec, self.slow_period)?,
            "ema" => self.calculate_ema(&price_vec, self.slow_period)?,
            _ => {
                return Err(StrategyError::InvalidParameter(format!(
                    "Unknown MA type: {}",
                    self.ma_type
                )))
            }
        };

        // Detect crossovers and generate signals
        let signals = self.detect_crossovers(&fast_ma, &slow_ma);

        // Convert signals to integer series
        let signal_ints: Vec<i32> = signals.iter().map(|s| s.to_int()).collect();
        Ok(Series::new("signals".into(), signal_ints))
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn required_columns(&self) -> Vec<&str> {
        vec![&self.price_column]
    }

    fn config(&self) -> &StrategyConfig {
        &self.config
    }

    fn min_data_points(&self) -> usize {
        self.slow_period + 10 // Need enough data for slow MA plus buffer
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Create a test DataFrame with trending data
    fn create_test_data() -> DataFrame {
        let close = Series::new(
            "close".into(),
            &[
                100.0, 101.0, 102.0, 103.0, 104.0, 105.0, 106.0, 107.0, 108.0, 109.0, 110.0, 111.0,
                112.0, 113.0, 114.0, 115.0, 116.0, 117.0, 118.0, 119.0, 120.0, 121.0, 122.0, 123.0,
                124.0,
            ],
        );

        DataFrame::new(vec![close.into()]).unwrap()
    }

    #[test]
    fn test_strategy_creation() {
        let strategy = MovingAverageCrossover::sma_crossover(5, 10);
        assert_eq!(strategy.name(), "MA_Crossover_5_10_SMA");
        assert!(strategy.description().contains("5 and 10 period SMA"));
        assert_eq!(strategy.required_columns(), vec!["close"]);
        assert_eq!(strategy.min_data_points(), 20); // 10 + 10 buffer
    }

    #[test]
    fn test_ema_strategy_creation() {
        let strategy = MovingAverageCrossover::ema_crossover(8, 21);
        assert_eq!(strategy.name(), "MA_Crossover_8_21_EMA");
        assert!(strategy.description().contains("8 and 21 period EMA"));
    }

    #[test]
    fn test_generate_signals() {
        let df = create_test_data();
        let strategy = MovingAverageCrossover::sma_crossover(5, 10);

        let signals = strategy.generate_signals(&df).unwrap();
        assert_eq!(signals.len(), df.height());

        // Verify signals are valid integers
        let signal_vals = signals.i32().unwrap();
        for i in 0..signal_vals.len() {
            let val = signal_vals.get(i).unwrap_or(0);
            assert!(val >= 0 && val <= 2); // Valid signal range
        }
    }

    #[test]
    fn test_missing_price_column() {
        let wrong_col = Series::new("wrong_col".into(), &[1.0, 2.0, 3.0]);
        let df = DataFrame::new(vec![wrong_col.into()]).unwrap();
        let strategy = MovingAverageCrossover::sma_crossover(5, 10);

        let result = strategy.generate_signals(&df);

        assert!(result.is_err());
        match result.unwrap_err() {
            StrategyError::MissingData(msg) => {
                assert!(msg.contains("close"));
            }
            _ => panic!("Expected MissingData error"),
        }
    }

    #[test]
    fn test_insufficient_data() {
        let config = StrategyConfig::new()
            .with_parameter("fast_period", 5_i64)
            .with_parameter("slow_period", 10_i64);

        let strategy = MovingAverageCrossover::new(config);

        let close = Series::new("close".into(), &[100.0, 101.0]); // Only 2 data points
        let df = DataFrame::new(vec![close.into()]).unwrap();

        let result = strategy.generate_signals(&df);
        assert!(result.is_err());

        if let Err(StrategyError::ValidationError(msg)) = result {
            assert!(msg.contains("Insufficient data"));
        } else {
            panic!("Expected ValidationError");
        }
    }

    #[test]
    fn test_config_access() {
        let strategy = MovingAverageCrossover::sma_crossover(12, 26);
        let config = strategy.config();

        assert_eq!(config.get_int("fast_period").unwrap(), 12);
        assert_eq!(config.get_int("slow_period").unwrap(), 26);
        assert_eq!(config.get_string("ma_type").unwrap(), "sma");
    }
}
