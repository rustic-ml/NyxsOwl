//! Volatility Breakout Strategy for intraday trading
//!
//! This strategy enters trades after periods of low volatility,
//! anticipating a volatility expansion and price breakout.
//!
//! # Strategy Logic
//!
//! The volatility breakout strategy:
//! 1. Identifies periods of low volatility (contraction)
//! 2. Sets breakout levels above/below the contraction range
//! 3. Enters trades when price breaks out of the contraction range
//! 4. Uses volatility-based stop loss and take profit levels
//!
//! # Example
//!
//! ```no_run
//! use minute_trade::{VolatilityBreakoutStrategy, IntradayStrategy};
//! use minute_trade::utils::generate_minute_data;
//!
//! // Create a volatility breakout strategy
//! let strategy = VolatilityBreakoutStrategy::new(20, 5, 1.5).unwrap();
//!
//! // Generate test data
//! let data = generate_minute_data(5, 390, 100.0, 0.01, 0.0);
//!
//! // Generate trading signals
//! let signals = strategy.generate_signals(&data).unwrap();
//! ```

use crate::utils::{calculate_basic_performance, validate_period, validate_positive};
use crate::{IntradayStrategy, MinuteOhlcv, Signal, TradeError};

/// Volatility Breakout Strategy for trading after low volatility periods
#[derive(Debug, Clone)]
pub struct VolatilityBreakoutStrategy {
    /// Period for detecting volatility contraction
    lookback_period: usize,
    /// Number of periods to detect low volatility
    contraction_period: usize,
    /// Multiplier for breakout level (e.g., 1.5 = 150% of range)
    breakout_multiplier: f64,
    /// Strategy name
    name: String,
}

impl VolatilityBreakoutStrategy {
    /// Create a new volatility breakout strategy
    ///
    /// # Arguments
    ///
    /// * `lookback_period` - Period for calculating volatility (typically 15-30 minutes)
    /// * `contraction_period` - Number of periods to confirm volatility contraction (typically 3-10)
    /// * `breakout_multiplier` - Multiplier for breakout level (typically 1.2-2.0)
    ///
    /// # Returns
    ///
    /// * `Result<Self, String>` - New strategy instance or error message
    pub fn new(
        lookback_period: usize,
        contraction_period: usize,
        breakout_multiplier: f64,
    ) -> Result<Self, String> {
        validate_period(lookback_period, 5)?;
        validate_period(contraction_period, 2)?;
        validate_positive(breakout_multiplier, "Breakout multiplier")?;

        if contraction_period >= lookback_period {
            return Err("Contraction period must be smaller than lookback period".to_string());
        }

        if breakout_multiplier < 1.0 {
            return Err(
                "Breakout multiplier should be at least 1.0 to avoid false breakouts".to_string(),
            );
        }

        Ok(Self {
            lookback_period,
            contraction_period,
            breakout_multiplier,
            name: format!(
                "Volatility Breakout ({}, {}, {}x)",
                lookback_period, contraction_period, breakout_multiplier
            ),
        })
    }

    /// Get the lookback period
    pub fn lookback_period(&self) -> usize {
        self.lookback_period
    }

    /// Get the contraction period
    pub fn contraction_period(&self) -> usize {
        self.contraction_period
    }

    /// Get the breakout multiplier
    pub fn breakout_multiplier(&self) -> f64 {
        self.breakout_multiplier
    }

    /// Calculate average true range for volatility measurement
    fn calculate_atr(&self, data: &[MinuteOhlcv], index: usize) -> Option<f64> {
        if index < self.lookback_period {
            return None;
        }

        let start_index = index - self.lookback_period;
        let mut true_ranges = Vec::with_capacity(self.lookback_period);

        // First true range doesn't have a previous close
        let first_candle = &data[start_index].data;
        let first_tr = first_candle.high - first_candle.low;
        true_ranges.push(first_tr);

        // Calculate true ranges for the rest of the window
        for i in (start_index + 1)..=index {
            let current = &data[i].data;
            let previous = &data[i - 1].data;

            // True Range = max(high - low, |high - prev_close|, |low - prev_close|)
            let tr1 = current.high - current.low;
            let tr2 = (current.high - previous.close).abs();
            let tr3 = (current.low - previous.close).abs();

            let tr = tr1.max(tr2).max(tr3);
            true_ranges.push(tr);
        }

        // Calculate average
        let sum: f64 = true_ranges.iter().sum();
        Some(sum / true_ranges.len() as f64)
    }

    /// Detect volatility contraction
    fn is_volatility_contracting(&self, data: &[MinuteOhlcv], index: usize) -> bool {
        if index < self.lookback_period + self.contraction_period {
            return false;
        }

        // Calculate current ATR
        let current_atr = match self.calculate_atr(data, index) {
            Some(atr) => atr,
            None => return false,
        };

        // Check if ATR has been decreasing for contraction_period
        for i in 1..=self.contraction_period {
            let previous_atr = match self.calculate_atr(data, index - i) {
                Some(atr) => atr,
                None => return false,
            };

            // If any previous ATR is lower than current, not contracting
            if previous_atr <= current_atr {
                return false;
            }
        }

        true
    }

    /// Calculate breakout levels
    fn calculate_breakout_levels(&self, data: &[MinuteOhlcv], index: usize) -> Option<(f64, f64)> {
        if index < self.lookback_period {
            return None;
        }

        // Find highest high and lowest low in lookback period
        let start_index = index - self.lookback_period;
        let mut highest_high = data[start_index].data.high;
        let mut lowest_low = data[start_index].data.low;

        for i in (start_index + 1)..=index {
            highest_high = highest_high.max(data[i].data.high);
            lowest_low = lowest_low.min(data[i].data.low);
        }

        // Calculate range and breakout levels
        let range = highest_high - lowest_low;
        let extended_range = range * self.breakout_multiplier;

        let upper_breakout = highest_high + (extended_range - range) / 2.0;
        let lower_breakout = lowest_low - (extended_range - range) / 2.0;

        Some((upper_breakout, lower_breakout))
    }
}

impl IntradayStrategy for VolatilityBreakoutStrategy {
    fn name(&self) -> &str {
        &self.name
    }

    fn generate_signals(&self, data: &[MinuteOhlcv]) -> Result<Vec<Signal>, TradeError> {
        if data.len() < self.lookback_period + self.contraction_period + 1 {
            return Err(TradeError::InsufficientData(format!(
                "Need at least {} data points for volatility breakout strategy",
                self.lookback_period + self.contraction_period + 1
            )));
        }

        let mut signals = Vec::with_capacity(data.len());

        // Fill initial periods with Hold signals
        for _ in 0..self.lookback_period + self.contraction_period {
            signals.push(Signal::Hold);
        }

        // Track if we're in a setup state (detected volatility contraction)
        let mut in_setup = false;
        let mut breakout_levels: Option<(f64, f64)> = None;

        // Generate signals for the remaining data points
        for i in (self.lookback_period + self.contraction_period)..data.len() {
            let current = &data[i].data;

            // If not in setup state, check for volatility contraction
            if !in_setup {
                in_setup = self.is_volatility_contracting(data, i - 1);
                if in_setup {
                    // Calculate breakout levels when entering setup
                    breakout_levels = self.calculate_breakout_levels(data, i - 1);
                }
                signals.push(Signal::Hold);
                continue;
            }

            // In setup state, check for breakouts
            if let Some((upper_level, lower_level)) = breakout_levels {
                if current.close > upper_level {
                    // Upside breakout
                    signals.push(Signal::Buy);
                    in_setup = false;
                    breakout_levels = None;
                } else if current.close < lower_level {
                    // Downside breakout
                    signals.push(Signal::Sell);
                    in_setup = false;
                    breakout_levels = None;
                } else {
                    // No breakout yet
                    signals.push(Signal::Hold);
                }
            } else {
                // Fallback in case breakout levels couldn't be calculated
                signals.push(Signal::Hold);
                in_setup = false;
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
    fn test_volatility_breakout_parameters() {
        // Test valid parameters
        let strategy = VolatilityBreakoutStrategy::new(20, 5, 1.5);
        assert!(strategy.is_ok());

        // Test invalid lookback period
        let strategy = VolatilityBreakoutStrategy::new(2, 5, 1.5);
        assert!(strategy.is_err());

        // Test invalid contraction period
        let strategy = VolatilityBreakoutStrategy::new(20, 1, 1.5);
        assert!(strategy.is_err());

        // Test invalid contraction vs lookback
        let strategy = VolatilityBreakoutStrategy::new(10, 10, 1.5);
        assert!(strategy.is_err());

        // Test invalid breakout multiplier
        let strategy = VolatilityBreakoutStrategy::new(20, 5, 0.5);
        assert!(strategy.is_err());
    }

    #[test]
    fn test_atr_calculation() {
        let data = create_test_data(100);
        let strategy = VolatilityBreakoutStrategy::new(10, 3, 1.5).unwrap();

        // Test with valid index
        let atr = strategy.calculate_atr(&data, 20);
        assert!(atr.is_some());

        // Test with invalid index
        let atr = strategy.calculate_atr(&data, 5);
        assert!(atr.is_none());
    }

    #[test]
    fn test_signal_generation() {
        let data = create_test_data(100);
        let strategy = VolatilityBreakoutStrategy::new(10, 3, 1.5).unwrap();

        let signals = strategy.generate_signals(&data).unwrap();

        // Check that we have the correct number of signals
        assert_eq!(signals.len(), data.len());

        // Check that the first signals are Hold
        let min_holds = strategy.lookback_period() + strategy.contraction_period();
        for i in 0..min_holds {
            assert_eq!(signals[i], Signal::Hold);
        }
    }
}
