//! Moving Average Convergence Divergence (MACD) trading strategy

use crate::mock_indicators::Macd;
use crate::{DailyOhlcv, Signal, TradeError, TradingStrategy};

/// MACD (Moving Average Convergence Divergence) strategy implementation
pub struct MacdStrategy {
    /// Fast EMA period
    fast_period: usize,
    /// Slow EMA period
    slow_period: usize,
    /// Signal line period
    signal_period: usize,
}

impl MacdStrategy {
    /// Create a new MACD strategy with the given parameters
    pub fn new(fast_period: usize, slow_period: usize, signal_period: usize) -> Self {
        Self {
            fast_period,
            slow_period,
            signal_period,
        }
    }

    /// Create a new MACD strategy with default parameters (12, 26, 9)
    pub fn default() -> Self {
        Self {
            fast_period: 12,
            slow_period: 26,
            signal_period: 9,
        }
    }
}

impl TradingStrategy for MacdStrategy {
    fn generate_signals(&self, data: &[DailyOhlcv]) -> Result<Vec<Signal>, TradeError> {
        if data.len() < self.slow_period + self.signal_period {
            return Err(TradeError::InsufficientData(format!(
                "Need at least {} data points for MACD calculation",
                self.slow_period + self.signal_period
            )));
        }

        let close_prices: Vec<f64> = data.iter().map(|d| d.data.close).collect();
        let mut signals = vec![Signal::Hold; data.len()];

        // Create MACD indicator using rustalib
        let mut macd =
            Macd::new(self.fast_period, self.slow_period, self.signal_period).map_err(|e| {
                TradeError::CalculationError(format!("Failed to create MACD indicator: {}", e))
            })?;

        let mut prev_histogram: Option<f64> = None;

        for (i, &price) in close_prices.iter().enumerate() {
            // Update indicator with current price
            macd.update(price).map_err(|e| {
                TradeError::CalculationError(format!("Failed to update MACD: {}", e))
            })?;

            // Skip until we have enough data points
            if i < self.slow_period + self.signal_period - 1 {
                continue;
            }

            // Get MACD values
            let macd_line = macd.macd_value().map_err(|e| {
                TradeError::CalculationError(format!("Failed to get MACD line: {}", e))
            })?;

            let signal_line = macd.signal_value().map_err(|e| {
                TradeError::CalculationError(format!("Failed to get signal line: {}", e))
            })?;

            let histogram = macd_line - signal_line;

            // Check for crossovers (sign changes in histogram)
            if let Some(prev) = prev_histogram {
                // Zero crossover (from negative to positive) is buy signal
                if histogram > 0.0 && prev <= 0.0 {
                    signals[i] = Signal::Buy;
                }
                // Zero crossover (from positive to negative) is sell signal
                else if histogram < 0.0 && prev >= 0.0 {
                    signals[i] = Signal::Sell;
                }
            }

            prev_histogram = Some(histogram);
        }

        Ok(signals)
    }

    fn calculate_performance(
        &self,
        data: &[DailyOhlcv],
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
    use crate::OhlcvData;
    use chrono::NaiveDate;

    fn create_test_data() -> Vec<DailyOhlcv> {
        // Create data for testing MACD strategy
        let mut data = Vec::new();

        // Starting price
        let mut price = 100.0;

        // First create a downtrend to establish negative MACD
        for day in 1..=20 {
            let date = NaiveDate::from_ymd_opt(2023, 1, day).unwrap();

            // Slight downtrend
            let price_change = -0.5;

            data.push(DailyOhlcv {
                date,
                data: OhlcvData {
                    open: price,
                    high: price + 0.2,
                    low: price - 0.3,
                    close: price + price_change,
                    volume: 1000 + ((day % 5) * 100) as u64,
                },
            });

            price = data.last().unwrap().data.close;
        }

        // Then create a strong uptrend to trigger a buy signal
        for day in 21..=30 {
            let date = NaiveDate::from_ymd_opt(2023, 1, day).unwrap();

            // Strong uptrend
            let price_change = 2.0;

            data.push(DailyOhlcv {
                date,
                data: OhlcvData {
                    open: price,
                    high: price + 2.5,
                    low: price - 0.1,
                    close: price + price_change,
                    volume: 1500 + ((day % 5) * 150) as u64,
                },
            });

            price = data.last().unwrap().data.close;
        }

        // Then create a downtrend again to trigger a sell signal
        for day in 31..=40 {
            let date = NaiveDate::from_ymd_opt(2023, 2, day % 28 + 1).unwrap();

            // Downtrend
            let price_change = -1.5;

            data.push(DailyOhlcv {
                date,
                data: OhlcvData {
                    open: price,
                    high: price + 0.3,
                    low: price - 2.0,
                    close: price + price_change,
                    volume: 1200 + ((day % 5) * 120) as u64,
                },
            });

            price = data.last().unwrap().data.close;
        }

        // Add some more data points with a slight uptrend
        for day in 41..=50 {
            let date = NaiveDate::from_ymd_opt(2023, 2, day % 28 + 1).unwrap();

            // Slight uptrend
            let price_change = 0.8;

            data.push(DailyOhlcv {
                date,
                data: OhlcvData {
                    open: price,
                    high: price + 1.0,
                    low: price - 0.2,
                    close: price + price_change,
                    volume: 1100 + ((day % 5) * 110) as u64,
                },
            });

            price = data.last().unwrap().data.close;
        }

        data
    }

    #[test]
    fn test_macd_default_parameters() {
        let strategy = MacdStrategy::default();
        assert_eq!(strategy.fast_period, 12);
        assert_eq!(strategy.slow_period, 26);
        assert_eq!(strategy.signal_period, 9);
    }

    #[test]
    fn test_macd_signal_generation() {
        let data = create_test_data();
        let strategy = MacdStrategy::default();

        let signals = strategy.generate_signals(&data).unwrap();

        // We should have some signals in our test data
        let buy_count = signals.iter().filter(|&&s| s == Signal::Buy).count();
        let sell_count = signals.iter().filter(|&&s| s == Signal::Sell).count();

        // In a cyclical pattern, we should get multiple buy and sell signals
        assert!(buy_count > 0, "Expected at least one buy signal");
        assert!(sell_count > 0, "Expected at least one sell signal");
    }
}
