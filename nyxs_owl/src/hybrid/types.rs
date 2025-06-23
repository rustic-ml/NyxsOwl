//! Core data types and structures for the Hybrid Strategy Framework
//!
//! This module defines the fundamental data types used throughout the hybrid
//! strategy framework, including signals, features, and market data structures.

use crate::prelude::*;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configuration for technical indicators calculation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechnicalConfig {
    /// RSI period
    pub rsi_period: usize,
    /// CCI period
    pub cci_period: usize,
    /// MFI period
    pub mfi_period: usize,
    /// ROC period
    pub roc_period: usize,
    /// SMA short period
    pub sma_short_period: usize,
    /// SMA long period
    pub sma_long_period: usize,
    /// EMA short period
    pub ema_short_period: usize,
    /// EMA long period
    pub ema_long_period: usize,
    /// ATR period
    pub atr_period: usize,
    /// Bollinger Bands period
    pub bollinger_period: usize,
    /// Bollinger Bands standard deviation
    pub bollinger_std_dev: f64,
    /// VWAP period
    pub vwap_period: usize,
}

impl TechnicalConfig {
    /// Create configuration from technical indicator configs
    pub fn from_indicators(configs: &[crate::hybrid::config::TechnicalIndicatorConfig]) -> Self {
        let mut config = Self::default();
        
        for indicator_config in configs {
            match indicator_config {
                crate::hybrid::config::TechnicalIndicatorConfig::RSI { period, .. } => {
                    config.rsi_period = *period;
                }
                crate::hybrid::config::TechnicalIndicatorConfig::CCI { period, .. } => {
                    config.cci_period = *period;
                }
                crate::hybrid::config::TechnicalIndicatorConfig::MFI { period, .. } => {
                    config.mfi_period = *period;
                }
                crate::hybrid::config::TechnicalIndicatorConfig::ROC { period } => {
                    config.roc_period = *period;
                }
                _ => {}
            }
        }
        
        config
    }
}

impl Default for TechnicalConfig {
    fn default() -> Self {
        Self {
            rsi_period: 14,
            cci_period: 20,
            mfi_period: 14,
            roc_period: 10,
            sma_short_period: 10,
            sma_long_period: 20,
            ema_short_period: 12,
            ema_long_period: 26,
            atr_period: 14,
            bollinger_period: 20,
            bollinger_std_dev: 2.0,
            vwap_period: 14,
        }
    }
}

/// Result of technical indicators calculation
#[derive(Debug, Clone)]
pub struct TechnicalIndicatorsResult {
    /// Momentum indicators
    pub momentum: MomentumIndicators,
    /// Trend indicators
    pub trend: TrendIndicators,
    /// Volatility indicators
    pub volatility: VolatilityIndicators,
    /// Volume indicators
    pub volume: VolumeIndicators,
}

/// Momentum indicators
#[derive(Debug, Clone)]
pub struct MomentumIndicators {
    /// RSI values
    pub rsi: polars::prelude::Series,
    /// CCI values
    pub cci: polars::prelude::Series,
    /// MFI values
    pub mfi: polars::prelude::Series,
    /// ROC values
    pub roc: polars::prelude::Series,
}

/// Trend indicators
#[derive(Debug, Clone)]
pub struct TrendIndicators {
    /// Short SMA values
    pub sma_short: polars::prelude::Series,
    /// Long SMA values
    pub sma_long: polars::prelude::Series,
    /// Short EMA values
    pub ema_short: polars::prelude::Series,
    /// Long EMA values
    pub ema_long: polars::prelude::Series,
    /// SMA crossover signals
    pub sma_crossover: polars::prelude::Series,
    /// EMA crossover signals
    pub ema_crossover: polars::prelude::Series,
}

/// Volatility indicators
#[derive(Debug, Clone)]
pub struct VolatilityIndicators {
    /// ATR values
    pub atr: polars::prelude::Series,
    /// Bollinger Bands upper
    pub bollinger_upper: polars::prelude::Series,
    /// Bollinger Bands middle
    pub bollinger_middle: polars::prelude::Series,
    /// Bollinger Bands lower
    pub bollinger_lower: polars::prelude::Series,
}

/// Volume indicators
#[derive(Debug, Clone)]
pub struct VolumeIndicators {
    /// VWAP values
    pub vwap: polars::prelude::Series,
    /// Volume VWAP values
    pub volume_vwap: polars::prelude::Series,
}

/// Market data structure for hybrid strategy processing
///
/// This structure represents OHLCV data with timestamps, optimized for
/// efficient processing in the hybrid strategy framework.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketData {
    /// Timestamp of the data point
    pub timestamp: DateTime<Utc>,
    /// Opening price
    pub open: f64,
    /// High price
    pub high: f64,
    /// Low price
    pub low: f64,
    /// Closing price
    pub close: f64,
    /// Trading volume
    pub volume: f64,
    /// Additional metadata (optional)
    pub metadata: HashMap<String, f64>,
}

impl MarketData {
    /// Create new market data from OHLCV values
    pub fn new(
        timestamp: DateTime<Utc>,
        open: f64,
        high: f64,
        low: f64,
        close: f64,
        volume: f64,
    ) -> Self {
        Self {
            timestamp,
            open,
            high,
            low,
            close,
            volume,
            metadata: HashMap::new(),
        }
    }

    /// Create market data from a Polars DataFrame row
    pub fn from_dataframe_row(row: &[AnyValue]) -> Result<Self, HybridError> {
        if row.len() < 6 {
            return Err(HybridError::DataError("Insufficient columns for OHLCV data".to_string()));
        }

        let timestamp = row[0].datetime()?;
        let open = row[1].f64()?;
        let high = row[2].f64()?;
        let low = row[3].f64()?;
        let close = row[4].f64()?;
        let volume = row[5].f64()?;

        Ok(Self::new(timestamp, open, high, low, close, volume))
    }

    /// Get the typical price (high + low + close) / 3
    pub fn typical_price(&self) -> f64 {
        (self.high + self.low + self.close) / 3.0
    }

    /// Get the true range (max of high-low, high-prev_close, low-prev_close)
    pub fn true_range(&self, prev_close: f64) -> f64 {
        let hl = self.high - self.low;
        let hc = (self.high - prev_close).abs();
        let lc = (self.low - prev_close).abs();
        hl.max(hc).max(lc)
    }

    /// Get the price change from previous close
    pub fn price_change(&self, prev_close: f64) -> f64 {
        self.close - prev_close
    }

    /// Get the price change percentage from previous close
    pub fn price_change_pct(&self, prev_close: f64) -> f64 {
        if prev_close == 0.0 {
            0.0
        } else {
            (self.close - prev_close) / prev_close
        }
    }
}

/// Technical signal generated by technical indicators
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechnicalSignal {
    /// Name of the technical indicator
    pub indicator_name: String,
    /// Current value of the indicator
    pub value: f64,
    /// Signal type (Buy, Sell, Hold)
    pub signal_type: SignalType,
    /// Signal strength (-1.0 to 1.0)
    pub strength: f64,
    /// Confidence level (0.0 to 1.0)
    pub confidence: f64,
    /// Timestamp of the signal
    pub timestamp: DateTime<Utc>,
    /// Additional metadata
    pub metadata: HashMap<String, f64>,
    /// Detected patterns (if any)
    pub patterns: Vec<Pattern>,
    /// Detected divergences (if any)
    pub divergences: Vec<Divergence>,
}

impl TechnicalSignal {
    /// Create a new technical signal
    pub fn new(
        indicator_name: String,
        value: f64,
        signal_type: SignalType,
        strength: f64,
        confidence: f64,
        timestamp: DateTime<Utc>,
    ) -> Self {
        Self {
            indicator_name,
            value,
            signal_type,
            strength,
            confidence,
            timestamp,
            metadata: HashMap::new(),
            patterns: Vec::new(),
            divergences: Vec::new(),
        }
    }

    /// Add metadata to the signal
    pub fn with_metadata(mut self, key: String, value: f64) -> Self {
        self.metadata.insert(key, value);
        self
    }

    /// Add a pattern to the signal
    pub fn with_pattern(mut self, pattern: Pattern) -> Self {
        self.patterns.push(pattern);
        self
    }

    /// Add a divergence to the signal
    pub fn with_divergence(mut self, divergence: Divergence) -> Self {
        self.divergences.push(divergence);
        self
    }
}

/// Forecasting signal generated by forecasting models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForecastSignal {
    /// Name of the forecasting model
    pub model_name: String,
    /// Forecasted value
    pub forecast: f64,
    /// Confidence level (0.0 to 1.0)
    pub confidence: f64,
    /// Detected market regime
    pub regime: MarketRegime,
    /// Forecast horizon (number of periods ahead)
    pub horizon: usize,
    /// Timestamp of the forecast
    pub timestamp: DateTime<Utc>,
    /// Additional metadata
    pub metadata: HashMap<String, f64>,
}

impl ForecastSignal {
    /// Create a new forecast signal
    pub fn new(
        model_name: String,
        forecast: f64,
        confidence: f64,
        regime: MarketRegime,
        horizon: usize,
        timestamp: DateTime<Utc>,
    ) -> Self {
        Self {
            model_name,
            forecast,
            confidence,
            regime,
            horizon,
            timestamp,
            metadata: HashMap::new(),
        }
    }

    /// Add metadata to the forecast signal
    pub fn with_metadata(mut self, key: String, value: f64) -> Self {
        self.metadata.insert(key, value);
        self
    }
}

/// Feature extracted from technical indicators or forecasting models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Feature {
    /// Name of the feature
    pub name: String,
    /// Feature value
    pub value: f64,
    /// Feature type
    pub feature_type: FeatureType,
    /// Timestamp of the feature
    pub timestamp: DateTime<Utc>,
    /// Additional metadata
    pub metadata: HashMap<String, f64>,
}

impl Feature {
    /// Create a new feature
    pub fn new(name: String, value: f64, feature_type: FeatureType, timestamp: DateTime<Utc>) -> Self {
        Self {
            name,
            value,
            feature_type,
            timestamp,
            metadata: HashMap::new(),
        }
    }

    /// Add metadata to the feature
    pub fn with_metadata(mut self, key: String, value: f64) -> Self {
        self.metadata.insert(key, value);
        self
    }
}

/// Feature type classification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FeatureType {
    /// Momentum features (RSI, MACD, etc.)
    Momentum,
    /// Volatility features (Bollinger Bands, ATR, etc.)
    Volatility,
    /// Trend features (Moving averages, ADX, etc.)
    Trend,
    /// Volume features (OBV, VWAP, etc.)
    Volume,
    /// Forecasting features
    Forecasting,
    /// Derived features (interactions, lags, etc.)
    Derived,
    /// Custom features
    Custom(String),
}

/// Pattern detected in technical analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pattern {
    /// Pattern name
    pub name: String,
    /// Pattern type
    pub pattern_type: PatternType,
    /// Pattern strength (0.0 to 1.0)
    pub strength: f64,
    /// Pattern direction
    pub direction: PatternDirection,
    /// Additional metadata
    pub metadata: HashMap<String, f64>,
}

/// Pattern type classification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PatternType {
    /// Candlestick patterns
    Candlestick,
    /// Chart patterns
    Chart,
    /// Harmonic patterns
    Harmonic,
    /// Elliott Wave patterns
    ElliottWave,
    /// Custom patterns
    Custom(String),
}

/// Pattern direction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PatternDirection {
    /// Bullish pattern
    Bullish,
    /// Bearish pattern
    Bearish,
    /// Neutral pattern
    Neutral,
}

/// Divergence detected in technical analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Divergence {
    /// Divergence type
    pub divergence_type: DivergenceType,
    /// Divergence strength (0.0 to 1.0)
    pub strength: f64,
    /// Time period of the divergence
    pub period: usize,
    /// Additional metadata
    pub metadata: HashMap<String, f64>,
}

/// Divergence type classification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DivergenceType {
    /// Bullish divergence (price makes lower low, indicator makes higher low)
    Bullish,
    /// Bearish divergence (price makes higher high, indicator makes lower high)
    Bearish,
    /// Hidden bullish divergence
    HiddenBullish,
    /// Hidden bearish divergence
    HiddenBearish,
}

/// Final hybrid signal combining technical and forecasting signals
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HybridSignal {
    /// Signal type (Buy, Sell, Hold)
    pub signal_type: SignalType,
    /// Overall signal strength (-1.0 to 1.0)
    pub strength: f64,
    /// Overall confidence level (0.0 to 1.0)
    pub confidence: f64,
    /// Technical confidence component
    pub technical_confidence: f64,
    /// Forecasting confidence component
    pub forecast_confidence: f64,
    /// Confirmation score (0.0 to 1.0)
    pub confirmation_score: f64,
    /// Timestamp of the signal
    pub timestamp: DateTime<Utc>,
    /// Current price
    pub price: f64,
    /// Technical signals that contributed to this signal
    pub technical_signals: Vec<TechnicalSignal>,
    /// Forecast signals that contributed to this signal
    pub forecast_signals: Vec<ForecastSignal>,
    /// Features used in signal generation
    pub features: Vec<Feature>,
    /// Additional metadata
    pub metadata: HashMap<String, f64>,
}

impl HybridSignal {
    /// Create a new hybrid signal
    pub fn new(
        signal_type: SignalType,
        strength: f64,
        confidence: f64,
        technical_confidence: f64,
        forecast_confidence: f64,
        confirmation_score: f64,
        timestamp: DateTime<Utc>,
        price: f64,
    ) -> Self {
        Self {
            signal_type,
            strength,
            confidence,
            technical_confidence,
            forecast_confidence,
            confirmation_score,
            timestamp,
            price,
            technical_signals: Vec::new(),
            forecast_signals: Vec::new(),
            features: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    /// Add technical signals to the hybrid signal
    pub fn with_technical_signals(mut self, signals: Vec<TechnicalSignal>) -> Self {
        self.technical_signals = signals;
        self
    }

    /// Add forecast signals to the hybrid signal
    pub fn with_forecast_signals(mut self, signals: Vec<ForecastSignal>) -> Self {
        self.forecast_signals = signals;
        self
    }

    /// Add features to the hybrid signal
    pub fn with_features(mut self, features: Vec<Feature>) -> Self {
        self.features = features;
        self
    }

    /// Add metadata to the hybrid signal
    pub fn with_metadata(mut self, key: String, value: f64) -> Self {
        self.metadata.insert(key, value);
        self
    }

    /// Check if the signal is strong enough to act on
    pub fn is_actionable(&self, min_confidence: f64, min_strength: f64) -> bool {
        self.confidence >= min_confidence && self.strength.abs() >= min_strength
    }

    /// Get the signal direction as a string
    pub fn direction(&self) -> &'static str {
        match self.signal_type {
            SignalType::Buy => "BUY",
            SignalType::Sell => "SELL",
            SignalType::Hold => "HOLD",
        }
    }
}

/// Confirmed signal with additional validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfirmedSignal {
    /// Original hybrid signal
    pub original_signal: HybridSignal,
    /// Overall confirmation score (0.0 to 1.0)
    pub confirmation_score: f64,
    /// Component confirmation scores
    pub component_scores: HashMap<String, f64>,
    /// Risk assessment
    pub risk_assessment: RiskAssessment,
    /// Final confidence after confirmation
    pub confidence: f64,
}

impl From<ConfirmedSignal> for HybridSignal {
    fn from(confirmed: ConfirmedSignal) -> Self {
        let mut signal = confirmed.original_signal;
        signal.confidence = confirmed.confidence;
        signal.confirmation_score = confirmed.confirmation_score;
        signal
    }
}

/// Risk assessment for a signal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskAssessment {
    /// Overall risk level
    pub risk_level: RiskLevel,
    /// Maximum drawdown risk
    pub max_drawdown_risk: f64,
    /// Volatility risk
    pub volatility_risk: f64,
    /// Liquidity risk
    pub liquidity_risk: f64,
    /// Additional risk factors
    pub additional_risks: HashMap<String, f64>,
}

/// Risk level classification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskLevel {
    /// Low risk
    Low,
    /// Medium risk
    Medium,
    /// High risk
    High,
    /// Very high risk
    VeryHigh,
}

/// Market regime classification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MarketRegime {
    /// Trending market
    Trending,
    /// Ranging/sideways market
    Ranging,
    /// Volatile market
    Volatile,
    /// Low volatility market
    LowVolatility,
    /// High volatility market
    HighVolatility,
    /// Unknown regime
    Unknown,
}

impl MarketRegime {
    /// Get the regime name as a string
    pub fn name(&self) -> &'static str {
        match self {
            MarketRegime::Trending => "Trending",
            MarketRegime::Ranging => "Ranging",
            MarketRegime::Volatile => "Volatile",
            MarketRegime::LowVolatility => "LowVolatility",
            MarketRegime::HighVolatility => "HighVolatility",
            MarketRegime::Unknown => "Unknown",
        }
    }
}

/// Feature set containing multiple features
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureSet {
    /// Features in the set
    pub features: Vec<Feature>,
    /// Timestamp of the feature set
    pub timestamp: DateTime<Utc>,
    /// Additional metadata
    pub metadata: HashMap<String, f64>,
}

impl FeatureSet {
    /// Create a new feature set
    pub fn new(features: Vec<Feature>, timestamp: DateTime<Utc>) -> Self {
        Self {
            features,
            timestamp,
            metadata: HashMap::new(),
        }
    }

    /// Add a feature to the set
    pub fn add_feature(&mut self, feature: Feature) {
        self.features.push(feature);
    }

    /// Get a feature by name
    pub fn get_feature(&self, name: &str) -> Option<&Feature> {
        self.features.iter().find(|f| f.name == name)
    }

    /// Get feature value by name
    pub fn get_value(&self, name: &str) -> Option<f64> {
        self.get_feature(name).map(|f| f.value)
    }

    /// Get all feature names
    pub fn feature_names(&self) -> Vec<String> {
        self.features.iter().map(|f| f.name.clone()).collect()
    }

    /// Get all feature values as a vector
    pub fn feature_values(&self) -> Vec<f64> {
        self.features.iter().map(|f| f.value).collect()
    }

    /// Add metadata to the feature set
    pub fn with_metadata(mut self, key: String, value: f64) -> Self {
        self.metadata.insert(key, value);
        self
    }
}

impl Extend<Feature> for FeatureSet {
    fn extend<T: IntoIterator<Item = Feature>>(&mut self, iter: T) {
        self.features.extend(iter);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_market_data_creation() {
        let timestamp = Utc::now();
        let data = MarketData::new(timestamp, 100.0, 102.0, 99.0, 101.0, 1000.0);
        
        assert_eq!(data.open, 100.0);
        assert_eq!(data.high, 102.0);
        assert_eq!(data.low, 99.0);
        assert_eq!(data.close, 101.0);
        assert_eq!(data.volume, 1000.0);
    }

    #[test]
    fn test_technical_signal_creation() {
        let timestamp = Utc::now();
        let signal = TechnicalSignal::new(
            "RSI".to_string(),
            65.0,
            SignalType::Buy,
            0.7,
            0.8,
            timestamp,
        );
        
        assert_eq!(signal.indicator_name, "RSI");
        assert_eq!(signal.value, 65.0);
        assert_eq!(signal.signal_type, SignalType::Buy);
        assert_eq!(signal.strength, 0.7);
        assert_eq!(signal.confidence, 0.8);
    }

    #[test]
    fn test_forecast_signal_creation() {
        let timestamp = Utc::now();
        let signal = ForecastSignal::new(
            "ARIMA".to_string(),
            105.0,
            0.9,
            MarketRegime::Trending,
            5,
            timestamp,
        );
        
        assert_eq!(signal.model_name, "ARIMA");
        assert_eq!(signal.forecast, 105.0);
        assert_eq!(signal.confidence, 0.9);
        assert_eq!(signal.regime, MarketRegime::Trending);
        assert_eq!(signal.horizon, 5);
    }

    #[test]
    fn test_hybrid_signal_creation() {
        let timestamp = Utc::now();
        let signal = HybridSignal::new(
            SignalType::Buy,
            0.8,
            0.85,
            0.8,
            0.9,
            0.82,
            timestamp,
            100.0,
        );
        
        assert_eq!(signal.signal_type, SignalType::Buy);
        assert_eq!(signal.strength, 0.8);
        assert_eq!(signal.confidence, 0.85);
        assert_eq!(signal.technical_confidence, 0.8);
        assert_eq!(signal.forecast_confidence, 0.9);
        assert_eq!(signal.confirmation_score, 0.82);
        assert_eq!(signal.price, 100.0);
    }

    #[test]
    fn test_feature_set_operations() {
        let timestamp = Utc::now();
        let mut feature_set = FeatureSet::new(Vec::new(), timestamp);
        
        let feature1 = Feature::new("rsi".to_string(), 65.0, FeatureType::Momentum, timestamp);
        let feature2 = Feature::new("macd".to_string(), 0.5, FeatureType::Momentum, timestamp);
        
        feature_set.add_feature(feature1);
        feature_set.add_feature(feature2);
        
        assert_eq!(feature_set.features.len(), 2);
        assert_eq!(feature_set.get_value("rsi"), Some(65.0));
        assert_eq!(feature_set.get_value("macd"), Some(0.5));
        assert_eq!(feature_set.get_value("nonexistent"), None);
    }

    #[test]
    fn test_market_regime_names() {
        assert_eq!(MarketRegime::Trending.name(), "Trending");
        assert_eq!(MarketRegime::Ranging.name(), "Ranging");
        assert_eq!(MarketRegime::Volatile.name(), "Volatile");
        assert_eq!(MarketRegime::Unknown.name(), "Unknown");
    }
} 