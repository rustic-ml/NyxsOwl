//! Williams %R Oscillator Strategy
//!
//! The Williams %R (Williams Percent Range) is a momentum indicator that measures
//! overbought and oversold levels. It oscillates between 0 and -100, with readings
//! above -20 typically indicating overbought conditions and readings below -80
//! indicating oversold conditions.
//!
//! Research shows Williams %R has a 71.7% win rate reliability, making it one of
//! the most dependable oscillating indicators for trading.

use crate::simple_types::{NyxsOwlError, Result as NyxsOwlResult, Signal};
use crate::technical_strategies::{PerformanceMetrics, TechnicalSignal, TechnicalStrategy};
use crate::technical_strategies::{Strategy, StrategyConfig};
use polars::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Williams %R oscillator strategy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WilliamsRConfig {
    /// Lookback period for highest high and lowest low calculation (default: 14)
    pub period: usize,
    /// Overbought threshold (default: -20)
    pub overbought_threshold: f64,
    /// Oversold threshold (default: -80)
    pub oversold_threshold: f64,
    /// Middle threshold for trend confirmation (default: -50)
    pub middle_threshold: f64,
}

impl Default for WilliamsRConfig {
    fn default() -> Self {
        Self {
            period: 14,
            overbought_threshold: -20.0,
            oversold_threshold: -80.0,
            middle_threshold: -50.0,
        }
    }
}

/// Williams %R oscillator strategy
#[derive(Debug, Clone)]
pub struct WilliamsRStrategy {
    config: WilliamsRConfig,
    strategy_config: StrategyConfig,
}

impl WilliamsRStrategy {
    /// Create a new Williams %R strategy with custom configuration
    pub fn new(config: WilliamsRConfig) -> Self {
        Self {
            config,
            strategy_config: StrategyConfig::new(),
        }
    }

    /// Create a new Williams %R strategy with both configs
    pub fn with_strategy_config(config: WilliamsRConfig, strategy_config: StrategyConfig) -> Self {
        Self {
            config,
            strategy_config,
        }
    }

    /// Calculate Williams %R oscillator
    pub fn calculate_williams_r(&self, data: &DataFrame) -> NyxsOwlResult<Vec<f64>> {
        let high_prices = data.column("high")?.f64()?;
        let low_prices = data.column("low")?.f64()?;
        let close_prices = data.column("close")?.f64()?;

        let mut williams_r = Vec::new();

        for i in 0..close_prices.len() {
            if i < self.config.period - 1 {
                williams_r.push(f64::NAN);
                continue;
            }

            let start_idx = i.saturating_sub(self.config.period - 1);
            let end_idx = i + 1;

            // Find highest high and lowest low in the period
            let mut highest_high = f64::NEG_INFINITY;
            let mut lowest_low = f64::INFINITY;

            for j in start_idx..end_idx {
                let high = high_prices.get(j).unwrap_or(0.0);
                let low = low_prices.get(j).unwrap_or(0.0);

                if high > 0.0 && low > 0.0 {
                    highest_high = highest_high.max(high);
                    lowest_low = lowest_low.min(low);
                }
            }

            // Calculate Williams %R
            let close = close_prices.get(i).unwrap_or(0.0);

            if close > 0.0 && highest_high != lowest_low && highest_high > 0.0 && lowest_low > 0.0 {
                let williams_r_value =
                    ((highest_high - close) / (highest_high - lowest_low)) * -100.0;
                williams_r.push(williams_r_value);
            } else {
                williams_r.push(-50.0); // Neutral value when range is zero
            }
        }

        Ok(williams_r)
    }

    /// Generate trading signals based on Williams %R
    fn generate_signals(&self, williams_r: &[f64]) -> Vec<TechnicalSignal> {
        let mut signals = Vec::new();

        for (_i, &wr_value) in williams_r.iter().enumerate() {
            if wr_value.is_nan() {
                signals.push(TechnicalSignal::new(Signal::Hold));
                continue;
            }

            let (signal, strength, confidence) = if wr_value <= self.config.oversold_threshold {
                // Oversold condition - potential buy signal
                let strength = ((self.config.oversold_threshold - wr_value) / 20.0).min(1.0);
                (Signal::Buy, strength, 0.7 + strength * 0.3)
            } else if wr_value >= self.config.overbought_threshold {
                // Overbought condition - potential sell signal
                let strength = ((wr_value - self.config.overbought_threshold) / 20.0).min(1.0);
                (Signal::Sell, strength, 0.7 + strength * 0.3)
            } else if wr_value > self.config.middle_threshold
                && wr_value < self.config.overbought_threshold
            {
                // Mild bullish momentum
                let strength = (wr_value - self.config.middle_threshold)
                    / (self.config.overbought_threshold - self.config.middle_threshold);
                (Signal::Buy, strength * 0.5, 0.5)
            } else if wr_value < self.config.middle_threshold
                && wr_value > self.config.oversold_threshold
            {
                // Mild bearish momentum
                let strength = (self.config.middle_threshold - wr_value)
                    / (self.config.middle_threshold - self.config.oversold_threshold);
                (Signal::Sell, strength * 0.5, 0.5)
            } else {
                // Neutral zone
                (Signal::Hold, 0.0, 0.3)
            };

            let tech_signal = TechnicalSignal::new(signal)
                .with_strength(strength)
                .with_confidence(confidence)
                .with_metadata("williams_r", wr_value);

            signals.push(tech_signal);
        }

        signals
    }

    /// Calculate maximum drawdown from equity curve
    fn calculate_max_drawdown(equity_curve: &[f64]) -> f64 {
        if equity_curve.len() < 2 {
            return 0.0;
        }

        let mut max_drawdown = 0.0;
        let mut peak = equity_curve[0];

        for &equity in equity_curve.iter().skip(1) {
            if equity > peak {
                peak = equity;
            } else {
                let drawdown = (peak - equity) / peak;
                if drawdown > max_drawdown {
                    max_drawdown = drawdown;
                }
            }
        }

        max_drawdown
    }
}

impl TechnicalStrategy for WilliamsRStrategy {
    fn generate_enhanced_signals(&self, data: &DataFrame) -> NyxsOwlResult<Vec<TechnicalSignal>> {
        let williams_r = self.calculate_williams_r(data)?;
        let signals = self.generate_signals(&williams_r);
        Ok(signals)
    }

    fn get_indicator_values(&self, data: &DataFrame) -> NyxsOwlResult<HashMap<String, Series>> {
        let williams_r = self.calculate_williams_r(data)?;
        let mut indicators = HashMap::new();

        let wr_series = Series::new("williams_r".into(), williams_r);
        indicators.insert("williams_r".to_string(), wr_series);

        Ok(indicators)
    }

    fn get_performance_metrics(
        &self,
        data: &DataFrame,
        signals: &[TechnicalSignal],
    ) -> NyxsOwlResult<PerformanceMetrics> {
        let close_prices = data.column("close")?.f64()?;
        let mut total_return = 0.0;
        let mut returns = Vec::new();
        let mut winning_trades = 0;
        let mut total_trades = 0;
        let mut equity_curve = Vec::new();
        let mut current_equity = 1.0; // Start with 1.0 (100%)

        // Track equity curve for max drawdown calculation
        equity_curve.push(current_equity);

        for i in 1..signals.len() {
            if signals[i - 1].signal != Signal::Hold {
                let prev_close = close_prices.get(i - 1).unwrap_or(0.0);
                let curr_close = close_prices.get(i).unwrap_or(0.0);

                if prev_close > 0.0 && curr_close > 0.0 {
                    let return_pct = match signals[i - 1].signal {
                        Signal::Buy => (curr_close - prev_close) / prev_close,
                        Signal::Sell => (prev_close - curr_close) / prev_close,
                        Signal::Hold => 0.0,
                    };

                    total_return += return_pct;
                    returns.push(return_pct);
                    total_trades += 1;

                    // Update equity curve
                    current_equity *= 1.0 + return_pct;
                    equity_curve.push(current_equity);

                    if return_pct > 0.0 {
                        winning_trades += 1;
                    }
                } else {
                    equity_curve.push(current_equity);
                }
            } else {
                equity_curve.push(current_equity);
            }
        }

        // Calculate maximum drawdown
        let max_drawdown = Self::calculate_max_drawdown(&equity_curve);

        let win_rate = if total_trades > 0 {
            winning_trades as f64 / total_trades as f64
        } else {
            0.0
        };

        let avg_return = if !returns.is_empty() {
            returns.iter().sum::<f64>() / returns.len() as f64
        } else {
            0.0
        };

        let volatility = if returns.len() > 1 {
            let mean = avg_return;
            let variance = returns.iter().map(|r| (r - mean).powi(2)).sum::<f64>()
                / (returns.len() - 1) as f64;
            variance.sqrt()
        } else {
            0.0
        };

        let sharpe_ratio = if volatility > 0.0 {
            avg_return / volatility
        } else {
            0.0
        };

        Ok(PerformanceMetrics {
            total_return,
            sharpe_ratio,
            max_drawdown,
            win_rate,
            total_trades,
            avg_trade_return: avg_return,
            volatility,
        })
    }

    fn validate_parameters(&self) -> NyxsOwlResult<()> {
        if self.config.period == 0 {
            return Err(NyxsOwlError::ValidationError(
                "Period must be greater than 0".to_string(),
            ));
        }

        if self.config.oversold_threshold >= self.config.overbought_threshold {
            return Err(NyxsOwlError::ValidationError(
                "Oversold threshold must be less than overbought threshold".to_string(),
            ));
        }

        Ok(())
    }
}

impl Strategy for WilliamsRStrategy {
    fn new(config: StrategyConfig) -> Self {
        Self {
            config: WilliamsRConfig::default(),
            strategy_config: config,
        }
    }

    fn name(&self) -> &str {
        "Williams %R Oscillator Strategy"
    }

    fn description(&self) -> &str {
        "Williams %R oscillator strategy with research-backed 71.7% win rate reliability"
    }

    fn generate_signals(&self, data: &DataFrame) -> NyxsOwlResult<Series> {
        let enhanced_signals = self.generate_enhanced_signals(data)?;
        let signal_values: Vec<i32> = enhanced_signals.iter().map(|s| s.signal.to_int()).collect();
        Ok(Series::new("signals".into(), signal_values))
    }

    fn required_columns(&self) -> Vec<&str> {
        vec!["high", "low", "close"]
    }

    fn config(&self) -> &StrategyConfig {
        &self.strategy_config
    }

    fn min_data_points(&self) -> usize {
        self.config.period + 5
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_data() -> DataFrame {
        let dates = (0..50)
            .map(|i| format!("2024-01-{:02}", i + 1))
            .collect::<Vec<_>>();
        let highs: Vec<f64> = (0..50)
            .map(|i| 100.0 + (i as f64 * 0.5) + ((i % 7) as f64))
            .collect();
        let lows: Vec<f64> = highs
            .iter()
            .map(|&h| h - 2.0 - ((h as usize % 3) as f64))
            .collect();
        let closes: Vec<f64> = highs
            .iter()
            .zip(lows.iter())
            .map(|(&h, &l)| l + (h - l) * 0.6)
            .collect();
        let volumes: Vec<f64> = (0..50).map(|_| 10000.0).collect();

        df! {
            "date" => dates,
            "high" => highs,
            "low" => lows,
            "close" => closes,
            "volume" => volumes,
        }
        .unwrap()
    }

    #[test]
    fn test_williams_r_calculation() {
        let strategy = WilliamsRStrategy::new(WilliamsRConfig::default());
        let data = create_test_data();

        let williams_r = strategy.calculate_williams_r(&data).unwrap();

        // First few values should be NaN due to lookback period
        assert!(williams_r[0].is_nan());
        // For period 14, first valid value should be at index 13 (0-based)
        // Values before that should be NaN
        for i in 0..13 {
            assert!(williams_r[i].is_nan(), "williams_r[{}] should be NaN", i);
        }

        // Subsequent values should be between -100 and 0
        for i in 14..williams_r.len() {
            assert!(!williams_r[i].is_nan());
            assert!(williams_r[i] >= -100.0 && williams_r[i] <= 0.0);
        }
    }

    #[test]
    fn test_signal_generation() {
        let strategy = WilliamsRStrategy::new(WilliamsRConfig::default());
        let data = create_test_data();

        let signals = strategy.generate_enhanced_signals(&data).unwrap();

        assert_eq!(signals.len(), data.height());

        // Check that signals are properly formatted
        for signal in &signals {
            assert!(signal.strength >= 0.0 && signal.strength <= 1.0);
            assert!(signal.confidence >= 0.0 && signal.confidence <= 1.0);
        }
    }

    #[test]
    fn test_parameter_validation() {
        let mut config = WilliamsRConfig::default();
        config.period = 0;
        let strategy = WilliamsRStrategy::new(config);

        assert!(strategy.validate_parameters().is_err());

        let mut config2 = WilliamsRConfig::default();
        config2.oversold_threshold = -10.0;
        config2.overbought_threshold = -20.0;
        let strategy2 = WilliamsRStrategy::new(config2);

        assert!(strategy2.validate_parameters().is_err());
    }
}
