//! Breakout Trading Strategy
//!
//! Identifies when price breaks through support or resistance levels
//! Breakouts can signal the start of a new trend

use crate::{DailyOhlcv, Signal, TradeError, TradingStrategy};
use trade_math::volatility::AverageTrueRange;

/// Breakout trading strategy parameters
pub struct BreakoutStrategy {
    /// Lookback period for finding support/resistance
    lookback_period: usize,
    /// ATR multiplier for confirming breakout
    atr_multiplier: f64,
    /// ATR period for calculating volatility
    atr_period: usize,
}

impl BreakoutStrategy {
    /// Create a new breakout strategy with given parameters
    pub fn new(
        lookback_period: usize,
        atr_multiplier: f64,
        atr_period: usize,
    ) -> Result<Self, String> {
        if lookback_period < 2 {
            return Err("Lookback period must be at least 2".to_string());
        }

        if atr_multiplier <= 0.0 {
            return Err("ATR multiplier must be positive".to_string());
        }

        if atr_period < 2 {
            return Err("ATR period must be at least 2".to_string());
        }

        Ok(Self {
            lookback_period,
            atr_multiplier,
            atr_period,
        })
    }

    /// Create a default breakout strategy
    /// Uses lookback_period=20, atr_multiplier=1.5, atr_period=14
    pub fn default() -> Self {
        Self {
            lookback_period: 20,
            atr_multiplier: 1.5,
            atr_period: 14,
        }
    }

    /// Find the highest high in the lookback period
    fn find_resistance(&self, data: &[DailyOhlcv], current_index: usize) -> f64 {
        let start = if current_index >= self.lookback_period {
            current_index - self.lookback_period
        } else {
            0
        };

        let mut highest = data[start].data.high;
        for i in start..current_index {
            if data[i].data.high > highest {
                highest = data[i].data.high;
            }
        }

        highest
    }

    /// Find the lowest low in the lookback period
    fn find_support(&self, data: &[DailyOhlcv], current_index: usize) -> f64 {
        let start = if current_index >= self.lookback_period {
            current_index - self.lookback_period
        } else {
            0
        };

        let mut lowest = data[start].data.low;
        for i in start..current_index {
            if data[i].data.low < lowest {
                lowest = data[i].data.low;
            }
        }

        lowest
    }
}

impl TradingStrategy for BreakoutStrategy {
    fn generate_signals(&self, data: &[DailyOhlcv]) -> Result<Vec<Signal>, TradeError> {
        let min_required = self.lookback_period.max(self.atr_period);

        if data.len() < min_required + 1 {
            return Err(TradeError::InsufficientData(format!(
                "Need at least {} data points for breakout strategy",
                min_required + 1
            )));
        }

        let mut signals = Vec::with_capacity(data.len());
        let mut atr = AverageTrueRange::new(self.atr_period)
            .map_err(|e| TradeError::CalculationError(e.to_string()))?;

        // Initialize ATR with initial data points
        for i in 0..self.atr_period {
            atr.update(data[i].data.high, data[i].data.low, data[i].data.close)
                .map_err(|e| TradeError::CalculationError(e.to_string()))?;
            signals.push(Signal::Hold);
        }

        // Hold for the rest of the lookback period
        for i in self.atr_period..min_required {
            atr.update(data[i].data.high, data[i].data.low, data[i].data.close)
                .map_err(|e| TradeError::CalculationError(e.to_string()))?;
            signals.push(Signal::Hold);
        }

        // Generate signals for the rest of the data
        for i in min_required..data.len() {
            atr.update(data[i].data.high, data[i].data.low, data[i].data.close)
                .map_err(|e| TradeError::CalculationError(e.to_string()))?;

            let atr_value = atr
                .value()
                .map_err(|e| TradeError::CalculationError(e.to_string()))?;

            let resistance = self.find_resistance(data, i);
            let support = self.find_support(data, i);

            let breakout_threshold = atr_value * self.atr_multiplier;

            // Check for breakouts
            if data[i].data.close > resistance + breakout_threshold {
                // Bullish breakout
                signals.push(Signal::Buy);
            } else if data[i].data.close < support - breakout_threshold {
                // Bearish breakout
                signals.push(Signal::Sell);
            } else {
                // No breakout
                signals.push(Signal::Hold);
            }
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
                "Data and signals arrays must be the same length".to_string(),
            ));
        }

        if data.len() <= 1 {
            return Err(TradeError::InsufficientData(
                "Need at least 2 data points to calculate performance".to_string(),
            ));
        }

        let mut cash = 10000.0; // Initial cash
        let mut shares = 0.0; // Initial shares

        for i in 1..data.len() {
            match signals[i - 1] {
                Signal::Buy => {
                    // Buy shares with all available cash
                    shares = cash / data[i].data.open;
                    cash = 0.0;
                }
                Signal::Sell => {
                    // Sell all shares
                    cash += shares * data[i].data.open;
                    shares = 0.0;
                }
                Signal::Hold => {} // Do nothing
            }
        }

        // Calculate final portfolio value
        let final_value = cash + shares * data.last().unwrap().data.close;

        // Calculate performance as percent return
        let performance = (final_value / 10000.0 - 1.0) * 100.0;

        Ok(performance)
    }
}
