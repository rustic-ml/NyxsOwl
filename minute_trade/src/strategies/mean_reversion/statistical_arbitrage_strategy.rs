//! Statistical Arbitrage Strategy for intraday trading
//!
//! This strategy exploits price divergence between correlated assets, assuming they
//! will revert to their historical relationship.
//!
//! # Strategy Logic
//!
//! The statistical arbitrage strategy:
//! 1. Calculates correlation and spread between two assets
//! 2. Identifies when the spread deviates significantly from its mean
//! 3. Takes positions expecting the spread to revert to normal
//! 4. Exits when the relationship normalizes
//!
//! # Example
//!
//! ```no_run
//! use intraday_trade::{StatisticalArbitrageStrategy, IntradayStrategy};
//! use intraday_trade::utils::generate_minute_data;
//!
//! // Create a statistical arbitrage strategy
//! let strategy = StatisticalArbitrageStrategy::new(30, 2.0).unwrap();
//!
//! // Generate test data
//! let data = generate_minute_data(5, 390, 100.0, 0.01, 0.0);
//!
//! // Generate trading signals
//! let signals = strategy.generate_signals(&data).unwrap();
//! ```

use crate::utils::{
    calculate_basic_performance, calculate_sma, validate_period, validate_positive,
};
use crate::{IntradayStrategy, MinuteOhlcv, Signal, TradeError};

/// Statistical Arbitrage Strategy for exploiting price divergence between related assets
#[derive(Debug, Clone)]
pub struct StatisticalArbitrageStrategy {
    /// Lookback window for calculating the mean and standard deviation
    lookback_period: usize,
    /// Z-score threshold for entry (number of standard deviations)
    zscore_threshold: f64,
    /// Strategy name
    name: String,
}

impl StatisticalArbitrageStrategy {
    /// Create a new statistical arbitrage strategy
    ///
    /// # Arguments
    ///
    /// * `lookback_period` - Number of periods to calculate statistical measures (typically 20-60 minutes)
    /// * `zscore_threshold` - Z-score threshold for trade signals (typically 1.5-3.0)
    ///
    /// # Returns
    ///
    /// * `Result<Self, String>` - New strategy instance or error message
    pub fn new(lookback_period: usize, zscore_threshold: f64) -> Result<Self, String> {
        validate_period(lookback_period, 10)?;
        validate_positive(zscore_threshold, "Z-score threshold")?;

        if zscore_threshold < 1.0 {
            return Err("Z-score threshold seems too low (<1.0). Statistical significance typically requires at least 1.0, with 1.5-3.0 being common.".to_string());
        }

        Ok(Self {
            lookback_period,
            zscore_threshold,
            name: format!(
                "Statistical Arbitrage ({}, {}σ)",
                lookback_period, zscore_threshold
            ),
        })
    }

    /// Get the lookback period
    pub fn lookback_period(&self) -> usize {
        self.lookback_period
    }

    /// Get the z-score threshold
    pub fn zscore_threshold(&self) -> f64 {
        self.zscore_threshold
    }

    /// Calculate the z-score for a time series
    fn calculate_zscore(&self, values: &[f64], current_value: f64) -> Option<f64> {
        if values.len() < 2 {
            return None;
        }

        // Calculate mean
        let sum: f64 = values.iter().sum();
        let mean = sum / values.len() as f64;

        // Calculate standard deviation
        let variance: f64 = values
            .iter()
            .map(|&value| (value - mean).powi(2))
            .sum::<f64>()
            / (values.len() - 1) as f64;

        let std_dev = variance.sqrt();

        if std_dev.abs() < f64::EPSILON {
            return None; // Avoid division by zero
        }

        // Calculate z-score
        Some((current_value - mean) / std_dev)
    }
}

impl IntradayStrategy for StatisticalArbitrageStrategy {
    fn name(&self) -> &str {
        &self.name
    }

    fn generate_signals(&self, data: &[MinuteOhlcv]) -> Result<Vec<Signal>, TradeError> {
        if data.len() < self.lookback_period + 1 {
            return Err(TradeError::InsufficientData(format!(
                "Need at least {} data points for statistical arbitrage strategy",
                self.lookback_period + 1
            )));
        }

        let mut signals = Vec::with_capacity(data.len());

        // Extract close prices
        let closes: Vec<f64> = data.iter().map(|d| d.data.close).collect();

        // Calculate moving average for the spread
        let sma = calculate_sma(&closes, self.lookback_period);

        // First entries are hold signals due to insufficient data
        for _ in 0..self.lookback_period {
            signals.push(Signal::Hold);
        }

        // Generate signals for the remaining data points
        for i in self.lookback_period..data.len() {
            // Extract the window for calculations
            let window = &closes[i - self.lookback_period..i];
            let current_price = closes[i];
            let ma = sma[i].unwrap();

            // Calculate z-score
            if let Some(zscore) = self.calculate_zscore(window, current_price) {
                // Generate signals based on z-score
                let signal = if zscore <= -self.zscore_threshold {
                    // Price is significantly below the mean - buy expecting reversion upward
                    Signal::Buy
                } else if zscore >= self.zscore_threshold {
                    // Price is significantly above the mean - sell expecting reversion downward
                    Signal::Sell
                } else {
                    // Price is within normal range - hold
                    Signal::Hold
                };

                signals.push(signal);
            } else {
                signals.push(Signal::Hold);
            }
        }

        Ok(signals)
    }

    fn calculate_performance(
        &self,
        data: &[MinuteOhlcv],
        signals: &[Signal],
    ) -> Result<f64, TradeError> {
        // Use a moderate commission rate
        let commission = 0.03; // 0.03% per trade
        calculate_basic_performance(data, signals, 10000.0, commission)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::create_test_data;

    #[test]
    fn test_statistical_arbitrage_parameters() {
        // Test valid parameters
        let strategy = StatisticalArbitrageStrategy::new(30, 2.0);
        assert!(strategy.is_ok());

        // Test invalid period
        let strategy = StatisticalArbitrageStrategy::new(5, 2.0);
        assert!(strategy.is_err());

        // Test invalid threshold
        let strategy = StatisticalArbitrageStrategy::new(30, 0.0);
        assert!(strategy.is_err());

        // Test warning threshold
        let strategy = StatisticalArbitrageStrategy::new(30, 0.5);
        assert!(strategy.is_err());
    }

    #[test]
    fn test_zscore_calculation() {
        let strategy = StatisticalArbitrageStrategy::new(20, 2.0).unwrap();

        // Test with known values
        let values = vec![100.0, 101.0, 99.0, 102.0, 98.0];
        let mean = 100.0; // Average of the values
        let std_dev = 1.5811; // Standard deviation approx

        // Z-score for value 103.0 should be (103.0 - 100.0) / 1.5811 ≈ 1.897
        let zscore = strategy.calculate_zscore(&values, 103.0).unwrap();
        let expected = (103.0 - mean) / std_dev;

        assert!((zscore - expected).abs() < 0.01);
    }

    #[test]
    fn test_signal_generation() {
        let data = create_test_data(100);
        let strategy = StatisticalArbitrageStrategy::new(20, 2.0).unwrap();

        let signals = strategy.generate_signals(&data).unwrap();

        // Check that we have the correct number of signals
        assert_eq!(signals.len(), data.len());

        // Check that the first 'lookback_period' signals are Hold
        for i in 0..strategy.lookback_period() {
            assert_eq!(signals[i], Signal::Hold);
        }
    }
}
