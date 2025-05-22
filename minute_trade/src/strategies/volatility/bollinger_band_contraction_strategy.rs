//! Bollinger Band Contraction Strategy for intraday trading
//!
//! This strategy trades volatility expansions that follow periods of tight
//! Bollinger Band contraction, indicating potential explosive price moves.
//!
//! # Strategy Logic
//!
//! The Bollinger Band contraction strategy:
//! 1. Monitors Bollinger Band width to identify contractions
//! 2. Sets up for trades when bands are unusually narrow
//! 3. Enters positions in the direction of the initial breakout
//! 4. Uses volatility-based profit targets
//!
//! # Example
//!
//! ```no_run
//! use intraday_trade::{BollingerBandContractionStrategy, IntradayStrategy};
//! use intraday_trade::utils::generate_minute_data;
//!
//! // Create a Bollinger Band contraction strategy
//! let strategy = BollingerBandContractionStrategy::new(20, 2.0, 0.3).unwrap();
//!
//! // Generate test data
//! let data = generate_minute_data(5, 390, 100.0, 0.01, 0.0);
//!
//! // Generate trading signals
//! let signals = strategy.generate_signals(&data).unwrap();
//! ```

use crate::utils::{
    calculate_basic_performance, calculate_bollinger_bands, validate_period, validate_positive,
};
use crate::{IntradayStrategy, MinuteOhlcv, Signal, TradeError};

/// Bollinger Band Contraction Strategy for trading volatility breakouts
#[derive(Debug, Clone)]
pub struct BollingerBandContractionStrategy {
    /// Period for Bollinger Band calculation
    bollinger_period: usize,
    /// Standard deviation multiplier for Bollinger Bands
    std_dev_multiplier: f64,
    /// Threshold for band width to trigger contraction (as percentage of price)
    contraction_threshold: f64,
    /// Strategy name
    name: String,
}

impl BollingerBandContractionStrategy {
    /// Create a new Bollinger Band contraction strategy
    ///
    /// # Arguments
    ///
    /// * `bollinger_period` - Period for Bollinger Band calculation (typically 20 minutes)
    /// * `std_dev_multiplier` - Standard deviation multiplier (typically 2.0)
    /// * `contraction_threshold` - Band contraction threshold as percentage (e.g., 0.3 = 0.3% of price)
    ///
    /// # Returns
    ///
    /// * `Result<Self, String>` - New strategy instance or error message
    pub fn new(
        bollinger_period: usize,
        std_dev_multiplier: f64,
        contraction_threshold: f64,
    ) -> Result<Self, String> {
        validate_period(bollinger_period, 5)?;
        validate_positive(std_dev_multiplier, "Standard deviation multiplier")?;
        validate_positive(contraction_threshold, "Contraction threshold")?;

        if !(1.0..=3.0).contains(&std_dev_multiplier) {
            return Err(
                "Standard deviation multiplier should typically be between 1.0 and 3.0".to_string(),
            );
        }

        if contraction_threshold > 2.0 {
            return Err("Contraction threshold seems too high (>2%). For intraday trading, values between 0.2% and 1.0% are typical.".to_string());
        }

        Ok(Self {
            bollinger_period,
            std_dev_multiplier,
            contraction_threshold,
            name: format!(
                "Bollinger Contraction ({}, {}σ, {}%)",
                bollinger_period,
                std_dev_multiplier,
                contraction_threshold * 100.0
            ),
        })
    }

    /// Get the Bollinger Band period
    pub fn bollinger_period(&self) -> usize {
        self.bollinger_period
    }

    /// Get the standard deviation multiplier
    pub fn std_dev_multiplier(&self) -> f64 {
        self.std_dev_multiplier
    }

    /// Get the contraction threshold
    pub fn contraction_threshold(&self) -> f64 {
        self.contraction_threshold
    }

    /// Calculate Bollinger Band width as percentage of the middle band
    fn calculate_band_width(&self, upper: f64, middle: f64, lower: f64) -> f64 {
        if middle <= 0.0 {
            return 0.0; // Avoid division by zero
        }

        ((upper - lower) / middle) * 100.0
    }

    /// Check if bands are contracted based on threshold
    fn is_contracted(&self, band_width: f64) -> bool {
        band_width < self.contraction_threshold * 100.0
    }
}

impl IntradayStrategy for BollingerBandContractionStrategy {
    fn name(&self) -> &str {
        &self.name
    }

    fn generate_signals(&self, data: &[MinuteOhlcv]) -> Result<Vec<Signal>, TradeError> {
        if data.len() < self.bollinger_period + 1 {
            return Err(TradeError::InsufficientData(format!(
                "Need at least {} data points for Bollinger Band contraction strategy",
                self.bollinger_period + 1
            )));
        }

        let mut signals = Vec::with_capacity(data.len());

        // Extract close prices
        let closes: Vec<f64> = data.iter().map(|d| d.data.close).collect();

        // Calculate Bollinger Bands
        let (upper_band, middle_band, lower_band) =
            calculate_bollinger_bands(&closes, self.bollinger_period, self.std_dev_multiplier);

        // First entries are hold signals due to insufficient data
        for _ in 0..self.bollinger_period {
            signals.push(Signal::Hold);
        }

        // Tracking variables for strategy state
        let mut last_contracted = false;
        let mut breakout_direction: Option<Signal> = None;

        // Generate signals for the remaining data points
        for i in self.bollinger_period..data.len() {
            let current_price = closes[i];

            // Get current Bollinger Band values
            let upper = match upper_band[i] {
                Some(val) => val,
                None => {
                    signals.push(Signal::Hold);
                    continue;
                }
            };

            let middle = match middle_band[i] {
                Some(val) => val,
                None => {
                    signals.push(Signal::Hold);
                    continue;
                }
            };

            let lower = match lower_band[i] {
                Some(val) => val,
                None => {
                    signals.push(Signal::Hold);
                    continue;
                }
            };

            // Calculate band width
            let band_width = self.calculate_band_width(upper, middle, lower);
            let is_contracted = self.is_contracted(band_width);

            // Strategy logic
            let mut signal = Signal::Hold;

            if !last_contracted && is_contracted {
                // Just entered contraction state
                breakout_direction = None;
            } else if last_contracted && !is_contracted {
                // Just exited contraction - potential breakout
                if current_price > middle {
                    // Upside breakout
                    signal = Signal::Buy;
                    breakout_direction = Some(Signal::Buy);
                } else if current_price < middle {
                    // Downside breakout
                    signal = Signal::Sell;
                    breakout_direction = Some(Signal::Sell);
                }
            } else if let Some(direction) = breakout_direction {
                // Already in a breakout trade
                if (direction == Signal::Buy && current_price < middle)
                    || (direction == Signal::Sell && current_price > middle)
                {
                    // Price crossed middle band in opposite direction - exit
                    signal = match direction {
                        Signal::Buy => Signal::Sell,
                        Signal::Sell => Signal::Buy,
                        _ => Signal::Hold,
                    };
                    breakout_direction = None;
                }
            }

            // Update contraction state for next iteration
            last_contracted = is_contracted;

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
        let commission = 0.03; // 0.03% per trade
        calculate_basic_performance(data, signals, 10000.0, commission)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::create_test_data;

    #[test]
    fn test_bollinger_parameters() {
        // Test valid parameters
        let strategy = BollingerBandContractionStrategy::new(20, 2.0, 0.3);
        assert!(strategy.is_ok());

        // Test invalid bollinger period
        let strategy = BollingerBandContractionStrategy::new(2, 2.0, 0.3);
        assert!(strategy.is_err());

        // Test invalid standard deviation
        let strategy = BollingerBandContractionStrategy::new(20, 0.0, 0.3);
        assert!(strategy.is_err());

        // Test standard deviation warning
        let strategy = BollingerBandContractionStrategy::new(20, 4.0, 0.3);
        assert!(strategy.is_err());

        // Test invalid contraction threshold
        let strategy = BollingerBandContractionStrategy::new(20, 2.0, 0.0);
        assert!(strategy.is_err());

        // Test contraction threshold warning
        let strategy = BollingerBandContractionStrategy::new(20, 2.0, 3.0);
        assert!(strategy.is_err());
    }

    #[test]
    fn test_band_width_calculation() {
        let strategy = BollingerBandContractionStrategy::new(20, 2.0, 0.3).unwrap();

        // Test normal case
        let width = strategy.calculate_band_width(110.0, 100.0, 90.0);
        assert_eq!(width, 20.0); // (110-90)/100 * 100 = 20%

        // Test zero middle
        let width = strategy.calculate_band_width(10.0, 0.0, -10.0);
        assert_eq!(width, 0.0); // Should handle division by zero
    }

    #[test]
    fn test_contraction_detection() {
        let strategy = BollingerBandContractionStrategy::new(20, 2.0, 0.3).unwrap();

        // Test below threshold (contracted)
        let contracted = strategy.is_contracted(0.2);
        assert!(contracted);

        // Test above threshold (not contracted)
        let contracted = strategy.is_contracted(0.5);
        assert!(!contracted);

        // Test at threshold
        let contracted = strategy.is_contracted(0.3 * 100.0);
        assert!(!contracted); // Should be strictly less than
    }

    #[test]
    fn test_signal_generation() {
        let data = create_test_data(100);
        let strategy = BollingerBandContractionStrategy::new(20, 2.0, 0.3).unwrap();

        let signals = strategy.generate_signals(&data).unwrap();

        // Check that we have the correct number of signals
        assert_eq!(signals.len(), data.len());

        // Check that the first 'bollinger_period' signals are Hold
        for i in 0..strategy.bollinger_period() {
            assert_eq!(signals[i], Signal::Hold);
        }
    }
}
