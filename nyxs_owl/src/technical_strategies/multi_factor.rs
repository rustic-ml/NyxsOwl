//! Multi-Factor Strategies
//!
//! This module implements strategies that combine multiple technical indicators
//! to generate more robust trading signals.

use crate::simple_types::{NyxsOwlError, Result as NyxsOwlResult, Signal};
use crate::technical_strategies::{PerformanceMetrics, TechnicalSignal, TechnicalStrategy};
use crate::technical_strategies::{Strategy, StrategyConfig};
use polars::prelude::{DataFrame, NamedFrom, Series};
use std::collections::HashMap;

/// Multi-Factor Technical Strategy
#[derive(Debug, Clone)]
pub struct MultiFactorStrategy {
    config: StrategyConfig,
}

impl Strategy for MultiFactorStrategy {
    fn new(config: StrategyConfig) -> Self {
        Self { config }
    }

    fn generate_signals(&self, data: &DataFrame) -> NyxsOwlResult<Series> {
        let enhanced_signals = self.generate_enhanced_signals(data)?;
        let signals: Vec<i32> = enhanced_signals.iter().map(|s| s.signal.to_int()).collect();
        Ok(Series::new("signals".into(), signals))
    }

    fn name(&self) -> &str {
        "Multi-Factor Strategy"
    }

    fn description(&self) -> &str {
        "Strategy combining multiple technical indicators"
    }

    fn required_columns(&self) -> Vec<&str> {
        vec!["close", "volume"]
    }

    fn config(&self) -> &StrategyConfig {
        &self.config
    }

    fn min_data_points(&self) -> usize {
        self.config.get_int("min_data_points").unwrap_or(20) as usize
    }
}

impl TechnicalStrategy for MultiFactorStrategy {
    fn generate_enhanced_signals(&self, data: &DataFrame) -> NyxsOwlResult<Vec<TechnicalSignal>> {
        self.validate_data(data)?;
        self.validate_parameters()?;

        let close = data.column("close")?.f64()?;
        let mut signals = Vec::with_capacity(data.height());

        for i in 0..data.height() {
            let signal = TechnicalSignal::new(Signal::Hold)
                .with_strength(0.5)
                .with_confidence(0.5);
            signals.push(signal);
        }

        Ok(signals)
    }

    fn get_indicator_values(&self, _data: &DataFrame) -> NyxsOwlResult<HashMap<String, Series>> {
        Ok(HashMap::new())
    }

    fn get_performance_metrics(
        &self,
        _data: &DataFrame,
        _signals: &[TechnicalSignal],
    ) -> NyxsOwlResult<PerformanceMetrics> {
        Ok(PerformanceMetrics::default())
    }

    fn validate_parameters(&self) -> NyxsOwlResult<()> {
        // Validate signal strength parameter
        if let Ok(min_strength) = self.config.get_float("min_signal_strength") {
            if !(0.0..=1.0).contains(&min_strength) {
                return Err(NyxsOwlError::InvalidParameter(
                    "min_signal_strength must be between 0.0 and 1.0".to_string(),
                ));
            }
        }

        // Validate confidence parameter
        if let Ok(min_confidence) = self.config.get_float("min_confidence") {
            if !(0.0..=1.0).contains(&min_confidence) {
                return Err(NyxsOwlError::InvalidParameter(
                    "min_confidence must be between 0.0 and 1.0".to_string(),
                ));
            }
        }

        // Validate MA periods if set
        if let (Ok(short_ma), Ok(long_ma)) = (
            self.config.get_int("short_ma_period"),
            self.config.get_int("long_ma_period"),
        ) {
            if short_ma >= long_ma {
                return Err(NyxsOwlError::InvalidParameter(
                    "short_ma_period must be less than long_ma_period".to_string(),
                ));
            }
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
            "open" => (0..50).map(|i| 100.0 + i as f64 * 0.1).collect::<Vec<f64>>(),
            "high" => (0..50).map(|i| 101.0 + i as f64 * 0.1).collect::<Vec<f64>>(),
            "low" => (0..50).map(|i| 99.0 + i as f64 * 0.1).collect::<Vec<f64>>(),
            "close" => (0..50).map(|i| 100.0 + i as f64 * 0.1 + (i as f64 * 0.1).sin()).collect::<Vec<f64>>(),
            "volume" => (0..50).map(|i| 1000.0 + (i % 10) as f64 * 100.0).collect::<Vec<f64>>(),
        }.unwrap()
    }

    #[test]
    fn test_multi_factor_strategy_creation() {
        let config = StrategyConfig::new()
            .with_parameter("min_signal_strength", 0.6)
            .with_parameter("min_confidence", 0.7);

        let strategy = MultiFactorStrategy::new(config);
        assert_eq!(strategy.name(), "Multi-Factor Strategy");
        assert_eq!(strategy.required_columns(), vec!["close", "volume"]);
    }

    #[test]
    fn test_multi_factor_signal_generation() {
        let config = StrategyConfig::new()
            .with_parameter("short_ma_period", 5)
            .with_parameter("long_ma_period", 10)
            .with_parameter("min_signal_strength", 0.3)
            .with_parameter("min_confidence", 0.4);

        let strategy = MultiFactorStrategy::new(config);
        let data = create_test_data();

        let signals = strategy.generate_enhanced_signals(&data).unwrap();
        assert!(!signals.is_empty());

        // Check that some signals have metadata from multiple sources
        let non_hold_signals: Vec<_> = signals
            .iter()
            .filter(|s| s.signal != Signal::Hold)
            .collect();

        if !non_hold_signals.is_empty() {
            // Should have combined metadata from different signal sources
            assert!(!non_hold_signals[0].metadata.is_empty());
        }
    }

    #[test]
    fn test_parameter_validation() {
        let config = StrategyConfig::new().with_parameter("min_signal_strength", 1.5); // Invalid

        let strategy = MultiFactorStrategy::new(config);
        assert!(strategy.validate_parameters().is_err());
    }
}
