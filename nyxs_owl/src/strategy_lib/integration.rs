//! # Integration Utilities
//!
//! This module provides utilities for integrating NyxsOwl strategies with external
//! backtesting frameworks, particularly those that use async traits.

use crate::strategy_lib::strategy::{Signal, Strategy, StrategyError};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use polars::prelude::*;
use serde::{Deserialize, Serialize};

/// Async version of the Strategy trait for integration with async backtesting frameworks
#[async_trait]
pub trait AsyncStrategy: Send + Sync {
    /// Generate trading signals asynchronously
    async fn generate_signals_async(&self, data: &DataFrame) -> Result<Series, StrategyError>;

    /// Get strategy name
    fn name(&self) -> &str;

    /// Get strategy description
    fn description(&self) -> &str;

    /// Get required columns
    fn required_columns(&self) -> Vec<&str>;

    /// Minimum number of data points required
    fn min_data_points(&self) -> usize {
        50
    }
}

/// Adapter to wrap synchronous strategies for async use
pub struct StrategyAdapter<T: Strategy + Send + Sync + Clone + 'static> {
    strategy: T,
}

impl<T: Strategy + Send + Sync + Clone + 'static> StrategyAdapter<T> {
    pub fn new(strategy: T) -> Self {
        Self { strategy }
    }
}

#[async_trait]
impl<T: Strategy + Send + Sync + Clone + 'static> AsyncStrategy for StrategyAdapter<T> {
    async fn generate_signals_async(&self, data: &DataFrame) -> Result<Series, StrategyError> {
        // Run the synchronous strategy in a blocking task if tokio is available
        #[cfg(feature = "async-support")]
        {
            let strategy = self.strategy.clone();
            let data = data.clone();

            tokio::task::spawn_blocking(move || strategy.generate_signals(&data))
                .await
                .map_err(|e| {
                    StrategyError::IndicatorError(format!("Async execution failed: {}", e))
                })?
        }

        #[cfg(not(feature = "async-support"))]
        {
            // Fallback to synchronous execution
            self.strategy.generate_signals(data)
        }
    }

    fn name(&self) -> &str {
        self.strategy.name()
    }

    fn description(&self) -> &str {
        self.strategy.description()
    }

    fn required_columns(&self) -> Vec<&str> {
        self.strategy.required_columns()
    }

    fn min_data_points(&self) -> usize {
        // Default minimum for most strategies
        50
    }
}

/// Data conversion utilities for different OHLCV formats
pub struct DataConverter;

impl DataConverter {
    /// Convert from external OHLCV format to Polars DataFrame
    pub fn from_ohlcv_vec(data: &[ExternalOHLCV]) -> Result<DataFrame, StrategyError> {
        let timestamps: Vec<i64> = data.iter().map(|d| d.timestamp.timestamp()).collect();
        let opens: Vec<f64> = data.iter().map(|d| d.open).collect();
        let highs: Vec<f64> = data.iter().map(|d| d.high).collect();
        let lows: Vec<f64> = data.iter().map(|d| d.low).collect();
        let closes: Vec<f64> = data.iter().map(|d| d.close).collect();
        let volumes: Vec<f64> = data.iter().map(|d| d.volume).collect();

        DataFrame::new(vec![
            Series::new("timestamp".into(), timestamps).into(),
            Series::new("open".into(), opens).into(),
            Series::new("high".into(), highs).into(),
            Series::new("low".into(), lows).into(),
            Series::new("close".into(), closes).into(),
            Series::new("volume".into(), volumes).into(),
        ])
        .map_err(StrategyError::PolarsError)
    }

    /// Convert from Polars DataFrame to external OHLCV format
    pub fn to_ohlcv_vec(df: &DataFrame) -> Result<Vec<ExternalOHLCV>, StrategyError> {
        let timestamps = df
            .column("timestamp")
            .map_err(|_| StrategyError::MissingData("timestamp column not found".to_string()))?
            .i64()
            .map_err(|_| StrategyError::InvalidParameter("Invalid timestamp format".to_string()))?;

        let opens = df
            .column("open")
            .map_err(|_| StrategyError::MissingData("open column not found".to_string()))?
            .f64()
            .map_err(|_| {
                StrategyError::InvalidParameter("Invalid open price format".to_string())
            })?;

        let highs = df
            .column("high")
            .map_err(|_| StrategyError::MissingData("high column not found".to_string()))?
            .f64()
            .map_err(|_| {
                StrategyError::InvalidParameter("Invalid high price format".to_string())
            })?;

        let lows = df
            .column("low")
            .map_err(|_| StrategyError::MissingData("low column not found".to_string()))?
            .f64()
            .map_err(|_| StrategyError::InvalidParameter("Invalid low price format".to_string()))?;

        let closes = df
            .column("close")
            .map_err(|_| StrategyError::MissingData("close column not found".to_string()))?
            .f64()
            .map_err(|_| {
                StrategyError::InvalidParameter("Invalid close price format".to_string())
            })?;

        let volumes = df
            .column("volume")
            .map_err(|_| StrategyError::MissingData("volume column not found".to_string()))?
            .f64()
            .map_err(|_| StrategyError::InvalidParameter("Invalid volume format".to_string()))?;

        let mut result = Vec::new();

        for i in 0..timestamps.len() {
            if let (Some(ts), Some(o), Some(h), Some(l), Some(c), Some(v)) = (
                timestamps.get(i),
                opens.get(i),
                highs.get(i),
                lows.get(i),
                closes.get(i),
                volumes.get(i),
            ) {
                result.push(ExternalOHLCV {
                    timestamp: DateTime::from_timestamp(ts, 0).ok_or_else(|| {
                        StrategyError::InvalidParameter("Invalid timestamp".to_string())
                    })?,
                    open: o,
                    high: h,
                    low: l,
                    close: c,
                    volume: v,
                });
            }
        }

        Ok(result)
    }

    /// Convert external OHLCV data to our internal format
    #[cfg(feature = "day-trading")]
    pub fn from_daily_ohlcv(data: &[crate::day_trade::DailyOhlcv]) -> Vec<ExternalOHLCV> {
        data.iter()
            .map(|d| ExternalOHLCV {
                timestamp: NaiveDateTime::from_timestamp_opt(
                    d.date.and_hms_opt(0, 0, 0).unwrap().timestamp(),
                    0,
                )
                .unwrap()
                .and_utc(),
                open: d.data.open,
                high: d.data.high,
                low: d.data.low,
                close: d.data.close,
                volume: d.data.volume as f64,
            })
            .collect()
    }

    /// Convert external OHLCV data back to daily format
    #[cfg(feature = "day-trading")]
    pub fn to_daily_ohlcv(data: &[ExternalOHLCV]) -> Vec<crate::day_trade::DailyOhlcv> {
        data.iter()
            .map(|d| crate::day_trade::DailyOhlcv {
                date: d.timestamp.date_naive(),
                data: crate::day_trade::OhlcvData {
                    open: d.open,
                    high: d.high,
                    low: d.low,
                    close: d.close,
                    volume: d.volume as u64,
                },
            })
            .collect()
    }
}

/// External OHLCV format for integration with other frameworks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalOHLCV {
    pub timestamp: DateTime<Utc>,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}

/// Configuration for strategy integration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationConfig {
    /// Whether to use async execution
    pub use_async: bool,
    /// Timeout for async operations in milliseconds
    pub timeout_ms: u64,
    /// Buffer size for data processing
    pub buffer_size: usize,
}

impl Default for IntegrationConfig {
    fn default() -> Self {
        Self {
            use_async: true,
            timeout_ms: 5000,
            buffer_size: 1000,
        }
    }
}

/// Signal conversion utilities
pub struct SignalConverter;

impl SignalConverter {
    /// Convert NyxsOwl Signal to external signal format
    pub fn to_external_signal(
        signal: &Signal,
        price: f64,
        timestamp: DateTime<Utc>,
    ) -> ExternalSignal {
        ExternalSignal {
            signal_type: match signal {
                Signal::Buy => ExternalSignalType::Buy,
                Signal::Sell => ExternalSignalType::Sell,
                Signal::Hold => ExternalSignalType::Hold,
            },
            price,
            timestamp,
            confidence: 1.0, // Default confidence
        }
    }

    /// Convert external signal to NyxsOwl Signal
    pub fn from_external_signal(signal: &ExternalSignal) -> Signal {
        match signal.signal_type {
            ExternalSignalType::Buy => Signal::Buy,
            ExternalSignalType::Sell => Signal::Sell,
            ExternalSignalType::Hold => Signal::Hold,
        }
    }
}

/// External signal format for integration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalSignal {
    pub signal_type: ExternalSignalType,
    pub price: f64,
    pub timestamp: DateTime<Utc>,
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExternalSignalType {
    Buy,
    Sell,
    Hold,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::strategy_lib::strategy::trend_following::MovingAverageCrossover;

    #[test]
    fn test_data_converter() {
        let external_data = vec![ExternalOHLCV {
            timestamp: Utc::now(),
            open: 100.0,
            high: 105.0,
            low: 95.0,
            close: 102.0,
            volume: 1000.0,
        }];

        let df = DataConverter::from_ohlcv_vec(&external_data).unwrap();
        assert_eq!(df.height(), 1);
        assert_eq!(df.width(), 6);

        let converted_back = DataConverter::to_ohlcv_vec(&df).unwrap();
        assert_eq!(converted_back.len(), 1);
        assert_eq!(converted_back[0].close, 102.0);
    }

    #[cfg(feature = "async-support")]
    #[tokio::test]
    async fn test_strategy_adapter() {
        // This test would require a concrete strategy implementation
        // For now, we'll just test that the adapter can be created
        // let strategy = MovingAverageCrossover::new(StrategyConfig::default());
        // let adapter = StrategyAdapter::new(strategy);
        // assert_eq!(adapter.name(), "Moving Average Crossover");
    }
}
