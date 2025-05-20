//! Statistical intraday trading strategies
//!
//! This module will contain strategies that use statistical methods for trading.

// Re-export strategies
pub use self::regression_strategy::RegressionStrategy;
pub use self::z_score_strategy::ZScoreStrategy;

// These will be implemented in the future
mod regression_strategy {
    use crate::{IntradayStrategy, MinuteOhlcv, Signal, TradeError};

    /// Placeholder for the Regression Strategy
    #[derive(Debug, Clone)]
    pub struct RegressionStrategy;

    impl RegressionStrategy {
        /// Create a new instance (placeholder)
        pub fn new() -> Self {
            Self
        }
    }

    impl IntradayStrategy for RegressionStrategy {
        fn name(&self) -> &str {
            "Regression Strategy (placeholder)"
        }

        fn generate_signals(&self, data: &[MinuteOhlcv]) -> Result<Vec<Signal>, TradeError> {
            let mut signals = vec![Signal::Hold; data.len()];
            Ok(signals)
        }

        fn calculate_performance(
            &self,
            data: &[MinuteOhlcv],
            signals: &[Signal],
        ) -> Result<f64, TradeError> {
            Ok(0.0) // Placeholder
        }
    }
}

mod z_score_strategy {
    use crate::utils::{
        calculate_basic_performance, validate_period, validate_positive, validate_range,
    };
    use crate::{IntradayStrategy, MinuteOhlcv, Signal, TradeError};

    /// Z-Score strategy for trading statistical deviations from the mean
    #[derive(Debug, Clone)]
    pub struct ZScoreStrategy {
        /// Period for calculating mean and standard deviation
        lookback_period: usize,
        /// Z-score threshold for entry (number of standard deviations)
        entry_threshold: f64,
        /// Z-score threshold for exit (number of standard deviations)
        exit_threshold: f64,
        /// Strategy name
        name: String,
    }

    impl ZScoreStrategy {
        /// Create a new Z-Score strategy
        ///
        /// # Arguments
        ///
        /// * `lookback_period` - Period for calculating statistics (typically 20-100)
        /// * `entry_threshold` - Z-score threshold for entry signals (typically 1.5-3.0)
        /// * `exit_threshold` - Z-score threshold for exit signals (typically 0.5-1.0)
        ///
        /// # Returns
        ///
        /// * `Result<Self, String>` - New strategy instance or error message
        pub fn new(
            lookback_period: usize,
            entry_threshold: f64,
            exit_threshold: f64,
        ) -> Result<Self, String> {
            validate_period(lookback_period, 10)?;
            validate_positive(entry_threshold, "Entry threshold")?;
            validate_positive(exit_threshold, "Exit threshold")?;

            if entry_threshold < 1.0 {
                return Err("Entry Z-score threshold seems too low (<1.0). Statistical significance typically requires at least 1.0.".to_string());
            }

            if exit_threshold >= entry_threshold {
                return Err("Exit threshold should be lower than entry threshold to prevent immediate exits.".to_string());
            }

            Ok(Self {
                lookback_period,
                entry_threshold,
                exit_threshold,
                name: format!(
                    "Z-Score ({}, {}σ/{}σ)",
                    lookback_period, entry_threshold, exit_threshold
                ),
            })
        }

        /// Get the lookback period
        pub fn lookback_period(&self) -> usize {
            self.lookback_period
        }

        /// Get the entry threshold
        pub fn entry_threshold(&self) -> f64 {
            self.entry_threshold
        }

        /// Get the exit threshold
        pub fn exit_threshold(&self) -> f64 {
            self.exit_threshold
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

    impl IntradayStrategy for ZScoreStrategy {
        fn name(&self) -> &str {
            &self.name
        }

        fn generate_signals(&self, data: &[MinuteOhlcv]) -> Result<Vec<Signal>, TradeError> {
            if data.len() < self.lookback_period + 1 {
                return Err(TradeError::InsufficientData(format!(
                    "Need at least {} data points for Z-Score strategy",
                    self.lookback_period + 1
                )));
            }

            let mut signals = Vec::with_capacity(data.len());

            // Extract close prices
            let closes: Vec<f64> = data.iter().map(|d| d.data.close).collect();

            // First entries are hold signals due to insufficient data
            for _ in 0..self.lookback_period {
                signals.push(Signal::Hold);
            }

            // Track position state
            let mut in_long = false;
            let mut in_short = false;

            // Generate signals for the remaining data points
            for i in self.lookback_period..data.len() {
                // Extract the window for calculations
                let window = &closes[i - self.lookback_period..i];
                let current_price = closes[i];

                // Calculate z-score
                let signal = if let Some(zscore) = self.calculate_zscore(window, current_price) {
                    if !in_long && !in_short {
                        // No position - look for entries
                        if zscore <= -self.entry_threshold {
                            // Price is significantly below the mean - buy expecting reversion upward
                            in_long = true;
                            Signal::Buy
                        } else if zscore >= self.entry_threshold {
                            // Price is significantly above the mean - sell expecting reversion downward
                            in_short = true;
                            Signal::Sell
                        } else {
                            // No significant deviation - hold
                            Signal::Hold
                        }
                    } else if in_long {
                        // In long position - check for exit
                        if zscore >= -self.exit_threshold {
                            // Price has reverted enough - exit
                            in_long = false;
                            Signal::Sell
                        } else {
                            // Hold position
                            Signal::Hold
                        }
                    } else {
                        // Must be in short position - check for exit
                        if zscore <= self.exit_threshold {
                            // Price has reverted enough - exit
                            in_short = false;
                            Signal::Buy
                        } else {
                            // Hold position
                            Signal::Hold
                        }
                    }
                } else {
                    Signal::Hold
                };

                signals.push(signal);
            }

            Ok(signals)
        }

        fn calculate_performance(
            &self,
            data: &[MinuteOhlcv],
            signals: &[Signal],
        ) -> Result<f64, TradeError> {
            // Use a moderate commission rate
            let commission = 0.02; // 0.02% per trade
            calculate_basic_performance(data, signals, 10000.0, commission)
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::tests::create_test_data;

        #[test]
        fn test_zscore_parameters() {
            // Test valid parameters
            let strategy = ZScoreStrategy::new(30, 2.0, 0.5);
            assert!(strategy.is_ok());

            // Test invalid period
            let strategy = ZScoreStrategy::new(5, 2.0, 0.5);
            assert!(strategy.is_err());

            // Test invalid entry threshold
            let strategy = ZScoreStrategy::new(30, 0.5, 0.2);
            assert!(strategy.is_err());

            // Test invalid exit threshold
            let strategy = ZScoreStrategy::new(30, 2.0, -0.5);
            assert!(strategy.is_err());

            // Test exit >= entry
            let strategy = ZScoreStrategy::new(30, 2.0, 2.5);
            assert!(strategy.is_err());
        }

        #[test]
        fn test_zscore_calculation() {
            let strategy = ZScoreStrategy::new(20, 2.0, 0.5).unwrap();

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
            let strategy = ZScoreStrategy::new(20, 2.0, 0.5).unwrap();

            let signals = strategy.generate_signals(&data).unwrap();

            // Check that we have the correct number of signals
            assert_eq!(signals.len(), data.len());

            // Check that the first 'lookback_period' signals are Hold
            for i in 0..strategy.lookback_period() {
                assert_eq!(signals[i], Signal::Hold);
            }
        }
    }
}
