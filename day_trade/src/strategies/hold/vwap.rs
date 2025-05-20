//! VWAP (Volume-Weighted Average Price) intraday trading strategy
//!
//! This strategy uses VWAP as a reference point for trading decisions, including
//! mean reversion and trend following approaches.

use crate::{IntradayTradingStrategy, MinuteOhlcv, Signal, TradeError};
use chrono::{DateTime, Duration, Timelike, Utc};

/// VWAP calculation implementation
#[derive(Debug)]
struct VwapCalculator {
    period: usize,              // Number of minutes to include in the calculation
    reset_on_new_session: bool, // Whether to reset calculations at market open
}

impl VwapCalculator {
    pub fn new(period: usize, reset_on_new_session: bool) -> Self {
        Self {
            period,
            reset_on_new_session,
        }
    }

    /// Calculate VWAP for the given minute data
    pub fn calculate(&self, data: &[MinuteOhlcv], current_idx: usize) -> Result<f64, String> {
        if data.is_empty() || current_idx >= data.len() {
            return Err("Invalid data or index for VWAP calculation".to_string());
        }

        let current_time = data[current_idx].timestamp;
        let start_idx = self.find_start_index(data, current_idx, current_time);

        if start_idx > current_idx {
            return Err("Invalid start index for VWAP calculation".to_string());
        }

        let mut volume_sum = 0u64;
        let mut typical_price_volume_sum = 0f64;

        for i in start_idx..=current_idx {
            let ohlcv = &data[i].data;
            let typical_price = (ohlcv.high + ohlcv.low + ohlcv.close) / 3.0;
            typical_price_volume_sum += typical_price * ohlcv.volume as f64;
            volume_sum += ohlcv.volume;
        }

        if volume_sum == 0 {
            return Err("Zero volume in VWAP calculation period".to_string());
        }

        Ok(typical_price_volume_sum / volume_sum as f64)
    }

    /// Find the starting index for VWAP calculation based on period and session reset
    fn find_start_index(
        &self,
        data: &[MinuteOhlcv],
        current_idx: usize,
        current_time: DateTime<Utc>,
    ) -> usize {
        if self.reset_on_new_session {
            // Assuming market opens at 9:30 AM EST - find the most recent market open
            let market_open_hour = 9;
            let market_open_minute = 30;

            for i in (0..=current_idx).rev() {
                let time = data[i].timestamp;
                if time.hour() == market_open_hour && time.minute() == market_open_minute {
                    return i;
                }

                // If we've gone back to the previous day, use this as the start
                if time.date_naive() < current_time.date_naive() {
                    return i + 1; // Start with the first point of the current day
                }
            }

            return 0; // If no market open found, use all available data
        } else {
            // Use rolling window based on period
            current_idx.saturating_sub(self.period - 1)
        }
    }
}

/// Strategy for intraday trading using VWAP (Volume-Weighted Average Price)
pub struct VwapStrategy {
    vwap_calculator: VwapCalculator,
    deviation_threshold: f64,  // % deviation from VWAP to trigger signals
    mean_reversion_mode: bool, // true = mean reversion, false = trend following
    lookback_period: usize,    // periods to look back for trend analysis
}

impl VwapStrategy {
    /// Create a new VWAP strategy with custom parameters
    pub fn new(
        period: usize,
        reset_on_new_session: bool,
        deviation_threshold: f64,
        mean_reversion_mode: bool,
        lookback_period: usize,
    ) -> Self {
        Self {
            vwap_calculator: VwapCalculator::new(period, reset_on_new_session),
            deviation_threshold,
            mean_reversion_mode,
            lookback_period,
        }
    }

    /// Create a VWAP strategy with default parameters
    pub fn default() -> Self {
        Self {
            vwap_calculator: VwapCalculator::new(390, true), // Full trading day (6.5 hours = 390 minutes)
            deviation_threshold: 1.0,                        // 1% deviation from VWAP
            mean_reversion_mode: true,                       // Default to mean reversion
            lookback_period: 20,                             // Look back 20 minutes for trend
        }
    }

    /// Create a VWAP strategy optimized for mean reversion
    pub fn mean_reversion() -> Self {
        Self {
            vwap_calculator: VwapCalculator::new(390, true), // Full trading day
            deviation_threshold: 1.5,                        // 1.5% deviation from VWAP
            mean_reversion_mode: true,                       // Mean reversion
            lookback_period: 15,                             // 15 minutes lookback
        }
    }

    /// Create a VWAP strategy optimized for trend following
    pub fn trend_following() -> Self {
        Self {
            vwap_calculator: VwapCalculator::new(60, false), // Rolling 60 minutes
            deviation_threshold: 0.5,                        // 0.5% deviation
            mean_reversion_mode: false,                      // Trend following
            lookback_period: 30,                             // 30 minutes lookback
        }
    }

    /// Check if price is trending in relation to VWAP
    fn is_trending_up(&self, data: &[MinuteOhlcv], vwap_values: &[f64], idx: usize) -> bool {
        if idx < self.lookback_period {
            return false;
        }

        let start_idx = idx - self.lookback_period;
        let mut up_count = 0;

        for i in start_idx..idx {
            if data[i].data.close > vwap_values[i] {
                up_count += 1;
            }
        }

        // Consider trending if more than 70% of recent closes are above VWAP
        up_count as f64 / self.lookback_period as f64 > 0.7
    }

    /// Check if price is trending down in relation to VWAP
    fn is_trending_down(&self, data: &[MinuteOhlcv], vwap_values: &[f64], idx: usize) -> bool {
        if idx < self.lookback_period {
            return false;
        }

        let start_idx = idx - self.lookback_period;
        let mut down_count = 0;

        for i in start_idx..idx {
            if data[i].data.close < vwap_values[i] {
                down_count += 1;
            }
        }

        // Consider trending if more than 70% of recent closes are below VWAP
        down_count as f64 / self.lookback_period as f64 > 0.7
    }
}

impl IntradayTradingStrategy for VwapStrategy {
    fn generate_signals(&self, data: &[MinuteOhlcv]) -> Result<Vec<Signal>, TradeError> {
        if data.len() < self.lookback_period {
            return Err(TradeError::InsufficientData(format!(
                "Need at least {} data points for VWAP strategy",
                self.lookback_period
            )));
        }

        let mut signals = vec![Signal::Hold; data.len()];
        let mut vwap_values = vec![0.0; data.len()];

        // Calculate VWAP for each data point
        for i in 0..data.len() {
            let vwap = self.vwap_calculator.calculate(&data, i).map_err(|e| {
                TradeError::CalculationError(format!("VWAP calculation error: {}", e))
            })?;
            vwap_values[i] = vwap;
        }

        // Generate signals starting from the lookback period
        for i in self.lookback_period..data.len() {
            let current_price = data[i].data.close;
            let vwap = vwap_values[i];

            // Calculate deviation percentage
            let deviation_pct = (current_price - vwap) / vwap * 100.0;

            if self.mean_reversion_mode {
                // Mean reversion mode: Buy when price drops below VWAP, sell when it goes above
                if deviation_pct < -self.deviation_threshold {
                    signals[i] = Signal::Buy;
                } else if deviation_pct > self.deviation_threshold {
                    signals[i] = Signal::Sell;
                }
            } else {
                // Trend following mode: Buy when price breaks above VWAP with momentum, sell on drops
                if deviation_pct > self.deviation_threshold
                    && self.is_trending_up(&data, &vwap_values, i)
                {
                    signals[i] = Signal::Buy;
                } else if deviation_pct < -self.deviation_threshold
                    && self.is_trending_down(&data, &vwap_values, i)
                {
                    signals[i] = Signal::Sell;
                }
            }
        }

        Ok(signals)
    }

    fn calculate_performance(
        &self,
        data: &[MinuteOhlcv],
        signals: &[Signal],
    ) -> Result<f64, TradeError> {
        if data.len() != signals.len() {
            return Err(TradeError::InvalidData(
                "Data and signals count mismatch".to_string(),
            ));
        }

        let mut cash = 1000.0;
        let mut shares = 0.0;

        for (i, signal) in signals.iter().enumerate() {
            match signal {
                Signal::Buy if cash > 0.0 => {
                    shares = cash / data[i].data.close;
                    cash = 0.0;
                }
                Signal::Sell if shares > 0.0 => {
                    cash = shares * data[i].data.close;
                    shares = 0.0;
                }
                _ => {}
            }
        }

        // Final portfolio value
        let final_value = cash + shares * data.last().map(|d| d.data.close).unwrap_or(0.0);
        let initial_value = 1000.0;

        Ok((final_value - initial_value) / initial_value * 100.0) // Return as percentage
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Timelike};

    fn create_test_minute_data() -> Vec<MinuteOhlcv> {
        let mut data = Vec::new();
        let base_timestamp = Utc.with_ymd_and_hms(2023, 5, 1, 9, 30, 0).unwrap();

        // Create test data for one trading day (9:30 AM to 4:00 PM)
        let mut price = 100.0;
        let mut volume = 1000u64;

        for minute in 0..390 {
            // 390 minutes in a trading day
            let timestamp = base_timestamp + Duration::minutes(minute);

            // Add some price movement patterns - rising in morning, dip at lunch, recovery in afternoon
            let hour = timestamp.hour();
            let minute_of_hour = timestamp.minute();

            let mut price_change = 0.0;

            if hour < 11 {
                // Morning uptrend
                price_change = 0.01 + (minute_of_hour as f64 % 10.0) / 500.0;
            } else if hour < 13 {
                // Midday dip
                price_change = -0.01 - (minute_of_hour as f64 % 10.0) / 500.0;
            } else {
                // Afternoon recovery
                price_change = 0.005 + (minute_of_hour as f64 % 15.0) / 700.0;
            }

            // Add some randomization
            price_change += (minute % 3) as f64 * 0.001 - 0.001;

            // Update price
            price *= 1.0 + price_change;

            // Generate some volume variation
            volume = 1000 + (minute % 5) * 100 + if minute % 15 == 0 { 500 } else { 0 };

            data.push(MinuteOhlcv {
                timestamp,
                data: OhlcvData {
                    open: price * 0.999,
                    high: price * 1.002,
                    low: price * 0.998,
                    close: price,
                    volume,
                },
            });
        }

        data
    }

    #[test]
    fn test_vwap_calculation() {
        let data = create_test_minute_data();
        let calculator = VwapCalculator::new(60, false);

        // Test VWAP calculation at different points
        let vwap_start = calculator.calculate(&data, 60).unwrap();
        let vwap_middle = calculator.calculate(&data, 200).unwrap();
        let vwap_end = calculator.calculate(&data, 389).unwrap();

        assert!(vwap_start > 0.0, "VWAP should be positive");
        assert!(vwap_middle > 0.0, "VWAP should be positive");
        assert!(vwap_end > 0.0, "VWAP should be positive");
    }

    #[test]
    fn test_strategy_signals() {
        let data = create_test_minute_data();

        // Test mean reversion strategy
        let mean_reversion_strategy = VwapStrategy::mean_reversion();
        let signals_mr = mean_reversion_strategy.generate_signals(&data).unwrap();

        // Test trend following strategy
        let trend_strategy = VwapStrategy::trend_following();
        let signals_tf = trend_strategy.generate_signals(&data).unwrap();

        // Verify we have signals for every data point
        assert_eq!(signals_mr.len(), data.len());
        assert_eq!(signals_tf.len(), data.len());

        // Verify we have a mix of signals
        let mr_buy_count = signals_mr.iter().filter(|&&s| s == Signal::Buy).count();
        let mr_sell_count = signals_mr.iter().filter(|&&s| s == Signal::Sell).count();

        let tf_buy_count = signals_tf.iter().filter(|&&s| s == Signal::Buy).count();
        let tf_sell_count = signals_tf.iter().filter(|&&s| s == Signal::Sell).count();

        assert!(
            mr_buy_count > 0,
            "Mean reversion strategy should generate buy signals"
        );
        assert!(
            mr_sell_count > 0,
            "Mean reversion strategy should generate sell signals"
        );

        assert!(
            tf_buy_count > 0,
            "Trend following strategy should generate buy signals"
        );
        assert!(
            tf_sell_count > 0,
            "Trend following strategy should generate sell signals"
        );
    }
}
