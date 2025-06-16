//! ATR (Average True Range) Strategy
//!
//! The Average True Range (ATR) is a volatility indicator that measures the
//! magnitude of price movements. It's essential for position sizing, stop-loss
//! placement, and identifying breakout opportunities in day trading.
//!
//! ATR is calculated as the moving average of True Range values, where True Range
//! is the maximum of:
//! - Current High - Current Low
//! - |Current High - Previous Close|
//! - |Current Low - Previous Close|

use crate::simple_types::{NyxsOwlError, Result as NyxsOwlResult, Signal};
use crate::technical_strategies::{PerformanceMetrics, TechnicalSignal, TechnicalStrategy};
use crate::technical_strategies::{Strategy, StrategyConfig};
use crate::trade_math::volatility::calculate_atr;
use polars::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// ATR strategy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ATRConfig {
    /// ATR calculation period (default: 14)
    pub period: usize,
    /// ATR expansion threshold for volatility breakout (default: 1.5)
    pub expansion_threshold: f64,
    /// ATR contraction threshold for low volatility (default: 0.7)
    pub contraction_threshold: f64,
    /// ATR multiplier for stop-loss placement (default: 2.0)
    pub stop_loss_multiplier: f64,
    /// ATR multiplier for position sizing (default: 0.02)
    pub position_size_multiplier: f64,
}

impl Default for ATRConfig {
    fn default() -> Self {
        Self {
            period: 14,
            expansion_threshold: 1.5,
            contraction_threshold: 0.7,
            stop_loss_multiplier: 2.0,
            position_size_multiplier: 0.02,
        }
    }
}

/// ATR-based volatility trading strategy
#[derive(Debug, Clone)]
pub struct ATRStrategy {
    config: ATRConfig,
    strategy_config: StrategyConfig,
}

impl ATRStrategy {
    /// Create a new ATR strategy
    pub fn new(config: ATRConfig) -> Self {
        Self {
            config,
            strategy_config: StrategyConfig::default(),
        }
    }

    /// Calculate ATR values
    pub fn calculate_atr_values(&self, data: &DataFrame) -> NyxsOwlResult<Vec<f64>> {
        let atr_series = calculate_atr(data, self.config.period)?;
        let atr_values = atr_series.f64()?;

        let mut result = Vec::new();
        for i in 0..atr_values.len() {
            result.push(atr_values.get(i).unwrap_or(0.0));
        }

        Ok(result)
    }

    /// Calculate ATR-based volatility ratio
    fn calculate_volatility_ratio(&self, atr_values: &[f64], index: usize) -> f64 {
        if index < self.config.period || atr_values.is_empty() {
            return 1.0;
        }

        let current_atr = atr_values[index];
        if current_atr <= 0.0 {
            return 1.0;
        }

        // Calculate average ATR over the past period
        let start_idx = index.saturating_sub(self.config.period);
        let avg_atr: f64 = atr_values[start_idx..index]
            .iter()
            .filter(|&&x| x > 0.0)
            .sum::<f64>()
            / (index - start_idx) as f64;

        if avg_atr <= 0.0 {
            return 1.0;
        }

        current_atr / avg_atr
    }

    /// Calculate position size based on ATR
    pub fn calculate_position_size(&self, price: f64, atr: f64, account_balance: f64) -> f64 {
        if atr <= 0.0 || price <= 0.0 {
            return 0.0;
        }

        // Risk percentage of account based on ATR
        let risk_amount = account_balance * self.config.position_size_multiplier;
        let position_value = risk_amount / atr;

        position_value / price
    }

    /// Calculate stop-loss level based on ATR
    pub fn calculate_stop_loss(&self, entry_price: f64, atr: f64, is_long: bool) -> f64 {
        if atr <= 0.0 {
            return entry_price;
        }

        let stop_distance = atr * self.config.stop_loss_multiplier;

        if is_long {
            entry_price - stop_distance
        } else {
            entry_price + stop_distance
        }
    }

    /// Generate trading signals based on ATR
    fn generate_signals(
        &self,
        data: &DataFrame,
        atr_values: &[f64],
    ) -> NyxsOwlResult<Vec<TechnicalSignal>> {
        let close_prices = data.column("close")?.f64()?;
        let mut signals = Vec::new();

        for i in 0..atr_values.len() {
            if i < self.config.period {
                signals.push(TechnicalSignal::new(Signal::Hold));
                continue;
            }

            let current_atr = atr_values[i];
            let close_price = close_prices.get(i).unwrap_or(0.0);

            if current_atr <= 0.0 || close_price <= 0.0 {
                signals.push(TechnicalSignal::new(Signal::Hold));
                continue;
            }

            let volatility_ratio = self.calculate_volatility_ratio(atr_values, i);

            // Generate signals based on volatility patterns
            let (signal, strength, confidence) = if volatility_ratio
                >= self.config.expansion_threshold
            {
                // High volatility - potential breakout opportunity
                // Look for direction based on recent price action
                let recent_trend = if i >= 3 {
                    let prev_close = close_prices.get(i - 3).unwrap_or(close_price);
                    if prev_close > 0.0 {
                        (close_price - prev_close) / prev_close
                    } else {
                        0.0
                    }
                } else {
                    0.0
                };

                if recent_trend > 0.01 {
                    // Upward breakout
                    let strength = (volatility_ratio - self.config.expansion_threshold).min(1.0);
                    (Signal::Buy, strength, 0.7 + strength * 0.2)
                } else if recent_trend < -0.01 {
                    // Downward breakout
                    let strength = (volatility_ratio - self.config.expansion_threshold).min(1.0);
                    (Signal::Sell, strength, 0.7 + strength * 0.2)
                } else {
                    // High volatility but no clear direction
                    (Signal::Hold, 0.3, 0.4)
                }
            } else if volatility_ratio <= self.config.contraction_threshold {
                // Low volatility - expect upcoming breakout, but hold for now
                (Signal::Hold, 0.2, 0.8) // High confidence in low volatility
            } else {
                // Normal volatility range
                (Signal::Hold, 0.1, 0.5)
            };

            let position_size = self.calculate_position_size(close_price, current_atr, 10000.0); // Assume $10k account
            let stop_loss_long = self.calculate_stop_loss(close_price, current_atr, true);
            let stop_loss_short = self.calculate_stop_loss(close_price, current_atr, false);

            let tech_signal = TechnicalSignal::new(signal)
                .with_strength(strength)
                .with_confidence(confidence)
                .with_metadata("atr", current_atr)
                .with_metadata("volatility_ratio", volatility_ratio)
                .with_metadata("position_size", position_size)
                .with_metadata("stop_loss_long", stop_loss_long)
                .with_metadata("stop_loss_short", stop_loss_short);

            signals.push(tech_signal);
        }

        Ok(signals)
    }
}

impl TechnicalStrategy for ATRStrategy {
    fn generate_enhanced_signals(&self, data: &DataFrame) -> NyxsOwlResult<Vec<TechnicalSignal>> {
        let atr_values = self.calculate_atr_values(data)?;
        self.generate_signals(data, &atr_values)
    }

    fn get_indicator_values(&self, data: &DataFrame) -> NyxsOwlResult<HashMap<String, Series>> {
        let atr_values = self.calculate_atr_values(data)?;
        let mut indicators = HashMap::new();

        let atr_series = Series::new("atr".into(), atr_values.clone());
        indicators.insert("atr".to_string(), atr_series);

        // Calculate volatility ratios
        let volatility_ratios: Vec<f64> = (0..atr_values.len())
            .map(|i| self.calculate_volatility_ratio(&atr_values, i))
            .collect();

        let volatility_ratio_series = Series::new("volatility_ratio".into(), volatility_ratios);
        indicators.insert("volatility_ratio".to_string(), volatility_ratio_series);

        Ok(indicators)
    }

    fn get_performance_metrics(
        &self,
        data: &DataFrame,
        signals: &[TechnicalSignal],
    ) -> NyxsOwlResult<PerformanceMetrics> {
        let close_prices = data.column("close")?.f64()?;
        let mut total_return = 0.0;
        let mut total_trades = 0;
        let mut winning_trades = 0;
        let mut returns = Vec::new();

        // Calculate returns considering ATR-based position sizing
        for i in 1..signals.len().min(close_prices.len()) {
            if signals[i - 1].signal != Signal::Hold {
                let prev_close = close_prices.get(i - 1).unwrap_or(0.0);
                let curr_close = close_prices.get(i).unwrap_or(0.0);

                if prev_close > 0.0 && curr_close > 0.0 {
                    let base_return = match signals[i - 1].signal {
                        Signal::Buy => (curr_close - prev_close) / prev_close,
                        Signal::Sell => (prev_close - curr_close) / prev_close,
                        Signal::Hold => 0.0,
                    };

                    // Scale return by signal strength (which represents volatility-adjusted confidence)
                    let adjusted_return = base_return * signals[i - 1].strength;

                    total_return += adjusted_return;
                    returns.push(adjusted_return);
                    total_trades += 1;

                    if adjusted_return > 0.0 {
                        winning_trades += 1;
                    }
                }
            }
        }

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
            max_drawdown: 0.0,
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

        if self.config.expansion_threshold <= self.config.contraction_threshold {
            return Err(NyxsOwlError::ValidationError(
                "Expansion threshold must be greater than contraction threshold".to_string(),
            ));
        }

        if self.config.stop_loss_multiplier <= 0.0 {
            return Err(NyxsOwlError::ValidationError(
                "Stop loss multiplier must be positive".to_string(),
            ));
        }

        if self.config.position_size_multiplier <= 0.0 || self.config.position_size_multiplier > 0.1
        {
            return Err(NyxsOwlError::ValidationError(
                "Position size multiplier must be between 0 and 0.1".to_string(),
            ));
        }

        Ok(())
    }
}

impl Strategy for ATRStrategy {
    fn new(config: StrategyConfig) -> Self {
        Self {
            config: ATRConfig::default(),
            strategy_config: config,
        }
    }

    fn name(&self) -> &str {
        "ATR Volatility Strategy"
    }

    fn description(&self) -> &str {
        "Average True Range based volatility strategy for position sizing and breakout detection"
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
        self.config.period + 10
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
            .map(|i| 100.0 + (i as f64 * 0.5) + ((i % 5) as f64) * 2.0)
            .collect();
        let lows: Vec<f64> = highs
            .iter()
            .map(|&h| h - 3.0 - ((h as usize % 4) as f64))
            .collect();
        let closes: Vec<f64> = highs
            .iter()
            .zip(lows.iter())
            .map(|(&h, &l)| l + (h - l) * 0.7)
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
    fn test_atr_calculation() {
        let strategy = ATRStrategy::new(ATRConfig::default());
        let data = create_test_data();

        let atr_values = strategy.calculate_atr_values(&data).unwrap();

        // First few values might be NaN due to the calculation period
        for i in 14..atr_values.len() {
            assert!(atr_values[i] >= 0.0, "ATR values should be non-negative");
        }
    }

    #[test]
    fn test_position_sizing() {
        let strategy = ATRStrategy::new(ATRConfig::default());

        let position_size = strategy.calculate_position_size(100.0, 2.0, 10000.0);
        assert!(position_size > 0.0, "Position size should be positive");

        // Higher ATR should result in smaller position size (for same risk)
        let position_size_high_atr = strategy.calculate_position_size(100.0, 4.0, 10000.0);
        assert!(
            position_size_high_atr < position_size,
            "Higher ATR should yield smaller position size"
        );
    }

    #[test]
    fn test_stop_loss_calculation() {
        let strategy = ATRStrategy::new(ATRConfig::default());

        let long_stop = strategy.calculate_stop_loss(100.0, 2.0, true);
        let short_stop = strategy.calculate_stop_loss(100.0, 2.0, false);

        assert!(
            long_stop < 100.0,
            "Long stop loss should be below entry price"
        );
        assert!(
            short_stop > 100.0,
            "Short stop loss should be above entry price"
        );
    }

    #[test]
    fn test_signal_generation() {
        let strategy = ATRStrategy::new(ATRConfig::default());
        let data = create_test_data();

        let signals = strategy.generate_enhanced_signals(&data).unwrap();

        assert_eq!(signals.len(), data.height());

        // Check that signals have ATR metadata
        for signal in &signals {
            if signal.signal != Signal::Hold {
                assert!(signal.metadata.contains_key("atr"));
                assert!(signal.metadata.contains_key("volatility_ratio"));
                assert!(signal.metadata.contains_key("position_size"));
            }
        }
    }

    #[test]
    fn test_parameter_validation() {
        let mut config = ATRConfig::default();
        config.period = 0;
        let strategy = ATRStrategy::new(config);
        assert!(strategy.validate_parameters().is_err());

        let mut config2 = ATRConfig::default();
        config2.expansion_threshold = 0.5;
        config2.contraction_threshold = 1.0;
        let strategy2 = ATRStrategy::new(config2);
        assert!(strategy2.validate_parameters().is_err());
    }
}
