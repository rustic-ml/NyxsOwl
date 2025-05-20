//! Adaptive Moving Average Strategy
//!
//! Adjusts the moving average period based on market volatility
//! Uses shorter periods in volatile markets and longer periods in stable markets

use crate::{DailyOhlcv, Signal, TradeError, TradingStrategy};
use trade_math::moving_averages::SimpleMovingAverage;
use trade_math::volatility::AverageTrueRange;

/// Adaptive Moving Average strategy parameters
pub struct AdaptiveMovingAverageStrategy {
    /// Base period for moving average
    base_period: usize,
    /// Minimum period to use in very volatile markets
    min_period: usize,
    /// Maximum period to use in very stable markets
    max_period: usize,
    /// ATR period for volatility calculation
    atr_period: usize,
    /// ATR multiplier to determine volatility thresholds
    volatility_factor: f64,
}

impl AdaptiveMovingAverageStrategy {
    /// Create a new adaptive moving average strategy with given parameters
    pub fn new(
        base_period: usize,
        min_period: usize,
        max_period: usize,
        atr_period: usize,
        volatility_factor: f64,
    ) -> Result<Self, String> {
        if base_period < 2 {
            return Err("Base period must be at least 2".to_string());
        }

        if min_period < 2 || min_period >= base_period {
            return Err("Minimum period must be at least 2 and less than base period".to_string());
        }

        if max_period <= base_period {
            return Err("Maximum period must be greater than base period".to_string());
        }

        if atr_period < 5 {
            return Err("ATR period must be at least 5".to_string());
        }

        if volatility_factor <= 0.0 {
            return Err("Volatility factor must be positive".to_string());
        }

        Ok(Self {
            base_period,
            min_period,
            max_period,
            atr_period,
            volatility_factor,
        })
    }

    /// Create a default adaptive moving average strategy
    /// Uses base_period=20, min_period=10, max_period=40, atr_period=14, volatility_factor=2.0
    pub fn default() -> Self {
        Self {
            base_period: 20,
            min_period: 10,
            max_period: 40,
            atr_period: 14,
            volatility_factor: 2.0,
        }
    }

    /// Calculate the adaptive period based on market volatility
    fn calculate_adaptive_period(
        &self,
        data: &[DailyOhlcv],
        current_index: usize,
        atr_value: f64,
    ) -> usize {
        // Calculate average price for reference
        let slice_start = if current_index >= 20 {
            current_index - 20
        } else {
            0
        };
        let avg_price: f64 = data[slice_start..=current_index]
            .iter()
            .map(|d| d.data.close)
            .sum::<f64>()
            / (current_index - slice_start + 1) as f64;

        // Calculate normalized volatility (ATR as percentage of price)
        let normalized_volatility = atr_value / avg_price;

        // Map volatility to period:
        // - High volatility -> shorter period
        // - Low volatility -> longer period
        let volatility_threshold = 0.01 * self.volatility_factor; // 1% × factor as threshold

        if normalized_volatility > volatility_threshold * 2.0 {
            // Very high volatility
            self.min_period
        } else if normalized_volatility > volatility_threshold {
            // High volatility
            let range = self.base_period - self.min_period;
            let volatility_ratio =
                (normalized_volatility - volatility_threshold) / volatility_threshold;
            let period_adjustment = (range as f64 * (1.0 - volatility_ratio)).round() as usize;
            self.min_period + period_adjustment
        } else if normalized_volatility < volatility_threshold / 2.0 {
            // Very low volatility
            self.max_period
        } else {
            // Low volatility
            let range = self.max_period - self.base_period;
            let volatility_ratio =
                (volatility_threshold - normalized_volatility) / (volatility_threshold / 2.0);
            let period_adjustment = (range as f64 * volatility_ratio).round() as usize;
            self.base_period + period_adjustment
        }
    }
}

impl TradingStrategy for AdaptiveMovingAverageStrategy {
    fn generate_signals(&self, data: &[DailyOhlcv]) -> Result<Vec<Signal>, TradeError> {
        let min_required = self.max_period.max(self.atr_period);

        if data.len() < min_required + 1 {
            return Err(TradeError::InsufficientData(format!(
                "Need at least {} data points for adaptive moving average strategy",
                min_required + 1
            )));
        }

        let mut signals = Vec::with_capacity(data.len());
        let mut atr = AverageTrueRange::new(self.atr_period)
            .map_err(|e| TradeError::CalculationError(e.to_string()))?;

        // Initialize ATR with initial data points without generating signals
        for i in 0..self.atr_period {
            atr.update(data[i].data.high, data[i].data.low, data[i].data.close)
                .map_err(|e| TradeError::CalculationError(e.to_string()))?;
            signals.push(Signal::Hold);
        }

        // Add hold signals until we have enough data
        for i in self.atr_period..min_required {
            atr.update(data[i].data.high, data[i].data.low, data[i].data.close)
                .map_err(|e| TradeError::CalculationError(e.to_string()))?;
            signals.push(Signal::Hold);
        }

        // Generate signals with adaptive MA
        let mut prev_ma_value: Option<f64> = None;

        for i in min_required..data.len() {
            atr.update(data[i].data.high, data[i].data.low, data[i].data.close)
                .map_err(|e| TradeError::CalculationError(e.to_string()))?;

            let atr_value = atr
                .value()
                .map_err(|e| TradeError::CalculationError(e.to_string()))?;

            // Calculate adaptive period based on volatility
            let adaptive_period = self.calculate_adaptive_period(data, i, atr_value);

            // Create MA with adaptive period
            let mut adaptive_ma = SimpleMovingAverage::new(adaptive_period)
                .map_err(|e| TradeError::CalculationError(e.to_string()))?;

            // Update MA with data
            let start_idx = if i >= adaptive_period {
                i - adaptive_period + 1
            } else {
                0
            };
            for j in start_idx..=i {
                adaptive_ma
                    .update(data[j].data.close)
                    .map_err(|e| TradeError::CalculationError(e.to_string()))?;
            }

            let ma_value = adaptive_ma
                .value()
                .map_err(|e| TradeError::CalculationError(e.to_string()))?;

            // Generate signal based on price crossing MA
            let signal = if let Some(prev_ma) = prev_ma_value {
                if data[i].data.close > ma_value && data[i - 1].data.close <= prev_ma {
                    // Price crossed above MA - bullish
                    Signal::Buy
                } else if data[i].data.close < ma_value && data[i - 1].data.close >= prev_ma {
                    // Price crossed below MA - bearish
                    Signal::Sell
                } else {
                    Signal::Hold
                }
            } else {
                Signal::Hold
            };

            signals.push(signal);
            prev_ma_value = Some(ma_value);
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
