//! Configuration structures for the Hybrid Strategy Framework
//!
//! This module defines all configuration types used throughout the hybrid
//! strategy framework, ensuring type safety and validation.

use crate::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Main configuration for hybrid strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HybridStrategyConfig {
    /// Technical indicator configurations
    pub technical_indicators: Vec<TechnicalIndicatorConfig>,
    /// Forecasting model configurations
    pub forecasting_models: Vec<ForecastingModelConfig>,
    /// Feature engineering configuration
    pub feature_engineering: FeatureEngineeringConfig,
    /// Signal confirmation configuration
    pub signal_confirmation: SignalConfirmationConfig,
    /// Integration configuration
    pub integration: IntegrationConfig,
}

impl HybridStrategyConfig {
    /// Create a new hybrid strategy configuration
    pub fn new() -> Self {
        Self {
            technical_indicators: Vec::new(),
            forecasting_models: Vec::new(),
            feature_engineering: FeatureEngineeringConfig::default(),
            signal_confirmation: SignalConfirmationConfig::default(),
            integration: IntegrationConfig::default(),
        }
    }

    /// Add a technical indicator configuration
    pub fn with_technical_indicator(mut self, config: TechnicalIndicatorConfig) -> Self {
        self.technical_indicators.push(config);
        self
    }

    /// Add a forecasting model configuration
    pub fn with_forecasting_model(mut self, config: ForecastingModelConfig) -> Self {
        self.forecasting_models.push(config);
        self
    }

    /// Set feature engineering configuration
    pub fn with_feature_engineering(mut self, config: FeatureEngineeringConfig) -> Self {
        self.feature_engineering = config;
        self
    }

    /// Set signal confirmation configuration
    pub fn with_signal_confirmation(mut self, config: SignalConfirmationConfig) -> Self {
        self.signal_confirmation = config;
        self
    }

    /// Set integration configuration
    pub fn with_integration(mut self, config: IntegrationConfig) -> Self {
        self.integration = config;
        self
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<(), HybridError> {
        // Check that we have at least one technical indicator
        if self.technical_indicators.is_empty() {
            return Err(HybridError::validation("At least one technical indicator is required"));
        }

        // Check that we have at least one forecasting model
        if self.forecasting_models.is_empty() {
            return Err(HybridError::validation("At least one forecasting model is required"));
        }

        // Validate individual components
        for (i, indicator) in self.technical_indicators.iter().enumerate() {
            indicator.validate().map_err(|e| {
                HybridError::validation(format!("Technical indicator {}: {}", i, e))
            })?;
        }

        for (i, model) in self.forecasting_models.iter().enumerate() {
            model.validate().map_err(|e| {
                HybridError::validation(format!("Forecasting model {}: {}", i, e))
            })?;
        }

        self.feature_engineering.validate()?;
        self.signal_confirmation.validate()?;
        self.integration.validate()?;

        Ok(())
    }
}

impl Default for HybridStrategyConfig {
    fn default() -> Self {
        Self {
            technical_indicators: vec![
                TechnicalIndicatorConfig::RSI {
                    period: 14,
                    oversold: 30.0,
                    overbought: 70.0,
                },
                TechnicalIndicatorConfig::MACD {
                    fast_period: 12,
                    slow_period: 26,
                    signal_period: 9,
                },
                TechnicalIndicatorConfig::CCI {
                    period: 20,
                    threshold: 100.0,
                },
                TechnicalIndicatorConfig::MFI {
                    period: 14,
                    oversold: 20.0,
                    overbought: 80.0,
                },
                TechnicalIndicatorConfig::ROC {
                    period: 10,
                },
                TechnicalIndicatorConfig::BollingerBands {
                    period: 20,
                    std_dev: 2.0,
                },
            ],
            forecasting_models: vec![
                ForecastingModelConfig::ARIMA {
                    auto_order: true,
                    ensemble_forecasting: true,
                    regime_detection: true,
                    outlier_detection: true,
                },
            ],
            feature_engineering: FeatureEngineeringConfig::default(),
            signal_confirmation: SignalConfirmationConfig::default(),
            integration: IntegrationConfig::default(),
        }
    }
}

/// Configuration for technical indicators
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TechnicalIndicatorConfig {
    /// RSI (Relative Strength Index) configuration
    RSI {
        /// Period for RSI calculation
        period: usize,
        /// Oversold threshold
        oversold: f64,
        /// Overbought threshold
        overbought: f64,
    },
    /// MACD (Moving Average Convergence Divergence) configuration
    MACD {
        /// Fast period
        fast_period: usize,
        /// Slow period
        slow_period: usize,
        /// Signal period
        signal_period: usize,
    },
    /// Bollinger Bands configuration
    BollingerBands {
        /// Period for moving average
        period: usize,
        /// Standard deviation multiplier
        std_dev: f64,
    },
    /// CCI (Commodity Channel Index) configuration
    CCI {
        /// Period for CCI calculation
        period: usize,
        /// Threshold for overbought/oversold
        threshold: f64,
    },
    /// MFI (Money Flow Index) configuration
    MFI {
        /// Period for MFI calculation
        period: usize,
        /// Oversold threshold
        oversold: f64,
        /// Overbought threshold
        overbought: f64,
    },
    /// ROC (Rate of Change) configuration
    ROC {
        /// Period for ROC calculation
        period: usize,
    },
    /// Custom indicator configuration
    Custom {
        /// Indicator name
        name: String,
        /// Custom parameters
        parameters: HashMap<String, String>,
    },
}

impl TechnicalIndicatorConfig {
    /// Validate the technical indicator configuration
    pub fn validate(&self) -> Result<(), String> {
        match self {
            Self::RSI { period, oversold, overbought } => {
                if *period == 0 {
                    return Err("RSI period must be greater than 0".to_string());
                }
                if *oversold >= *overbought {
                    return Err("RSI oversold must be less than overbought".to_string());
                }
                if *oversold < 0.0 || *overbought > 100.0 {
                    return Err("RSI thresholds must be between 0 and 100".to_string());
                }
            }
            Self::MACD { fast_period, slow_period, signal_period } => {
                if *fast_period == 0 || *slow_period == 0 || *signal_period == 0 {
                    return Err("MACD periods must be greater than 0".to_string());
                }
                if *fast_period >= *slow_period {
                    return Err("MACD fast period must be less than slow period".to_string());
                }
            }
            Self::BollingerBands { period, std_dev } => {
                if *period == 0 {
                    return Err("Bollinger Bands period must be greater than 0".to_string());
                }
                if *std_dev <= 0.0 {
                    return Err("Bollinger Bands standard deviation must be positive".to_string());
                }
            }
            Self::CCI { period, threshold } => {
                if *period == 0 {
                    return Err("CCI period must be greater than 0".to_string());
                }
                if *threshold <= 0.0 {
                    return Err("CCI threshold must be positive".to_string());
                }
            }
            Self::MFI { period, oversold, overbought } => {
                if *period == 0 {
                    return Err("MFI period must be greater than 0".to_string());
                }
                if *oversold >= *overbought {
                    return Err("MFI oversold must be less than overbought".to_string());
                }
                if *oversold < 0.0 || *overbought > 100.0 {
                    return Err("MFI thresholds must be between 0 and 100".to_string());
                }
            }
            Self::ROC { period } => {
                if *period == 0 {
                    return Err("ROC period must be greater than 0".to_string());
                }
            }
            Self::Custom { name, parameters } => {
                if name.is_empty() {
                    return Err("Custom indicator name cannot be empty".to_string());
                }
                if parameters.is_empty() {
                    return Err("Custom indicator must have at least one parameter".to_string());
                }
            }
        }
        Ok(())
    }
}

/// Configuration for forecasting models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ForecastingModelConfig {
    /// ARIMA model configuration
    ARIMA {
        /// Enable automatic order selection
        auto_order: bool,
        /// Enable ensemble forecasting
        ensemble_forecasting: bool,
        /// Enable regime detection
        regime_detection: bool,
        /// Enable outlier detection
        outlier_detection: bool,
    },
    /// Ensemble model configuration
    Ensemble {
        /// Model names to include in ensemble
        models: Vec<String>,
        /// Enable adaptive weighting
        adaptive_weighting: bool,
    },
    /// Exponential Smoothing configuration
    ExponentialSmoothing {
        /// Alpha parameter
        alpha: f64,
        /// Beta parameter
        beta: f64,
        /// Gamma parameter
        gamma: f64,
    },
    /// Custom model configuration
    Custom {
        /// Model name
        name: String,
        /// Custom parameters
        parameters: HashMap<String, String>,
    },
}

impl ForecastingModelConfig {
    /// Validate the forecasting model configuration
    pub fn validate(&self) -> Result<(), String> {
        match self {
            Self::ARIMA { .. } => {
                // ARIMA validation is handled by OxiDiviner
                Ok(())
            }
            Self::Ensemble { models, .. } => {
                if models.is_empty() {
                    return Err("Ensemble must contain at least one model".to_string());
                }
                for model in models {
                    if model.is_empty() {
                        return Err("Ensemble model names cannot be empty".to_string());
                    }
                }
            }
            Self::ExponentialSmoothing { alpha, beta, gamma } => {
                if !(0.0..=1.0).contains(alpha) {
                    return Err("Exponential smoothing alpha must be between 0 and 1".to_string());
                }
                if !(0.0..=1.0).contains(beta) {
                    return Err("Exponential smoothing beta must be between 0 and 1".to_string());
                }
                if !(0.0..=1.0).contains(gamma) {
                    return Err("Exponential smoothing gamma must be between 0 and 1".to_string());
                }
            }
            Self::Custom { name, parameters } => {
                if name.is_empty() {
                    return Err("Custom model name cannot be empty".to_string());
                }
                if parameters.is_empty() {
                    return Err("Custom model must have at least one parameter".to_string());
                }
            }
        }
        Ok(())
    }
}

/// Configuration for feature engineering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureEngineeringConfig {
    /// Enable technical features
    pub technical_features: bool,
    /// Enable forecasting features
    pub forecasting_features: bool,
    /// Enable derived features
    pub derived_features: bool,
    /// Custom features
    pub custom_features: Vec<CustomFeatureConfig>,
    /// Enable feature selection
    pub feature_selection: bool,
    /// Enable feature scaling
    pub feature_scaling: bool,
}

impl FeatureEngineeringConfig {
    /// Validate the feature engineering configuration
    pub fn validate(&self) -> Result<(), HybridError> {
        // At least one feature type must be enabled
        if !self.technical_features && !self.forecasting_features && !self.derived_features {
            return Err(HybridError::validation("At least one feature type must be enabled"));
        }

        // Validate custom features
        for (i, feature) in self.custom_features.iter().enumerate() {
            feature.validate().map_err(|e| {
                HybridError::validation(format!("Custom feature {}: {}", i, e))
            })?;
        }

        Ok(())
    }
}

impl Default for FeatureEngineeringConfig {
    fn default() -> Self {
        Self {
            technical_features: true,
            forecasting_features: true,
            derived_features: true,
            custom_features: Vec::new(),
            feature_selection: true,
            feature_scaling: true,
        }
    }
}

/// Configuration for custom features
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomFeatureConfig {
    /// Feature name
    pub name: String,
    /// Feature calculation expression
    pub calculation: String,
}

impl CustomFeatureConfig {
    /// Validate the custom feature configuration
    pub fn validate(&self) -> Result<(), String> {
        if self.name.is_empty() {
            return Err("Custom feature name cannot be empty".to_string());
        }
        if self.calculation.is_empty() {
            return Err("Custom feature calculation cannot be empty".to_string());
        }
        Ok(())
    }
}

/// Configuration for signal confirmation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalConfirmationConfig {
    /// Enable technical confirmation
    pub technical_confirmation: bool,
    /// Enable forecasting confirmation
    pub forecasting_confirmation: bool,
    /// Enable volume confirmation
    pub volume_confirmation: bool,
    /// Enable pattern confirmation
    pub pattern_confirmation: bool,
    /// Enable multi-timeframe confirmation
    pub multi_timeframe: bool,
    /// Custom confirmation methods
    pub custom_confirmation: Vec<CustomConfirmationConfig>,
    /// Minimum confirmation score
    pub min_confirmation_score: f64,
}

impl SignalConfirmationConfig {
    /// Validate the signal confirmation configuration
    pub fn validate(&self) -> Result<(), HybridError> {
        // At least one confirmation method must be enabled
        if !self.technical_confirmation
            && !self.forecasting_confirmation
            && !self.volume_confirmation
            && !self.pattern_confirmation
            && !self.multi_timeframe
        {
            return Err(HybridError::validation("At least one confirmation method must be enabled"));
        }

        // Validate confirmation score range
        if !(0.0..=1.0).contains(&self.min_confirmation_score) {
            return Err(HybridError::validation("Minimum confirmation score must be between 0 and 1"));
        }

        // Validate custom confirmation methods
        for (i, confirmation) in self.custom_confirmation.iter().enumerate() {
            confirmation.validate().map_err(|e| {
                HybridError::validation(format!("Custom confirmation {}: {}", i, e))
            })?;
        }

        Ok(())
    }
}

impl Default for SignalConfirmationConfig {
    fn default() -> Self {
        Self {
            technical_confirmation: true,
            forecasting_confirmation: true,
            volume_confirmation: true,
            pattern_confirmation: true,
            multi_timeframe: true,
            custom_confirmation: Vec::new(),
            min_confirmation_score: 0.7,
        }
    }
}

/// Configuration for custom confirmation methods
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomConfirmationConfig {
    /// Confirmation method name
    pub name: String,
    /// Weight for this confirmation method
    pub weight: f64,
    /// Threshold for this confirmation method
    pub threshold: f64,
}

impl CustomConfirmationConfig {
    /// Validate the custom confirmation configuration
    pub fn validate(&self) -> Result<(), String> {
        if self.name.is_empty() {
            return Err("Custom confirmation name cannot be empty".to_string());
        }
        if !(0.0..=1.0).contains(&self.weight) {
            return Err("Custom confirmation weight must be between 0 and 1".to_string());
        }
        if !(0.0..=1.0).contains(&self.threshold) {
            return Err("Custom confirmation threshold must be between 0 and 1".to_string());
        }
        Ok(())
    }
}

/// Configuration for signal integration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IntegrationConfig {
    /// Weighted consensus integration
    WeightedConsensus {
        /// Weight for technical signals
        technical_weight: f64,
        /// Weight for forecasting signals
        forecast_weight: f64,
        /// Minimum confidence threshold
        min_confidence: f64,
        /// Confirmation window size
        confirmation_window: usize,
    },
    /// Adaptive integration
    Adaptive {
        /// Base weights for different signal types
        base_weights: HashMap<String, f64>,
        /// Adaptation rate
        adaptation_rate: f64,
        /// Minimum confidence threshold
        min_confidence: f64,
    },
}

impl IntegrationConfig {
    /// Validate the integration configuration
    pub fn validate(&self) -> Result<(), HybridError> {
        match self {
            Self::WeightedConsensus {
                technical_weight,
                forecast_weight,
                min_confidence,
                confirmation_window,
            } => {
                if *technical_weight < 0.0 || *forecast_weight < 0.0 {
                    return Err(HybridError::validation("Weights must be non-negative"));
                }
                if (*technical_weight + *forecast_weight - 1.0).abs() > 1e-6 {
                    return Err(HybridError::validation("Weights must sum to 1.0"));
                }
                if !(0.0..=1.0).contains(min_confidence) {
                    return Err(HybridError::validation("Minimum confidence must be between 0 and 1"));
                }
                if *confirmation_window == 0 {
                    return Err(HybridError::validation("Confirmation window must be greater than 0"));
                }
            }
            Self::Adaptive {
                base_weights,
                adaptation_rate,
                min_confidence,
            } => {
                if base_weights.is_empty() {
                    return Err(HybridError::validation("Base weights cannot be empty"));
                }
                for (name, weight) in base_weights {
                    if name.is_empty() {
                        return Err(HybridError::validation("Weight name cannot be empty"));
                    }
                    if *weight < 0.0 {
                        return Err(HybridError::validation("Base weights must be non-negative"));
                    }
                }
                if !(0.0..=1.0).contains(adaptation_rate) {
                    return Err(HybridError::validation("Adaptation rate must be between 0 and 1"));
                }
                if !(0.0..=1.0).contains(min_confidence) {
                    return Err(HybridError::validation("Minimum confidence must be between 0 and 1"));
                }
            }
        }
        Ok(())
    }

    /// Get the minimum confidence threshold
    pub fn min_confidence(&self) -> f64 {
        match self {
            Self::WeightedConsensus { min_confidence, .. } => *min_confidence,
            Self::Adaptive { min_confidence, .. } => *min_confidence,
        }
    }
}

impl Default for IntegrationConfig {
    fn default() -> Self {
        Self::WeightedConsensus {
            technical_weight: 0.6,
            forecast_weight: 0.4,
            min_confidence: 0.7,
            confirmation_window: 5,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hybrid_strategy_config_creation() {
        let config = HybridStrategyConfig::new();
        assert!(config.technical_indicators.is_empty());
        assert!(config.forecasting_models.is_empty());
    }

    #[test]
    fn test_hybrid_strategy_config_validation() {
        let mut config = HybridStrategyConfig::new();
        
        // Should fail validation (no indicators or models)
        assert!(config.validate().is_err());

        // Add required components
        config = config
            .with_technical_indicator(TechnicalIndicatorConfig::RSI {
                period: 14,
                oversold: 30.0,
                overbought: 70.0,
            })
            .with_forecasting_model(ForecastingModelConfig::ARIMA {
                auto_order: true,
                ensemble_forecasting: true,
                regime_detection: true,
                outlier_detection: true,
            });

        // Should pass validation
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_technical_indicator_config_validation() {
        // Valid RSI config
        let valid_rsi = TechnicalIndicatorConfig::RSI {
            period: 14,
            oversold: 30.0,
            overbought: 70.0,
        };
        assert!(valid_rsi.validate().is_ok());

        // Invalid RSI config (oversold >= overbought)
        let invalid_rsi = TechnicalIndicatorConfig::RSI {
            period: 14,
            oversold: 70.0,
            overbought: 30.0,
        };
        assert!(invalid_rsi.validate().is_err());
    }

    #[test]
    fn test_forecasting_model_config_validation() {
        // Valid ARIMA config
        let valid_arima = ForecastingModelConfig::ARIMA {
            auto_order: true,
            ensemble_forecasting: true,
            regime_detection: true,
            outlier_detection: true,
        };
        assert!(valid_arima.validate().is_ok());

        // Invalid ensemble config (empty models)
        let invalid_ensemble = ForecastingModelConfig::Ensemble {
            models: Vec::new(),
            adaptive_weighting: true,
        };
        assert!(invalid_ensemble.validate().is_err());
    }

    #[test]
    fn test_integration_config_validation() {
        // Valid weighted consensus
        let valid_consensus = IntegrationConfig::WeightedConsensus {
            technical_weight: 0.6,
            forecast_weight: 0.4,
            min_confidence: 0.7,
            confirmation_window: 5,
        };
        assert!(valid_consensus.validate().is_ok());

        // Invalid weighted consensus (weights don't sum to 1.0)
        let invalid_consensus = IntegrationConfig::WeightedConsensus {
            technical_weight: 0.6,
            forecast_weight: 0.6, // Sum = 1.2
            min_confidence: 0.7,
            confirmation_window: 5,
        };
        assert!(invalid_consensus.validate().is_err());
    }

    #[test]
    fn test_default_configs() {
        let default_config = HybridStrategyConfig::default();
        assert!(!default_config.technical_indicators.is_empty());
        assert!(!default_config.forecasting_models.is_empty());
        assert!(default_config.validate().is_ok());
    }
} 