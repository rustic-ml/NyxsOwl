//! Intraday trading strategies module
//!
//! This module contains various intraday trading strategies organized by type:
//!
//! - **Momentum strategies**: Capitalize on price movement continuation
//!   - `ScalpingStrategy`: Quick entry/exit based on short-term price momentum
//!   - `MomentumBreakoutStrategy`: Trade breakouts with strong momentum
//!
//! - **Mean reversion strategies**: Trade on the assumption that prices revert to the mean
//!   - `StatisticalArbitrageStrategy`: Exploit price divergence between correlated assets
//!   - `MeanReversionOscillatorStrategy`: Trade oversold/overbought conditions using RSI
//!
//! - **Volatility strategies**: Trade based on market volatility patterns
//!   - `VolatilityBreakoutStrategy`: Enter trades after periods of low volatility
//!   - `BollingerBandContractionStrategy`: Trade volatility expansions following tight Bollinger Bands
//!
//! - **Pattern recognition strategies**: Identify and trade chart patterns
//!   - `ChartPatternStrategy`: Trade classic patterns like flags, double tops/bottoms, and head and shoulders
//!   - `SupportResistanceStrategy`: Trade bounces and breakouts from key price levels
//!
//! - **Time-based strategies**: Trade based on specific times of the day
//!   - `TimeOfDayStrategy`: Enter and exit positions at specific times
//!   - `SessionTransitionStrategy`: Trade during market session transitions (planned)
//!
//! - **Statistical strategies**: Use statistical methods to find trading opportunities
//!   - `ZScoreStrategy`: Trade statistically significant deviations using z-scores
//!   - `RegressionStrategy`: Trade using linear regression analysis (planned)
//!
//! - **Volume-based strategies**: Analyze volume patterns for trading signals
//!   - `VolumeProfileStrategy`: Identify and trade high-volume price levels
//!   - `RelativeVolumeStrategy`: Trade unusual relative volume spikes (planned)

pub mod mean_reversion;
pub mod momentum;
pub mod pattern;
pub mod statistical;
pub mod time_based;
pub mod volatility;
pub mod volume;

// Re-export all strategies for convenient access
pub use mean_reversion::{MeanReversionOscillatorStrategy, StatisticalArbitrageStrategy};
pub use momentum::{MomentumBreakoutStrategy, ScalpingStrategy};
pub use pattern::{ChartPatternStrategy, SupportResistanceStrategy};
pub use statistical::{RegressionStrategy, ZScoreStrategy};
pub use time_based::{SessionTransitionStrategy, TimeOfDayStrategy};
pub use volatility::{BollingerBandContractionStrategy, VolatilityBreakoutStrategy};
pub use volume::{RelativeVolumeStrategy, VolumeProfileStrategy};
