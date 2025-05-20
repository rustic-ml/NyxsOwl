//! Moving Average Crossover strategy implementation

use crate::mock_indicators::SimpleMovingAverage;
use crate::{DailyOhlcv, Signal, TradeError, TradingStrategy};

/// Moving Average Crossover strategy implementation
pub struct MACrossover {
    short_period: usize,
    long_period: usize,
}

impl MACrossover {
    /// Create a new MA Crossover strategy with given periods
    pub fn new(short_period: usize, long_period: usize) -> Self {
        Self {
            short_period,
            long_period,
        }
    }
}

impl TradingStrategy for MACrossover {
    fn generate_signals(&self, data: &[DailyOhlcv]) -> Result<Vec<Signal>, TradeError> {
        if data.len() < self.long_period {
            return Err(TradeError::InsufficientData(format!(
                "Need at least {} data points",
                self.long_period
            )));
        }

        let close_prices: Vec<f64> = data.iter().map(|d| d.data.close).collect();
        let mut signals = vec![Signal::Hold; data.len()];

        // Create SMA indicators using rustalib (through our adapter)
        let mut short_sma = SimpleMovingAverage::new(self.short_period).map_err(|e| {
            TradeError::CalculationError(format!("Failed to create short SMA: {}", e))
        })?;

        let mut long_sma = SimpleMovingAverage::new(self.long_period).map_err(|e| {
            TradeError::CalculationError(format!("Failed to create long SMA: {}", e))
        })?;

        let mut prev_short_value = None;
        let mut prev_long_value = None;

        for (i, &price) in close_prices.iter().enumerate() {
            // Update indicators with the current price
            short_sma.update(price).map_err(|e| {
                TradeError::CalculationError(format!("Failed to update short SMA: {}", e))
            })?;

            long_sma.update(price).map_err(|e| {
                TradeError::CalculationError(format!("Failed to update long SMA: {}", e))
            })?;

            // Skip until we have enough data points
            if i < self.long_period - 1 {
                continue;
            }

            // Get current values
            let short_value = short_sma.value().map_err(|e| {
                TradeError::CalculationError(format!("Failed to get short SMA value: {}", e))
            })?;

            let long_value = long_sma.value().map_err(|e| {
                TradeError::CalculationError(format!("Failed to get long SMA value: {}", e))
            })?;

            // Check for crossovers if we have previous values
            if let (Some(prev_short), Some(prev_long)) = (prev_short_value, prev_long_value) {
                if short_value > long_value && prev_short <= prev_long {
                    signals[i] = Signal::Buy;
                } else if short_value < long_value && prev_short >= prev_long {
                    signals[i] = Signal::Sell;
                }
            }

            prev_short_value = Some(short_value);
            prev_long_value = Some(long_value);
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
