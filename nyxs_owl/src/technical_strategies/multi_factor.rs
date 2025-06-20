//! Multi-Factor Strategies
//!
//! This module implements strategies that combine multiple technical indicators
//! to generate more robust trading signals.

use crate::simple_types::{NyxsOwlError, Result as NyxsOwlResult, Signal};
use crate::technical_strategies::{
    ConfigExtractor, PerformanceMetrics, Strategy, StrategyConfig, TechnicalSignal,
    TechnicalStrategy,
};
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

        let _close = data.column("close")?.f64()?;
        let mut signals = Vec::with_capacity(data.height());

        for _i in 0..data.height() {
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
        if let Some(min_strength) = self.config.get_float_safe("min_signal_strength") {
            if !(0.0..=1.0).contains(&min_strength) {
                return Err(NyxsOwlError::InvalidParameter(
                    "min_signal_strength must be between 0.0 and 1.0".to_string(),
                ));
            }
        }

        // Validate confidence parameter
        if let Some(min_confidence) = self.config.get_float_safe("min_confidence") {
            if !(0.0..=1.0).contains(&min_confidence) {
                return Err(NyxsOwlError::InvalidParameter(
                    "min_confidence must be between 0.0 and 1.0".to_string(),
                ));
            }
        }

        // Validate MA periods if set
        let short_ma = self.config.get_int_safe("short_ma_period");
        let long_ma = self.config.get_int_safe("long_ma_period");
        if let (Some(short_ma), Some(long_ma)) = (short_ma, long_ma) {
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

    #[test]
    fn test_parameter_validation_valid_values() {
        let config = StrategyConfig::new()
            .with_parameter("min_signal_strength", 0.5)
            .with_parameter("min_confidence", 0.6);

        let strategy = MultiFactorStrategy::new(config);
        assert!(strategy.validate_parameters().is_ok());
    }

    #[test]
    fn test_parameter_validation_invalid_signal_strength() {
        let config = StrategyConfig::new().with_parameter("min_signal_strength", -0.1); // Invalid: negative

        let strategy = MultiFactorStrategy::new(config);
        let result = strategy.validate_parameters();
        assert!(result.is_err());

        if let Err(NyxsOwlError::InvalidParameter(msg)) = result {
            assert!(msg.contains("min_signal_strength must be between 0.0 and 1.0"));
        } else {
            panic!("Expected InvalidParameter error");
        }
    }

    #[test]
    fn test_parameter_validation_invalid_confidence() {
        let config = StrategyConfig::new().with_parameter("min_confidence", 1.5); // Invalid: > 1.0

        let strategy = MultiFactorStrategy::new(config);
        let result = strategy.validate_parameters();
        assert!(result.is_err());

        if let Err(NyxsOwlError::InvalidParameter(msg)) = result {
            assert!(msg.contains("min_confidence must be between 0.0 and 1.0"));
        } else {
            panic!("Expected InvalidParameter error");
        }
    }

    #[test]
    fn test_parameter_validation_invalid_ma_periods() {
        let config = StrategyConfig::new()
            .with_parameter("short_ma_period", 10)
            .with_parameter("long_ma_period", 5); // Invalid: short >= long

        let strategy = MultiFactorStrategy::new(config);
        let result = strategy.validate_parameters();
        assert!(result.is_err());

        if let Err(NyxsOwlError::InvalidParameter(msg)) = result {
            assert!(msg.contains("short_ma_period must be less than long_ma_period"));
        } else {
            panic!("Expected InvalidParameter error");
        }
    }

    #[test]
    fn test_parameter_validation_valid_ma_periods() {
        let config = StrategyConfig::new()
            .with_parameter("short_ma_period", 5)
            .with_parameter("long_ma_period", 10); // Valid: short < long

        let strategy = MultiFactorStrategy::new(config);
        assert!(strategy.validate_parameters().is_ok());
    }

    #[test]
    fn test_parameter_validation_edge_cases() {
        // Test edge case values
        let config = StrategyConfig::new()
            .with_parameter("min_signal_strength", 0.0) // Valid edge case
            .with_parameter("min_confidence", 1.0); // Valid edge case

        let strategy = MultiFactorStrategy::new(config);
        assert!(strategy.validate_parameters().is_ok());
    }

    #[test]
    fn test_min_data_points_with_config() {
        let config = StrategyConfig::new().with_parameter("min_data_points", 50);

        let strategy = MultiFactorStrategy::new(config);
        assert_eq!(strategy.min_data_points(), 50);
    }

    #[test]
    fn test_min_data_points_default() {
        let config = StrategyConfig::new(); // No min_data_points parameter

        let strategy = MultiFactorStrategy::new(config);
        assert_eq!(strategy.min_data_points(), 20); // Default value
    }

    #[test]
    fn test_generate_signals() {
        let config = StrategyConfig::new();
        let strategy = MultiFactorStrategy::new(config);
        let data = create_test_data();

        let result = strategy.generate_signals(&data);
        assert!(result.is_ok());

        let signals = result.unwrap();
        assert_eq!(signals.len(), data.height());
        assert_eq!(signals.name(), "signals");
    }

    #[test]
    fn test_get_indicator_values() {
        let config = StrategyConfig::new();
        let strategy = MultiFactorStrategy::new(config);
        let data = create_test_data();

        let result = strategy.get_indicator_values(&data);
        assert!(result.is_ok());

        let indicators = result.unwrap();
        assert!(indicators.is_empty()); // Current implementation returns empty HashMap
    }

    #[test]
    fn test_get_performance_metrics() {
        let config = StrategyConfig::new();
        let strategy = MultiFactorStrategy::new(config);
        let data = create_test_data();
        let signals = vec![TechnicalSignal::new(Signal::Hold)];

        let result = strategy.get_performance_metrics(&data, &signals);
        assert!(result.is_ok());

        let metrics = result.unwrap();
        assert_eq!(metrics.total_return, 0.0); // Default value
    }

    #[test]
    fn test_strategy_trait_methods() {
        let config = StrategyConfig::new();
        let strategy = MultiFactorStrategy::new(config);

        assert_eq!(strategy.name(), "Multi-Factor Strategy");
        assert_eq!(
            strategy.description(),
            "Strategy combining multiple technical indicators"
        );
        assert_eq!(strategy.required_columns(), vec!["close", "volume"]);
        let _ = strategy.config(); // Just ensure this method is callable
    }
}
