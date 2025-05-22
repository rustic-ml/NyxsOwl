//! Chart Pattern Strategy for intraday trading
//!
//! This strategy identifies common chart patterns (flags, pennants, triangles)
//! and trades their breakouts or continuations.
//!
//! # Strategy Logic
//!
//! The chart pattern strategy:
//! 1. Identifies key chart patterns in price action
//! 2. Calculates expected breakout directions and price targets
//! 3. Enters trades when price confirms the pattern breakout
//! 4. Uses pattern-specific stops and targets
//!
//! # Example
//!
//! ```no_run
//! use minute_trade::{ChartPatternStrategy, IntradayStrategy};
//! use minute_trade::utils::generate_minute_data;
//!
//! // Create a chart pattern strategy
//! let strategy = ChartPatternStrategy::new(
//!     30,     // lookback period
//!     5,      // min pattern size
//!     0.5,    // pattern threshold
//!     "flag", // pattern type
//! ).unwrap();
//!
//! // Generate test data
//! let data = generate_minute_data(5, 390, 100.0, 0.02, 0.001);
//!
//! // Generate trading signals
//! let signals = strategy.generate_signals(&data).unwrap();
//! ```

use crate::utils::{calculate_basic_performance, validate_period, validate_positive};
use crate::{IntradayStrategy, MinuteOhlcv, Signal, TradeError};
use std::fmt;

/// Types of chart patterns supported by the strategy
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PatternType {
    /// Flag pattern (continuation)
    Flag,
    /// Double top/bottom (reversal)
    DoubleTopBottom,
    /// Triangle pattern (consolidation)
    Triangle,
    /// Head and shoulders pattern (reversal)
    HeadAndShoulders,
}

impl PatternType {
    /// Parse pattern type from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "flag" => Some(PatternType::Flag),
            "double" | "double_top_bottom" => Some(PatternType::DoubleTopBottom),
            "triangle" => Some(PatternType::Triangle),
            "head_and_shoulders" | "head_shoulders" => Some(PatternType::HeadAndShoulders),
            _ => None,
        }
    }
}

impl fmt::Display for PatternType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PatternType::Flag => write!(f, "Flag"),
            PatternType::DoubleTopBottom => write!(f, "Double Top/Bottom"),
            PatternType::Triangle => write!(f, "Triangle"),
            PatternType::HeadAndShoulders => write!(f, "Head and Shoulders"),
        }
    }
}

/// Chart Pattern Strategy for identifying and trading common chart patterns
#[derive(Debug, Clone)]
pub struct ChartPatternStrategy {
    /// Lookback period to identify patterns
    lookback_period: usize,
    /// Minimum size of pattern (in bars)
    min_pattern_size: usize,
    /// Threshold for pattern recognition (0.0-1.0)
    pattern_threshold: f64,
    /// Type of pattern to trade
    pattern_type: PatternType,
    /// Strategy name
    name: String,
}

impl ChartPatternStrategy {
    /// Create a new chart pattern strategy
    ///
    /// # Arguments
    ///
    /// * `lookback_period` - Period to look back for patterns (typically 20-50 minutes)
    /// * `min_pattern_size` - Minimum size of pattern in bars (typically 5-15)
    /// * `pattern_threshold` - Threshold for pattern recognition (0.0-1.0)
    /// * `pattern_type` - Type of pattern to trade ("flag", "double", "triangle", "head_and_shoulders")
    ///
    /// # Returns
    ///
    /// * `Result<Self, String>` - New strategy instance or error message
    pub fn new(
        lookback_period: usize,
        min_pattern_size: usize,
        pattern_threshold: f64,
        pattern_type: &str,
    ) -> Result<Self, String> {
        validate_period(lookback_period, 10)?;
        validate_period(min_pattern_size, 3)?;

        if min_pattern_size >= lookback_period {
            return Err("Minimum pattern size must be smaller than lookback period".to_string());
        }

        validate_positive(pattern_threshold, "Pattern threshold")?;
        if pattern_threshold > 1.0 {
            return Err("Pattern threshold must be between 0.0 and 1.0".to_string());
        }

        let pattern = PatternType::from_str(pattern_type)
            .ok_or_else(|| format!("Invalid pattern type: {}", pattern_type))?;

        Ok(Self {
            lookback_period,
            min_pattern_size,
            pattern_threshold,
            pattern_type: pattern.clone(),
            name: format!(
                "Chart Pattern ({}, {}, {:.1}%, {})",
                lookback_period,
                min_pattern_size,
                pattern_threshold * 100.0,
                pattern
            ),
        })
    }

    /// Get the lookback period
    pub fn lookback_period(&self) -> usize {
        self.lookback_period
    }

    /// Get the minimum pattern size
    pub fn min_pattern_size(&self) -> usize {
        self.min_pattern_size
    }

    /// Get the pattern threshold
    pub fn pattern_threshold(&self) -> f64 {
        self.pattern_threshold
    }

    /// Get the pattern type
    pub fn pattern_type(&self) -> &PatternType {
        &self.pattern_type
    }

    /// Detect a bull flag pattern
    fn detect_bull_flag(&self, data: &[MinuteOhlcv], index: usize) -> Option<(usize, f64)> {
        if index < self.lookback_period {
            return None;
        }

        let start_index = index - self.lookback_period;

        // Look for strong uptrend followed by consolidation
        let mut max_high = data[start_index].data.high;
        let min_low = data[start_index].data.low;
        let trend_start = start_index;
        let mut trend_end = start_index;
        let mut max_move = 0.0;

        // Find the strong uptrend (flag pole)
        for i in (start_index + 1)..=index {
            let curr_high = data[i].data.high;

            if curr_high > max_high {
                max_high = curr_high;
                trend_end = i;
            }

            let move_size = (max_high - data[trend_start].data.low) / data[trend_start].data.low;
            if move_size > max_move {
                max_move = move_size;
            }
        }

        // Need significant move
        if max_move < 0.005 {
            // At least 0.5% move
            return None;
        }

        // Need minimum pole size
        let pole_size = trend_end - trend_start;
        if pole_size < self.min_pattern_size {
            return None;
        }

        // Verify consolidation (flag)
        if index - trend_end < self.min_pattern_size / 2 {
            return None; // Not enough consolidation time
        }

        // Check if price is in a tight range during consolidation
        let mut max_consolidation = data[trend_end].data.high;
        let mut min_consolidation = data[trend_end].data.low;

        for i in (trend_end + 1)..=index {
            max_consolidation = max_consolidation.max(data[i].data.high);
            min_consolidation = min_consolidation.min(data[i].data.low);
        }

        let consolidation_range = (max_consolidation - min_consolidation) / min_consolidation;
        let trend_range = max_move;

        // Consolidation should be smaller than the trend and in a proper channel
        if consolidation_range > trend_range * 0.5 || consolidation_range < 0.001 {
            return None;
        }

        // Calculate pattern quality (0-1)
        let quality = if trend_range > 0.0 {
            1.0 - (consolidation_range / trend_range)
        } else {
            0.0
        };

        if quality >= self.pattern_threshold {
            Some((trend_end, max_consolidation))
        } else {
            None
        }
    }

    /// Detect a double bottom pattern
    fn detect_double_bottom(&self, data: &[MinuteOhlcv], index: usize) -> Option<(usize, f64)> {
        if index < self.lookback_period {
            return None;
        }

        let start_index = index - self.lookback_period;

        // Need enough bars to form double bottom
        if index - start_index < self.min_pattern_size * 2 {
            return None;
        }

        // Find first bottom
        let mut first_bottom_idx = start_index;
        let mut first_bottom_val = data[start_index].data.low;

        for i in (start_index + 1)..(start_index + self.lookback_period / 2) {
            if data[i].data.low < first_bottom_val {
                first_bottom_val = data[i].data.low;
                first_bottom_idx = i;
            }
        }

        // Find second bottom
        let mut second_bottom_idx = first_bottom_idx + self.min_pattern_size;
        let mut second_bottom_val = data[second_bottom_idx].data.low;

        for i in (second_bottom_idx - 1)..index {
            if data[i].data.low < second_bottom_val {
                second_bottom_val = data[i].data.low;
                second_bottom_idx = i;
            }
        }

        // Check if bottoms are similar in price
        let bottoms_diff = (first_bottom_val - second_bottom_val).abs() / first_bottom_val;
        if bottoms_diff > 0.01 {
            // Bottoms should be within 1%
            return None;
        }

        // Check if there's a significant peak between bottoms
        let mut middle_peak = first_bottom_val;
        for i in (first_bottom_idx + 1)..second_bottom_idx {
            middle_peak = middle_peak.max(data[i].data.high);
        }

        let peak_height = (middle_peak - first_bottom_val) / first_bottom_val;
        if peak_height < 0.005 {
            // Need at least 0.5% peak
            return None;
        }

        // Pattern confirmed with neckline break
        if data[index].data.close > middle_peak {
            Some((second_bottom_idx, first_bottom_val * 0.99))
        } else {
            None
        }
    }

    /// Detect a triangle pattern
    fn detect_triangle(&self, data: &[MinuteOhlcv], index: usize) -> Option<(usize, f64)> {
        // Basic triangle detection (simplified)
        if index < self.lookback_period {
            return None;
        }

        let start_index = index - self.lookback_period;

        // Find highest high and lowest low in first third of lookback
        let initial_range_end = start_index + self.lookback_period / 3;
        let mut highest_high = data[start_index].data.high;
        let mut lowest_low = data[start_index].data.low;

        for i in (start_index + 1)..=initial_range_end {
            highest_high = highest_high.max(data[i].data.high);
            lowest_low = lowest_low.min(data[i].data.low);
        }

        let initial_range = highest_high - lowest_low;
        if initial_range < data[start_index].data.close * 0.005 {
            return None; // Range too small
        }

        // Check for narrowing price action
        let last_range_start = index - self.lookback_period / 3;
        let mut last_highest = data[last_range_start].data.high;
        let mut last_lowest = data[last_range_start].data.low;

        for i in (last_range_start + 1)..=index {
            last_highest = last_highest.max(data[i].data.high);
            last_lowest = last_lowest.min(data[i].data.low);
        }

        let last_range = last_highest - last_lowest;

        // Triangle should narrow significantly
        if last_range > initial_range * 0.7 {
            return None;
        }

        // Break direction depends on most recent moves

        if data[index].data.close > data[index - 1].data.high {
            // Upside break
            Some((last_range_start, last_lowest * 0.99))
        } else if data[index].data.close < data[index - 1].data.low {
            // Downside break
            Some((last_range_start, last_highest * 1.01))
        } else {
            None
        }
    }

    /// Detect a head and shoulders pattern
    fn detect_head_and_shoulders(
        &self,
        data: &[MinuteOhlcv],
        index: usize,
    ) -> Option<(usize, f64)> {
        if index < self.lookback_period {
            return None;
        }

        let start_index = index - self.lookback_period;

        // Need enough bars to form head and shoulders pattern
        // Minimum 5 points: left shoulder, left trough, head, right trough, right shoulder
        if index - start_index < self.min_pattern_size * 2 {
            return None;
        }

        // Find potential shoulders and head
        let third = self.lookback_period / 3;

        // Find left shoulder (first peak)
        let mut left_shoulder_idx = start_index;
        let mut left_shoulder_val = data[start_index].data.high;

        for i in start_index..(start_index + third) {
            if data[i].data.high > left_shoulder_val {
                left_shoulder_val = data[i].data.high;
                left_shoulder_idx = i;
            }
        }

        // Find head (middle and highest peak)
        let head_start = left_shoulder_idx + 2;
        let head_end = head_start + third;

        if head_end >= index {
            return None;
        }

        let mut head_idx = head_start;
        let mut head_val = data[head_start].data.high;

        for i in head_start..head_end {
            if data[i].data.high > head_val {
                head_val = data[i].data.high;
                head_idx = i;
            }
        }

        // Head should be higher than left shoulder
        if head_val <= left_shoulder_val {
            return None;
        }

        // Find right shoulder (last peak)
        let right_start = head_idx + 2;
        let right_end = index;

        if right_start >= right_end {
            return None;
        }

        let mut right_shoulder_idx = right_start;
        let mut right_shoulder_val = data[right_start].data.high;

        for i in right_start..right_end {
            if data[i].data.high > right_shoulder_val {
                right_shoulder_val = data[i].data.high;
                right_shoulder_idx = i;
            }
        }

        // Right shoulder should be lower than head
        if right_shoulder_val >= head_val {
            return None;
        }

        // Check if right and left shoulders are at similar levels (within 5%)
        let shoulder_diff = (right_shoulder_val - left_shoulder_val).abs() / left_shoulder_val;
        if shoulder_diff > 0.05 {
            return None; // Shoulders should be at similar levels
        }

        // Find neckline based on troughs between shoulders and head
        let mut left_trough_idx = left_shoulder_idx;
        let mut left_trough_val = data[left_shoulder_idx].data.low;

        for i in left_shoulder_idx..head_idx {
            if data[i].data.low < left_trough_val {
                left_trough_val = data[i].data.low;
                left_trough_idx = i;
            }
        }

        let mut right_trough_idx = head_idx;
        let mut right_trough_val = data[head_idx].data.low;

        for i in head_idx..right_shoulder_idx {
            if data[i].data.low < right_trough_val {
                right_trough_val = data[i].data.low;
                right_trough_idx = i;
            }
        }

        // Calculate neckline level (connect the troughs)
        let neckline = (left_trough_val + right_trough_val) / 2.0;

        // Pattern is valid if price breaks below neckline
        if data[index].data.close < neckline {
            // Calculate target based on pattern height
            let pattern_height = head_val - neckline;
            let target = neckline - pattern_height; // Projection below neckline

            // Return stop level slightly above the right shoulder
            Some((right_shoulder_idx, right_shoulder_val * 1.01))
        } else {
            None
        }
    }
}

impl IntradayStrategy for ChartPatternStrategy {
    fn name(&self) -> &str {
        &self.name
    }

    fn generate_signals(&self, data: &[MinuteOhlcv]) -> Result<Vec<Signal>, TradeError> {
        if data.len() < self.lookback_period + 1 {
            return Err(TradeError::InsufficientData(format!(
                "Need at least {} data points for chart pattern strategy",
                self.lookback_period + 1
            )));
        }

        let mut signals = Vec::with_capacity(data.len());

        // First entries are hold signals due to insufficient data
        for _ in 0..self.lookback_period {
            signals.push(Signal::Hold);
        }

        // Track pattern formation and breakout states
        let mut in_pattern = false;
        let mut pattern_stop = 0.0;
        let mut pattern_start_idx = 0;

        // Generate signals for the remaining data points
        for i in self.lookback_period..data.len() {
            let current_price = data[i].data.close;
            let mut signal = Signal::Hold;

            // Pattern detection based on pattern type
            let pattern_result = match self.pattern_type {
                PatternType::Flag => self.detect_bull_flag(data, i),
                PatternType::DoubleTopBottom => self.detect_double_bottom(data, i),
                PatternType::Triangle => self.detect_triangle(data, i),
                PatternType::HeadAndShoulders => self.detect_head_and_shoulders(data, i),
            };

            if let Some((pattern_idx, stop_level)) = pattern_result {
                if !in_pattern {
                    // New pattern detected
                    in_pattern = true;
                    pattern_stop = stop_level;
                    pattern_start_idx = pattern_idx;

                    // Enter position on pattern detection
                    match self.pattern_type {
                        PatternType::Flag => {
                            // Bull flag: buy on break above consolidation
                            signal = Signal::Buy;
                        }
                        PatternType::DoubleTopBottom => {
                            // Double bottom: buy on neckline break
                            signal = Signal::Buy;
                        }
                        PatternType::Triangle => {
                            // Triangle: direction depends on breakout direction
                            if current_price > data[i - 1].data.high {
                                signal = Signal::Buy;
                            } else if current_price < data[i - 1].data.low {
                                signal = Signal::Sell;
                            }
                        }
                        PatternType::HeadAndShoulders => {
                            // Head and shoulders: sell on neckline break
                            signal = Signal::Sell;
                        }
                    }
                }
            } else if in_pattern {
                // Already in pattern, check for stop or target
                let days_in_pattern = i - pattern_start_idx;

                // Exit if pattern takes too long (failed pattern)
                if days_in_pattern > self.lookback_period / 2 {
                    in_pattern = false;
                    signal = if signals[i - 1] == Signal::Buy {
                        Signal::Sell // Exit long
                    } else if signals[i - 1] == Signal::Sell {
                        Signal::Buy // Exit short
                    } else {
                        Signal::Hold
                    };
                }
                // Check for stop loss
                else if (signals[i - 1] == Signal::Buy && current_price < pattern_stop)
                    || (signals[i - 1] == Signal::Sell && current_price > pattern_stop)
                {
                    in_pattern = false;
                    signal = if signals[i - 1] == Signal::Buy {
                        Signal::Sell // Stop loss on long
                    } else {
                        Signal::Buy // Stop loss on short
                    };
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
        let commission = 0.03; // 0.03% per trade
        calculate_basic_performance(data, signals, 10000.0, commission)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::create_test_data;

    #[test]
    fn test_pattern_type_parsing() {
        assert_eq!(PatternType::from_str("flag"), Some(PatternType::Flag));
        assert_eq!(
            PatternType::from_str("DOUBLE"),
            Some(PatternType::DoubleTopBottom)
        );
        assert_eq!(
            PatternType::from_str("triangle"),
            Some(PatternType::Triangle)
        );
        assert_eq!(
            PatternType::from_str("head_and_shoulders"),
            Some(PatternType::HeadAndShoulders)
        );
        assert_eq!(PatternType::from_str("invalid"), None);
    }

    #[test]
    fn test_strategy_parameters() {
        // Test valid parameters
        let strategy = ChartPatternStrategy::new(30, 5, 0.5, "flag");
        assert!(strategy.is_ok());

        // Test invalid lookback period
        let strategy = ChartPatternStrategy::new(5, 3, 0.5, "flag");
        assert!(strategy.is_err());

        // Test invalid pattern size
        let strategy = ChartPatternStrategy::new(20, 1, 0.5, "flag");
        assert!(strategy.is_err());

        // Test pattern size > lookback
        let strategy = ChartPatternStrategy::new(20, 21, 0.5, "flag");
        assert!(strategy.is_err());

        // Test invalid threshold
        let strategy = ChartPatternStrategy::new(30, 5, 1.5, "flag");
        assert!(strategy.is_err());

        // Test invalid pattern type
        let strategy = ChartPatternStrategy::new(30, 5, 0.5, "unknown");
        assert!(strategy.is_err());
    }

    #[test]
    fn test_signal_generation() {
        let data = create_test_data(100);
        let strategy = ChartPatternStrategy::new(30, 5, 0.5, "flag").unwrap();

        let signals = strategy.generate_signals(&data).unwrap();

        // Check that we have the correct number of signals
        assert_eq!(signals.len(), data.len());

        // Check that the first 'lookback_period' signals are Hold
        for i in 0..strategy.lookback_period() {
            assert_eq!(signals[i], Signal::Hold);
        }
    }

    #[test]
    fn test_head_and_shoulders_pattern() {
        let data = create_test_data(150); // Need more data for this complex pattern
        let strategy = ChartPatternStrategy::new(60, 10, 0.6, "head_and_shoulders").unwrap();

        // Just testing that signals are generated without errors
        let signals = strategy.generate_signals(&data).unwrap();

        // Check that we have the correct number of signals
        assert_eq!(signals.len(), data.len());

        // Create a manual test case with known data that forms a H&S pattern
        let mut custom_data = Vec::new();
        use chrono::{TimeZone, Utc};
        let base_time = Utc.with_ymd_and_hms(2023, 1, 1, 9, 30, 0).unwrap();

        for i in 0..100 {
            let base_price = 100.0;
            let mut price = match i {
                10..=15 => base_price + 2.0, // Left shoulder
                20..=22 => base_price - 0.5, // Left trough
                25..=30 => base_price + 4.0, // Head
                35..=37 => base_price - 0.5, // Right trough
                40..=45 => base_price + 2.2, // Right shoulder
                50..=60 => base_price - 1.0, // Break below neckline
                _ => base_price,
            };

            // Add some noise
            price += (i as f64 * 0.01).sin();

            let ohlcv = MinuteOhlcv {
                timestamp: base_time + chrono::Duration::minutes(i as i64),
                data: crate::OhlcvData {
                    open: price,
                    high: price + 0.5,
                    low: price - 0.5,
                    close: price,
                    volume: 1000.0,
                },
            };
            custom_data.push(ohlcv);
        }

        let pattern_strategy =
            ChartPatternStrategy::new(60, 10, 0.5, "head_and_shoulders").unwrap();
        let result = pattern_strategy.detect_head_and_shoulders(&custom_data, 60);

        // Should detect the pattern after it breaks below the neckline
        assert!(
            result.is_some(),
            "Head and Shoulders pattern was not detected"
        );
    }
}
