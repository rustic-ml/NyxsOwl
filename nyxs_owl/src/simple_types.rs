use polars::prelude::*;
use thiserror::Error;

/// Custom error type for NyxsOwl operations
#[derive(Error, Debug)]
pub enum NyxsOwlError {
    #[error("Data processing error: {0}")]
    DataError(String),

    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),

    #[error("Missing data: {0}")]
    MissingData(String),

    #[error("Strategy error: {0}")]
    StrategyError(String),

    #[error("Backtest execution error: {0}")]
    BacktestError(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Indicator calculation error: {0}")]
    IndicatorError(String),

    #[error("Model error: {0}")]
    ModelError(String),

    #[error("Not implemented: {0}")]
    NotImplemented(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Polars error: {0}")]
    PolarsError(#[from] PolarsError),
}

/// Result type alias for NyxsOwl operations
pub type Result<T> = std::result::Result<T, NyxsOwlError>;

/// Price type alias for clarity
pub type Price = f64;

/// Trading signal enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Signal {
    /// Hold current position
    Hold = 0,
    /// Buy signal
    Buy = 1,
    /// Sell signal
    Sell = 2,
}

impl Signal {
    /// Convert signal to integer representation
    pub fn to_int(self) -> i32 {
        self as i32
    }

    /// Create signal from integer representation
    pub fn from_int(value: i32) -> Self {
        match value {
            1 => Signal::Buy,
            -1 | 2 => Signal::Sell,
            _ => Signal::Hold,
        }
    }
}

impl std::fmt::Display for Signal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Signal::Buy => write!(f, "BUY"),
            Signal::Sell => write!(f, "SELL"),
            Signal::Hold => write!(f, "HOLD"),
        }
    }
}

/// Position type for trading signals
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum PositionType {
    /// Long position
    Long,
    /// Short position
    Short,
    /// Hold position
    Hold,
}

/// Enhanced trading signal with additional metadata
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct SignalData {
    /// The trading signal
    pub signal: Signal,
    /// The position type
    pub position_type: PositionType,
    /// Optional timestamp of the signal
    pub timestamp: Option<chrono::DateTime<chrono::Utc>>,
    /// Confidence level of the signal (0.0 to 1.0)
    pub confidence: f64,
    /// Optional metadata with additional signal information
    pub metadata: Option<std::collections::HashMap<String, f64>>,
}

impl SignalData {
    /// Create a new SignalData with default values
    pub fn new(signal: Signal) -> Self {
        let position_type = match signal {
            Signal::Buy => PositionType::Long,
            Signal::Sell => PositionType::Short,
            Signal::Hold => PositionType::Hold,
        };

        Self {
            signal,
            position_type,
            timestamp: None,
            confidence: 1.0,
            metadata: None,
        }
    }

    /// Set the confidence level of the signal
    pub fn with_confidence(mut self, confidence: f64) -> Self {
        self.confidence = confidence.clamp(0.0, 1.0);
        self
    }

    /// Set the timestamp of the signal
    pub fn with_timestamp(mut self, timestamp: chrono::DateTime<chrono::Utc>) -> Self {
        self.timestamp = Some(timestamp);
        self
    }

    /// Set the metadata of the signal
    pub fn with_metadata(mut self, metadata: std::collections::HashMap<String, f64>) -> Self {
        self.metadata = Some(metadata);
        self
    }

    /// Add a single metadata entry
    pub fn add_metadata(mut self, key: &str, value: f64) -> Self {
        let mut metadata = self.metadata.unwrap_or_default();
        metadata.insert(key.to_string(), value);
        self.metadata = Some(metadata);
        self
    }
}

impl Default for SignalData {
    fn default() -> Self {
        Self::new(Signal::Hold)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signal_conversion() {
        assert_eq!(Signal::Buy.to_int(), 1);
        assert_eq!(Signal::Sell.to_int(), 2);
        assert_eq!(Signal::Hold.to_int(), 0);

        assert_eq!(Signal::from_int(1), Signal::Buy);
        assert_eq!(Signal::from_int(2), Signal::Sell);
        assert_eq!(Signal::from_int(-1), Signal::Sell);
        assert_eq!(Signal::from_int(0), Signal::Hold);
        assert_eq!(Signal::from_int(99), Signal::Hold);
    }

    #[test]
    fn test_signal_display() {
        assert_eq!(format!("{}", Signal::Buy), "BUY");
        assert_eq!(format!("{}", Signal::Sell), "SELL");
        assert_eq!(format!("{}", Signal::Hold), "HOLD");
    }

    #[test]
    fn test_signal_data_creation() {
        let signal_data = SignalData::new(Signal::Buy)
            .with_confidence(0.8)
            .add_metadata("rsi", 75.0);

        assert_eq!(signal_data.signal, Signal::Buy);
        assert_eq!(signal_data.position_type, PositionType::Long);
        assert_eq!(signal_data.confidence, 0.8);
        assert_eq!(
            signal_data.metadata.as_ref().unwrap().get("rsi"),
            Some(&75.0)
        );
    }

    #[test]
    fn test_confidence_clamping() {
        let signal_data = SignalData::new(Signal::Buy).with_confidence(1.5);
        assert_eq!(signal_data.confidence, 1.0);

        let signal_data = SignalData::new(Signal::Buy).with_confidence(-0.5);
        assert_eq!(signal_data.confidence, 0.0);
    }
}
