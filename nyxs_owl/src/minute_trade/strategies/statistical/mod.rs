//! Statistical intraday trading strategies
//!
//! This module will contain strategies that use statistical methods for trading.

// Re-export strategies
pub use self::regression_strategy::RegressionStrategy;
pub use self::z_score_strategy::ZScoreStrategy;

// These will be implemented in the future
mod regression_strategy {
    use crate::minute_trade::utils::{calculate_basic_performance, validate_period, validate_positive};
    use crate::minute_trade::{IntradayStrategy, MinuteOhlcv, Signal, TradeError};

    /// Regression Strategy - uses linear regression to identify trends and generate signals
    #[derive(Debug, Clone)]
    pub struct RegressionStrategy {
        /// Period for calculating regression
        lookback_period: usize,
        /// R-squared threshold for trend significance (0.0 to 1.0)
        r_squared_threshold: f64,
        /// Slope threshold for signal generation (minimum slope steepness)
        slope_threshold: f64,
        /// Distance threshold from regression line for entry (percentage)
        distance_threshold: f64,
        /// Strategy name
        name: String,
    }

    impl Default for RegressionStrategy {
        fn default() -> Self {
            Self::new(20, 0.7, 0.1, 2.0).unwrap()
        }
    }

    impl RegressionStrategy {
        /// Create a new Regression Strategy
        ///
        /// # Arguments
        ///
        /// * `lookback_period` - Period for regression calculation (typically 15-50)
        /// * `r_squared_threshold` - Minimum R² for trend significance (typically 0.6-0.9)
        /// * `slope_threshold` - Minimum absolute slope for signal generation (typically 0.05-0.5)
        /// * `distance_threshold` - Percentage distance from regression line for entry (typically 1.0-3.0)
        ///
        /// # Returns
        ///
        /// * `Result<Self, String>` - New strategy instance or error message
        pub fn new(
            lookback_period: usize,
            r_squared_threshold: f64,
            slope_threshold: f64,
            distance_threshold: f64,
        ) -> Result<Self, String> {
            validate_period(lookback_period, 10)?;
            validate_positive(slope_threshold, "Slope threshold")?;
            validate_positive(distance_threshold, "Distance threshold")?;

            if r_squared_threshold < 0.0 || r_squared_threshold > 1.0 {
                return Err("R-squared threshold must be between 0.0 and 1.0".to_string());
            }

            if r_squared_threshold < 0.5 {
                return Err("R-squared threshold seems too low (<0.5). Consider using a higher value for more reliable trends.".to_string());
            }

            if distance_threshold > 10.0 {
                return Err("Distance threshold seems too high (>10%). Consider using a lower value.".to_string());
            }

            Ok(Self {
                lookback_period,
                r_squared_threshold,
                slope_threshold,
                distance_threshold,
                name: format!(
                    "Regression ({}p, R²>{:.2}, slope>{:.2}, dist<{:.1}%)",
                    lookback_period, r_squared_threshold, slope_threshold, distance_threshold
                ),
            })
        }

        /// Get the lookback period
        pub fn lookback_period(&self) -> usize {
            self.lookback_period
        }

        /// Get the R-squared threshold
        pub fn r_squared_threshold(&self) -> f64 {
            self.r_squared_threshold
        }

        /// Get the slope threshold
        pub fn slope_threshold(&self) -> f64 {
            self.slope_threshold
        }

        /// Get the distance threshold
        pub fn distance_threshold(&self) -> f64 {
            self.distance_threshold
        }

        /// Calculate linear regression for price data
        fn calculate_regression(&self, prices: &[f64]) -> Option<RegressionResult> {
            if prices.len() < 3 {
                return None;
            }

            let n = prices.len() as f64;
            let x_values: Vec<f64> = (0..prices.len()).map(|i| i as f64).collect();

            // Calculate means
            let x_mean = x_values.iter().sum::<f64>() / n;
            let y_mean = prices.iter().sum::<f64>() / n;

            // Calculate slope and intercept using least squares
            let numerator: f64 = x_values
                .iter()
                .zip(prices.iter())
                .map(|(x, y)| (x - x_mean) * (y - y_mean))
                .sum();

            let denominator: f64 = x_values
                .iter()
                .map(|x| (x - x_mean).powi(2))
                .sum();

            if denominator.abs() < f64::EPSILON {
                return None; // Avoid division by zero
            }

            let slope = numerator / denominator;
            let intercept = y_mean - slope * x_mean;

            // Calculate R-squared
            let predicted_values: Vec<f64> = x_values
                .iter()
                .map(|x| slope * x + intercept)
                .collect();

            let ss_res: f64 = prices
                .iter()
                .zip(predicted_values.iter())
                .map(|(actual, predicted)| (actual - predicted).powi(2))
                .sum();

            let ss_tot: f64 = prices
                .iter()
                .map(|y| (y - y_mean).powi(2))
                .sum();

            let r_squared = if ss_tot.abs() < f64::EPSILON {
                0.0
            } else {
                1.0 - (ss_res / ss_tot)
            };

            Some(RegressionResult {
                slope,
                intercept,
                r_squared,
            })
        }

        /// Calculate distance from current price to regression line (as percentage)
        fn calculate_distance_from_line(
            &self,
            current_price: f64,
            regression: &RegressionResult,
            current_x: f64,
        ) -> f64 {
            let expected_price = regression.slope * current_x + regression.intercept;
            if expected_price <= 0.0 {
                return 0.0;
            }
            ((current_price - expected_price) / expected_price).abs() * 100.0
        }

        /// Generate signal based on regression analysis
        fn evaluate_regression_signal(
            &self,
            current_price: f64,
            regression: &RegressionResult,
            current_x: f64,
        ) -> Signal {
            // Check if the trend is significant enough
            if regression.r_squared < self.r_squared_threshold {
                return Signal::Hold;
            }

            // Check if the slope is significant enough
            if regression.slope.abs() < self.slope_threshold {
                return Signal::Hold;
            }

            let expected_price = regression.slope * current_x + regression.intercept;
            let distance = self.calculate_distance_from_line(current_price, regression, current_x);

            // Only trade if price is far enough from the regression line
            if distance < self.distance_threshold {
                return Signal::Hold;
            }

            // Determine signal based on trend direction and price position relative to line
            if regression.slope > 0.0 {
                // Upward trend
                if current_price < expected_price {
                    // Price below upward trend line - buy (expecting reversion to trend)
                    Signal::Buy
                } else {
                    // Price above upward trend line - sell (expecting pullback)
                    Signal::Sell
                }
            } else {
                // Downward trend
                if current_price > expected_price {
                    // Price above downward trend line - sell (expecting reversion to trend)
                    Signal::Sell
                } else {
                    // Price below downward trend line - buy (expecting bounce)
                    Signal::Buy
                }
            }
        }
    }

    #[derive(Debug, Clone)]
    struct RegressionResult {
        slope: f64,
        intercept: f64,
        r_squared: f64,
    }

    impl IntradayStrategy for RegressionStrategy {
        fn name(&self) -> &str {
            &self.name
        }

        fn generate_signals(&self, data: &[MinuteOhlcv]) -> Result<Vec<Signal>, TradeError> {
            if data.len() < self.lookback_period + 1 {
                return Err(TradeError::InsufficientData(format!(
                    "Need at least {} data points for Regression strategy",
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
            let mut in_position = false;
            let mut position_is_long = false;

            // Generate signals for the remaining data points
            for i in self.lookback_period..data.len() {
                // Extract the window for regression calculation
                let window = &closes[i - self.lookback_period..i];
                let current_price = closes[i];

                let signal = if let Some(regression) = self.calculate_regression(window) {
                    if !in_position {
                        // No position - look for entry signals
                        let entry_signal = self.evaluate_regression_signal(
                            current_price,
                            &regression,
                            (self.lookback_period - 1) as f64, // Current position in the window
                        );

                        match entry_signal {
                            Signal::Buy => {
                                in_position = true;
                                position_is_long = true;
                                Signal::Buy
                            }
                            Signal::Sell => {
                                in_position = true;
                                position_is_long = false;
                                Signal::Sell
                            }
                            Signal::Hold => Signal::Hold,
                        }
                    } else {
                        // In position - look for exit conditions
                        let distance = self.calculate_distance_from_line(
                            current_price,
                            &regression,
                            (self.lookback_period - 1) as f64,
                        );

                        // Exit if price moves back close to regression line or trend weakens
                        let should_exit = distance < (self.distance_threshold * 0.5) || 
                                        regression.r_squared < (self.r_squared_threshold * 0.8);

                        if should_exit {
                            in_position = false;
                            if position_is_long {
                                Signal::Sell // Close long position
                            } else {
                                Signal::Buy // Close short position
                            }
                        } else {
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
            // Use moderate commission for regression strategies
            let commission = 0.025; // 0.025% per trade
            calculate_basic_performance(data, signals, 10000.0, commission)
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::minute_trade::create_test_data;

        #[test]
        fn test_regression_parameters() {
            // Test valid parameters
            let strategy = RegressionStrategy::new(20, 0.7, 0.1, 2.0);
            assert!(strategy.is_ok());

            // Test invalid lookback period
            let strategy = RegressionStrategy::new(5, 0.7, 0.1, 2.0);
            assert!(strategy.is_err());

            // Test invalid R-squared threshold
            let strategy = RegressionStrategy::new(20, 1.5, 0.1, 2.0);
            assert!(strategy.is_err());

            // Test invalid slope threshold
            let strategy = RegressionStrategy::new(20, 0.7, -0.1, 2.0);
            assert!(strategy.is_err());

            // Test warning for low R-squared
            let strategy = RegressionStrategy::new(20, 0.3, 0.1, 2.0);
            assert!(strategy.is_err());
        }

        #[test]
        fn test_regression_calculation() {
            let strategy = RegressionStrategy::new(20, 0.7, 0.1, 2.0).unwrap();
            
            // Test with a clear upward trend
            let prices = vec![1.0, 2.0, 3.0, 4.0, 5.0];
            let regression = strategy.calculate_regression(&prices);
            assert!(regression.is_some());

            if let Some(reg) = regression {
                assert!(reg.slope > 0.0); // Should be positive slope
                assert!(reg.r_squared > 0.9); // Should have high R-squared for perfect linear data
            }

            // Test with insufficient data
            let prices = vec![1.0, 2.0];
            let regression = strategy.calculate_regression(&prices);
            assert!(regression.is_none());
        }

        #[test]
        fn test_distance_calculation() {
            let strategy = RegressionStrategy::new(20, 0.7, 0.1, 2.0).unwrap();
            
            let regression = RegressionResult {
                slope: 1.0,
                intercept: 0.0,
                r_squared: 0.9,
            };

            // Test distance calculation
            let distance = strategy.calculate_distance_from_line(5.0, &regression, 4.0);
            assert!((distance - 25.0).abs() < f64::EPSILON); // (5-4)/4 * 100 = 25%
        }

        #[test]
        fn test_signal_generation() {
            let data = create_test_data(50);
            let strategy = RegressionStrategy::new(20, 0.7, 0.1, 2.0).unwrap();

            let signals = strategy.generate_signals(&data).unwrap();

            // Check that we have the correct number of signals
            assert_eq!(signals.len(), data.len());

            // Check that the first 'lookback_period' signals are Hold
            for i in 0..strategy.lookback_period() {
                assert_eq!(signals[i], Signal::Hold);
            }
        }

        #[test]
        fn test_regression_with_flat_data() {
            let strategy = RegressionStrategy::new(20, 0.7, 0.1, 2.0).unwrap();
            
            // Test with flat data (no trend)
            let prices = vec![5.0; 10]; // All prices the same
            let regression = strategy.calculate_regression(&prices);
            assert!(regression.is_some());

            if let Some(reg) = regression {
                assert!(reg.slope.abs() < f64::EPSILON); // Should be near zero slope
            }
        }
    }
}

mod z_score_strategy {
    use crate::minute_trade::create_test_data;
    use crate::minute_trade::utils::{
        calculate_basic_performance, validate_period, validate_positive,
    };
    use crate::minute_trade::{IntradayStrategy, MinuteOhlcv, Signal, TradeError};

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
        use crate::minute_trade::create_test_data;

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
