//! Relative Strength Index (RSI) trading strategy

use crate::mock_indicators::RelativeStrengthIndex;
use crate::{DailyOhlcv, Signal, TradeError, TradingStrategy};

/// RSI (Relative Strength Index) strategy implementation
pub struct RsiStrategy {
    period: usize,
    overbought_threshold: f64,
    oversold_threshold: f64,
}

impl RsiStrategy {
    /// Create a new RSI strategy with the given parameters
    pub fn new(period: usize, overbought_threshold: f64, oversold_threshold: f64) -> Self {
        Self {
            period,
            overbought_threshold,
            oversold_threshold,
        }
    }

    /// Create a new RSI strategy with default parameters (14, 70, 30)
    pub fn default() -> Self {
        Self {
            period: 14,
            overbought_threshold: 70.0,
            oversold_threshold: 30.0,
        }
    }
}

impl TradingStrategy for RsiStrategy {
    fn generate_signals(&self, data: &[DailyOhlcv]) -> Result<Vec<Signal>, TradeError> {
        if data.len() <= self.period + 1 {
            return Err(TradeError::InsufficientData(format!(
                "Need at least {} data points for RSI calculation",
                self.period + 2
            )));
        }

        let close_prices: Vec<f64> = data.iter().map(|d| d.data.close).collect();
        let mut signals = vec![Signal::Hold; data.len()];

        // Create RSI indicator using rustalib
        let mut rsi = RelativeStrengthIndex::new(self.period).map_err(|e| {
            TradeError::CalculationError(format!("Failed to create RSI indicator: {}", e))
        })?;

        let mut prev_rsi_value: Option<f64> = None;

        for (i, &price) in close_prices.iter().enumerate() {
            // Update indicator with current price
            rsi.update(price).map_err(|e| {
                TradeError::CalculationError(format!("Failed to update RSI: {}", e))
            })?;

            // Skip until we have enough data points for RSI calculation
            if i < self.period {
                continue;
            }

            // Get current RSI value
            let rsi_value = rsi.value().map_err(|e| {
                TradeError::CalculationError(format!("Failed to get RSI value: {}", e))
            })?;

            // Generate signals based on RSI thresholds
            if let Some(prev_value) = prev_rsi_value {
                if rsi_value < self.oversold_threshold && prev_value >= self.oversold_threshold {
                    // RSI crossed below oversold threshold - buy signal
                    signals[i] = Signal::Buy;
                } else if rsi_value > self.overbought_threshold
                    && prev_value <= self.overbought_threshold
                {
                    // RSI crossed above overbought threshold - sell signal
                    signals[i] = Signal::Sell;
                }
            }

            prev_rsi_value = Some(rsi_value);
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
        // Creating price data with a more extreme pattern to trigger RSI signals
        let mut data = Vec::new();

        // Starting price
        let mut price = 100.0;

        // Add enough initial data points for RSI calculation with simple pattern
        for day in 1..=20 {
            let date = NaiveDate::from_ymd_opt(2023, 1, day).unwrap();

            // Add slight variation to create some ups and downs
            let price_change = match day % 3 {
                0 => 1.0,  // Up
                1 => -0.5, // Down
                _ => 0.25, // Up slightly
            };

            data.push(DailyOhlcv {
                date,
                data: OhlcvData {
                    open: price,
                    high: price * 1.01,
                    low: price * 0.99,
                    close: price + price_change,
                    volume: 1000,
                },
            });

            price = data.last().unwrap().data.close;
        }

        // Add strong uptrend to generate overbought condition
        for day in 21..=30 {
            let date = NaiveDate::from_ymd_opt(2023, 1, day).unwrap();
            price *= 1.03; // 3% increase each day

            data.push(DailyOhlcv {
                date,
                data: OhlcvData {
                    open: price / 1.03,
                    high: price * 1.01,
                    low: price * 0.98,
                    close: price,
                    volume: 2000,
                },
            });
        }

        // Add a few neutral days
        for day in 1..=3 {
            let date = NaiveDate::from_ymd_opt(2023, 2, day).unwrap();

            data.push(DailyOhlcv {
                date,
                data: OhlcvData {
                    open: price,
                    high: price * 1.01,
                    low: price * 0.99,
                    close: price,
                    volume: 1500,
                },
            });
        }

        // Add strong downtrend to generate oversold condition
        for day in 4..=15 {
            let date = NaiveDate::from_ymd_opt(2023, 2, day).unwrap();
            price *= 0.97; // 3% decrease each day

            data.push(DailyOhlcv {
                date,
                data: OhlcvData {
                    open: price / 0.97,
                    high: price * 1.01,
                    low: price * 0.98,
                    close: price,
                    volume: 3000,
                },
            });
        }

        data
    }

    #[test]
    fn test_rsi_signal_generation() {
        let data = create_test_data();
        let strategy = RsiStrategy::new(14, 70.0, 30.0);

        let signals = strategy.generate_signals(&data).unwrap();

        // We expect at least one buy and one sell signal
        let buy_count = signals.iter().filter(|&&s| s == Signal::Buy).count();
        let sell_count = signals.iter().filter(|&&s| s == Signal::Sell).count();

        assert!(buy_count > 0, "Expected at least one buy signal");
        assert!(sell_count > 0, "Expected at least one sell signal");
    }

    #[test]
    fn test_rsi_default_parameters() {
        let strategy = RsiStrategy::default();
        assert_eq!(strategy.period, 14);
        assert_eq!(strategy.overbought_threshold, 70.0);
        assert_eq!(strategy.oversold_threshold, 30.0);
    }
}
