//! # Minute Trade
//!
//! [![Minute Trade](https://raw.githubusercontent.com/yourusername/nyxs_owl/main/IMG_2167_250px.jpg)](https://github.com/yourusername/nyxs_owl)
//!
//! `minute_trade` is a Rust library for implementing intraday trading strategies
//! using minute-by-minute OHLCV (Open, High, Low, Close, Volume) data spanning multiple days.
//!
//! This crate provides high-frequency trading strategies optimized for minute-level analysis.
//! Strategies are organized into several categories based on their approach:
//!
//! - **Momentum strategies**: Capitalize on price movement continuation
//! - **Mean reversion strategies**: Trade on the assumption that prices revert to the mean
//! - **Volatility strategies**: Trade based on market volatility patterns
//! - **Pattern recognition strategies**: Identify and trade chart patterns
//! - **Time-based strategies**: Trade based on specific times of the day
//! - **Statistical strategies**: Use statistical methods to find trading opportunities
//! - **Volume-based strategies**: Analyze volume patterns for trading signals
//!
//! ## Usage Example
//!
//! ```no_run
//! use minute_trade::{ScalpingStrategy, IntradayStrategy};
//! use minute_trade::utils::load_minute_data;
//!
//! // Load minute-by-minute data
//! let data = load_minute_data("AAPL_minute_data.csv").unwrap();
//!
//! // Create a scalping strategy
//! let strategy = ScalpingStrategy::new(5, 0.1)?;
//!
//! // Generate trading signals
//! let signals = strategy.generate_signals(&data)?;
//!
//! // Calculate performance
//! let performance = strategy.calculate_performance(&data, &signals)?;
//! println!("Strategy performance: {}%", performance);
//! ```

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

// Strategy modules
mod strategies;
pub mod utils;

// Re-export all strategies for convenient access
pub use strategies::mean_reversion::{
    MeanReversionOscillatorStrategy, StatisticalArbitrageStrategy,
};
pub use strategies::momentum::{MomentumBreakoutStrategy, ScalpingStrategy};
pub use strategies::pattern::{ChartPatternStrategy, SupportResistanceStrategy};
pub use strategies::statistical::{RegressionStrategy, ZScoreStrategy};
pub use strategies::time_based::{SessionTransitionStrategy, TimeOfDayStrategy};
pub use strategies::volatility::{BollingerBandContractionStrategy, VolatilityBreakoutStrategy};
pub use strategies::volume::{RelativeVolumeStrategy, VolumeProfileStrategy};

/// Errors that can occur in intraday trading operations
#[derive(Error, Debug)]
pub enum TradeError {
    #[error("Invalid data: {0}")]
    InvalidData(String),

    #[error("Insufficient data for strategy: {0}")]
    InsufficientData(String),

    #[error("Strategy calculation error: {0}")]
    CalculationError(String),

    #[error("Data loading error: {0}")]
    DataLoadError(String),

    #[error("Parameter validation error: {0}")]
    ParameterError(String),
}

/// Represents OHLCV (Open, High, Low, Close, Volume) data for a specific minute
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OhlcvData {
    /// Open price
    pub open: f64,
    /// High price
    pub high: f64,
    /// Low price
    pub low: f64,
    /// Close price
    pub close: f64,
    /// Volume
    pub volume: f64,
}

/// Minute-level OHLCV data with timestamp
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinuteOhlcv {
    /// Timestamp of the data point
    pub timestamp: DateTime<Utc>,
    /// OHLCV data
    pub data: OhlcvData,
}

/// Trading signal type representing buy/sell/hold decisions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Signal {
    /// Buy signal - enter a long position
    Buy,
    /// Sell signal - exit a position or enter a short position
    Sell,
    /// Hold signal - maintain current position
    Hold,
}

impl fmt::Display for Signal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Signal::Buy => write!(f, "Buy"),
            Signal::Sell => write!(f, "Sell"),
            Signal::Hold => write!(f, "Hold"),
        }
    }
}

/// Signal strength for more nuanced trading signals
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SignalStrength {
    /// Strong buy signal
    StrongBuy = 2,
    /// Normal buy signal
    Buy = 1,
    /// Hold/neutral signal
    Neutral = 0,
    /// Normal sell signal
    Sell = -1,
    /// Strong sell signal
    StrongSell = -2,
}

impl From<SignalStrength> for Signal {
    fn from(strength: SignalStrength) -> Self {
        match strength {
            SignalStrength::StrongBuy | SignalStrength::Buy => Signal::Buy,
            SignalStrength::StrongSell | SignalStrength::Sell => Signal::Sell,
            SignalStrength::Neutral => Signal::Hold,
        }
    }
}

/// Trait defining an intraday trading strategy
pub trait IntradayStrategy {
    /// Get the name of the strategy
    fn name(&self) -> &str;

    /// Analyze minute data and generate trading signals
    fn generate_signals(&self, data: &[MinuteOhlcv]) -> Result<Vec<Signal>, TradeError>;

    /// Calculate performance metrics for the strategy
    fn calculate_performance(
        &self,
        data: &[MinuteOhlcv],
        signals: &[Signal],
    ) -> Result<f64, TradeError>;

    /// Reset the strategy state if needed
    fn reset(&mut self) {}
}

/// Trade execution details
#[derive(Debug, Clone)]
pub struct Trade {
    /// Entry timestamp
    pub entry_time: DateTime<Utc>,
    /// Exit timestamp
    pub exit_time: Option<DateTime<Utc>>,
    /// Entry price
    pub entry_price: f64,
    /// Exit price
    pub exit_price: Option<f64>,
    /// Position size
    pub size: f64,
    /// Trade direction (true for long, false for short)
    pub is_long: bool,
    /// Trade profit/loss
    pub pnl: Option<f64>,
}

/// Performance metrics for a strategy
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    /// Total return percentage
    pub total_return: f64,
    /// Annualized return
    pub annualized_return: f64,
    /// Sharpe ratio
    pub sharpe_ratio: f64,
    /// Maximum drawdown
    pub max_drawdown: f64,
    /// Win rate (percentage of winning trades)
    pub win_rate: f64,
    /// Profit factor (gross profit / gross loss)
    pub profit_factor: f64,
    /// Total number of trades
    pub total_trades: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    pub fn create_test_data(length: usize) -> Vec<MinuteOhlcv> {
        let mut data = Vec::with_capacity(length);
        let base_price = 100.0;

        for i in 0..length {
            // Create timestamp with minute increments
            let timestamp = Utc
                .with_ymd_and_hms(2023, 1, 1, 9, i as u32 % 60, 0)
                .unwrap();

            // Create some price variation
            let close = base_price + (i as f64 * 0.01).sin() * 2.0;
            let open = close - 0.1 + (i as f64 * 0.005).cos() * 0.2;
            let high = close.max(open) + 0.1 + (i as f64 * 0.02).cos() * 0.1;
            let low = close.min(open) - 0.1 + (i as f64 * 0.02).sin() * 0.1;
            let volume = 1000.0 + (i as f64 * 0.1).cos() * 500.0;

            data.push(MinuteOhlcv {
                timestamp,
                data: OhlcvData {
                    open,
                    high,
                    low,
                    close,
                    volume,
                },
            });
        }

        data
    }

    #[test]
    fn test_signal_conversion() {
        assert_eq!(Signal::from(SignalStrength::StrongBuy), Signal::Buy);
        assert_eq!(Signal::from(SignalStrength::Buy), Signal::Buy);
        assert_eq!(Signal::from(SignalStrength::Neutral), Signal::Hold);
        assert_eq!(Signal::from(SignalStrength::Sell), Signal::Sell);
        assert_eq!(Signal::from(SignalStrength::StrongSell), Signal::Sell);
    }
}
