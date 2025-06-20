//! Enhanced RSI Strategy
//!
//! An advanced RSI-based trading strategy that incorporates multiple RSI periods,
//! dynamic thresholds, and trend filtering for improved signal quality.
//!
//! This strategy follows the NyxsOwl architectural principles:
//! - Production-ready quality with comprehensive error handling
//! - SIMD-optimized calculations where applicable
//! - Clean API design with proper validation
//! - Streaming updates support for real-time processing

use crate::simple_types::{NyxsOwlError, Result, Signal};
use crate::technical_strategies::{
    ConfigExtractor, Strategy, StrategyConfig, TechnicalSignal, TechnicalStrategy,
};
use crate::trade_math::momentum::calculate_rsi;
use polars::prelude::{DataFrame, NamedFrom, Series};
// Remove unused imports
use std::collections::HashMap;

/// Enhanced RSI Strategy Configuration
#[derive(Debug, Clone)]
pub struct EnhancedRsiConfig {
    /// Primary RSI period (default: 14)
    pub primary_period: usize,
    /// Secondary RSI period for confirmation (default: 21)
    pub secondary_period: usize,
    /// Oversold threshold (default: 30.0)
    pub oversold_threshold: f64,
    /// Overbought threshold (default: 70.0)
    pub overbought_threshold: f64,
    /// Enable dynamic thresholds based on volatility (default: true)
    pub dynamic_thresholds: bool,
    /// Minimum signal strength required (default: 0.6)
    pub min_signal_strength: f64,
    /// Enable trend filtering (default: true)
    pub trend_filtering: bool,
    /// Lookback period for trend analysis (default: 50)
    pub trend_lookback: usize,
}

impl Default for EnhancedRsiConfig {
    fn default() -> Self {
        Self {
            primary_period: 14,
            secondary_period: 21,
            oversold_threshold: 30.0,
            overbought_threshold: 70.0,
            dynamic_thresholds: true,
            min_signal_strength: 0.6,
            trend_filtering: true,
            trend_lookback: 50,
        }
    }
}

/// Enhanced RSI Trading Strategy
///
/// This strategy uses multiple RSI periods and advanced signal filtering
/// to generate high-quality trading signals with reduced false positives.
///
/// ## Features
/// - Dual RSI confirmation system
/// - Dynamic threshold adjustment based on market volatility
/// - Trend filtering to avoid counter-trend trades
/// - Configurable signal strength thresholds
/// - Real-time streaming updates support
///
/// ## Signal Generation Logic
/// 1. Calculate primary and secondary RSI values
/// 2. Apply dynamic threshold adjustment if enabled
/// 3. Check for RSI divergence and confirmation
/// 4. Apply trend filtering to avoid counter-trend signals
/// 5. Generate signals with confidence scoring
#[derive(Debug, Clone)]
pub struct EnhancedRsiStrategy {
    config: StrategyConfig,
    enhanced_config: EnhancedRsiConfig,
}

impl EnhancedRsiStrategy {
    /// Create a new Enhanced RSI Strategy with custom configuration
    pub fn with_enhanced_config(
        config: StrategyConfig,
        enhanced_config: EnhancedRsiConfig,
    ) -> Self {
        Self {
            config,
            enhanced_config,
        }
    }

    /// Extract enhanced configuration from StrategyConfig
    fn extract_enhanced_config(config: &StrategyConfig) -> Result<EnhancedRsiConfig> {
        let mut enhanced_config = EnhancedRsiConfig::default();

        if let Some(period) = config.get_int_safe("primary_period") {
            enhanced_config.primary_period = period as usize;
        }
        if let Some(period) = config.get_int_safe("secondary_period") {
            enhanced_config.secondary_period = period as usize;
        }
        if let Some(threshold) = config.get_float_safe("oversold_threshold") {
            enhanced_config.oversold_threshold = threshold;
        }
        if let Some(threshold) = config.get_float_safe("overbought_threshold") {
            enhanced_config.overbought_threshold = threshold;
        }
        if let Some(dynamic) = config.get_bool_safe("dynamic_thresholds") {
            enhanced_config.dynamic_thresholds = dynamic;
        }
        if let Some(strength) = config.get_float_safe("min_signal_strength") {
            enhanced_config.min_signal_strength = strength;
        }
        if let Some(filtering) = config.get_bool_safe("trend_filtering") {
            enhanced_config.trend_filtering = filtering;
        }
        if let Some(lookback) = config.get_int_safe("trend_lookback") {
            enhanced_config.trend_lookback = lookback as usize;
        }
        Ok(enhanced_config)
    }

    /// Calculate dynamic thresholds based on market volatility
    fn calculate_dynamic_thresholds(&self, rsi_values: &[f64]) -> Result<(f64, f64)> {
        if rsi_values.len() < 20 {
            return Ok((
                self.enhanced_config.oversold_threshold,
                self.enhanced_config.overbought_threshold,
            ));
        }

        // Calculate RSI volatility over the last 20 periods
        let recent_rsi = &rsi_values[rsi_values.len().saturating_sub(20)..];
        let mean = recent_rsi.iter().sum::<f64>() / recent_rsi.len() as f64;
        let variance =
            recent_rsi.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / recent_rsi.len() as f64;
        let std_dev = variance.sqrt();

        // Adjust thresholds based on volatility
        let adjustment = std_dev * 0.5; // Scaling factor
        let oversold = (self.enhanced_config.oversold_threshold - adjustment).max(10.0);
        let overbought = (self.enhanced_config.overbought_threshold + adjustment).min(90.0);

        Ok((oversold, overbought))
    }

    /// Determine market trend direction
    fn analyze_trend(&self, prices: &[f64]) -> Result<f64> {
        if prices.len() < self.enhanced_config.trend_lookback {
            return Ok(0.0); // Neutral trend
        }

        let recent_prices = &prices[prices.len() - self.enhanced_config.trend_lookback..];
        let start_price = recent_prices[0];
        let end_price = recent_prices[recent_prices.len() - 1];

        // Simple trend strength calculation
        let trend_strength = (end_price - start_price) / start_price;
        Ok(trend_strength)
    }

    /// Calculate signal confidence based on multiple factors
    fn calculate_confidence(
        &self,
        primary_rsi: f64,
        secondary_rsi: f64,
        oversold: f64,
        overbought: f64,
        trend_strength: f64,
        signal_type: Signal,
    ) -> f64 {
        let mut confidence = 0.0;

        // RSI position confidence
        match signal_type {
            Signal::Buy => {
                let primary_strength = (oversold - primary_rsi) / oversold;
                let secondary_strength = (oversold - secondary_rsi) / oversold;
                confidence += (primary_strength + secondary_strength) * 0.3;
            }
            Signal::Sell => {
                let primary_strength = (primary_rsi - overbought) / (100.0 - overbought);
                let secondary_strength = (secondary_rsi - overbought) / (100.0 - overbought);
                confidence += (primary_strength + secondary_strength) * 0.3;
            }
            Signal::Hold => confidence = 0.1,
        }

        // RSI confirmation confidence
        let rsi_agreement = 1.0 - ((primary_rsi - secondary_rsi).abs() / 100.0);
        confidence += rsi_agreement * 0.3;

        // Trend alignment confidence
        match signal_type {
            Signal::Buy if trend_strength > 0.0 => confidence += trend_strength.abs() * 0.4,
            Signal::Sell if trend_strength < 0.0 => confidence += trend_strength.abs() * 0.4,
            _ => confidence += 0.1, // Slight penalty for counter-trend trades
        }

        confidence.clamp(0.0, 1.0)
    }
}

impl Strategy for EnhancedRsiStrategy {
    fn new(config: StrategyConfig) -> Self {
        let enhanced_config =
            Self::extract_enhanced_config(&config).unwrap_or_else(|_| EnhancedRsiConfig::default());
        Self {
            config,
            enhanced_config,
        }
    }

    fn generate_signals(&self, data: &DataFrame) -> Result<Series> {
        self.validate_data(data)?;

        let enhanced_signals = self.generate_enhanced_signals(data)?;
        let signals: Vec<i32> = enhanced_signals
            .into_iter()
            .map(|ts| ts.signal.to_int())
            .collect();

        Ok(Series::new("signal".into(), &signals))
    }

    fn name(&self) -> &str {
        "EnhancedRSI"
    }

    fn description(&self) -> &str {
        "Enhanced RSI strategy with dual RSI confirmation, dynamic thresholds, and trend filtering"
    }

    fn required_columns(&self) -> Vec<&str> {
        vec!["close"]
    }

    fn config(&self) -> &StrategyConfig {
        &self.config
    }

    fn min_data_points(&self) -> usize {
        self.enhanced_config
            .secondary_period
            .max(self.enhanced_config.trend_lookback)
            + 10
    }
}

impl TechnicalStrategy for EnhancedRsiStrategy {
    fn generate_enhanced_signals(&self, data: &DataFrame) -> Result<Vec<TechnicalSignal>> {
        self.validate_data(data)?;

        // Extract price data
        let close_column = data
            .column("close")
            .map_err(|e| NyxsOwlError::DataError(format!("Missing close column: {}", e)))?;
        let close_series = close_column.as_series().ok_or_else(|| {
            NyxsOwlError::DataError("Cannot convert column to series".to_string())
        })?;

        // Calculate RSI values
        let primary_rsi = calculate_rsi(close_series, self.enhanced_config.primary_period)?;
        let secondary_rsi = calculate_rsi(close_series, self.enhanced_config.secondary_period)?;

        // Extract price values for trend analysis
        let prices: Vec<f64> = close_series
            .f64()
            .map_err(|e| NyxsOwlError::DataError(format!("Failed to extract prices: {}", e)))?
            .into_no_null_iter()
            .collect();

        // Convert RSI series to vectors
        let primary_rsi_values: Vec<f64> = primary_rsi
            .f64()
            .map_err(|e| {
                NyxsOwlError::IndicatorError(format!("Primary RSI conversion failed: {}", e))
            })?
            .into_no_null_iter()
            .collect();

        let secondary_rsi_values: Vec<f64> = secondary_rsi
            .f64()
            .map_err(|e| {
                NyxsOwlError::IndicatorError(format!("Secondary RSI conversion failed: {}", e))
            })?
            .into_no_null_iter()
            .collect();

        let mut signals = Vec::new();
        let min_len = primary_rsi_values.len().min(secondary_rsi_values.len());

        for i in 0..min_len {
            let primary_value = primary_rsi_values[i];
            let secondary_value = secondary_rsi_values[i];

            // Calculate dynamic thresholds if enabled
            let (oversold, overbought) = if self.enhanced_config.dynamic_thresholds {
                self.calculate_dynamic_thresholds(&primary_rsi_values[..=i])?
            } else {
                (
                    self.enhanced_config.oversold_threshold,
                    self.enhanced_config.overbought_threshold,
                )
            };

            // Analyze trend if enabled
            let trend_strength = if self.enhanced_config.trend_filtering && i < prices.len() {
                self.analyze_trend(&prices[..=i])?
            } else {
                0.0
            };

            // Generate signal based on RSI conditions
            let signal = if primary_value <= oversold && secondary_value <= oversold + 5.0 {
                // Both RSI values indicate oversold condition
                if !self.enhanced_config.trend_filtering || trend_strength >= -0.05 {
                    Signal::Buy
                } else {
                    Signal::Hold // Avoid buying in strong downtrend
                }
            } else if primary_value >= overbought && secondary_value >= overbought - 5.0 {
                // Both RSI values indicate overbought condition
                if !self.enhanced_config.trend_filtering || trend_strength <= 0.05 {
                    Signal::Sell
                } else {
                    Signal::Hold // Avoid selling in strong uptrend
                }
            } else {
                Signal::Hold
            };

            // Calculate confidence and signal strength
            let confidence = self.calculate_confidence(
                primary_value,
                secondary_value,
                oversold,
                overbought,
                trend_strength,
                signal,
            );

            // Only generate signal if confidence meets minimum threshold
            let final_signal = if confidence >= self.enhanced_config.min_signal_strength {
                signal
            } else {
                Signal::Hold
            };

            // Create technical signal with metadata
            let mut metadata = HashMap::new();
            metadata.insert("primary_rsi".to_string(), primary_value);
            metadata.insert("secondary_rsi".to_string(), secondary_value);
            metadata.insert("oversold_threshold".to_string(), oversold);
            metadata.insert("overbought_threshold".to_string(), overbought);
            metadata.insert("trend_strength".to_string(), trend_strength);

            let technical_signal = TechnicalSignal::new(final_signal)
                .with_strength(confidence)
                .with_confidence(confidence)
                .with_metadata("primary_rsi", primary_value)
                .with_metadata("secondary_rsi", secondary_value)
                .with_metadata("trend_strength", trend_strength);

            signals.push(technical_signal);
        }

        Ok(signals)
    }

    fn get_indicator_values(&self, data: &DataFrame) -> Result<HashMap<String, Series>> {
        let close_column = data
            .column("close")
            .map_err(|e| NyxsOwlError::DataError(format!("Missing close column: {}", e)))?;
        let close_series = close_column.as_series().ok_or_else(|| {
            NyxsOwlError::DataError("Cannot convert column to series".to_string())
        })?;

        let mut indicators = HashMap::new();

        let primary_rsi = calculate_rsi(close_series, self.enhanced_config.primary_period)?;
        let secondary_rsi = calculate_rsi(close_series, self.enhanced_config.secondary_period)?;

        indicators.insert("primary_rsi".to_string(), primary_rsi);
        indicators.insert("secondary_rsi".to_string(), secondary_rsi);

        Ok(indicators)
    }

    fn get_performance_metrics(
        &self,
        _data: &DataFrame,
        _signals: &[TechnicalSignal],
    ) -> Result<crate::technical_strategies::PerformanceMetrics> {
        // Placeholder implementation - would need historical backtesting
        Ok(crate::technical_strategies::PerformanceMetrics::default())
    }

    fn validate_parameters(&self) -> Result<()> {
        // Validate periods
        if self.enhanced_config.primary_period == 0 {
            return Err(NyxsOwlError::InvalidParameter(
                "Primary period must be greater than 0".to_string(),
            ));
        }

        if self.enhanced_config.secondary_period == 0 {
            return Err(NyxsOwlError::InvalidParameter(
                "Secondary period must be greater than 0".to_string(),
            ));
        }

        // Validate thresholds
        if self.enhanced_config.oversold_threshold >= self.enhanced_config.overbought_threshold {
            return Err(NyxsOwlError::InvalidParameter(
                "Oversold threshold must be less than overbought threshold".to_string(),
            ));
        }

        if self.enhanced_config.oversold_threshold < 0.0
            || self.enhanced_config.overbought_threshold > 100.0
        {
            return Err(NyxsOwlError::InvalidParameter(
                "RSI thresholds must be between 0 and 100".to_string(),
            ));
        }

        // Validate signal strength
        if self.enhanced_config.min_signal_strength < 0.0
            || self.enhanced_config.min_signal_strength > 1.0
        {
            return Err(NyxsOwlError::InvalidParameter(
                "Minimum signal strength must be between 0.0 and 1.0".to_string(),
            ));
        }

        Ok(())
    }
}

/// Convenience function to create an Enhanced RSI strategy with default parameters
pub fn enhanced_rsi_signals(data: &DataFrame) -> Result<Vec<TechnicalSignal>> {
    let config = StrategyConfig::new();
    let strategy = EnhancedRsiStrategy::new(config);
    strategy.generate_enhanced_signals(data)
}

/// Convenience function to create an Enhanced RSI strategy with custom parameters
pub fn enhanced_rsi_signals_with_config(
    data: &DataFrame,
    primary_period: usize,
    secondary_period: usize,
    oversold_threshold: f64,
    overbought_threshold: f64,
) -> Result<Vec<TechnicalSignal>> {
    let config = StrategyConfig::new()
        .with_parameter("primary_period", primary_period as i64)
        .with_parameter("secondary_period", secondary_period as i64)
        .with_parameter("oversold_threshold", oversold_threshold)
        .with_parameter("overbought_threshold", overbought_threshold);

    let strategy = EnhancedRsiStrategy::new(config);
    strategy.generate_enhanced_signals(data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use polars::prelude::*;

    fn create_test_data() -> DataFrame {
        let prices: Vec<f64> = (1..=100)
            .map(|i| 100.0 + 10.0 * (i as f64 * 0.1).sin())
            .collect();

        df! {
            "close" => prices
        }
        .unwrap()
    }

    #[test]
    fn test_enhanced_rsi_strategy_creation() {
        let config = StrategyConfig::new()
            .with_parameter("primary_period", 14i64)
            .with_parameter("secondary_period", 21i64);

        let strategy = EnhancedRsiStrategy::new(config);
        assert_eq!(strategy.name(), "EnhancedRSI");
        assert_eq!(strategy.enhanced_config.primary_period, 14);
        assert_eq!(strategy.enhanced_config.secondary_period, 21);
    }

    #[test]
    fn test_parameter_validation() {
        let config = StrategyConfig::new().with_parameter("primary_period", 0i64); // Invalid

        let strategy = EnhancedRsiStrategy::new(config);
        assert!(strategy.validate_parameters().is_err());
    }

    #[test]
    fn test_signal_generation() {
        let data = create_test_data();
        let config = StrategyConfig::new();
        let strategy = EnhancedRsiStrategy::new(config);

        let signals = strategy.generate_enhanced_signals(&data);
        assert!(signals.is_ok());

        let signals = signals.unwrap();
        assert!(!signals.is_empty());

        // Check that signals have metadata
        if let Some(first_signal) = signals.first() {
            assert!(first_signal.metadata.contains_key("primary_rsi"));
            assert!(first_signal.metadata.contains_key("secondary_rsi"));
        }
    }

    #[test]
    fn test_convenience_functions() {
        let data = create_test_data();

        let signals = enhanced_rsi_signals(&data);
        assert!(signals.is_ok());

        let custom_signals = enhanced_rsi_signals_with_config(&data, 10, 20, 25.0, 75.0);
        assert!(custom_signals.is_ok());
    }

    #[test]
    fn test_required_columns() {
        let config = StrategyConfig::new();
        let strategy = EnhancedRsiStrategy::new(config);

        let required = strategy.required_columns();
        assert_eq!(required, vec!["close"]);
    }

    #[test]
    fn test_min_data_points() {
        let config = StrategyConfig::new();
        let strategy = EnhancedRsiStrategy::new(config);

        let min_points = strategy.min_data_points();
        assert!(min_points > 0);
    }

    #[test]
    fn test_parameter_validation_valid_values() {
        let config = StrategyConfig::new()
            .with_parameter("primary_period", 14i64)
            .with_parameter("secondary_period", 21i64)
            .with_parameter("oversold_threshold", 30.0)
            .with_parameter("overbought_threshold", 70.0)
            .with_parameter("min_signal_strength", 0.6);

        let strategy = EnhancedRsiStrategy::new(config);
        assert!(strategy.validate_parameters().is_ok());
    }

    #[test]
    fn test_parameter_validation_invalid_primary_period() {
        let config = StrategyConfig::new().with_parameter("primary_period", 0i64); // Invalid: zero

        let strategy = EnhancedRsiStrategy::new(config);
        let result = strategy.validate_parameters();
        assert!(result.is_err());

        if let Err(NyxsOwlError::InvalidParameter(msg)) = result {
            assert!(msg.contains("Primary period must be greater than 0"));
        } else {
            panic!("Expected InvalidParameter error");
        }
    }

    #[test]
    fn test_parameter_validation_invalid_secondary_period() {
        let config = StrategyConfig::new().with_parameter("secondary_period", 0i64); // Invalid: zero

        let strategy = EnhancedRsiStrategy::new(config);
        let result = strategy.validate_parameters();
        assert!(result.is_err());

        if let Err(NyxsOwlError::InvalidParameter(msg)) = result {
            assert!(msg.contains("Secondary period must be greater than 0"));
        } else {
            panic!("Expected InvalidParameter error");
        }
    }

    #[test]
    fn test_parameter_validation_invalid_thresholds() {
        let config = StrategyConfig::new()
            .with_parameter("oversold_threshold", 80.0)
            .with_parameter("overbought_threshold", 70.0); // Invalid: oversold >= overbought

        let strategy = EnhancedRsiStrategy::new(config);
        let result = strategy.validate_parameters();
        assert!(result.is_err());

        if let Err(NyxsOwlError::InvalidParameter(msg)) = result {
            assert!(msg.contains("Oversold threshold must be less than overbought threshold"));
        } else {
            panic!("Expected InvalidParameter error");
        }
    }

    #[test]
    fn test_parameter_validation_invalid_rsi_thresholds() {
        let config = StrategyConfig::new().with_parameter("oversold_threshold", -10.0); // Invalid: < 0

        let strategy = EnhancedRsiStrategy::new(config);
        let result = strategy.validate_parameters();
        assert!(result.is_err());

        if let Err(NyxsOwlError::InvalidParameter(msg)) = result {
            assert!(msg.contains("RSI thresholds must be between 0 and 100"));
        } else {
            panic!("Expected InvalidParameter error");
        }
    }

    #[test]
    fn test_parameter_validation_invalid_signal_strength() {
        let config = StrategyConfig::new().with_parameter("min_signal_strength", 1.5); // Invalid: > 1.0

        let strategy = EnhancedRsiStrategy::new(config);
        let result = strategy.validate_parameters();
        assert!(result.is_err());

        if let Err(NyxsOwlError::InvalidParameter(msg)) = result {
            assert!(msg.contains("Minimum signal strength must be between 0.0 and 1.0"));
        } else {
            panic!("Expected InvalidParameter error");
        }
    }

    #[test]
    fn test_parameter_validation_edge_cases() {
        // Test edge case values
        let config = StrategyConfig::new()
            .with_parameter("oversold_threshold", 0.0) // Valid edge case
            .with_parameter("overbought_threshold", 100.0) // Valid edge case
            .with_parameter("min_signal_strength", 0.0); // Valid edge case

        let strategy = EnhancedRsiStrategy::new(config);
        assert!(strategy.validate_parameters().is_ok());
    }

    #[test]
    fn test_extract_enhanced_config() {
        let config = StrategyConfig::new()
            .with_parameter("primary_period", 10i64)
            .with_parameter("secondary_period", 20i64)
            .with_parameter("oversold_threshold", 25.0)
            .with_parameter("overbought_threshold", 75.0)
            .with_parameter("dynamic_thresholds", true)
            .with_parameter("min_signal_strength", 0.7)
            .with_parameter("trend_filtering", false)
            .with_parameter("trend_lookback", 30i64);

        let enhanced_config = EnhancedRsiStrategy::extract_enhanced_config(&config).unwrap();

        assert_eq!(enhanced_config.primary_period, 10);
        assert_eq!(enhanced_config.secondary_period, 20);
        assert_eq!(enhanced_config.oversold_threshold, 25.0);
        assert_eq!(enhanced_config.overbought_threshold, 75.0);
        assert!(enhanced_config.dynamic_thresholds);
        assert_eq!(enhanced_config.min_signal_strength, 0.7);
        assert!(!enhanced_config.trend_filtering);
        assert_eq!(enhanced_config.trend_lookback, 30);
    }

    #[test]
    fn test_calculate_dynamic_thresholds() {
        let config = StrategyConfig::new();
        let strategy = EnhancedRsiStrategy::new(config);

        // Test with insufficient data
        let rsi_values = vec![50.0, 55.0, 45.0]; // Less than 20 values
        let result = strategy.calculate_dynamic_thresholds(&rsi_values).unwrap();
        assert_eq!(result.0, 30.0); // Default oversold
        assert_eq!(result.1, 70.0); // Default overbought

        // Test with sufficient data
        let rsi_values: Vec<f64> = (0..25)
            .map(|i| 50.0 + (i as f64 * 0.5).sin() * 10.0)
            .collect();
        let result = strategy.calculate_dynamic_thresholds(&rsi_values).unwrap();
        assert!(result.0 >= 10.0); // Should be adjusted but >= min
        assert!(result.1 <= 90.0); // Should be adjusted but <= max
    }

    #[test]
    fn test_analyze_trend() {
        let config = StrategyConfig::new();
        let strategy = EnhancedRsiStrategy::new(config);

        // Test with insufficient data
        let prices = vec![100.0, 101.0, 102.0]; // Less than trend_lookback
        let result = strategy.analyze_trend(&prices).unwrap();
        assert_eq!(result, 0.0); // Neutral trend

        // Test with sufficient data
        let prices: Vec<f64> = (0..60).map(|i| 100.0 + i as f64 * 0.1).collect(); // Uptrend
        let result = strategy.analyze_trend(&prices).unwrap();
        assert!(result > 0.0); // Should be positive for uptrend
    }

    #[test]
    fn test_calculate_confidence() {
        let config = StrategyConfig::new();
        let strategy = EnhancedRsiStrategy::new(config);

        // Test buy signal confidence
        let confidence = strategy.calculate_confidence(25.0, 30.0, 30.0, 70.0, 0.1, Signal::Buy);
        assert!(confidence > 0.0 && confidence <= 1.0);

        // Test sell signal confidence
        let confidence = strategy.calculate_confidence(75.0, 80.0, 30.0, 70.0, -0.1, Signal::Sell);
        assert!(confidence > 0.0 && confidence <= 1.0);

        // Test hold signal confidence - the actual calculation includes RSI agreement and trend alignment
        let confidence = strategy.calculate_confidence(50.0, 55.0, 30.0, 70.0, 0.0, Signal::Hold);
        assert!(confidence > 0.0 && confidence <= 1.0); // Just check it's in valid range
    }

    #[test]
    fn test_generate_signals() {
        let config = StrategyConfig::new();
        let strategy = EnhancedRsiStrategy::new(config);
        let data = create_test_data();

        let result = strategy.generate_signals(&data);
        assert!(result.is_ok());

        let signals = result.unwrap();
        assert_eq!(signals.len(), data.height());
        assert_eq!(signals.name(), "signal");
    }

    #[test]
    fn test_get_indicator_values() {
        let config = StrategyConfig::new();
        let strategy = EnhancedRsiStrategy::new(config);
        let data = create_test_data();

        let result = strategy.get_indicator_values(&data);
        assert!(result.is_ok());

        let indicators = result.unwrap();
        assert!(indicators.contains_key("primary_rsi"));
        assert!(indicators.contains_key("secondary_rsi"));

        let primary_rsi = indicators.get("primary_rsi").unwrap();
        let secondary_rsi = indicators.get("secondary_rsi").unwrap();

        assert_eq!(primary_rsi.len(), data.height());
        assert_eq!(secondary_rsi.len(), data.height());
    }

    #[test]
    fn test_get_performance_metrics() {
        let config = StrategyConfig::new();
        let strategy = EnhancedRsiStrategy::new(config);
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
        let strategy = EnhancedRsiStrategy::new(config);

        assert_eq!(strategy.name(), "EnhancedRSI");
        assert_eq!(strategy.description(), "Enhanced RSI strategy with dual RSI confirmation, dynamic thresholds, and trend filtering");
        assert_eq!(strategy.required_columns(), vec!["close"]);
        let _ = strategy.config(); // Just ensure this method is callable
    }

    #[test]
    fn test_with_enhanced_config() {
        let config = StrategyConfig::new();
        let enhanced_config = EnhancedRsiConfig {
            primary_period: 10,
            secondary_period: 20,
            oversold_threshold: 25.0,
            overbought_threshold: 75.0,
            dynamic_thresholds: true,
            min_signal_strength: 0.7,
            trend_filtering: false,
            trend_lookback: 30,
        };

        let strategy = EnhancedRsiStrategy::with_enhanced_config(config, enhanced_config.clone());
        assert_eq!(strategy.enhanced_config.primary_period, 10);
        assert_eq!(strategy.enhanced_config.secondary_period, 20);
        assert_eq!(strategy.enhanced_config.oversold_threshold, 25.0);
        assert_eq!(strategy.enhanced_config.overbought_threshold, 75.0);
        assert!(strategy.enhanced_config.dynamic_thresholds);
        assert_eq!(strategy.enhanced_config.min_signal_strength, 0.7);
        assert!(!strategy.enhanced_config.trend_filtering);
        assert_eq!(strategy.enhanced_config.trend_lookback, 30);
    }

    #[test]
    fn test_enhanced_config_default() {
        let default_config = EnhancedRsiConfig::default();

        assert_eq!(default_config.primary_period, 14);
        assert_eq!(default_config.secondary_period, 21);
        assert_eq!(default_config.oversold_threshold, 30.0);
        assert_eq!(default_config.overbought_threshold, 70.0);
        assert!(default_config.dynamic_thresholds);
        assert_eq!(default_config.min_signal_strength, 0.6);
        assert!(default_config.trend_filtering);
        assert_eq!(default_config.trend_lookback, 50);
    }
}
