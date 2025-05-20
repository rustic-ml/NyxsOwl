//! Momentum Breakout Strategy for intraday trading
//!
//! This strategy aims to capitalize on strong price movements that break through
//! established support or resistance levels with increased volume.
//!
//! # Strategy Logic
//!
//! The momentum breakout strategy works by:
//! 1. Identifying recent high and low price levels over a lookback period
//! 2. Monitoring for price breakouts above resistance or below support
//! 3. Confirming breakouts with increased trading volume
//! 4. Entering positions in the direction of the breakout
//!
//! # Example
//!
//! ```no_run
//! use intraday_trade::{MomentumBreakoutStrategy, IntradayStrategy};
//! use intraday_trade::utils::generate_minute_data;
//!
//! // Create a momentum breakout strategy with a 30-minute lookback and 1.5 volume threshold
//! let strategy = MomentumBreakoutStrategy::new(30, 1.5).unwrap();
//!
//! // Generate test data
//! let data = generate_minute_data(5, 390, 100.0, 0.03, 0.0);
//!
//! // Generate trading signals
//! let signals = strategy.generate_signals(&data).unwrap();
//!
//! // Calculate performance
//! let performance = strategy.calculate_performance(&data, &signals).unwrap();
//! println!("Momentum breakout strategy performance: {}%", performance);
//! ```

use crate::utils::{calculate_basic_performance, validate_period, validate_positive};
use crate::{IntradayStrategy, MinuteOhlcv, Signal, TradeError};

/// Momentum Breakout Strategy for capturing strong price movements
#[derive(Debug, Clone)]
pub struct MomentumBreakoutStrategy {
    /// Lookback period for establishing support/resistance
    period: usize,
    /// Volume threshold multiplier to confirm breakouts
    volume_threshold: f64,
    /// Strategy name
    name: String,
}

impl MomentumBreakoutStrategy {
    /// Create a new momentum breakout strategy
    ///
    /// # Arguments
    ///
    /// * `period` - Lookback period for establishing support/resistance (typically 15-60 minutes)
    /// * `volume_threshold` - Volume multiplier to confirm breakouts (e.g., 1.5 means 150% of average volume)
    ///
    /// # Returns
    ///
    /// * `Result<Self, String>` - New strategy instance or error message
    pub fn new(period: usize, volume_threshold: f64) -> Result<Self, String> {
        validate_period(period, 5)?;
        validate_positive(volume_threshold, "Volume threshold")?;

        Ok(Self {
            period,
            volume_threshold,
            name: format!("Momentum Breakout ({}m, {}x vol)", period, volume_threshold),
        })
    }

    /// Get the lookback period
    pub fn period(&self) -> usize {
        self.period
    }

    /// Get the volume threshold multiplier
    pub fn volume_threshold(&self) -> f64 {
        self.volume_threshold
    }
}

impl IntradayStrategy for MomentumBreakoutStrategy {
    fn name(&self) -> &str {
        &self.name
    }

    fn generate_signals(&self, data: &[MinuteOhlcv]) -> Result<Vec<Signal>, TradeError> {
        if data.len() < self.period + 1 {
            return Err(TradeError::InsufficientData(format!(
                "Need at least {} data points for momentum breakout strategy",
                self.period + 1
            )));
        }

        let mut signals = Vec::with_capacity(data.len());

        // First entries are hold signals due to insufficient data
        for _ in 0..self.period {
            signals.push(Signal::Hold);
        }

        // Generate signals for the remaining data points
        for i in self.period..data.len() {
            // Get the lookback window
            let window = &data[i - self.period..i];

            // Calculate the highest high and lowest low in the lookback period
            let highest_high = window
                .iter()
                .map(|d| d.data.high)
                .fold(f64::NEG_INFINITY, f64::max);
            let lowest_low = window
                .iter()
                .map(|d| d.data.low)
                .fold(f64::INFINITY, f64::min);

            // Calculate average volume in the lookback period
            let avg_volume = window.iter().map(|d| d.data.volume).sum::<f64>() / self.period as f64;

            // Current candlestick data
            let current = &data[i].data;

            // Check for breakouts with volume confirmation
            let signal = if current.close > highest_high
                && current.volume > avg_volume * self.volume_threshold
            {
                // Upside breakout with volume confirmation
                Signal::Buy
            } else if current.close < lowest_low
                && current.volume > avg_volume * self.volume_threshold
            {
                // Downside breakout with volume confirmation
                Signal::Sell
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
        // Standard commission for momentum trading
        let commission = 0.05; // 0.05% per trade
        calculate_basic_performance(data, signals, 10000.0, commission)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::create_test_data;

    #[test]
    fn test_momentum_breakout_params() {
        // Test valid parameters
        let strategy = MomentumBreakoutStrategy::new(30, 1.5);
        assert!(strategy.is_ok());

        // Test invalid period
        let strategy = MomentumBreakoutStrategy::new(4, 1.5);
        assert!(strategy.is_err());

        // Test invalid volume threshold
        let strategy = MomentumBreakoutStrategy::new(30, 0.0);
        assert!(strategy.is_err());
    }

    #[test]
    fn test_momentum_breakout_signals() {
        let data = create_test_data(200);
        let strategy = MomentumBreakoutStrategy::new(20, 1.2).unwrap();

        let signals = strategy.generate_signals(&data).unwrap();

        // Check that we have the correct number of signals
        assert_eq!(signals.len(), data.len());

        // Check that the first 'period' signals are Hold
        for i in 0..strategy.period() {
            assert_eq!(signals[i], Signal::Hold);
        }

        // Since this is a breakout strategy, it should have fewer signals
        // than a scalping strategy, but we should still have some
        let action_count = signals.iter().filter(|&&s| s != Signal::Hold).count();

        println!("Momentum breakout signals generated: {}", action_count);
    }

    #[test]
    fn test_performance_calculation() {
        let data = create_test_data(200);
        let strategy = MomentumBreakoutStrategy::new(20, 1.2).unwrap();

        let signals = strategy.generate_signals(&data).unwrap();
        let performance = strategy.calculate_performance(&data, &signals).unwrap();

        // Just check that the calculation completes successfully
        println!("Test momentum breakout performance: {}%", performance);
    }
}
