//! Volume-based intraday trading strategies
//!
//! This module will contain strategies that analyze volume patterns for trading signals.

// Re-export strategies
pub use self::relative_volume_strategy::RelativeVolumeStrategy;
pub use self::volume_profile_strategy::VolumeProfileStrategy;

// These will be implemented in the future
mod volume_profile_strategy {
    use crate::utils::{calculate_basic_performance, validate_period, validate_positive};
    use crate::{IntradayStrategy, MinuteOhlcv, Signal, TradeError};
    use std::collections::HashMap;

    /// Volume Profile Strategy for identifying high volume price levels (support/resistance)
    #[derive(Debug, Clone)]
    pub struct VolumeProfileStrategy {
        /// Period for volume profile calculation
        lookback_period: usize,
        /// Number of price levels to use in profile
        num_price_levels: usize,
        /// Volume threshold for significant levels (percentage of total)
        volume_threshold: f64,
        /// Strategy name
        name: String,
    }

    impl VolumeProfileStrategy {
        /// Create a new Volume Profile strategy
        ///
        /// # Arguments
        ///
        /// * `lookback_period` - Period to calculate volume profile (typically 60-240 minutes)
        /// * `num_price_levels` - Number of price levels to divide the range (typically 10-50)
        /// * `volume_threshold` - Threshold for significant volumes (0.0-1.0, as percentage of total)
        ///
        /// # Returns
        ///
        /// * `Result<Self, String>` - New strategy instance or error message
        pub fn new(
            lookback_period: usize,
            num_price_levels: usize,
            volume_threshold: f64,
        ) -> Result<Self, String> {
            validate_period(lookback_period, 20)?;

            if num_price_levels < 5 {
                return Err(
                    "Number of price levels should be at least 5 for meaningful analysis"
                        .to_string(),
                );
            }

            validate_positive(volume_threshold, "Volume threshold")?;
            if volume_threshold > 0.5 {
                return Err("Volume threshold seems too high (>0.5). Typical values are 0.05-0.2 (5-20% of total volume).".to_string());
            }

            Ok(Self {
                lookback_period,
                num_price_levels,
                volume_threshold,
                name: format!(
                    "Volume Profile ({}, {}, {:.0}%)",
                    lookback_period,
                    num_price_levels,
                    volume_threshold * 100.0
                ),
            })
        }

        /// Get the lookback period
        pub fn lookback_period(&self) -> usize {
            self.lookback_period
        }

        /// Get the number of price levels
        pub fn num_price_levels(&self) -> usize {
            self.num_price_levels
        }

        /// Get the volume threshold
        pub fn volume_threshold(&self) -> f64 {
            self.volume_threshold
        }

        /// Build volume profile for price range
        fn build_volume_profile(
            &self,
            data: &[MinuteOhlcv],
            index: usize,
        ) -> Option<(HashMap<usize, f64>, f64, f64, f64)> {
            if index < self.lookback_period {
                return None;
            }

            let start_index = index - self.lookback_period;

            // Find min and max prices in the lookback period
            let mut min_price = data[start_index].data.low;
            let mut max_price = data[start_index].data.high;
            let mut total_volume = 0.0;

            for i in start_index..=index {
                min_price = min_price.min(data[i].data.low);
                max_price = max_price.max(data[i].data.high);
                total_volume += data[i].data.volume;
            }

            let price_range = max_price - min_price;
            if price_range <= 0.0 {
                return None; // Avoid division by zero
            }

            // Size of each price level
            let level_size = price_range / self.num_price_levels as f64;

            // Allocate volume to price levels
            let mut profile: HashMap<usize, f64> = HashMap::new();

            for i in start_index..=index {
                let candle = &data[i].data;
                let candle_range = candle.high - candle.low;

                if candle_range <= 0.0 {
                    // For point candles, allocate to single level
                    let level = ((candle.close - min_price) / level_size).floor() as usize;
                    let level = level.min(self.num_price_levels - 1); // Ensure within bounds

                    *profile.entry(level).or_insert(0.0) += candle.volume;
                } else {
                    // Distribute volume proportionally across levels the candle spans
                    let low_level = ((candle.low - min_price) / level_size).floor() as usize;
                    let high_level = ((candle.high - min_price) / level_size).floor() as usize;
                    let low_level = low_level.min(self.num_price_levels - 1);
                    let high_level = high_level.min(self.num_price_levels - 1);

                    let levels_spanned = (high_level - low_level) + 1;

                    // Simple approach: divide volume equally among levels
                    let volume_per_level = candle.volume / levels_spanned as f64;

                    for level in low_level..=high_level {
                        *profile.entry(level).or_insert(0.0) += volume_per_level;
                    }
                }
            }

            Some((profile, min_price, level_size, total_volume))
        }

        /// Identify high volume nodes (HVNs) in the profile
        fn find_high_volume_nodes(
            &self,
            profile: &HashMap<usize, f64>,
            total_volume: f64,
            min_price: f64,
            level_size: f64,
        ) -> Vec<f64> {
            let mut hvn_prices = Vec::new();
            let threshold = total_volume * self.volume_threshold;

            for (level, volume) in profile {
                if *volume >= threshold {
                    // Convert level back to price (use middle of the level)
                    let level_price = min_price + (*level as f64 + 0.5) * level_size;
                    hvn_prices.push(level_price);
                }
            }

            // Sort by price
            hvn_prices.sort_by(|a, b| a.partial_cmp(b).unwrap());

            hvn_prices
        }
    }

    impl IntradayStrategy for VolumeProfileStrategy {
        fn name(&self) -> &str {
            &self.name
        }

        fn generate_signals(&self, data: &[MinuteOhlcv]) -> Result<Vec<Signal>, TradeError> {
            if data.len() < self.lookback_period + 1 {
                return Err(TradeError::InsufficientData(format!(
                    "Need at least {} data points for volume profile strategy",
                    self.lookback_period + 1
                )));
            }

            let mut signals = Vec::with_capacity(data.len());

            // First entries are hold signals due to insufficient data
            for _ in 0..self.lookback_period {
                signals.push(Signal::Hold);
            }

            // Track position state
            let mut in_long = false;
            let mut in_short = false;
            let mut support_level = 0.0;
            let mut resistance_level = 0.0;

            // Generate signals for the remaining data points
            for i in self.lookback_period..data.len() {
                let current = &data[i].data;

                // Build volume profile
                let profile_result = self.build_volume_profile(data, i - 1); // Use up to previous candle

                let signal = if let Some((profile, min_price, level_size, total_volume)) =
                    profile_result
                {
                    // Find high volume nodes
                    let hvn_prices =
                        self.find_high_volume_nodes(&profile, total_volume, min_price, level_size);

                    if hvn_prices.is_empty() {
                        Signal::Hold
                    } else {
                        // Find closest HVN below and above current price
                        let mut closest_below = None;
                        let mut closest_above = None;

                        for price in &hvn_prices {
                            if *price < current.close {
                                if closest_below.is_none() || *price > closest_below.unwrap() {
                                    closest_below = Some(*price);
                                }
                            } else if *price > current.close
                                && (closest_above.is_none() || *price < closest_above.unwrap())
                            {
                                closest_above = Some(*price);
                            }
                        }

                        // Update support/resistance levels
                        if let Some(below) = closest_below {
                            support_level = below;
                        }

                        if let Some(above) = closest_above {
                            resistance_level = above;
                        }

                        // Trading logic
                        if !in_long && !in_short {
                            // Not in a position - look for signals
                            if let Some(support) = closest_below {
                                // If price is close to support, buy
                                if (current.close - support) / support < 0.003 {
                                    // Within 0.3%
                                    in_long = true;
                                    Signal::Buy
                                } else if let Some(resistance) = closest_above {
                                    // If price is close to resistance, sell
                                    if (resistance - current.close) / current.close < 0.003 {
                                        // Within 0.3%
                                        in_short = true;
                                        Signal::Sell
                                    } else {
                                        Signal::Hold
                                    }
                                } else {
                                    Signal::Hold
                                }
                            } else {
                                Signal::Hold
                            }
                        } else if in_long {
                            // In long position - check for exit at resistance
                            if let Some(resistance) = closest_above {
                                if current.close >= resistance {
                                    in_long = false;
                                    Signal::Sell // Take profit at resistance
                                } else {
                                    Signal::Hold
                                }
                            } else {
                                Signal::Hold
                            }
                        } else {
                            // in_short
                            // In short position - check for exit at support
                            if let Some(support) = closest_below {
                                if current.close <= support {
                                    in_short = false;
                                    Signal::Buy // Take profit at support
                                } else {
                                    Signal::Hold
                                }
                            } else {
                                Signal::Hold
                            }
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
            let commission = 0.03; // 0.03% per trade
            calculate_basic_performance(data, signals, 10000.0, commission)
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::tests::create_test_data;

        #[test]
        fn test_volume_profile_parameters() {
            // Test valid parameters
            let strategy = VolumeProfileStrategy::new(60, 20, 0.1);
            assert!(strategy.is_ok());

            // Test invalid lookback period
            let strategy = VolumeProfileStrategy::new(10, 20, 0.1);
            assert!(strategy.is_err());

            // Test invalid price levels
            let strategy = VolumeProfileStrategy::new(60, 3, 0.1);
            assert!(strategy.is_err());

            // Test invalid volume threshold
            let strategy = VolumeProfileStrategy::new(60, 20, 0.0);
            assert!(strategy.is_err());

            // Test volume threshold warning
            let strategy = VolumeProfileStrategy::new(60, 20, 0.7);
            assert!(strategy.is_err());
        }

        #[test]
        fn test_volume_profile_building() {
            let strategy = VolumeProfileStrategy::new(60, 10, 0.1).unwrap();
            let data = create_test_data(100);

            let profile_result = strategy.build_volume_profile(&data, 80);
            assert!(
                profile_result.is_some(),
                "Volume profile should be built successfully"
            );

            if let Some((profile, min_price, level_size, total_volume)) = profile_result {
                // Basic validation
                assert!(!profile.is_empty(), "Profile should not be empty");
                assert!(min_price > 0.0, "Min price should be positive");
                assert!(level_size > 0.0, "Level size should be positive");
                assert!(total_volume > 0.0, "Total volume should be positive");
            }
        }

        #[test]
        fn test_signal_generation() {
            let data = create_test_data(120);
            let strategy = VolumeProfileStrategy::new(60, 10, 0.1).unwrap();

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

mod relative_volume_strategy {
    use crate::{IntradayStrategy, MinuteOhlcv, Signal, TradeError};

    /// Placeholder for the Relative Volume Strategy
    #[derive(Debug, Clone)]
    pub struct RelativeVolumeStrategy;

    impl Default for RelativeVolumeStrategy {
        fn default() -> Self {
            Self::new()
        }
    }

    impl RelativeVolumeStrategy {
        /// Create a new instance (placeholder)
        pub fn new() -> Self {
            Self
        }
    }

    impl IntradayStrategy for RelativeVolumeStrategy {
        fn name(&self) -> &str {
            "Relative Volume Strategy (placeholder)"
        }

        fn generate_signals(&self, data: &[MinuteOhlcv]) -> Result<Vec<Signal>, TradeError> {
            let signals = vec![Signal::Hold; data.len()];
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
