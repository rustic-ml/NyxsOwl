//! Volume-based technical strategies
//!
//! This module implements trading strategies based on volume analysis,
//! including Volume Weighted Average Price (VWAP), On-Balance Volume (OBV),
//! and other volume-based indicators.

use crate::simple_types::{NyxsOwlError, Result as NyxsOwlResult, Signal};
use crate::technical_strategies::{PerformanceMetrics, TechnicalSignal, TechnicalStrategy};
use crate::technical_strategies::{Strategy, StrategyConfig};
use crate::trade_math::{calculate_obv, calculate_vwap};
use polars::prelude::{DataFrame, NamedFrom, Series};
use std::collections::HashMap;

/// Volume Weighted Average Price (VWAP) Strategy
#[derive(Debug, Clone)]
pub struct VWAPStrategy {
    config: StrategyConfig,
}

impl Strategy for VWAPStrategy {
    fn new(config: StrategyConfig) -> Self {
        Self { config }
    }

    fn generate_signals(&self, data: &DataFrame) -> NyxsOwlResult<Series> {
        let enhanced_signals = self.generate_enhanced_signals(data)?;
        let signals: Vec<i32> = enhanced_signals.iter().map(|s| s.signal.to_int()).collect();
        Ok(Series::new("signals".into(), signals))
    }

    fn name(&self) -> &str {
        "VWAP Strategy"
    }

    fn description(&self) -> &str {
        "Volume Weighted Average Price based trading strategy"
    }

    fn required_columns(&self) -> Vec<&str> {
        vec!["close", "volume", "high", "low"]
    }

    fn config(&self) -> &StrategyConfig {
        &self.config
    }

    fn min_data_points(&self) -> usize {
        self.config.get_int("min_data_points").unwrap_or(20) as usize
    }
}

impl TechnicalStrategy for VWAPStrategy {
    fn generate_enhanced_signals(&self, data: &DataFrame) -> NyxsOwlResult<Vec<TechnicalSignal>> {
        self.validate_data(data)?;
        self.validate_parameters()?;

        let close = data.column("close")?;
        let volume = data.column("volume")?;
        let high = data.column("high")?;
        let low = data.column("low")?;

        // Calculate VWAP
        let vwap = calculate_vwap(data)?;
        let vwap_values = vwap.f64()?;
        let close_values = close.f64()?;

        let threshold = self.config.get_float("signal_threshold").unwrap_or(0.01);
        let min_volume_ratio = self.config.get_float("min_volume_ratio").unwrap_or(1.5);

        // Calculate volume ratio
        let volume_values = volume.f64()?;
        let avg_volume: f64 = volume_values
            .into_iter()
            .sum::<Option<f64>>()
            .unwrap_or(0.0)
            / volume_values.len() as f64;

        let mut signals = Vec::with_capacity(data.height());

        for i in 0..data.height() {
            let close_val = close_values.get(i).unwrap_or(0.0);
            let vwap_val = vwap_values.get(i).unwrap_or(0.0);
            let volume_val = volume_values.get(i).unwrap_or(0.0);

            if close_val == 0.0 || vwap_val == 0.0 {
                signals.push(TechnicalSignal::new(Signal::Hold));
                continue;
            }

            let price_deviation = (close_val - vwap_val) / vwap_val;
            let volume_ratio = if avg_volume > 0.0 {
                volume_val / avg_volume
            } else {
                1.0
            };

            let signal = if price_deviation > threshold && volume_ratio >= min_volume_ratio {
                Signal::Buy
            } else if price_deviation < -threshold && volume_ratio >= min_volume_ratio {
                Signal::Sell
            } else {
                Signal::Hold
            };

            let strength = (price_deviation.abs() / threshold).min(1.0);
            let confidence = (volume_ratio / min_volume_ratio).min(1.0);

            let tech_signal = TechnicalSignal::new(signal)
                .with_strength(strength)
                .with_confidence(confidence)
                .with_metadata("vwap", vwap_val)
                .with_metadata("price_deviation", price_deviation)
                .with_metadata("volume_ratio", volume_ratio);

            signals.push(tech_signal);
        }

        Ok(signals)
    }

    fn get_indicator_values(&self, data: &DataFrame) -> NyxsOwlResult<HashMap<String, Series>> {
        let vwap = calculate_vwap(data)?;
        let obv = calculate_obv(data)?;

        let mut indicators = HashMap::new();
        indicators.insert("vwap".to_string(), vwap);
        indicators.insert("obv".to_string(), obv);

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
        let max_drawdown = {
            if equity_curve.len() < 2 {
                0.0
            } else {
                let mut max_dd = 0.0;
                let mut peak = equity_curve[0];

                for &equity in equity_curve.iter().skip(1) {
                    if equity > peak {
                        peak = equity;
                    } else {
                        let drawdown = (peak - equity) / peak;
                        if drawdown > max_dd {
                            max_dd = drawdown;
                        }
                    }
                }
                max_dd
            }
        };

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
        let threshold = self.config.get_float("signal_threshold").unwrap_or(0.01);
        if threshold <= 0.0 || threshold > 0.5 {
            return Err(NyxsOwlError::InvalidParameter(
                "signal_threshold must be between 0.0 and 0.5".to_string(),
            ));
        }

        let min_volume_ratio = self.config.get_float("min_volume_ratio").unwrap_or(1.5);
        if min_volume_ratio < 1.0 {
            return Err(NyxsOwlError::InvalidParameter(
                "min_volume_ratio must be >= 1.0".to_string(),
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use polars::prelude::*;

    fn create_test_data() -> DataFrame {
        df! {
            "close" => [100.0, 102.0, 101.0, 103.0, 105.0],
            "volume" => [1000.0, 1500.0, 800.0, 2000.0, 1200.0],
            "high" => [101.0, 103.0, 102.0, 104.0, 106.0],
            "low" => [99.0, 101.0, 100.0, 102.0, 104.0],
        }
        .unwrap()
    }

    #[test]
    fn test_vwap_strategy_creation() {
        let config = StrategyConfig::new()
            .with_parameter("signal_threshold", 0.02)
            .with_parameter("min_volume_ratio", 1.5);

        let strategy = VWAPStrategy::new(config);
        assert_eq!(strategy.name(), "VWAP Strategy");
        assert_eq!(
            strategy.required_columns(),
            vec!["close", "volume", "high", "low"]
        );
    }

    #[test]
    fn test_vwap_signal_generation() {
        let config = StrategyConfig::new()
            .with_parameter("signal_threshold", 0.02)
            .with_parameter("min_volume_ratio", 1.2)
            .with_parameter("min_data_points", 3);

        let strategy = VWAPStrategy::new(config);
        let data = create_test_data();

        let signals = strategy.generate_enhanced_signals(&data).unwrap();
        assert_eq!(signals.len(), 5);

        // Check that signals have metadata
        for signal in &signals {
            assert!(signal.metadata.contains_key("vwap"));
            assert!(signal.metadata.contains_key("volume_ratio"));
        }
    }

    #[test]
    fn test_vwap_parameter_validation() {
        let config = StrategyConfig::new().with_parameter("signal_threshold", -0.01); // Invalid

        let strategy = VWAPStrategy::new(config);
        assert!(strategy.validate_parameters().is_err());

        let config = StrategyConfig::new().with_parameter("min_volume_ratio", 0.5); // Invalid

        let strategy = VWAPStrategy::new(config);
        assert!(strategy.validate_parameters().is_err());
    }
}
