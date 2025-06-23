//! Technical indicator integration for the Hybrid Strategy Framework
//!
//! This module provides integration between technical indicators and the hybrid
//! strategy framework, including signal generation, pattern detection, and
//! divergence analysis.
//!
//! ## Features
//!
//! - **Comprehensive Technical Indicators**: 125+ indicators including momentum,
//!   trend, volatility, and volume-based indicators
//! - **Feature Engineering**: Automatic feature matrix generation for forecasting models
//! - **Caching**: Efficient caching of calculated indicators for performance
//! - **Signal Generation**: Advanced signal generation with confirmation patterns
//! - **Pattern Recognition**: Built-in pattern detection for common chart patterns
//! - **Divergence Analysis**: Detection of price-indicator divergences

use crate::prelude::*;
use crate::hybrid::{config::*, types::*, error::*};
use polars::prelude::*;
use std::collections::HashMap;

mod indicators;
mod patterns;
mod divergences;

pub use indicators::*;
pub use patterns::*;
pub use divergences::*;

/// Technical engine for processing technical indicators
#[derive(Debug)]
pub struct TechnicalEngine {
    /// Technical indicator configurations
    configs: Vec<TechnicalIndicatorConfig>,
    /// Active indicator instances
    indicators: HashMap<String, Box<dyn TechnicalIndicator>>,
    /// Pattern detectors
    pattern_detectors: Vec<Box<dyn PatternDetector>>,
    /// Divergence detectors
    divergence_detectors: Vec<Box<dyn DivergenceDetector>>,
    /// Historical data for analysis
    historical_data: Vec<MarketData>,
    /// Technical indicators calculator
    technical_indicators: TechnicalIndicators,
}

impl TechnicalEngine {
    /// Create a new technical engine
    pub fn new(configs: &[TechnicalIndicatorConfig]) -> Result<Self, HybridError> {
        let mut indicators = HashMap::new();
        let mut pattern_detectors = Vec::new();
        let mut divergence_detectors = Vec::new();

        // Initialize indicators based on configurations
        for config in configs {
            let indicator = Self::create_indicator(config)?;
            let name = indicator.name().to_string();
            indicators.insert(name.clone(), indicator);

            // Add pattern detectors for relevant indicators
            if Self::supports_patterns(config) {
                pattern_detectors.push(Box::new(PatternDetectorImpl::new(name.clone())));
            }

            // Add divergence detectors for relevant indicators
            if Self::supports_divergences(config) {
                divergence_detectors.push(Box::new(DivergenceDetectorImpl::new(name.clone())));
            }
        }

        // Initialize technical indicators calculator
        let technical_config = TechnicalConfig::from_indicators(configs);
        let technical_indicators = TechnicalIndicators::new(technical_config);

        Ok(Self {
            configs: configs.to_vec(),
            indicators,
            pattern_detectors,
            divergence_detectors,
            historical_data: Vec::new(),
            technical_indicators,
        })
    }

    /// Generate technical signals from market data
    pub fn generate_signals(&mut self, market_data: &DataFrame) -> Result<Vec<TechnicalSignal>, HybridError> {
        let mut signals = Vec::new();

        // Convert DataFrame to MarketData
        let market_data_vec = self.convert_dataframe_to_market_data(market_data)?;
        
        // Update historical data
        self.historical_data.extend(market_data_vec.clone());

        // Generate signals for each indicator
        for (name, indicator) in &mut self.indicators {
            let indicator_signals = indicator.generate_signals(&market_data_vec)?;
            signals.extend(indicator_signals);
        }

        // Detect patterns
        let patterns = self.detect_patterns(&market_data_vec)?;
        
        // Detect divergences
        let divergences = self.detect_divergences(&market_data_vec)?;

        // Attach patterns and divergences to signals
        self.attach_patterns_and_divergences(&mut signals, &patterns, &divergences)?;

        Ok(signals)
    }

    /// Generate feature matrix for forecasting models
    pub fn generate_feature_matrix(&mut self, market_data: &DataFrame) -> Result<DataFrame, HybridError> {
        self.technical_indicators.generate_feature_matrix(market_data)
    }

    /// Calculate all technical indicators
    pub fn calculate_all_indicators(&mut self, market_data: &DataFrame) -> Result<TechnicalIndicatorsResult, HybridError> {
        self.technical_indicators.calculate_all_indicators(market_data)
    }

    /// Get cached indicator values
    pub fn get_cached_indicator(&self, key: &str) -> Option<&Series> {
        self.technical_indicators.get_cached(key)
    }

    /// Clear indicator cache
    pub fn clear_indicator_cache(&mut self) {
        self.technical_indicators.clear_cache();
    }

    /// Update the engine with new market data (streaming mode)
    pub fn update(&mut self, new_data: &MarketData) -> Result<(), HybridError> {
        // Update historical data
        self.historical_data.push(new_data.clone());

        // Update each indicator
        for indicator in self.indicators.values_mut() {
            indicator.update(new_data)?;
        }

        Ok(())
    }

    /// Check if the engine is ready to generate signals
    pub fn is_ready(&self) -> bool {
        // Check if we have enough historical data
        if self.historical_data.len() < 50 {
            return false;
        }

        // Check if all indicators are ready
        for indicator in self.indicators.values() {
            if !indicator.is_ready() {
                return false;
            }
        }

        true
    }

    /// Get current signals from all indicators
    pub fn get_current_signals(&self) -> Result<Vec<TechnicalSignal>, HybridError> {
        let mut signals = Vec::new();

        for (name, indicator) in &self.indicators {
            if indicator.is_ready() {
                let current_signal = indicator.get_current_signal()?;
                signals.push(current_signal);
            }
        }

        Ok(signals)
    }

    /// Reset the engine state
    pub fn reset(&mut self) -> Result<(), HybridError> {
        self.historical_data.clear();
        
        for indicator in self.indicators.values_mut() {
            indicator.reset()?;
        }

        Ok(())
    }

    /// Create a technical indicator from configuration
    fn create_indicator(config: &TechnicalIndicatorConfig) -> Result<Box<dyn TechnicalIndicator>, HybridError> {
        match config {
            TechnicalIndicatorConfig::RSI { period, oversold, overbought } => {
                Ok(Box::new(RSIIndicator::new(*period, *oversold, *overbought)))
            }
            TechnicalIndicatorConfig::MACD { fast_period, slow_period, signal_period } => {
                Ok(Box::new(MACDIndicator::new(*fast_period, *slow_period, *signal_period)))
            }
            TechnicalIndicatorConfig::BollingerBands { period, std_dev } => {
                Ok(Box::new(BollingerBandsIndicator::new(*period, *std_dev)))
            }
            TechnicalIndicatorConfig::CCI { period, threshold } => {
                Ok(Box::new(CCIIndicator::new(*period, *threshold)))
            }
            TechnicalIndicatorConfig::MFI { period, oversold, overbought } => {
                Ok(Box::new(MFIIndicator::new(*period, *oversold, *overbought)))
            }
            TechnicalIndicatorConfig::Custom { name, parameters } => {
                Ok(Box::new(CustomIndicator::new(name.clone(), parameters.clone())))
            }
        }
    }

    /// Check if an indicator supports pattern detection
    fn supports_patterns(config: &TechnicalIndicatorConfig) -> bool {
        matches!(
            config,
            TechnicalIndicatorConfig::RSI { .. } |
            TechnicalIndicatorConfig::MACD { .. } |
            TechnicalIndicatorConfig::BollingerBands { .. }
        )
    }

    /// Check if an indicator supports divergence detection
    fn supports_divergences(config: &TechnicalIndicatorConfig) -> bool {
        matches!(
            config,
            TechnicalIndicatorConfig::RSI { .. } |
            TechnicalIndicatorConfig::MACD { .. } |
            TechnicalIndicatorConfig::CCI { .. }
        )
    }

    /// Convert DataFrame to MarketData vector
    fn convert_dataframe_to_market_data(&self, df: &DataFrame) -> Result<Vec<MarketData>, HybridError> {
        let mut market_data = Vec::new();

        // Get column names
        let columns = df.get_column_names();
        
        // Find required columns
        let date_col = columns.iter().find(|&&col| col == "date" || col == "timestamp")
            .ok_or_else(|| HybridError::data("Date/timestamp column not found"))?;
        let open_col = columns.iter().find(|&&col| col == "open")
            .ok_or_else(|| HybridError::data("Open column not found"))?;
        let high_col = columns.iter().find(|&&col| col == "high")
            .ok_or_else(|| HybridError::data("High column not found"))?;
        let low_col = columns.iter().find(|&&col| col == "low")
            .ok_or_else(|| HybridError::data("Low column not found"))?;
        let close_col = columns.iter().find(|&&col| col == "close")
            .ok_or_else(|| HybridError::data("Close column not found"))?;
        let volume_col = columns.iter().find(|&&col| col == "volume")
            .ok_or_else(|| HybridError::data("Volume column not found"))?;

        // Convert each row
        for row in df.iter() {
            let timestamp = row[df.get_column_index(date_col).unwrap()].datetime()?;
            let open = row[df.get_column_index(open_col).unwrap()].f64()?;
            let high = row[df.get_column_index(high_col).unwrap()].f64()?;
            let low = row[df.get_column_index(low_col).unwrap()].f64()?;
            let close = row[df.get_column_index(close_col).unwrap()].f64()?;
            let volume = row[df.get_column_index(volume_col).unwrap()].f64()?;

            market_data.push(MarketData::new(timestamp, open, high, low, close, volume));
        }

        Ok(market_data)
    }

    /// Detect patterns in the market data
    fn detect_patterns(&self, market_data: &[MarketData]) -> Result<Vec<Pattern>, HybridError> {
        let mut patterns = Vec::new();

        for detector in &self.pattern_detectors {
            let detected_patterns = detector.detect_patterns(market_data)?;
            patterns.extend(detected_patterns);
        }

        Ok(patterns)
    }

    /// Detect divergences in the market data
    fn detect_divergences(&self, market_data: &[MarketData]) -> Result<Vec<Divergence>, HybridError> {
        let mut divergences = Vec::new();

        for detector in &self.divergence_detectors {
            let detected_divergences = detector.detect_divergences(market_data)?;
            divergences.extend(detected_divergences);
        }

        Ok(divergences)
    }

    /// Attach patterns and divergences to signals
    fn attach_patterns_and_divergences(
        &self,
        signals: &mut [TechnicalSignal],
        patterns: &[Pattern],
        divergences: &[Divergence],
    ) -> Result<(), HybridError> {
        for signal in signals {
            // Attach relevant patterns
            for pattern in patterns {
                if pattern.name.contains(&signal.indicator_name) {
                    signal.patterns.push(pattern.clone());
                }
            }

            // Attach relevant divergences
            for divergence in divergences {
                // This is a simplified matching - in practice, you'd want more sophisticated matching
                signal.divergences.push(divergence.clone());
            }
        }

        Ok(())
    }
}

/// Trait for technical indicators
pub trait TechnicalIndicator: Send + Sync {
    /// Get the indicator name
    fn name(&self) -> &str;

    /// Generate signals from market data
    fn generate_signals(&mut self, market_data: &[MarketData]) -> Result<Vec<TechnicalSignal>, HybridError>;

    /// Update the indicator with new data
    fn update(&mut self, new_data: &MarketData) -> Result<(), HybridError>;

    /// Check if the indicator is ready to generate signals
    fn is_ready(&self) -> bool;

    /// Get the current signal
    fn get_current_signal(&self) -> Result<TechnicalSignal, HybridError>;

    /// Reset the indicator state
    fn reset(&mut self) -> Result<(), HybridError>;
}

/// Trait for pattern detectors
pub trait PatternDetector: Send + Sync {
    /// Detect patterns in market data
    fn detect_patterns(&self, market_data: &[MarketData]) -> Result<Vec<Pattern>, HybridError>;
}

/// Trait for divergence detectors
pub trait DivergenceDetector: Send + Sync {
    /// Detect divergences in market data
    fn detect_divergences(&self, market_data: &[MarketData]) -> Result<Vec<Divergence>, HybridError>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_technical_engine_creation() {
        let configs = vec![
            TechnicalIndicatorConfig::RSI {
                period: 14,
                oversold: 30.0,
                overbought: 70.0,
            }
        ];
        
        let engine = TechnicalEngine::new(&configs);
        assert!(engine.is_ok());
    }

    #[test]
    fn test_technical_engine_ready_state() {
        let configs = vec![
            TechnicalIndicatorConfig::RSI {
                period: 14,
                oversold: 30.0,
                overbought: 70.0,
            }
        ];
        
        let mut engine = TechnicalEngine::new(&configs).unwrap();
        
        // Should not be ready initially
        assert!(!engine.is_ready());
        
        // Add some data
        for i in 0..100 {
            let data = MarketData::new(
                Utc::now(),
                100.0 + i as f64,
                102.0 + i as f64,
                99.0 + i as f64,
                101.0 + i as f64,
                1000.0,
            );
            engine.update(&data).unwrap();
        }
        
        // Should be ready after adding sufficient data
        assert!(engine.is_ready());
    }

    #[test]
    fn test_market_data_conversion() {
        let engine = TechnicalEngine::new(&[]).unwrap();
        
        // Create test DataFrame
        let dates = vec!["2024-01-01", "2024-01-02"];
        let opens = vec![100.0, 101.0];
        let highs = vec![102.0, 103.0];
        let lows = vec![99.0, 100.0];
        let closes = vec![101.0, 102.0];
        let volumes = vec![1000.0, 1100.0];

        let df = DataFrame::new(vec![
            Series::new("date", dates),
            Series::new("open", opens),
            Series::new("high", highs),
            Series::new("low", lows),
            Series::new("close", closes),
            Series::new("volume", volumes),
        ]).unwrap();

        let market_data = engine.convert_dataframe_to_market_data(&df);
        assert!(market_data.is_ok());
        
        let market_data = market_data.unwrap();
        assert_eq!(market_data.len(), 2);
        assert_eq!(market_data[0].open, 100.0);
        assert_eq!(market_data[1].close, 102.0);
    }
} 