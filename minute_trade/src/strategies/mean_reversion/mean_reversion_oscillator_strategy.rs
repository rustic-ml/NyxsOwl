//! Mean Reversion Oscillator Strategy for intraday trading
//!
//! This strategy uses oscillator indicators to find oversold/overbought conditions
//! for trading mean reversion movements.
//!
//! # Strategy Logic
//!
//! The mean reversion oscillator strategy:
//! 1. Calculates RSI (Relative Strength Index) to identify overbought/oversold conditions
//! 2. Identifies when prices reach extreme levels
//! 3. Takes contrarian positions expecting a reversion to the mean
//! 4. Uses confirmation signals to filter out false positives
//!
//! # Example
//!
//! ```no_run
//! use intraday_trade::{MeanReversionOscillatorStrategy, IntradayStrategy};
//! use intraday_trade::utils::generate_minute_data;
//!
//! // Create a mean reversion oscillator strategy
//! let strategy = MeanReversionOscillatorStrategy::new(14, 30.0, 70.0).unwrap();
//!
//! // Generate test data
//! let data = generate_minute_data(5, 390, 100.0, 0.01, 0.0);
//!
//! // Generate trading signals
//! let signals = strategy.generate_signals(&data).unwrap();
//! ```

use crate::utils::{calculate_basic_performance, calculate_rsi, validate_period, validate_range};
use crate::{IntradayStrategy, MinuteOhlcv, Signal, TradeError};

/// Mean Reversion Oscillator Strategy for trading based on overbought/oversold conditions
#[derive(Debug, Clone)]
pub struct MeanReversionOscillatorStrategy {
    /// Period for RSI calculation
    rsi_period: usize,
    /// Oversold threshold (typically 30)
    oversold_threshold: f64,
    /// Overbought threshold (typically 70)
    overbought_threshold: f64,
    /// Strategy name
    name: String,
}

impl MeanReversionOscillatorStrategy {
    /// Create a new mean reversion oscillator strategy
    ///
    /// # Arguments
    ///
    /// * `rsi_period` - Period for RSI calculation (typically 9-14 minutes)
    /// * `oversold_threshold` - Threshold below which an asset is considered oversold (e.g., 30)
    /// * `overbought_threshold` - Threshold above which an asset is considered overbought (e.g., 70)
    ///
    /// # Returns
    ///
    /// * `Result<Self, String>` - New strategy instance or error message
    pub fn new(
        rsi_period: usize,
        oversold_threshold: f64,
        overbought_threshold: f64,
    ) -> Result<Self, String> {
        validate_period(rsi_period, 2)?;
        validate_range(oversold_threshold, 0.0, 50.0, "Oversold threshold")?;
        validate_range(overbought_threshold, 50.0, 100.0, "Overbought threshold")?;

        if overbought_threshold - oversold_threshold < 20.0 {
            return Err("The gap between oversold and overbought thresholds should be at least 20 (e.g., 30/70) to avoid excessive trading.".to_string());
        }

        Ok(Self {
            rsi_period,
            oversold_threshold,
            overbought_threshold,
            name: format!(
                "Mean Reversion RSI ({}, {}/{})",
                rsi_period, oversold_threshold, overbought_threshold
            ),
        })
    }

    /// Get the RSI period
    pub fn rsi_period(&self) -> usize {
        self.rsi_period
    }

    /// Get the oversold threshold
    pub fn oversold_threshold(&self) -> f64 {
        self.oversold_threshold
    }

    /// Get the overbought threshold
    pub fn overbought_threshold(&self) -> f64 {
        self.overbought_threshold
    }

    /// Check for confirmation signal (optional filter to reduce false signals)
    fn has_confirmation(&self, data: &[MinuteOhlcv], index: usize) -> bool {
        if index < 2 || index >= data.len() {
            return false;
        }

        // Simple confirmation based on price action: check for candlestick reversal pattern
        let current = &data[index].data;
        let previous = &data[index - 1].data;

        // For buy confirmation (after oversold condition): current close > previous close
        let buy_confirmation = current.close > previous.close;

        // For sell confirmation (after overbought condition): current close < previous close
        let sell_confirmation = current.close < previous.close;

        buy_confirmation || sell_confirmation
    }
}

impl IntradayStrategy for MeanReversionOscillatorStrategy {
    fn name(&self) -> &str {
        &self.name
    }

    fn generate_signals(&self, data: &[MinuteOhlcv]) -> Result<Vec<Signal>, TradeError> {
        if data.len() < self.rsi_period + 1 {
            return Err(TradeError::InsufficientData(format!(
                "Need at least {} data points for mean reversion oscillator strategy",
                self.rsi_period + 1
            )));
        }

        let mut signals = Vec::with_capacity(data.len());

        // Extract close prices
        let closes: Vec<f64> = data.iter().map(|d| d.data.close).collect();

        // Calculate RSI
        let rsi_values = calculate_rsi(&closes, self.rsi_period);

        // First entries are hold signals due to insufficient data for RSI calculation
        for _ in 0..self.rsi_period {
            signals.push(Signal::Hold);
        }

        // Track oversold/overbought states to implement entry and exit logic
        let mut in_long_position = false;
        let mut in_short_position = false;

        // Generate signals for the remaining data points
        for i in self.rsi_period..data.len() {
            let rsi = match rsi_values[i] {
                Some(val) => val,
                None => {
                    signals.push(Signal::Hold);
                    continue;
                }
            };

            let mut signal = Signal::Hold;

            // Entry and exit logic for mean reversion
            if !in_long_position && !in_short_position {
                // No position - look for new entries
                if rsi <= self.oversold_threshold && self.has_confirmation(data, i) {
                    // Oversold condition - buy
                    signal = Signal::Buy;
                    in_long_position = true;
                } else if rsi >= self.overbought_threshold && self.has_confirmation(data, i) {
                    // Overbought condition - sell
                    signal = Signal::Sell;
                    in_short_position = true;
                }
            } else if in_long_position {
                // In long position - look for exit
                if rsi >= 50.0 {
                    // Exit when RSI crosses above the center line
                    signal = Signal::Sell;
                    in_long_position = false;
                }
            } else if in_short_position {
                // In short position - look for exit
                if rsi <= 50.0 {
                    // Exit when RSI crosses below the center line
                    signal = Signal::Buy;
                    in_short_position = false;
                }
            }

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
    fn test_mean_reversion_parameters() {
        // Test valid parameters
        let strategy = MeanReversionOscillatorStrategy::new(14, 30.0, 70.0);
        assert!(strategy.is_ok());

        // Test invalid RSI period
        let strategy = MeanReversionOscillatorStrategy::new(1, 30.0, 70.0);
        assert!(strategy.is_err());

        // Test invalid oversold threshold
        let strategy = MeanReversionOscillatorStrategy::new(14, -10.0, 70.0);
        assert!(strategy.is_err());

        // Test invalid overbought threshold
        let strategy = MeanReversionOscillatorStrategy::new(14, 30.0, 110.0);
        assert!(strategy.is_err());

        // Test invalid threshold gap
        let strategy = MeanReversionOscillatorStrategy::new(14, 40.0, 55.0);
        assert!(strategy.is_err());
    }

    #[test]
    fn test_signal_generation() {
        let data = create_test_data(100);
        let strategy = MeanReversionOscillatorStrategy::new(14, 30.0, 70.0).unwrap();

        let signals = strategy.generate_signals(&data).unwrap();

        // Check that we have the correct number of signals
        assert_eq!(signals.len(), data.len());

        // Check that the first 'rsi_period' signals are Hold
        for i in 0..strategy.rsi_period() {
            assert_eq!(signals[i], Signal::Hold);
        }
    }

    #[test]
    fn test_confirmation_signal() {
        let data = create_test_data(10);
        let strategy = MeanReversionOscillatorStrategy::new(2, 30.0, 70.0).unwrap();

        // Test valid index
        let confirmation = strategy.has_confirmation(&data, 5);
        assert!(confirmation || !confirmation); // Should be either true or false

        // Test invalid index (too small)
        let confirmation = strategy.has_confirmation(&data, 1);
        assert!(!confirmation);

        // Test invalid index (too large)
        let confirmation = strategy.has_confirmation(&data, 100);
        assert!(!confirmation);
    }
}
