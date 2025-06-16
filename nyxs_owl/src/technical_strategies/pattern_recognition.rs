//! Pattern Recognition Strategies
//!
//! This module implements trading strategies based on chart pattern recognition,
//! including candlestick patterns, geometric patterns, and trend patterns.

use crate::technical_strategies::{Strategy, StrategyConfig};
use crate::simple_types::{Result as NyxsOwlResult, Signal};
use crate::technical_strategies::{PerformanceMetrics, TechnicalSignal, TechnicalStrategy};
use polars::prelude::{DataFrame, NamedFrom, Series};
use std::collections::HashMap;

/// Candlestick Pattern Recognition Strategy
#[derive(Debug, Clone)]
pub struct CandlestickPatternStrategy {
    config: StrategyConfig,
}

impl Strategy for CandlestickPatternStrategy {
    fn new(config: StrategyConfig) -> Self {
        Self { config }
    }

    fn generate_signals(&self, data: &DataFrame) -> NyxsOwlResult<Series> {
        let enhanced_signals = self.generate_enhanced_signals(data)?;
        let signals: Vec<i32> = enhanced_signals.iter().map(|s| s.signal.to_int()).collect();
        Ok(Series::new("signals".into(), signals))
    }

    fn name(&self) -> &str {
        "Candlestick Pattern Strategy"
    }

    fn description(&self) -> &str {
        "Trading strategy based on candlestick pattern recognition"
    }

    fn required_columns(&self) -> Vec<&str> {
        vec!["open", "high", "low", "close"]
    }

    fn config(&self) -> &StrategyConfig {
        &self.config
    }

    fn min_data_points(&self) -> usize {
        self.config.get_int("min_data_points").unwrap_or(5) as usize
    }
}

impl TechnicalStrategy for CandlestickPatternStrategy {
    fn generate_enhanced_signals(&self, data: &DataFrame) -> NyxsOwlResult<Vec<TechnicalSignal>> {
        self.validate_data(data)?;
        self.validate_parameters()?;

        let open = data.column("open")?.f64()?;
        let high = data.column("high")?.f64()?;
        let low = data.column("low")?.f64()?;
        let close = data.column("close")?.f64()?;

        let mut signals = Vec::with_capacity(data.height());

        for i in 0..data.height() {
            let signal = if i < 2 {
                // Need at least 3 candles for pattern recognition
                TechnicalSignal::new(Signal::Hold)
            } else {
                self.detect_pattern(open, high, low, close, i)?
            };

            signals.push(signal);
        }

        Ok(signals)
    }

    fn get_indicator_values(&self, _data: &DataFrame) -> NyxsOwlResult<HashMap<String, Series>> {
        // Pattern recognition doesn't produce traditional indicators
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
        // Basic validation - pattern strategies have minimal parameters
        Ok(())
    }
}

impl CandlestickPatternStrategy {
    fn detect_pattern(
        &self,
        open: &polars::chunked_array::ChunkedArray<polars::datatypes::Float64Type>,
        high: &polars::chunked_array::ChunkedArray<polars::datatypes::Float64Type>,
        low: &polars::chunked_array::ChunkedArray<polars::datatypes::Float64Type>,
        close: &polars::chunked_array::ChunkedArray<polars::datatypes::Float64Type>,
        index: usize,
    ) -> NyxsOwlResult<TechnicalSignal> {
        // Get current and previous candles
        let curr_open = open.get(index).unwrap_or(0.0);
        let curr_high = high.get(index).unwrap_or(0.0);
        let curr_low = low.get(index).unwrap_or(0.0);
        let curr_close = close.get(index).unwrap_or(0.0);

        let prev_open = open.get(index - 1).unwrap_or(0.0);
        let prev_high = high.get(index - 1).unwrap_or(0.0);
        let prev_low = low.get(index - 1).unwrap_or(0.0);
        let prev_close = close.get(index - 1).unwrap_or(0.0);

        // Detect bullish engulfing pattern
        if self.is_bullish_engulfing(prev_open, prev_close, curr_open, curr_close) {
            return Ok(TechnicalSignal::new(Signal::Buy)
                .with_strength(0.8)
                .with_confidence(0.7)
                .with_metadata("pattern", 1.0)); // 1 = bullish engulfing
        }

        // Detect bearish engulfing pattern
        if self.is_bearish_engulfing(prev_open, prev_close, curr_open, curr_close) {
            return Ok(TechnicalSignal::new(Signal::Sell)
                .with_strength(0.8)
                .with_confidence(0.7)
                .with_metadata("pattern", 2.0)); // 2 = bearish engulfing
        }

        // Detect hammer pattern
        if self.is_hammer(curr_open, curr_high, curr_low, curr_close) {
            return Ok(TechnicalSignal::new(Signal::Buy)
                .with_strength(0.6)
                .with_confidence(0.6)
                .with_metadata("pattern", 3.0)); // 3 = hammer
        }

        // Detect shooting star pattern
        if self.is_shooting_star(curr_open, curr_high, curr_low, curr_close) {
            return Ok(TechnicalSignal::new(Signal::Sell)
                .with_strength(0.6)
                .with_confidence(0.6)
                .with_metadata("pattern", 4.0)); // 4 = shooting star
        }

        Ok(TechnicalSignal::new(Signal::Hold))
    }

    fn is_bullish_engulfing(
        &self,
        prev_open: f64,
        prev_close: f64,
        curr_open: f64,
        curr_close: f64,
    ) -> bool {
        // Previous candle is bearish (red)
        let prev_bearish = prev_close < prev_open;
        // Current candle is bullish (green) and engulfs previous
        let curr_bullish = curr_close > curr_open;
        let engulfs = curr_open < prev_close && curr_close > prev_open;

        prev_bearish && curr_bullish && engulfs
    }

    fn is_bearish_engulfing(
        &self,
        prev_open: f64,
        prev_close: f64,
        curr_open: f64,
        curr_close: f64,
    ) -> bool {
        // Previous candle is bullish (green)
        let prev_bullish = prev_close > prev_open;
        // Current candle is bearish (red) and engulfs previous
        let curr_bearish = curr_close < curr_open;
        let engulfs = curr_open > prev_close && curr_close < prev_open;

        prev_bullish && curr_bearish && engulfs
    }

    fn is_hammer(&self, open: f64, high: f64, low: f64, close: f64) -> bool {
        let body_size = (close - open).abs();
        let total_range = high - low;
        let lower_shadow = open.min(close) - low;
        let upper_shadow = high - open.max(close);

        // Hammer characteristics: small body, long lower shadow, small upper shadow
        if total_range == 0.0 {
            return false;
        }

        let body_ratio = body_size / total_range;
        let lower_ratio = lower_shadow / total_range;
        let upper_ratio = upper_shadow / total_range;

        body_ratio < 0.3 && lower_ratio > 0.6 && upper_ratio < 0.1
    }

    fn is_shooting_star(&self, open: f64, high: f64, low: f64, close: f64) -> bool {
        let body_size = (close - open).abs();
        let total_range = high - low;
        let lower_shadow = open.min(close) - low;
        let upper_shadow = high - open.max(close);

        // Shooting star characteristics: small body, long upper shadow, small lower shadow
        if total_range == 0.0 {
            return false;
        }

        let body_ratio = body_size / total_range;
        let lower_ratio = lower_shadow / total_range;
        let upper_ratio = upper_shadow / total_range;

        body_ratio < 0.3 && upper_ratio > 0.6 && lower_ratio < 0.1
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use polars::prelude::*;

    fn create_test_data() -> DataFrame {
        df! {
            "open" => [100.0, 102.0, 98.0, 101.0, 103.0],
            "high" => [101.0, 103.0, 102.0, 104.0, 106.0],
            "low" => [99.0, 101.0, 97.0, 100.0, 102.0],
            "close" => [100.5, 101.5, 101.5, 103.5, 105.0],
        }
        .unwrap()
    }

    #[test]
    fn test_pattern_strategy_creation() {
        let config = StrategyConfig::new();
        let strategy = CandlestickPatternStrategy::new(config);

        assert_eq!(strategy.name(), "Candlestick Pattern Strategy");
        assert_eq!(
            strategy.required_columns(),
            vec!["open", "high", "low", "close"]
        );
    }

    #[test]
    fn test_pattern_detection() {
        let config = StrategyConfig::new();
        let strategy = CandlestickPatternStrategy::new(config);
        let data = create_test_data();

        let signals = strategy.generate_enhanced_signals(&data).unwrap();
        assert_eq!(signals.len(), 5);

        // First two signals should be Hold due to insufficient history
        assert_eq!(signals[0].signal, Signal::Hold);
        assert_eq!(signals[1].signal, Signal::Hold);
    }

    #[test]
    fn test_bullish_engulfing() {
        let config = StrategyConfig::new();
        let strategy = CandlestickPatternStrategy::new(config);

        // Previous: bearish candle (open > close)
        // Current: bullish candle that engulfs previous
        assert!(strategy.is_bullish_engulfing(102.0, 100.0, 99.0, 103.0));
        assert!(!strategy.is_bullish_engulfing(100.0, 102.0, 99.0, 103.0)); // Previous not bearish
    }

    #[test]
    fn test_hammer_pattern() {
        let config = StrategyConfig::new();
        let strategy = CandlestickPatternStrategy::new(config);

        // Hammer: small body, long lower shadow
        assert!(strategy.is_hammer(100.0, 101.0, 95.0, 100.5)); // Small body, long lower shadow
        assert!(!strategy.is_hammer(100.0, 105.0, 99.0, 104.0)); // Large body
    }
}
