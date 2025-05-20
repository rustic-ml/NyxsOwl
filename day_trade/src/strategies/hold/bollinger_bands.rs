//! Bollinger Bands intraday trading strategy implementation
//!
//! This strategy uses Bollinger Bands for identifying volatility and potential reversal points
//! in intraday trading with minute-level data.

use crate::{IntradayTradingStrategy, MinuteOhlcv, Signal, TradeError};
use std::collections::VecDeque;

/// Bollinger Bands calculation implementation
#[derive(Debug)]
struct BollingerBands {
    period: usize,           // The period for SMA calculation
    std_dev_multiplier: f64, // Number of standard deviations for bands
    prices: VecDeque<f64>,   // Queue of recent prices
}

impl BollingerBands {
    pub fn new(period: usize, std_dev_multiplier: f64) -> Self {
        Self {
            period,
            std_dev_multiplier,
            prices: VecDeque::with_capacity(period + 1),
        }
    }

    /// Update the indicator with a new price
    pub fn update(&mut self, price: f64) {
        self.prices.push_back(price);
        if self.prices.len() > self.period {
            self.prices.pop_front();
        }
    }

    /// Calculate the middle band (Simple Moving Average)
    pub fn middle_band(&self) -> Result<f64, String> {
        if self.prices.len() < self.period {
            return Err(format!(
                "Not enough data for Bollinger Bands calculation. Need {} data points.",
                self.period
            ));
        }

        let sum: f64 = self.prices.iter().sum();
        Ok(sum / self.prices.len() as f64)
    }

    /// Calculate standard deviation of prices
    fn standard_deviation(&self, mean: f64) -> f64 {
        let variance: f64 = self
            .prices
            .iter()
            .map(|&price| {
                let diff = price - mean;
                diff * diff
            })
            .sum::<f64>()
            / self.prices.len() as f64;

        variance.sqrt()
    }

    /// Calculate the upper band
    pub fn upper_band(&self) -> Result<f64, String> {
        let middle = self.middle_band()?;
        let std_dev = self.standard_deviation(middle);
        Ok(middle + (std_dev * self.std_dev_multiplier))
    }

    /// Calculate the lower band
    pub fn lower_band(&self) -> Result<f64, String> {
        let middle = self.middle_band()?;
        let std_dev = self.standard_deviation(middle);
        Ok(middle - (std_dev * self.std_dev_multiplier))
    }

    /// Calculate Bollinger Band Width (volatility indicator)
    pub fn band_width(&self) -> Result<f64, String> {
        let upper = self.upper_band()?;
        let lower = self.lower_band()?;
        let middle = self.middle_band()?;

        Ok((upper - lower) / middle * 100.0) // Return as percentage
    }

    /// Calculate %B (Where price is relative to the bands)
    pub fn percent_b(&self, price: f64) -> Result<f64, String> {
        let upper = self.upper_band()?;
        let lower = self.lower_band()?;

        if upper - lower == 0.0 {
            return Err("Band width is zero, cannot calculate %B".to_string());
        }

        Ok((price - lower) / (upper - lower))
    }
}

/// Bollinger Bands strategy for intraday trading
pub struct BollingerBandsStrategy {
    period: usize,
    std_dev_multiplier: f64,
    oversold_threshold: f64,   // %B below this is considered oversold
    overbought_threshold: f64, // %B above this is considered overbought
    bandwidth_expansion_threshold: f64, // For volatility breakout detection
    trend_confirmation_length: usize, // Lookback periods for trend confirmation
}

impl BollingerBandsStrategy {
    /// Create a new Bollinger Bands strategy with custom parameters
    pub fn new(
        period: usize,
        std_dev_multiplier: f64,
        oversold_threshold: f64,
        overbought_threshold: f64,
        bandwidth_expansion_threshold: f64,
        trend_confirmation_length: usize,
    ) -> Self {
        Self {
            period,
            std_dev_multiplier,
            oversold_threshold,
            overbought_threshold,
            bandwidth_expansion_threshold,
            trend_confirmation_length,
        }
    }

    /// Create a strategy with default parameters
    pub fn default() -> Self {
        Self {
            period: 20,
            std_dev_multiplier: 2.0,
            oversold_threshold: 0.1,            // 10% from lower band
            overbought_threshold: 0.9,          // 90% from lower band (near upper)
            bandwidth_expansion_threshold: 5.0, // 5% expansion signals volatility
            trend_confirmation_length: 5,
        }
    }

    /// Create a strategy optimized for mean reversion
    pub fn mean_reversion() -> Self {
        Self {
            period: 20,
            std_dev_multiplier: 2.5, // Wider bands for stronger mean reversion
            oversold_threshold: 0.05, // More extreme oversold threshold
            overbought_threshold: 0.95, // More extreme overbought threshold
            bandwidth_expansion_threshold: 7.0, // Looking for higher volatility
            trend_confirmation_length: 3, // Quicker to enter trades
        }
    }

    /// Create a strategy optimized for volatility breakouts
    pub fn volatility_breakout() -> Self {
        Self {
            period: 20,
            std_dev_multiplier: 2.0,
            oversold_threshold: 0.2,   // Less extreme for breakout strategy
            overbought_threshold: 0.8, // Less extreme for breakout strategy
            bandwidth_expansion_threshold: 4.0, // More sensitive to volatility increases
            trend_confirmation_length: 3,
        }
    }

    /// Detect if price broke out above the upper band
    fn is_upside_breakout(&self, percent_b: f64, band_widths: &[f64], current_idx: usize) -> bool {
        // Price is above upper band
        if percent_b <= 1.0 {
            return false;
        }

        // Check if bandwidth is expanding (volatility increasing)
        if current_idx >= self.trend_confirmation_length
            && band_widths[current_idx]
                > band_widths[current_idx - self.trend_confirmation_length]
                    * (1.0 + self.bandwidth_expansion_threshold / 100.0)
        {
            return true;
        }

        false
    }

    /// Detect if price broke down below the lower band
    fn is_downside_breakout(
        &self,
        percent_b: f64,
        band_widths: &[f64],
        current_idx: usize,
    ) -> bool {
        // Price is below lower band
        if percent_b >= 0.0 {
            return false;
        }

        // Check if bandwidth is expanding (volatility increasing)
        if current_idx >= self.trend_confirmation_length
            && band_widths[current_idx]
                > band_widths[current_idx - self.trend_confirmation_length]
                    * (1.0 + self.bandwidth_expansion_threshold / 100.0)
        {
            return true;
        }

        false
    }

    /// Check if price is consistently near the upper band
    fn is_trending_up(&self, percent_bs: &[f64], current_idx: usize) -> bool {
        if current_idx < self.trend_confirmation_length {
            return false;
        }

        let mut count = 0;
        for i in 0..self.trend_confirmation_length {
            if percent_bs[current_idx - i] > 0.5 {
                count += 1;
            }
        }

        count >= self.trend_confirmation_length / 2 + 1
    }

    /// Check if price is consistently near the lower band
    fn is_trending_down(&self, percent_bs: &[f64], current_idx: usize) -> bool {
        if current_idx < self.trend_confirmation_length {
            return false;
        }

        let mut count = 0;
        for i in 0..self.trend_confirmation_length {
            if percent_bs[current_idx - i] < 0.5 {
                count += 1;
            }
        }

        count >= self.trend_confirmation_length / 2 + 1
    }
}

impl IntradayTradingStrategy for BollingerBandsStrategy {
    fn generate_signals(&self, data: &[MinuteOhlcv]) -> Result<Vec<Signal>, TradeError> {
        if data.len() < self.period + self.trend_confirmation_length {
            return Err(TradeError::InsufficientData(format!(
                "Need at least {} data points for Bollinger Bands strategy",
                self.period + self.trend_confirmation_length
            )));
        }

        let mut signals = vec![Signal::Hold; data.len()];
        let mut bb_indicator = BollingerBands::new(self.period, self.std_dev_multiplier);

        // Arrays to store calculated values for each data point
        let mut percent_bs = vec![0.5; data.len()]; // default to middle
        let mut band_widths = vec![0.0; data.len()];

        // Calculate Bollinger Bands for each data point
        for i in 0..data.len() {
            let price = data[i].data.close;
            bb_indicator.update(price);

            // Skip calculations until we have enough data
            if i < self.period - 1 {
                continue;
            }

            // Calculate %B and band width for trend analysis
            match bb_indicator.percent_b(price) {
                Ok(percent_b) => percent_bs[i] = percent_b,
                Err(e) => {
                    return Err(TradeError::CalculationError(format!(
                        "Failed to calculate %B: {}",
                        e
                    )))
                }
            }

            match bb_indicator.band_width() {
                Ok(band_width) => band_widths[i] = band_width,
                Err(e) => {
                    return Err(TradeError::CalculationError(format!(
                        "Failed to calculate band width: {}",
                        e
                    )))
                }
            }
        }

        // Generate signals starting after we have enough data
        for i in (self.period + self.trend_confirmation_length - 1)..data.len() {
            let percent_b = percent_bs[i];

            // Mean reversion signals (price bouncing from bands back to middle)
            if percent_b <= self.oversold_threshold && self.is_trending_down(&percent_bs, i) {
                // Oversold condition - potential buy
                signals[i] = Signal::Buy;
            } else if percent_b >= self.overbought_threshold && self.is_trending_up(&percent_bs, i)
            {
                // Overbought condition - potential sell
                signals[i] = Signal::Sell;
            }
            // Breakout signals (price breaking through bands with increased volatility)
            else if self.is_upside_breakout(percent_b, &band_widths, i) {
                // Upside breakout with volatility - momentum buy
                signals[i] = Signal::Buy;
            } else if self.is_downside_breakout(percent_b, &band_widths, i) {
                // Downside breakout with volatility - momentum sell
                signals[i] = Signal::Sell;
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

        // Start evaluation after we have enough data for the strategy
        let start_idx = self.period + self.trend_confirmation_length - 1;

        for i in start_idx..signals.len() {
            match signals[i] {
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
    use crate::OhlcvData;
    use chrono::{Duration, TimeZone, Utc};

    fn create_test_minute_data() -> Vec<MinuteOhlcv> {
        let mut data = Vec::new();
        let base_timestamp = Utc.with_ymd_and_hms(2023, 5, 1, 9, 30, 0).unwrap();

        // Create test data for one trading day (9:30 AM to 4:00 PM)
        let mut price = 100.0;

        for minute in 0..390 {
            // 390 minutes in a trading day
            let timestamp = base_timestamp + Duration::minutes(minute);

            // Create price patterns that would test Bollinger Bands
            // Starting with some volatility, then a trend, then mean reversion

            let mut price_change = 0.0;

            if minute < 100 {
                // Initial volatility with oscillations
                price_change = (minute as f64 / 10.0).sin() * 0.5;
            } else if minute < 200 {
                // Uptrend
                price_change = 0.05 + (minute as f64 % 10.0) * 0.002;
            } else if minute < 300 {
                // High volatility
                price_change = (minute as f64 / 5.0).sin() * 0.8;
            } else {
                // Mean reversion back to average
                price_change = (150.0 - price) * 0.01;
            }

            price += price_change;

            // Add some randomness
            price += (minute % 3) as f64 * 0.2 - 0.2;

            // Ensure price stays positive
            price = price.max(50.0);

            data.push(MinuteOhlcv {
                timestamp,
                data: OhlcvData {
                    open: price - 0.1,
                    high: price + 0.2,
                    low: price - 0.2,
                    close: price,
                    volume: 1000 + (minute % 10) * 100,
                },
            });
        }

        data
    }

    #[test]
    fn test_bollinger_bands_calculation() {
        let data = create_test_minute_data();
        let mut bb = BollingerBands::new(20, 2.0);

        // Update with initial data
        for i in 0..30 {
            bb.update(data[i].data.close);
        }

        let middle = bb.middle_band().unwrap();
        let upper = bb.upper_band().unwrap();
        let lower = bb.lower_band().unwrap();
        let width = bb.band_width().unwrap();

        assert!(middle > 0.0, "Middle band should be positive");
        assert!(upper > middle, "Upper band should be greater than middle");
        assert!(lower < middle, "Lower band should be less than middle");
        assert!(width > 0.0, "Band width should be positive");
    }

    #[test]
    fn test_strategy_signals() {
        let data = create_test_minute_data();

        // Test mean reversion strategy
        let mean_reversion_strategy = BollingerBandsStrategy::mean_reversion();
        let signals_mr = mean_reversion_strategy.generate_signals(&data).unwrap();

        // Test volatility breakout strategy
        let breakout_strategy = BollingerBandsStrategy::volatility_breakout();
        let signals_br = breakout_strategy.generate_signals(&data).unwrap();

        // Verify we have signals for every data point
        assert_eq!(signals_mr.len(), data.len());
        assert_eq!(signals_br.len(), data.len());

        // Verify we get some signals
        let mr_buy_count = signals_mr.iter().filter(|&&s| s == Signal::Buy).count();
        let mr_sell_count = signals_mr.iter().filter(|&&s| s == Signal::Sell).count();
        let br_buy_count = signals_br.iter().filter(|&&s| s == Signal::Buy).count();
        let br_sell_count = signals_br.iter().filter(|&&s| s == Signal::Sell).count();

        assert!(
            mr_buy_count > 0,
            "Mean reversion strategy should generate buy signals"
        );
        assert!(
            mr_sell_count > 0,
            "Mean reversion strategy should generate sell signals"
        );
        assert!(
            br_buy_count > 0,
            "Breakout strategy should generate buy signals"
        );
        assert!(
            br_sell_count > 0,
            "Breakout strategy should generate sell signals"
        );
    }
}
