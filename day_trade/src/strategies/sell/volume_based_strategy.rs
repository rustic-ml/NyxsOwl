//! Volume-based Trading Strategy
//!
//! Uses volume indicators to identify potential trend changes
//! This strategy combines On-Balance Volume and Volume Price Trend

use crate::{DailyOhlcv, Signal, TradeError, TradingStrategy};
use trade_math::moving_averages::SimpleMovingAverage;
use trade_math::volume::{OnBalanceVolume, VolumePriceTrend};

/// Volume-based trading strategy parameters
pub struct VolumeBasedStrategy {
    /// Period for OBV moving average
    obv_ma_period: usize,
    /// Period for VPT moving average
    vpt_ma_period: usize,
}

impl VolumeBasedStrategy {
    /// Create a new volume-based strategy with given parameters
    pub fn new(obv_ma_period: usize, vpt_ma_period: usize) -> Result<Self, String> {
        if obv_ma_period < 2 {
            return Err("OBV MA period must be at least 2".to_string());
        }

        if vpt_ma_period < 2 {
            return Err("VPT MA period must be at least 2".to_string());
        }

        Ok(Self {
            obv_ma_period,
            vpt_ma_period,
        })
    }

    /// Create a default volume-based strategy
    /// Uses obv_ma_period=20, vpt_ma_period=14
    pub fn default() -> Self {
        Self {
            obv_ma_period: 20,
            vpt_ma_period: 14,
        }
    }
}

impl TradingStrategy for VolumeBasedStrategy {
    fn generate_signals(&self, data: &[DailyOhlcv]) -> Result<Vec<Signal>, TradeError> {
        let min_required = self.obv_ma_period.max(self.vpt_ma_period);

        if data.len() < min_required + 1 {
            return Err(TradeError::InsufficientData(format!(
                "Need at least {} data points for volume-based strategy",
                min_required + 1
            )));
        }

        let mut signals = Vec::with_capacity(data.len());
        let mut obv = OnBalanceVolume::new();
        let mut vpt = VolumePriceTrend::new();

        let mut obv_ma = SimpleMovingAverage::new(self.obv_ma_period)
            .map_err(|e| TradeError::CalculationError(e.to_string()))?;
        let mut vpt_ma = SimpleMovingAverage::new(self.vpt_ma_period)
            .map_err(|e| TradeError::CalculationError(e.to_string()))?;

        let mut obv_values = Vec::with_capacity(data.len());
        let mut vpt_values = Vec::with_capacity(data.len());

        // Calculate OBV and VPT for all data points
        for i in 0..data.len() {
            obv.update(data[i].data.close, data[i].data.volume as f64)
                .map_err(|e| TradeError::CalculationError(e.to_string()))?;

            vpt.update(data[i].data.close, data[i].data.volume as f64)
                .map_err(|e| TradeError::CalculationError(e.to_string()))?;

            // Skip the first data point for OBV/VPT as they don't have valid values yet
            if i > 0 {
                obv_values.push(
                    obv.value()
                        .map_err(|e| TradeError::CalculationError(e.to_string()))?,
                );

                vpt_values.push(
                    vpt.value()
                        .map_err(|e| TradeError::CalculationError(e.to_string()))?,
                );
            }
        }

        // Add hold signals for the first data points
        for _ in 0..min_required {
            signals.push(Signal::Hold);
        }

        // Generate signals using moving averages of OBV and VPT
        for i in min_required..data.len() {
            // Index adjustment for obv_values and vpt_values (they start at index 1 of data)
            let index = i - 1;

            obv_ma
                .update(obv_values[index])
                .map_err(|e| TradeError::CalculationError(e.to_string()))?;

            vpt_ma
                .update(vpt_values[index])
                .map_err(|e| TradeError::CalculationError(e.to_string()))?;

            // Only start generating signals when we have enough data
            if i >= min_required + 1 {
                let obv_ma_value = obv_ma
                    .value()
                    .map_err(|e| TradeError::CalculationError(e.to_string()))?;

                let vpt_ma_value = vpt_ma
                    .value()
                    .map_err(|e| TradeError::CalculationError(e.to_string()))?;

                // Check if OBV and VPT are both above their MAs
                let obv_above_ma = obv_values[index] > obv_ma_value;
                let vpt_above_ma = vpt_values[index] > vpt_ma_value;

                // Check if price is trending with volume
                if obv_above_ma && vpt_above_ma && data[i].data.close > data[i - 1].data.close {
                    // Strong buy signal
                    signals.push(Signal::Buy);
                } else if !obv_above_ma
                    && !vpt_above_ma
                    && data[i].data.close < data[i - 1].data.close
                {
                    // Strong sell signal
                    signals.push(Signal::Sell);
                } else {
                    // No strong signal
                    signals.push(Signal::Hold);
                }
            } else {
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
