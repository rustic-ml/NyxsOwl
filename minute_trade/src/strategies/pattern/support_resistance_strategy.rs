//! Support/Resistance Strategy for intraday trading
//!
//! This strategy identifies key support and resistance levels and
//! trades bounces and breakouts from these levels.
//!
//! # Strategy Logic
//!
//! The support/resistance strategy:
//! 1. Identifies significant price levels where price has repeatedly reversed
//! 2. Enters trades when price approaches and bounces from these levels
//! 3. Alternatively, trades breakouts when price decisively moves through a level
//! 4. Uses tight stop losses to manage risk
//!
//! # Example
//!
//! ```no_run
//! use minute_trade::{SupportResistanceStrategy, IntradayStrategy};
//! use minute_trade::utils::generate_minute_data;
//!
//! // Create a support/resistance strategy
//! let strategy = SupportResistanceStrategy::new(
//!     60,     // lookback period
//!     3,      // level strength
//!     0.2,    // level zone percentage
//!     true,   // trade bounces
//! ).unwrap();
//!
//! // Generate test data
//! let data = generate_minute_data(5, 390, 100.0, 0.02, 0.0);
//!
//! // Generate trading signals
//! let signals = strategy.generate_signals(&data).unwrap();
//! ```

use crate::utils::{calculate_basic_performance, validate_period, validate_positive};
use crate::{IntradayStrategy, MinuteOhlcv, Signal, TradeError};
use std::collections::HashMap;

/// Type of level (support or resistance)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum LevelType {
    /// Support level (price tends to bounce up from this level)
    Support,
    /// Resistance level (price tends to bounce down from this level)
    Resistance,
}

/// Price level with strength and type
#[derive(Debug, Clone)]
struct PriceLevel {
    /// Price level
    price: f64,
    /// Number of times the level has been tested
    strength: usize,
    /// Type of level (support or resistance)
    level_type: LevelType,
    /// Last time the level was tested (index in data array)
    last_test: usize,
}

/// Support/Resistance Strategy for trading key price levels
#[derive(Debug, Clone)]
pub struct SupportResistanceStrategy {
    /// Lookback period to identify levels
    lookback_period: usize,
    /// Minimum strength (number of touches) to consider a level valid
    min_strength: usize,
    /// Zone around level to consider price in the level's zone (as percentage)
    level_zone_pct: f64,
    /// Whether to trade bounces (true) or breakouts (false)
    trade_bounces: bool,
    /// Strategy name
    name: String,
}

impl SupportResistanceStrategy {
    /// Create a new support/resistance strategy
    ///
    /// # Arguments
    ///
    /// * `lookback_period` - Period to look back for levels (typically 60-240 minutes)
    /// * `min_strength` - Minimum number of touches to consider a level valid (typically 2-4)
    /// * `level_zone_pct` - Zone around level to consider it active (as percentage, e.g., 0.2 = 0.2%)
    /// * `trade_bounces` - Whether to trade bounces (true) or breakouts (false)
    ///
    /// # Returns
    ///
    /// * `Result<Self, String>` - New strategy instance or error message
    pub fn new(
        lookback_period: usize,
        min_strength: usize,
        level_zone_pct: f64,
        trade_bounces: bool,
    ) -> Result<Self, String> {
        validate_period(lookback_period, 30)?;

        if min_strength < 2 {
            return Err("Minimum strength should be at least 2 touches".to_string());
        }

        validate_positive(level_zone_pct, "Level zone percentage")?;
        if level_zone_pct > 1.0 {
            return Err("Level zone percentage seems too high (>1%). For intraday trading, values between 0.1% and 0.5% are typical.".to_string());
        }

        let bounce_or_breakout = if trade_bounces {
            "Bounces"
        } else {
            "Breakouts"
        };

        Ok(Self {
            lookback_period,
            min_strength,
            level_zone_pct,
            trade_bounces,
            name: format!(
                "Support/Resistance {0} ({1}, {2}, {3:.1}%)",
                bounce_or_breakout,
                lookback_period,
                min_strength,
                level_zone_pct * 100.0
            ),
        })
    }

    /// Get the lookback period
    pub fn lookback_period(&self) -> usize {
        self.lookback_period
    }

    /// Get the minimum strength required
    pub fn min_strength(&self) -> usize {
        self.min_strength
    }

    /// Get the level zone percentage
    pub fn level_zone_pct(&self) -> f64 {
        self.level_zone_pct
    }

    /// Get whether trading bounces or breakouts
    pub fn trades_bounces(&self) -> bool {
        self.trade_bounces
    }

    /// Identify key support and resistance levels
    fn identify_levels(&self, data: &[MinuteOhlcv], current_index: usize) -> Vec<PriceLevel> {
        if current_index < self.lookback_period {
            return Vec::new();
        }

        let start_index = current_index - self.lookback_period;

        // Group nearby price points to identify levels
        let mut level_groups: HashMap<i64, (f64, usize, LevelType, usize)> = HashMap::new();

        // Identify potential pivot points (local highs and lows)
        for i in (start_index + 1)..(current_index - 1) {
            let prev = &data[i - 1].data;
            let curr = &data[i].data;
            let next = &data[i + 1].data;

            // Local high (potential resistance)
            if curr.high > prev.high && curr.high > next.high {
                // Round to nearest 0.1% to group nearby levels
                let bucket = (curr.high * 1000.0) as i64;
                level_groups
                    .entry(bucket)
                    .and_modify(|(price, count, level_type, last_idx)| {
                        *count += 1;
                        *last_idx = i;
                        // Calculate weighted average for level price
                        *price = (*price * (*count - 1) as f64 + curr.high) / *count as f64;
                    })
                    .or_insert((curr.high, 1, LevelType::Resistance, i));
            }

            // Local low (potential support)
            if curr.low < prev.low && curr.low < next.low {
                // Round to nearest 0.1% to group nearby levels
                let bucket = (curr.low * 1000.0) as i64;
                level_groups
                    .entry(bucket)
                    .and_modify(|(price, count, level_type, last_idx)| {
                        *count += 1;
                        *last_idx = i;
                        // Calculate weighted average for level price
                        *price = (*price * (*count - 1) as f64 + curr.low) / *count as f64;
                    })
                    .or_insert((curr.low, 1, LevelType::Support, i));
            }
        }

        // Convert to vector of price levels with sufficient strength
        let mut levels: Vec<PriceLevel> = level_groups
            .into_iter()
            .filter_map(|(_, (price, strength, level_type, last_test))| {
                if strength >= self.min_strength {
                    Some(PriceLevel {
                        price,
                        strength,
                        level_type,
                        last_test,
                    })
                } else {
                    None
                }
            })
            .collect();

        // Sort by strength (descending)
        levels.sort_by(|a, b| b.strength.cmp(&a.strength));

        levels
    }

    /// Check if price is near a level
    fn is_near_level(&self, price: f64, level: &PriceLevel) -> bool {
        let zone_size = level.price * self.level_zone_pct / 100.0;
        (price - level.price).abs() <= zone_size
    }

    /// Check if price has broken through a level
    fn is_breakout(&self, candle: &OhlcvData, level: &PriceLevel) -> bool {
        match level.level_type {
            LevelType::Support => candle.close < level.price * (1.0 - self.level_zone_pct / 200.0),
            LevelType::Resistance => {
                candle.close > level.price * (1.0 + self.level_zone_pct / 200.0)
            }
        }
    }

    /// Check if price is bouncing from a level
    fn is_bounce(&self, candle: &OhlcvData, prev_candle: &OhlcvData, level: &PriceLevel) -> bool {
        match level.level_type {
            LevelType::Support => {
                // Price approached support and bounced up
                prev_candle.low <= level.price * (1.0 + self.level_zone_pct / 100.0)
                    && candle.close > prev_candle.close
            }
            LevelType::Resistance => {
                // Price approached resistance and bounced down
                prev_candle.high >= level.price * (1.0 - self.level_zone_pct / 100.0)
                    && candle.close < prev_candle.close
            }
        }
    }
}

/// OHLCV data for easier reference
type OhlcvData = crate::OhlcvData;

impl IntradayStrategy for SupportResistanceStrategy {
    fn name(&self) -> &str {
        &self.name
    }

    fn generate_signals(&self, data: &[MinuteOhlcv]) -> Result<Vec<Signal>, TradeError> {
        if data.len() < self.lookback_period + 1 {
            return Err(TradeError::InsufficientData(format!(
                "Need at least {} data points for support/resistance strategy",
                self.lookback_period + 1
            )));
        }

        let mut signals = Vec::with_capacity(data.len());

        // First entries are hold signals due to insufficient data
        for _ in 0..self.lookback_period {
            signals.push(Signal::Hold);
        }

        // Track active trades to avoid repeated entries
        let mut in_long = false;
        let mut in_short = false;

        // Generate signals for the remaining data points
        for i in self.lookback_period..data.len() {
            // Identify key levels using lookback window
            let levels = self.identify_levels(data, i);

            let current = &data[i].data;
            let prev = &data[i - 1].data;

            let mut signal = Signal::Hold;

            // Check for level interactions
            for level in &levels {
                if self.trade_bounces {
                    // Trade bounces from levels
                    if self.is_bounce(current, prev, level) {
                        match level.level_type {
                            LevelType::Support if !in_long => {
                                // Bounce up from support - Buy
                                signal = Signal::Buy;
                                in_long = true;
                                in_short = false;
                                break;
                            }
                            LevelType::Resistance if !in_short => {
                                // Bounce down from resistance - Sell
                                signal = Signal::Sell;
                                in_short = true;
                                in_long = false;
                                break;
                            }
                            _ => {}
                        }
                    }
                } else {
                    // Trade breakouts of levels
                    if self.is_breakout(current, level) {
                        match level.level_type {
                            LevelType::Resistance if !in_long => {
                                // Break above resistance - Buy
                                signal = Signal::Buy;
                                in_long = true;
                                in_short = false;
                                break;
                            }
                            LevelType::Support if !in_short => {
                                // Break below support - Sell
                                signal = Signal::Sell;
                                in_short = true;
                                in_long = false;
                                break;
                            }
                            _ => {}
                        }
                    }
                }
            }

            // Exit logic - exit when we hit an opposite level
            if in_long || in_short {
                for level in &levels {
                    let is_near = self.is_near_level(current.close, level);

                    if in_long && level.level_type == LevelType::Resistance && is_near {
                        // Long position near resistance - exit
                        signal = Signal::Sell;
                        in_long = false;
                        break;
                    } else if in_short && level.level_type == LevelType::Support && is_near {
                        // Short position near support - exit
                        signal = Signal::Buy;
                        in_short = false;
                        break;
                    }
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
    fn test_strategy_parameters() {
        // Test valid parameters
        let strategy = SupportResistanceStrategy::new(60, 3, 0.2, true);
        assert!(strategy.is_ok());

        // Test invalid lookback period
        let strategy = SupportResistanceStrategy::new(20, 3, 0.2, true);
        assert!(strategy.is_err());

        // Test invalid strength
        let strategy = SupportResistanceStrategy::new(60, 1, 0.2, true);
        assert!(strategy.is_err());

        // Test invalid zone percentage
        let strategy = SupportResistanceStrategy::new(60, 3, 0.0, true);
        assert!(strategy.is_err());

        // Test zone percentage warning
        let strategy = SupportResistanceStrategy::new(60, 3, 2.0, true);
        assert!(strategy.is_err());
    }

    #[test]
    fn test_level_detection() {
        let data = create_test_data(100);
        let strategy = SupportResistanceStrategy::new(50, 2, 0.2, true).unwrap();

        // Test with valid index
        let levels = strategy.identify_levels(&data, 80);

        // There should be at least some levels detected
        assert!(!levels.is_empty(), "No levels detected");

        // Test with insufficient data
        let levels = strategy.identify_levels(&data, 20);
        assert!(
            levels.is_empty(),
            "Expected empty levels with insufficient data"
        );
    }

    #[test]
    fn test_level_proximity() {
        let strategy = SupportResistanceStrategy::new(60, 2, 0.2, true).unwrap();

        let level = PriceLevel {
            price: 100.0,
            strength: 3,
            level_type: LevelType::Support,
            last_test: 50,
        };

        // Test price within zone (0.2% = 0.2)
        assert!(strategy.is_near_level(100.1, &level));
        assert!(strategy.is_near_level(99.9, &level));

        // Test price outside zone
        assert!(!strategy.is_near_level(100.3, &level));
        assert!(!strategy.is_near_level(99.7, &level));
    }

    #[test]
    fn test_signal_generation() {
        let data = create_test_data(120);
        let strategy = SupportResistanceStrategy::new(60, 2, 0.2, true).unwrap();

        let signals = strategy.generate_signals(&data).unwrap();

        // Check that we have the correct number of signals
        assert_eq!(signals.len(), data.len());

        // Check that the first 'lookback_period' signals are Hold
        for i in 0..strategy.lookback_period() {
            assert_eq!(signals[i], Signal::Hold);
        }
    }
}
