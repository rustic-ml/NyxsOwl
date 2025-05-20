//! Mean Reversion Strategy based on statistical reversion to the mean
//!
//! Uses Bollinger Bands to identify overbought and oversold conditions.
//! This strategy assumes prices tend to revert to their mean over time.
//!
//! # Theory
//!
//! Mean reversion trading is based on the concept that asset prices tend to return to their average
//! level over time. This strategy uses Bollinger Bands to identify when prices are statistically
//! overextended in either direction.
//!
//! The strategy generates:
//! - Buy signals when price is near the lower band (oversold)
//! - Sell signals when price is near the upper band (overbought)
//! - Hold signals when price is between thresholds
//!
//! # Example
//!
//! ```
//! use day_trade::{MeanReversionStrategy, TradingStrategy};
//! use day_trade::utils::generate_test_data;
//!
//! // Create a mean reversion strategy with custom parameters
//! let strategy = MeanReversionStrategy::new(20, 2.0, 0.05, 0.95).unwrap();
//!
//! // Generate test data and trading signals
//! let data = generate_test_data(100, 100.0, 0.05);
//! let signals = strategy.generate_signals(&data).unwrap();
//!
//! // Calculate performance
//! let performance = strategy.calculate_performance(&data, &signals).unwrap();
//! println!("Strategy performance: {}%", performance);
//! ```

use crate::utils::{
    calculate_basic_performance, validate_period, validate_positive, validate_range,
};
use crate::{DailyOhlcv, Signal, TradeError, TradingStrategy};
use trade_math::volatility::BollingerBands;

/// Mean reversion strategy parameters
#[derive(Debug, Clone)]
pub struct MeanReversionStrategy {
    /// Bollinger Bands period
    period: usize,
    /// Standard deviation multiplier for Bollinger Bands
    std_dev_multiplier: f64,
    /// Threshold for %B to generate buy signal (oversold)
    oversold_threshold: f64,
    /// Threshold for %B to generate sell signal (overbought)
    overbought_threshold: f64,
}

impl MeanReversionStrategy {
    /// Create a new mean reversion strategy with given parameters
    ///
    /// # Arguments
    ///
    /// * `period` - Number of periods for Bollinger Bands calculation (minimum 2)
    /// * `std_dev_multiplier` - Standard deviation multiplier for band width (typically 2.0)
    /// * `oversold_threshold` - %B threshold for oversold conditions (0.0-1.0, e.g., 0.1)
    /// * `overbought_threshold` - %B threshold for overbought conditions (0.0-1.0, e.g., 0.9)
    ///
    /// # Returns
    ///
    /// * `Result<Self, String>` - New strategy instance or error message
    pub fn new(
        period: usize,
        std_dev_multiplier: f64,
        oversold_threshold: f64,
        overbought_threshold: f64,
    ) -> Result<Self, String> {
        validate_period(period, 2)?;
        validate_positive(std_dev_multiplier, "Standard deviation multiplier")?;
        validate_range(oversold_threshold, 0.0, 1.0, "Oversold threshold")?;
        validate_range(overbought_threshold, 0.0, 1.0, "Overbought threshold")?;

        if oversold_threshold >= overbought_threshold {
            return Err("Oversold threshold must be less than overbought threshold".to_string());
        }

        Ok(Self {
            period,
            std_dev_multiplier,
            oversold_threshold,
            overbought_threshold,
        })
    }

    /// Create a default mean reversion strategy
    ///
    /// Uses period=20, std_dev_multiplier=2.0, oversold_threshold=0.1, overbought_threshold=0.9
    pub fn default() -> Self {
        Self {
            period: 20,
            std_dev_multiplier: 2.0,
            oversold_threshold: 0.1,
            overbought_threshold: 0.9,
        }
    }

    /// Get the Bollinger Bands period
    pub fn period(&self) -> usize {
        self.period
    }

    /// Get the standard deviation multiplier
    pub fn std_dev_multiplier(&self) -> f64 {
        self.std_dev_multiplier
    }

    /// Get the oversold threshold
    pub fn oversold_threshold(&self) -> f64 {
        self.oversold_threshold
    }

    /// Get the overbought threshold
    pub fn overbought_threshold(&self) -> f64 {
        self.overbought_threshold
    }
}

impl TradingStrategy for MeanReversionStrategy {
    fn generate_signals(&self, data: &[DailyOhlcv]) -> Result<Vec<Signal>, TradeError> {
        if data.len() < self.period {
            return Err(TradeError::InsufficientData(format!(
                "Need at least {} data points for mean reversion strategy",
                self.period
            )));
        }

        let mut signals = Vec::with_capacity(data.len());
        let mut bb = BollingerBands::new(self.period, self.std_dev_multiplier)
            .map_err(|e| TradeError::CalculationError(e.to_string()))?;

        // Initialize with initial data points without generating signals
        for i in 0..self.period {
            bb.update(data[i].data.close)
                .map_err(|e| TradeError::CalculationError(e.to_string()))?;
            signals.push(Signal::Hold);
        }

        // Generate signals for the rest of the data
        for i in self.period..data.len() {
            bb.update(data[i].data.close)
                .map_err(|e| TradeError::CalculationError(e.to_string()))?;

            let percent_b = bb
                .percent_b(data[i].data.close)
                .map_err(|e| TradeError::CalculationError(e.to_string()))?;

            let signal = if percent_b <= self.oversold_threshold {
                Signal::Buy
            } else if percent_b >= self.overbought_threshold {
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
        data: &[DailyOhlcv],
        signals: &[Signal],
    ) -> Result<f64, TradeError> {
        calculate_basic_performance(data, signals, 10000.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::generate_test_data;

    #[test]
    fn test_mean_reversion_parameters() {
        // Test valid parameters
        let strategy = MeanReversionStrategy::new(20, 2.0, 0.1, 0.9);
        assert!(strategy.is_ok());

        // Test invalid period
        let strategy = MeanReversionStrategy::new(1, 2.0, 0.1, 0.9);
        assert!(strategy.is_err());

        // Test invalid std_dev_multiplier
        let strategy = MeanReversionStrategy::new(20, 0.0, 0.1, 0.9);
        assert!(strategy.is_err());

        // Test invalid thresholds
        let strategy = MeanReversionStrategy::new(20, 2.0, 0.9, 0.1);
        assert!(strategy.is_err());

        // Test out of range thresholds
        let strategy = MeanReversionStrategy::new(20, 2.0, -0.1, 0.9);
        assert!(strategy.is_err());
        let strategy = MeanReversionStrategy::new(20, 2.0, 0.1, 1.1);
        assert!(strategy.is_err());
    }

    #[test]
    fn test_mean_reversion_signals() {
        let data = generate_test_data(100, 100.0, 0.05);
        let strategy = MeanReversionStrategy::default();

        let signals = strategy.generate_signals(&data).unwrap();

        // Check that we have the correct number of signals
        assert_eq!(signals.len(), data.len());

        // Check that initial period has hold signals
        for i in 0..strategy.period() {
            assert_eq!(signals[i], Signal::Hold);
        }

        // Check that we have some buy and sell signals
        let buy_count = signals.iter().filter(|&&s| s == Signal::Buy).count();
        let sell_count = signals.iter().filter(|&&s| s == Signal::Sell).count();

        // In a random price series, we should expect some buy and sell signals
        assert!(buy_count > 0, "Expected at least one buy signal");
        assert!(sell_count > 0, "Expected at least one sell signal");
    }

    #[test]
    fn test_performance_calculation() {
        let data = generate_test_data(100, 100.0, 0.05);
        let strategy = MeanReversionStrategy::default();

        // Create a fixed set of signals for testing performance
        let mut signals = vec![Signal::Hold; data.len()];

        // Create a pattern of buys and sells that should be profitable in an uptrend
        signals[20] = Signal::Buy; // Buy at a low point
        signals[40] = Signal::Sell; // Sell after a rise
        signals[60] = Signal::Buy; // Buy again
        signals[80] = Signal::Sell; // Sell again

        let performance = strategy.calculate_performance(&data, &signals).unwrap();

        // Performance could be positive or negative depending on the random data
        // but the calculation should succeed
        println!("Test performance: {}%", performance);
    }
}
