//! Scalping Strategy for intraday trading
//!
//! This strategy aims to profit from small price changes by entering and
//! exiting positions quickly, focusing on short time frames (1-5 minutes).
//!
//! # Strategy Logic
//!
//! The scalping strategy detects short-term momentum by:
//! 1. Using a fast EMA to identify immediate trend direction
//! 2. Monitoring price movements that exceed a minimum threshold
//! 3. Entering trades on short bursts of momentum
//! 4. Exiting quickly to capture small profits
//!
//! # Example
//!
//! ```no_run
//! use intraday_trade::{ScalpingStrategy, IntradayStrategy};
//! use intraday_trade::utils::generate_minute_data;
//!
//! // Create a scalping strategy with a 5-minute lookback and 0.1% threshold
//! let strategy = ScalpingStrategy::new(5, 0.1).unwrap();
//!
//! // Generate test data
//! let data = generate_minute_data(5, 390, 100.0, 0.02, 0.0);
//!
//! // Generate trading signals
//! let signals = strategy.generate_signals(&data).unwrap();
//!
//! // Calculate performance
//! let performance = strategy.calculate_performance(&data, &signals).unwrap();
//! println!("Scalping strategy performance: {}%", performance);
//! ```

use crate::utils::{
    calculate_basic_performance, calculate_ema, validate_period, validate_positive,
};
use crate::{IntradayStrategy, MinuteOhlcv, Signal, TradeError};

/// Scalping Strategy for capturing short-term price movements
#[derive(Debug, Clone)]
pub struct ScalpingStrategy {
    /// Number of periods to look back for momentum calculation
    period: usize,
    /// Minimum price movement threshold (percentage) to trigger a trade
    threshold: f64,
    /// Strategy name
    name: String,
}

impl ScalpingStrategy {
    /// Create a new scalping strategy
    ///
    /// # Arguments
    ///
    /// * `period` - Number of periods to look back (typically 1-5 minutes)
    /// * `threshold` - Minimum price movement threshold as percentage (e.g., 0.1 means 0.1%)
    ///
    /// # Returns
    ///
    /// * `Result<Self, String>` - New strategy instance or error message
    pub fn new(period: usize, threshold: f64) -> Result<Self, String> {
        validate_period(period, 1)?;
        validate_positive(threshold, "Threshold")?;

        if threshold > 1.0 {
            return Err("Threshold seems too high (> 1%). For a scalping strategy, threshold is typically 0.05% to 0.5%. Use a decimal value like 0.1 for 0.1%.".to_string());
        }

        Ok(Self {
            period,
            threshold,
            name: format!("Scalping Strategy ({}m, {}%)", period, threshold * 100.0),
        })
    }

    /// Get the lookback period
    pub fn period(&self) -> usize {
        self.period
    }

    /// Get the price movement threshold
    pub fn threshold(&self) -> f64 {
        self.threshold
    }
}

impl IntradayStrategy for ScalpingStrategy {
    fn name(&self) -> &str {
        &self.name
    }

    fn generate_signals(&self, data: &[MinuteOhlcv]) -> Result<Vec<Signal>, TradeError> {
        if data.len() < self.period + 1 {
            return Err(TradeError::InsufficientData(format!(
                "Need at least {} data points for scalping strategy",
                self.period + 1
            )));
        }

        let mut signals = Vec::with_capacity(data.len());

        // Extract close prices
        let closes: Vec<f64> = data.iter().map(|d| d.data.close).collect();

        // Calculate fast EMA for trend direction
        let fast_ema = calculate_ema(&closes, self.period);

        // First entries are hold signals due to insufficient data
        for _ in 0..self.period {
            signals.push(Signal::Hold);
        }

        // Generate signals for the remaining data points
        for i in self.period..data.len() {
            let close = data[i].data.close;
            let prev_close = data[i - 1].data.close;
            let ema = fast_ema[i].unwrap();

            // Calculate price change percentage
            let price_change_pct = (close - prev_close) / prev_close * 100.0;

            // Determine signal based on price change exceeding threshold and EMA relationship
            let signal = if price_change_pct.abs() >= self.threshold * 100.0 {
                if close > ema && price_change_pct > 0.0 {
                    Signal::Buy
                } else if close < ema && price_change_pct < 0.0 {
                    Signal::Sell
                } else {
                    Signal::Hold
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
        // Use a low commission for high-frequency trading
        let commission = 0.01; // 0.01% per trade
        calculate_basic_performance(data, signals, 10000.0, commission)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::create_test_data;

    #[test]
    fn test_scalping_strategy_params() {
        // Test valid parameters
        let strategy = ScalpingStrategy::new(5, 0.1);
        assert!(strategy.is_ok());

        // Test invalid period
        let strategy = ScalpingStrategy::new(0, 0.1);
        assert!(strategy.is_err());

        // Test invalid threshold
        let strategy = ScalpingStrategy::new(5, 0.0);
        assert!(strategy.is_err());

        // Test threshold warning
        let strategy = ScalpingStrategy::new(5, 2.0);
        assert!(strategy.is_err());
    }

    #[test]
    fn test_scalping_signals() {
        let data = create_test_data(100);
        let strategy = ScalpingStrategy::new(5, 0.1).unwrap();

        let signals = strategy.generate_signals(&data).unwrap();

        // Check that we have the correct number of signals
        assert_eq!(signals.len(), data.len());

        // Check that the first 'period' signals are Hold
        for i in 0..strategy.period() {
            assert_eq!(signals[i], Signal::Hold);
        }

        // Check that we have some non-Hold signals
        let action_count = signals.iter().filter(|&&s| s != Signal::Hold).count();

        // A scalping strategy should generate a moderate number of signals
        assert!(action_count > 0, "Expected at least one non-Hold signal");
    }

    #[test]
    fn test_performance_calculation() {
        let data = create_test_data(100);
        let strategy = ScalpingStrategy::new(3, 0.05).unwrap();

        let signals = strategy.generate_signals(&data).unwrap();
        let performance = strategy.calculate_performance(&data, &signals).unwrap();

        // Just check that the calculation completes successfully
        println!("Test scalping performance: {}%", performance);
    }
}
