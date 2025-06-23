//! NyxsOwl Hybrid Strategy Framework
//!
//! This module provides a comprehensive framework for integrating technical indicators
//! with forecasting models to create robust, adaptive trading strategies.
//!
//! ## Overview
//!
//! The Hybrid Strategy Framework combines the strengths of technical analysis and
//! forecasting while addressing their individual limitations through:
//!
//! - **Multi-layer signal confirmation** - Reduces false signals through technical,
//!   forecasting, volume, and pattern confirmation
//! - **Regime-aware model selection** - Adapts to market conditions automatically
//! - **Advanced feature engineering** - Creates predictive features from both
//!   technical indicators and forecasting models
//! - **Ensemble methods** - Reduces overfitting through model combination
//! - **Outlier detection** - Improves data quality and signal reliability
//! - **Comprehensive technical indicators** - 125+ indicators including momentum,
//!   trend, volatility, and volume-based indicators
//!
//! ## Architecture
//!
//! ```
//! Technical Indicators → Feature Engineering → Signal Confirmation
//!         ↓                       ↓                    ↓
//! Forecasting Models → Feature Engineering → Signal Confirmation
//!         ↓                       ↓                    ↓
//!     Integration Engine → Final Hybrid Signal
//! ```
//!
//! ## Quick Start
//!
//! ```rust
//! use nyxs_owl::hybrid::*;
//! use nyxs_owl::prelude::*;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create hybrid strategy configuration
//!     let config = HybridStrategyConfig {
//!         technical_indicators: vec![
//!             TechnicalIndicatorConfig::RSI { period: 14, oversold: 30.0, overbought: 70.0 },
//!             TechnicalIndicatorConfig::MACD { fast_period: 12, slow_period: 26, signal_period: 9 },
//!             TechnicalIndicatorConfig::CCI { period: 20 },
//!             TechnicalIndicatorConfig::MFI { period: 14 },
//!             TechnicalIndicatorConfig::ROC { period: 10 },
//!         ],
//!         forecasting_models: vec![
//!             ForecastingModelConfig::ARIMA {
//!                 auto_order: true,
//!                 ensemble_forecasting: true,
//!                 regime_detection: true,
//!                 outlier_detection: true,
//!             },
//!         ],
//!         feature_engineering: FeatureEngineeringConfig::default(),
//!         signal_confirmation: SignalConfirmationConfig::default(),
//!         integration: IntegrationConfig::WeightedConsensus {
//!             technical_weight: 0.6,
//!             forecast_weight: 0.4,
//!             min_confidence: 0.7,
//!             confirmation_window: 5,
//!         },
//!     };
//!
//!     // Create and run hybrid strategy
//!     let mut strategy = HybridStrategy::new(config)?;
//!     let signals = strategy.generate_signals(&market_data).await?;
//!
//!     println!("Generated {} hybrid signals", signals.len());
//!     Ok(())
//! }
//! ```
//!
//! ## Module Structure
//!
//! - **`config.rs`** - Configuration structures for all hybrid components
//! - **`engine.rs`** - Core hybrid strategy engine implementation
//! - **`technical/`** - Technical indicator integration and signal generation
//!   - **`indicators.rs`** - Comprehensive technical indicators (125+ indicators)
//!   - **`signals.rs`** - Signal generation from technical indicators
//!   - **`patterns.rs`** - Pattern recognition and analysis
//! - **`forecasting/`** - Forecasting model integration and ensemble methods
//! - **`features/`** - Feature engineering pipeline
//! - **`confirmation/`** - Signal confirmation framework
//! - **`integration/`** - Signal integration and final output generation
//! - **`types.rs`** - Core data types and structures
//! - **`error.rs`** - Error handling specific to hybrid operations

use crate::prelude::*;
use polars::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Re-export main types for easy access
pub use config::*;
pub use engine::*;
pub use types::*;

// Sub-modules
pub mod config;
pub mod engine;
pub mod technical;
pub mod forecasting;
pub mod features;
pub mod confirmation;
pub mod integration;
pub mod types;
pub mod error;

/// Result type for hybrid strategy operations
pub type HybridResult<T> = Result<T, HybridError>;

/// Main hybrid strategy implementation
///
/// This is the primary interface for creating and running hybrid strategies
/// that combine technical indicators with forecasting models.
#[derive(Debug)]
pub struct HybridStrategy {
    config: HybridStrategyConfig,
    technical_engine: technical::TechnicalEngine,
    forecasting_engine: forecasting::ForecastingEngine,
    feature_engine: features::FeatureEngineeringEngine,
    signal_confirmation: confirmation::SignalConfirmationEngine,
    integration_engine: integration::IntegrationEngine,
}

impl HybridStrategy {
    /// Create a new hybrid strategy with the given configuration
    ///
    /// # Arguments
    ///
    /// * `config` - Configuration for the hybrid strategy
    ///
    /// # Returns
    ///
    /// Returns a new `HybridStrategy` instance or an error if configuration is invalid
    ///
    /// # Example
    ///
    /// ```rust
    /// use nyxs_owl::hybrid::*;
    ///
    /// let config = HybridStrategyConfig::default();
    /// let strategy = HybridStrategy::new(config)?;
    /// ```
    pub fn new(config: HybridStrategyConfig) -> HybridResult<Self> {
        // Validate configuration
        config.validate()?;

        // Initialize engines
        let technical_engine = technical::TechnicalEngine::new(&config.technical_indicators)?;
        let forecasting_engine = forecasting::ForecastingEngine::new(&config.forecasting_models)?;
        let feature_engine = features::FeatureEngineeringEngine::new(&config.feature_engineering)?;
        let signal_confirmation = confirmation::SignalConfirmationEngine::new(&config.signal_confirmation)?;
        let integration_engine = integration::IntegrationEngine::new(&config.integration)?;

        Ok(Self {
            config,
            technical_engine,
            forecasting_engine,
            feature_engine,
            signal_confirmation,
            integration_engine,
        })
    }

    /// Generate hybrid signals from market data
    ///
    /// This method processes the input market data through the complete hybrid
    /// pipeline: technical indicators, forecasting models, feature engineering,
    /// signal confirmation, and final integration.
    ///
    /// # Arguments
    ///
    /// * `market_data` - OHLCV market data with timestamps
    ///
    /// # Returns
    ///
    /// Returns a vector of confirmed hybrid signals
    ///
    /// # Example
    ///
    /// ```rust
    /// use nyxs_owl::hybrid::*;
    /// use polars::prelude::*;
    ///
    /// let mut strategy = HybridStrategy::new(config)?;
    /// let df = LazyFrame::scan_csv("data.csv", ScanArgsCSV::default())?.collect()?;
    /// let signals = strategy.generate_signals(&df).await?;
    /// ```
    pub async fn generate_signals(&mut self, market_data: &DataFrame) -> HybridResult<Vec<HybridSignal>> {
        // Step 1: Generate technical signals
        let technical_signals = self.technical_engine.generate_signals(market_data)?;

        // Step 2: Generate forecasting signals
        let forecast_signals = self.forecasting_engine.generate_forecasts(market_data).await?;

        // Step 3: Extract features
        let features = self.feature_engine.extract_features(
            &technical_signals,
            &forecast_signals,
            market_data,
        )?;

        // Step 4: Generate initial hybrid signals
        let mut hybrid_signals = self.integration_engine.generate_initial_signals(
            &technical_signals,
            &forecast_signals,
            &features,
        )?;

        // Step 5: Apply signal confirmation
        for signal in &mut hybrid_signals {
            let confirmed_signal = self.signal_confirmation.confirm_signal(signal, market_data)?;
            *signal = confirmed_signal.into();
        }

        // Step 6: Filter by confidence threshold
        hybrid_signals.retain(|signal| signal.confidence >= self.config.integration.min_confidence());

        Ok(hybrid_signals)
    }

    /// Update the strategy with new market data (streaming mode)
    ///
    /// This method is optimized for real-time streaming applications where
    /// new data arrives incrementally.
    ///
    /// # Arguments
    ///
    /// * `new_data` - New market data point
    ///
    /// # Returns
    ///
    /// Returns a signal if conditions are met, None otherwise
    ///
    /// # Example
    ///
    /// ```rust
    /// use nyxs_owl::hybrid::*;
    ///
    /// let mut strategy = HybridStrategy::new(config)?;
    /// 
    /// // In a streaming loop
    /// for tick in data_stream {
    ///     if let Some(signal) = strategy.update_streaming(&tick)? {
    ///         println!("Signal generated: {:?}", signal);
    ///     }
    /// }
    /// ```
    pub fn update_streaming(&mut self, new_data: &MarketData) -> HybridResult<Option<HybridSignal>> {
        // Update technical indicators
        self.technical_engine.update(new_data)?;

        // Update forecasting models
        self.forecasting_engine.update(new_data)?;

        // Check if we have enough data for signal generation
        if !self.technical_engine.is_ready() || !self.forecasting_engine.is_ready() {
            return Ok(None);
        }

        // Generate signals for the current state
        let technical_signals = self.technical_engine.get_current_signals()?;
        let forecast_signals = self.forecasting_engine.get_current_forecasts()?;

        // Extract features for current state
        let features = self.feature_engine.extract_current_features(
            &technical_signals,
            &forecast_signals,
            new_data,
        )?;

        // Generate and confirm signal
        let mut hybrid_signal = self.integration_engine.generate_single_signal(
            &technical_signals,
            &forecast_signals,
            &features,
        )?;

        if let Some(signal) = &mut hybrid_signal {
            let confirmed_signal = self.signal_confirmation.confirm_signal(signal, new_data)?;
            *signal = confirmed_signal.into();
        }

        Ok(hybrid_signal)
    }

    /// Get the current configuration
    pub fn config(&self) -> &HybridStrategyConfig {
        &self.config
    }

    /// Update the configuration (requires reinitialization of engines)
    pub fn update_config(&mut self, new_config: HybridStrategyConfig) -> HybridResult<()> {
        // Validate new configuration
        new_config.validate()?;

        // Reinitialize engines with new configuration
        self.technical_engine = technical::TechnicalEngine::new(&new_config.technical_indicators)?;
        self.forecasting_engine = forecasting::ForecastingEngine::new(&new_config.forecasting_models)?;
        self.feature_engine = features::FeatureEngineeringEngine::new(&new_config.feature_engineering)?;
        self.signal_confirmation = confirmation::SignalConfirmationEngine::new(&new_config.signal_confirmation)?;
        self.integration_engine = integration::IntegrationEngine::new(&new_config.integration)?;

        self.config = new_config;
        Ok(())
    }

    /// Reset the strategy state (useful for backtesting)
    pub fn reset(&mut self) -> HybridResult<()> {
        self.technical_engine.reset()?;
        self.forecasting_engine.reset()?;
        self.feature_engine.reset()?;
        self.signal_confirmation.reset()?;
        self.integration_engine.reset()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prelude::*;

    #[test]
    fn test_hybrid_strategy_creation() {
        let config = HybridStrategyConfig::default();
        let strategy = HybridStrategy::new(config);
        assert!(strategy.is_ok());
    }

    #[test]
    fn test_hybrid_strategy_config_validation() {
        let mut config = HybridStrategyConfig::default();
        config.technical_indicators.clear(); // Invalid: no technical indicators
        
        let strategy = HybridStrategy::new(config);
        assert!(strategy.is_err());
    }

    #[tokio::test]
    async fn test_hybrid_strategy_signal_generation() {
        let config = HybridStrategyConfig::default();
        let mut strategy = HybridStrategy::new(config).unwrap();

        // Create test market data
        let df = create_test_market_data();
        let signals = strategy.generate_signals(&df).await;
        
        assert!(signals.is_ok());
        let signals = signals.unwrap();
        assert!(signals.is_empty() || signals.iter().all(|s| s.confidence >= 0.0 && s.confidence <= 1.0));
    }

    fn create_test_market_data() -> DataFrame {
        let dates = vec![
            "2024-01-01", "2024-01-02", "2024-01-03", "2024-01-04", "2024-01-05"
        ];
        let opens = vec![100.0, 101.0, 102.0, 101.5, 103.0];
        let highs = vec![102.0, 103.0, 104.0, 103.5, 105.0];
        let lows = vec![99.0, 100.0, 101.0, 100.5, 102.0];
        let closes = vec![101.0, 102.0, 103.0, 102.5, 104.0];
        let volumes = vec![1000, 1100, 1200, 1150, 1300];

        DataFrame::new(vec![
            Series::new("date", dates),
            Series::new("open", opens),
            Series::new("high", highs),
            Series::new("low", lows),
            Series::new("close", closes),
            Series::new("volume", volumes),
        ]).unwrap()
    }
} 