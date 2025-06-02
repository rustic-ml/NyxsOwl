//! Volume-based intraday trading strategies
//!
//! This module will contain strategies that analyze volume patterns for trading signals.

// Re-export strategies
pub use self::relative_volume_strategy::RelativeVolumeStrategy;
pub use self::volume_profile_strategy::VolumeProfileStrategy;

// These will be implemented in the future
mod volume_profile_strategy {
    use crate::minute_trade::utils::{
        calculate_basic_performance, validate_period, validate_positive,
    };
    use crate::minute_trade::{IntradayStrategy, MinuteOhlcv, Signal, TradeError};
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
            up_to_index: usize,
        ) -> Option<(HashMap<usize, f64>, f64, f64, f64)> {
            if up_to_index < self.lookback_period {
                return None;
            }

            let start_index = up_to_index - self.lookback_period;

            // Find min and max prices in the lookback period
            let mut min_price = data[start_index].data.low;
            let mut max_price = data[start_index].data.high;
            let mut total_volume = 0.0;

            for i in start_index..=up_to_index {
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

            for i in start_index..=up_to_index {
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
            let signals = vec![Signal::Hold; data.len()];
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
    use crate::minute_trade::utils::{calculate_basic_performance, validate_period, validate_positive};
    use crate::minute_trade::{IntradayStrategy, MinuteOhlcv, Signal, TradeError};

    /// Relative Volume Strategy - trades based on volume anomalies compared to historical averages
    #[derive(Debug, Clone)]
    pub struct RelativeVolumeStrategy {
        /// Period for calculating average volume
        lookback_period: usize,
        /// Volume multiplier threshold for entry signals (e.g., 2.0 means 2x average volume)
        volume_threshold: f64,
        /// Price change threshold for confirmation (percentage)
        price_change_threshold: f64,
        /// Strategy name
        name: String,
    }

    impl Default for RelativeVolumeStrategy {
        fn default() -> Self {
            Self::new(20, 2.0, 0.5).unwrap()
        }
    }

    impl RelativeVolumeStrategy {
        /// Create a new Relative Volume Strategy
        ///
        /// # Arguments
        ///
        /// * `lookback_period` - Period for calculating average volume (typically 20-60)
        /// * `volume_threshold` - Volume multiplier for signal generation (typically 1.5-3.0)
        /// * `price_change_threshold` - Minimum price change percentage for confirmation (typically 0.3-1.0)
        ///
        /// # Returns
        ///
        /// * `Result<Self, String>` - New strategy instance or error message
        pub fn new(
            lookback_period: usize,
            volume_threshold: f64,
            price_change_threshold: f64,
        ) -> Result<Self, String> {
            validate_period(lookback_period, 10)?;
            validate_positive(volume_threshold, "Volume threshold")?;
            validate_positive(price_change_threshold, "Price change threshold")?;

            if volume_threshold < 1.2 {
                return Err("Volume threshold should be at least 1.2 to detect meaningful anomalies.".to_string());
            }

            if price_change_threshold > 5.0 {
                return Err("Price change threshold seems too high (>5%). Consider using a lower value.".to_string());
            }

            Ok(Self {
                lookback_period,
                volume_threshold,
                price_change_threshold,
                name: format!(
                    "Relative Volume ({}p, {}x vol, {}% price)",
                    lookback_period, volume_threshold, price_change_threshold
                ),
            })
        }

        /// Get the lookback period
        pub fn lookback_period(&self) -> usize {
            self.lookback_period
        }

        /// Get the volume threshold
        pub fn volume_threshold(&self) -> f64 {
            self.volume_threshold
        }

        /// Get the price change threshold
        pub fn price_change_threshold(&self) -> f64 {
            self.price_change_threshold
        }

        /// Calculate average volume over the lookback period
        fn calculate_average_volume(&self, data: &[MinuteOhlcv], end_index: usize) -> Option<f64> {
            if end_index < self.lookback_period {
                return None;
            }

            let start_index = end_index - self.lookback_period;
            let total_volume: f64 = data[start_index..end_index]
                .iter()
                .map(|d| d.data.volume)
                .sum();

            Some(total_volume / self.lookback_period as f64)
        }

        /// Calculate price change percentage
        fn calculate_price_change(&self, previous_close: f64, current_close: f64) -> f64 {
            if previous_close <= 0.0 {
                return 0.0;
            }
            ((current_close - previous_close) / previous_close) * 100.0
        }

        /// Check if volume and price conditions are met for a signal
        fn check_signal_conditions(&self, current_volume: f64, avg_volume: f64, price_change: f64) -> Option<Signal> {
            let volume_ratio = current_volume / avg_volume;
            
            if volume_ratio >= self.volume_threshold {
                let abs_price_change = price_change.abs();
                
                if abs_price_change >= self.price_change_threshold {
                    // High volume with significant price movement
                    if price_change > 0.0 {
                        // Price moving up with high volume - buy signal
                        Some(Signal::Buy)
                    } else {
                        // Price moving down with high volume - sell signal
                        Some(Signal::Sell)
                    }
                } else {
                    // High volume but insufficient price movement - hold
                    Some(Signal::Hold)
                }
            } else {
                // Normal volume - hold
                None
            }
        }
    }

    impl IntradayStrategy for RelativeVolumeStrategy {
        fn name(&self) -> &str {
            &self.name
        }

        fn generate_signals(&self, data: &[MinuteOhlcv]) -> Result<Vec<Signal>, TradeError> {
            if data.len() < self.lookback_period + 1 {
                return Err(TradeError::InsufficientData(format!(
                    "Need at least {} data points for Relative Volume strategy",
                    self.lookback_period + 1
                )));
            }

            let mut signals = Vec::with_capacity(data.len());

            // First entries are hold signals due to insufficient data
            for _ in 0..self.lookback_period {
                signals.push(Signal::Hold);
            }

            // Track position state
            let mut in_position = false;
            let mut position_is_long = false;

            // Generate signals for the remaining data points
            for i in self.lookback_period..data.len() {
                let current_candle = &data[i];
                let previous_candle = &data[i - 1];
                
                // Calculate average volume for the lookback period
                let avg_volume = self.calculate_average_volume(data, i);
                
                let signal = if let Some(avg_vol) = avg_volume {
                    let current_volume = current_candle.data.volume;
                    let price_change = self.calculate_price_change(
                        previous_candle.data.close,
                        current_candle.data.close,
                    );

                    if !in_position {
                        // No position - look for entry signals
                        if let Some(entry_signal) = self.check_signal_conditions(current_volume, avg_vol, price_change) {
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
                            Signal::Hold
                        }
                    } else {
                        // In position - look for exit conditions
                        let volume_ratio = current_volume / avg_vol;
                        
                        // Exit if volume drops significantly or price reverses
                        let should_exit = volume_ratio < (self.volume_threshold * 0.5) || 
                                        (position_is_long && price_change < -self.price_change_threshold) ||
                                        (!position_is_long && price_change > self.price_change_threshold);
                        
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
            // Use higher commission for volume-based strategies due to more frequent trading
            let commission = 0.03; // 0.03% per trade
            calculate_basic_performance(data, signals, 10000.0, commission)
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::minute_trade::create_test_data;

        #[test]
        fn test_relative_volume_parameters() {
            // Test valid parameters
            let strategy = RelativeVolumeStrategy::new(20, 2.0, 0.5);
            assert!(strategy.is_ok());

            // Test invalid lookback period
            let strategy = RelativeVolumeStrategy::new(5, 2.0, 0.5);
            assert!(strategy.is_err());

            // Test invalid volume threshold
            let strategy = RelativeVolumeStrategy::new(20, 1.0, 0.5);
            assert!(strategy.is_err());

            // Test invalid price change threshold
            let strategy = RelativeVolumeStrategy::new(20, 2.0, 6.0);
            assert!(strategy.is_err());
        }

        #[test]
        fn test_volume_calculations() {
            let strategy = RelativeVolumeStrategy::new(20, 2.0, 0.5).unwrap();
            let data = create_test_data(50);

            // Test average volume calculation
            let avg_volume = strategy.calculate_average_volume(&data, 30);
            assert!(avg_volume.is_some());
            assert!(avg_volume.unwrap() > 0.0);

            // Test insufficient data
            let avg_volume = strategy.calculate_average_volume(&data, 10);
            assert!(avg_volume.is_none());
        }

        #[test]
        fn test_signal_generation() {
            let data = create_test_data(50);
            let strategy = RelativeVolumeStrategy::new(20, 2.0, 0.5).unwrap();

            let signals = strategy.generate_signals(&data).unwrap();

            // Check that we have the correct number of signals
            assert_eq!(signals.len(), data.len());

            // Check that the first 'lookback_period' signals are Hold
            for i in 0..strategy.lookback_period() {
                assert_eq!(signals[i], Signal::Hold);
            }
        }

        #[test]
        fn test_price_change_calculation() {
            let strategy = RelativeVolumeStrategy::new(20, 2.0, 0.5).unwrap();

            // Test positive price change
            let change = strategy.calculate_price_change(100.0, 102.0);
            assert!((change - 2.0).abs() < f64::EPSILON);

            // Test negative price change
            let change = strategy.calculate_price_change(100.0, 98.0);
            assert!((change - (-2.0)).abs() < f64::EPSILON);

            // Test zero previous price
            let change = strategy.calculate_price_change(0.0, 100.0);
            assert_eq!(change, 0.0);
        }
    }
}
